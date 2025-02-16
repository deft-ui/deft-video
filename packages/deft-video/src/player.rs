use std::io::Seek;
use std::mem::MaybeUninit;
use std::ops::Range;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::anyhow;
use bytemuck::Pod;
use cpal::{Sample, SizedSample};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ffmpeg_next::decoder::{Audio, Video};
use ffmpeg_next::format::context::Input;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::software::resampling::Context;
use ffmpeg_next::{ffi, frame, Error, Rational};
use ffmpeg_next::ffi::AV_TIME_BASE;
use deft::{event, js_serialize};
use ringbuf::{HeapRb, Producer, SharedRb};
use serde::Serialize;
use crate::player_thread::PlayerThread;
use crate::thread_executor::{SingleThreadExecutor};

pub struct Player {
    player_thread: Option<PlayerThread>,
}

#[derive(Serialize, Clone)]
pub struct Meta {
    pub width: usize,
    pub height: usize,
    pub duration: f32,
}

pub struct PlayServer {
    path: String,
    video_stream_index: usize,
    packet_decoder: Video,

    audio_stream_index: usize,
    audio_packet_decoder: Audio,
    input_context: Input,
    timebase: Rational,
    rescale_context: Option<ffmpeg_next::software::scaling::Context>,
    audio_sender: Option<mpsc::Sender<frame::Audio>>,
    latest_frame: Option<frame::Video>,
    stream_clock: Option<StreamClock>,
}

pub enum ControlMessage {
    Play,
    Pause,
    Seek(f32),
    Stop,
}

impl PlayServer {
    pub fn new(path: String) -> Self {
        let mut input_context = ffmpeg_next::format::input(&path).unwrap();
        let video_stream =
            input_context.streams().best(ffmpeg_next::media::Type::Video).unwrap();
        let video_stream_index = video_stream.index();

        let decoder_context = ffmpeg_next::codec::Context::from_parameters(video_stream.parameters()).unwrap();
        let mut packet_decoder = decoder_context.decoder().video().unwrap();


        let audio_stream =
            input_context.streams().best(ffmpeg_next::media::Type::Audio).unwrap();
        let audio_stream_index = audio_stream.index();
        let audio_decoder_context = ffmpeg_next::codec::Context::from_parameters(audio_stream.parameters()).unwrap();
        let mut audio_packet_decoder = audio_decoder_context.decoder().audio().unwrap();
        let mut player = Self {
            path,
            video_stream_index,
            packet_decoder,
            audio_stream_index,
            audio_packet_decoder,
            timebase: video_stream.time_base(),
            input_context,
            audio_sender: None,
            rescale_context: None,
            latest_frame: None,
            stream_clock: None,
        };
        player.next_frame(false);
        player
    }

    pub fn get_duration(&self) -> f32 {
        let duration = self.input_context.duration();
        duration as f32 / ffmpeg_next::ffi::AV_TIME_BASE as f32
    }

    pub fn get_width(&self) -> Option<usize> {
        self.latest_frame.as_ref().map(|frame| frame.width() as usize)
    }

    pub fn get_height(&self) -> Option<usize> {
        self.latest_frame.as_ref().map(|frame| frame.height() as usize)
    }

    pub fn play(
        &mut self,
        mut renderer: Box<dyn FnMut(frame::Video) + Send + 'static>,
        mut progress_handler: Box<dyn  FnMut(f32)>,
        mut control_msg_receiver: mpsc::Receiver<ControlMessage>,
        mut stop_handler: Box<dyn FnMut()>,
    ) {
        let (audio_frame_sender, audio_frame_receiver) = mpsc::channel();
        self.audio_sender = Some(audio_frame_sender);
        let mut audio_playback = AudioPlayback::<f32>::new(&self.audio_packet_decoder, audio_frame_receiver);

        thread::spawn(move || {
            // Note: playback will stop when audio_frame_sender dropped
            audio_playback.run();
        });

        let mut seek_time = None;
        let mut playing = false;
        loop {
            let msg = if playing {
                control_msg_receiver.try_recv().ok()
            } else {
                control_msg_receiver.recv().ok()
            };

            if let Some(msg) = msg {
                match msg {
                    ControlMessage::Play => {
                        playing = true;
                    }
                    ControlMessage::Pause => {
                        playing = false;
                        self.stream_clock = None;
                        continue;
                    }
                    ControlMessage::Seek(time) => {
                        self.seek(time);
                        seek_time = Some(time);
                    }
                    ControlMessage::Stop => {
                        break;
                    }
                }
            }

            if let Some(st) = seek_time {
                if let Some(sc) = &self.stream_clock {
                    //TODO impl fast accurate seek
                    // let mut pts = 0;
                    // let expected_ts = sc.convert_time_to_pts(st as f64);
                    // while pts < expected_ts {
                    //     self.next_frame(false).unwrap();
                    //     pts = self.latest_frame.as_ref().unwrap().pts().unwrap_or(0);
                    // }
                    self.stream_clock = None;
                }
                seek_time = None;
            }

            let rgb_frame = match self.next_frame(true) {
                Ok(frame) => frame,
                Err(_err) => break,
            };
            let mut pts = self.latest_frame.as_ref().unwrap().pts();
            // println!("pts: {:?}", pts);
            if self.stream_clock.is_none() {
                let latest_pts = self.latest_frame.as_ref().map(|f| f.pts().unwrap_or(0)).unwrap_or(0);
                self.stream_clock = Some(StreamClock::new(self.timebase, latest_pts));
            }
            let stream_clock = self.stream_clock.as_ref().unwrap();
            if let Some(delay) = stream_clock.convert_pts_to_instant(pts) {
                //println!("delay: {:?}", delay);
                thread::sleep(delay);
            }
            renderer(rgb_frame);
            let time = stream_clock.convert_pts_to_time(pts.unwrap_or(0)) as f32;
            progress_handler(time);
        }
        stop_handler();
    }

    pub fn seek(&mut self, ts: f32) {
        //TODO check error
        let ts = (ts * AV_TIME_BASE as f32) as i64;
        if let Err(e) = self.input_context.seek(ts, (0..ts)) {
            println!("seek error: {:?}", e);
        }
        // Clear buffers
        self.packet_decoder.flush();
        self.audio_packet_decoder.flush();
    }

    fn next_frame(&mut self, play_audio: bool) -> Result<frame::Video, anyhow::Error> {
        let mut decoded_frame = ffmpeg_next::util::frame::Video::empty();
        if self.packet_decoder.receive_frame(&mut decoded_frame).is_ok() {
            let format = decoded_frame.format();
            let rebuild_rescale_context = self.rescale_context.as_ref().map_or(true, |r| r.input().format != format);
            if rebuild_rescale_context {
                self.rescale_context = Some(create_rescale_context(&decoded_frame));
            }

            let mut rgb_frame = ffmpeg_next::util::frame::Video::empty();
            let mut rescaler = self.rescale_context.as_mut().unwrap();
            rescaler.run(&decoded_frame, &mut rgb_frame).unwrap();
            self.latest_frame = Some(decoded_frame);
            return Ok(rgb_frame)
        }

        let (stream, packet) = self.input_context.packets().next().ok_or(anyhow!("eof"))?;

        if stream.index() == self.video_stream_index {
            self.packet_decoder.send_packet(&packet)?;
        } else if stream.index() == self.audio_stream_index {
            let mut decoded_frame = ffmpeg_next::util::frame::Audio::empty();
            self.audio_packet_decoder.send_packet(&packet).unwrap();
            while self.audio_packet_decoder.receive_frame(&mut decoded_frame).is_ok() {
                // println!("sending audio frame");
                if let Some(audio_frame_sender) = &mut self.audio_sender {
                    if play_audio {
                        audio_frame_sender.send(decoded_frame.clone())?;
                    }
                }
            }
        }
        self.next_frame(play_audio)
    }

}

fn create_rescale_context(frame: &ffmpeg_next::util::frame::Video) -> ffmpeg_next::software::scaling::Context {
    ffmpeg_next::software::scaling::Context::get(
        frame.format(),
        frame.width(),
        frame.height(),
        Pixel::RGBA,
        frame.width(),
        frame.height(),
        ffmpeg_next::software::scaling::Flags::BILINEAR,
    ).unwrap()
}


unsafe impl Send for AudioPlayback<f32> {}

struct AudioPlayback<T> {
    _stream: cpal::Stream,
    frame_receiver: mpsc::Receiver<ffmpeg_next::util::frame::Audio>,
    sample_producer: Producer<T, Arc<SharedRb<T, Vec<MaybeUninit<T>>>>>,
    context: Context,
}

impl<T: Send + Pod + SizedSample + 'static> AudioPlayback<T> {
    pub fn new(packet_decoder: &Audio, frame_receiver: mpsc::Receiver<ffmpeg_next::util::frame::Audio>) -> Self {
        let buffer = HeapRb::new(4096 * 2);
        let (mut sample_producer, mut sample_consumer) = buffer.split();

        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");

        let config = device.default_output_config().unwrap();
        let fmt = config.sample_format();
        println!("audio format: {:?}", fmt);


        let output_channel_layout = match config.channels() {
            1 => ffmpeg_next::util::channel_layout::ChannelLayout::MONO,
            2 => {
                ffmpeg_next::util::channel_layout::ChannelLayout::STEREO_LEFT
                    | ffmpeg_next::util::channel_layout::ChannelLayout::STEREO_RIGHT
            }
            _ => todo!(),
        };

        //TODO fix output format
        let output_format = ffmpeg_next::util::format::sample::Sample::F32(
            ffmpeg_next::util::format::sample::Type::Packed,
        );

        let cpal_stream = device
            .build_output_stream(
                &config.config(),
                move |data: &mut [T], _| {
                    // println!("filling data");
                    let filled = sample_consumer.pop_slice(data);
                    data[filled..].fill(T::EQUILIBRIUM);
                },
                move |err| {
                    eprintln!("error feeding audio stream to cpal: {}", err);
                },
                None,
            )
            .unwrap();

        cpal_stream.play().unwrap();


        let resampler = ffmpeg_next::software::resampling::Context::get(
            packet_decoder.format(),
            packet_decoder.channel_layout(),
            packet_decoder.rate(),
            output_format,
            output_channel_layout,
            config.sample_rate().0,
        )
            .unwrap();
        Self {
            frame_receiver,
            sample_producer,
            context: resampler,
            _stream: cpal_stream,
        }
    }

    pub fn run(mut self) {
        loop {
            let frame = match self.frame_receiver.recv() {
                Ok(frame) => frame,
                Err(_) => return,
            };
            // println!("receive audio frame");
            let mut audio_frame = ffmpeg_next::util::frame::Audio::empty();
            self.context.run(&frame, &mut audio_frame).unwrap();
            // println!("resampled audio frame");


            let expected_bytes =
                audio_frame.samples() * audio_frame.channels() as usize * size_of::<T>();
            let cpal_sample_data: &[T] =
                bytemuck::cast_slice(&audio_frame.data(0)[..expected_bytes]);

            while self.sample_producer.free_len() < cpal_sample_data.len() {
                //println!("audio sleeping");
                thread::sleep(Duration::from_millis(16));
            }

            // println!("pushing slice");
            // Buffer the samples for playback
            self.sample_producer.push_slice(cpal_sample_data);
        }
    }
}

struct StreamClock {
    time_base_seconds: f64,
    start_time: std::time::Instant,
    start_pts: i64,
}

impl StreamClock {
    fn new(time_base_seconds: Rational, start_pts: i64) -> Self {
        let time_base_seconds =
            time_base_seconds.numerator() as f64 / time_base_seconds.denominator() as f64;

        let start_time = std::time::Instant::now();

        Self { time_base_seconds, start_time, start_pts }
    }

    fn convert_pts_to_instant(&self, pts: Option<i64>) -> Option<std::time::Duration> {
        pts.and_then(|pts| {
            let pts_since_start =
                std::time::Duration::from_secs_f64((pts - self.start_pts) as f64 * self.time_base_seconds);
            self.start_time.checked_add(pts_since_start)
        })
            .map(|absolute_pts| absolute_pts.duration_since(std::time::Instant::now()))
    }

    fn convert_pts_to_time(&self, pts: i64) -> f64 {
        pts as f64 * self.time_base_seconds
    }

    fn convert_time_to_pts(&self, time: f64) -> i64 {
        (time / self.time_base_seconds) as i64
    }

}

#[derive(Clone)]
struct Signal {
    lit: Arc<Mutex<bool>>,
}

impl Signal {
    pub fn new() -> Self {
        Self {
            lit: Arc::new(Mutex::new(false))
        }
    }
    pub fn lit_up(&self) {
        *self.lit.lock().unwrap() = true;
    }
    pub fn is_lit(&self) -> bool {
        *self.lit.lock().unwrap()
    }
}
