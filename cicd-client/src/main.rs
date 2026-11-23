use gpui::{App, AppContext, Bounds, Window, WindowBounds, WindowOptions, size, px};
use gpui_platform::application;

mod models;
mod app;
mod jenkins_client;

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(980.0), px(780.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| app::CicdClientApp::new(cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
