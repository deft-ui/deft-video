use crate::video::VideoBackend;
use deft::element::register_component;
use deft::js::js_engine::JsEngine;

mod player;
mod player_thread;
mod video;

pub fn deft_video_init(js_engine: &mut JsEngine) {
    register_component::<VideoBackend>("video");
    js_engine.add_global_functions(VideoBackend::create_js_apis());
    js_engine
        .eval_module(include_str!("video.js"), "video.js")
        .unwrap();
}
