use gpui::{App, AppContext};

mod models;
mod app;
mod jenkins_client;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(Default::default(), |cx| {
            cx.new_view(|cx| app::CicdClientApp::new(cx))
        });
    });
}
