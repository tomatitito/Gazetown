// Pure GPUI agent orchestration application
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod views;

use assets::Assets;
use gpui::{AppContext, Application};

use crate::app::GazetownApp;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    env_logger::init();

    let app = Application::new().with_assets(Assets);

    app.run(|cx| {
        let _ = cx.open_window(Default::default(), |_, cx| cx.new(|_| GazetownApp::new()));
    });
}
