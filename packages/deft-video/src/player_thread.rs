use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use crate::player::{ControlMessage, Meta};

pub struct PlayerThread {
    sender: Sender<ControlMessage>
}

pub struct PlayParams {
    pub path: String,
    pub on_meta_loaded: Box<dyn FnOnce(Meta) + Send + 'static>,
    pub renderer: Box<dyn FnMut(ffmpeg_next::util::frame::Video) + Send + 'static>,
    pub on_progress: Box<dyn FnMut(f32) + Send + 'static>,
    pub on_stop: Box<dyn FnMut() + Send + 'static>,
}

impl PlayerThread {

    pub fn start(mut params: PlayParams) -> Self {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut player = crate::player::PlayServer::new(params.path.clone());
            let width = player.get_width().unwrap();
            let height = player.get_height().unwrap();
            let duration = player.get_duration();
            let meta = Meta { width, height, duration };
            (params.on_meta_loaded)(meta);
            player.play(params.renderer, params.on_progress, receiver, params.on_stop);
        });
        Self {
            sender
        }
    }

    pub fn seek(&mut self, time: f32) {
        let _ = self.sender.send(ControlMessage::Seek(time));
    }

    pub fn play(&self) {
        let _ = self.sender.send(ControlMessage::Play);
    }

    pub fn pause(&self) {
        let _ = self.sender.send(ControlMessage::Pause);
    }

    pub fn stop(&self) {
        let _ = self.sender.send(ControlMessage::Stop);
    }

}