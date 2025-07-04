use crate::player::Meta;
use crate::player_thread::{PlayParams, PlayerThread};
use deft::element::{Element, ElementBackend, ElementWeak};
use deft::event_loop::create_event_loop_fn_mut;
use deft::render::RenderFn;
use deft::{element_backend, event, js_methods, ok_or_return};
use ffmpeg_next::frame::Video;
use skia_safe::Image;
use skia_safe::{AlphaType, Bitmap, ColorSpace, ColorType, ImageInfo, Paint, Rect};
use std::ffi::{c_void};
use std::sync::{Arc, Mutex};
use libc::{memcpy, size_t};

#[element_backend]
pub struct VideoBackend {
    element: ElementWeak,
    frame: Arc<Mutex<Option<Video>>>,
    player: Option<PlayerThread>,
}

#[event]
struct ProgressEvent(f32);

#[event]
struct PlayEvent;

#[event]
struct PauseEvent;

#[event]
struct StopEvent;

#[event]
struct LoadedMetaData(Meta);

#[js_methods]
impl VideoBackend {
    #[js_func]
    pub fn set_src(&mut self, src: String) {
        println!("Setting src: {}", src);
        let mut el = ok_or_return!(self.element.upgrade());

        let frame = self.frame.clone();
        let weak_element = self.element.clone();
        let mut dirty_marker = create_event_loop_fn_mut(move |_| {
            if let Ok(mut el) = weak_element.upgrade() {
                el.mark_dirty(false);
            }
        });

        let meta_loaded_emitter = el.create_event_emitter();
        let progress_emitter = el.create_event_emitter();
        let stop_emitter = el.create_event_emitter();
        let play_params = PlayParams {
            path: src,
            on_meta_loaded: Box::new(move |meta| {
                meta_loaded_emitter.emit(LoadedMetaData(meta));
            }),
            on_progress: Box::new(move |progress| {
                progress_emitter.emit(ProgressEvent(progress));
            }),
            on_stop: Box::new(move || {
                stop_emitter.emit(StopEvent);
            }),
            renderer: Box::new(move |f| {
                let mut frame = frame.lock().unwrap();
                frame.replace(f);
                dirty_marker.call(());
            }),
        };
        self.player = Some(PlayerThread::start(play_params));
        // self.play();
    }

    #[js_func]
    pub fn play(&mut self) {
        let el = self.element.clone();
        if let Some(ref player) = self.player {
            player.play();
            el.emit(PlayEvent);
        }
    }

    #[js_func]
    pub fn seek(&mut self, value: f32) {
        if let Some(ref mut player) = self.player {
            player.seek(value);
        }
    }

    #[js_func]
    pub fn pause(&mut self) {
        if let Some(ref mut player) = self.player {
            player.pause();
            self.element.emit(PauseEvent);
        }
    }

    #[js_func]
    pub fn stop(&mut self) {
        if let Some(ref mut player) = self.player {
            player.stop();
        }
    }
}

impl ElementBackend for VideoBackend {
    fn create(element: &mut Element) -> Self
    where
        Self: Sized,
    {
        element.register_js_event::<ProgressEvent>("progress");
        element.register_js_event::<PlayEvent>("play");
        element.register_js_event::<PauseEvent>("pause");
        element.register_js_event::<StopEvent>("stop");
        element.register_js_event::<LoadedMetaData>("loadedmetadata");
        VideoBackendData {
            element: element.as_weak(),
            frame: Arc::new(Mutex::new(None)),
            player: None,
        }
        .to_ref()
    }

    fn render(&mut self) -> RenderFn {
        let img = {
            match self.frame.lock().unwrap().as_ref() {
                None => return RenderFn::empty(),
                Some(f) => {
                    let stride = f.stride(0);
                    load_image_from_rgba_bytes(
                        f.data(0),
                        stride,
                        f.width() as usize,
                        f.height() as usize,
                        ColorType::RGBA8888,
                    )
                }
            }
        };
        let element = ok_or_return!(self.element.upgrade_mut(), RenderFn::empty());
        let view_size = element.get_size();
        let (view_width, view_height) = (view_size.0 as i32, view_size.1 as i32);
        let (image_width, image_height) = (img.width(), img.height());
        let mut rect_width = view_width;
        let mut rect_height = image_height * rect_width / image_width;
        if rect_height > view_height {
            rect_height = view_height;
            rect_width = image_width * rect_height / image_height;
        }
        let left = (view_width - rect_width) / 2;
        let top = (view_height - rect_height) / 2;
        let rect = Rect::new(
            left as f32,
            top as f32,
            (left + rect_width) as f32,
            (top + rect_height) as f32,
        );
        RenderFn::new(move |painter| {
            painter
                .canvas
                .draw_image_rect(&img, None, &rect, &Paint::default());
        })
    }
}

fn load_image_from_rgba_bytes(
    data: &[u8],
    stride: usize,
    width: usize,
    height: usize,
    color_type: ColorType,
) -> Image {
    let width = width as i32;
    let height = height as i32;
    let image_info = ImageInfo::new(
        (width, height),
        color_type,
        AlphaType::Unpremul,
        ColorSpace::new_srgb(),
    );
    let mut bm = Bitmap::new();
    let _ = bm.set_info(&image_info, stride);
    bm.alloc_pixels();
    unsafe {
        let dest = bm.pixels();
        let src = data.as_ptr() as *const c_void;
        memcpy(dest, src, data.len() as size_t);
    }
    bm.as_image()
}
