#![windows_subsystem = "windows"]
use deft::app::{App, IApp};
use deft::bootstrap;
use deft::js::js_engine::JsEngine;
use deft_video::deft_video_init;
use std::env;

struct MyApp {}

impl IApp for MyApp {
    fn init_js_engine(&mut self, js_engine: &mut JsEngine) {
        js_engine
            .eval_module(include_str!("../dev-hack.js"), "dev-hack.js")
            .unwrap();
        deft_video_init(js_engine);
    }

    fn create_module_loader(
        &mut self,
    ) -> Box<dyn deft::js::loader::JsModuleLoader + Send + Sync + 'static> {
        include!(concat!(env!("OUT_DIR"), "/js_loader.code"))
    }
}

fn main() {
    env_logger::init();
    let app = App::new(MyApp {});
    bootstrap(app.clone());
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: deft::winit::platform::android::activity::AndroidApp) {
    let app = App::new(MyApp {});
    deft::android_bootstrap(android_app, app);
}
