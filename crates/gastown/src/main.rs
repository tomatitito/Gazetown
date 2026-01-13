use anyhow::Result;
use gpui::{
    actions, div, px, size, App, AppContext, Bounds, ParentElement as _, Render, Styled as _,
    ViewContext, Window, WindowBounds, WindowOptions,
};

actions!(gastown, [Quit]);

pub fn on_quit(_quit: &Quit, cx: &mut AppContext) {
    cx.quit();
}

struct GasTown;

impl Render for GasTown {
    fn render(&mut self, _window: &mut Window, _cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .child("Gas Town - Multi-Agent Development Workspace")
    }
}

fn main() -> Result<()> {
    env_logger::init();

    gpui::Application::new().run(|cx| {
        cx.activate(true);
        cx.on_action(on_quit);

        // Open main window
        let size = size(px(1200.), px(800.));
        let bounds = Bounds::centered(None, size, cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Gas Town".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| cx.new_view(|_window, _cx| GasTown),
        )
        .expect("Failed to open window");
    });

    Ok(())
}
