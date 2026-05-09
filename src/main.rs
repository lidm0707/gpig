use dotenv::dotenv;
use gpig::actions::Quit;
use gpig::garph::Garph;
use gpig::workspace::Workspace;
use gpui::{App, AppContext, Application, KeyBinding, QuitMode, WindowOptions};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let garph_value = Garph::new(None);

    Application::with_platform(gpui_platform::current_platform(false))
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx: &mut App| {
            let garph = cx.new(|_| garph_value.clone());

            cx.bind_keys([KeyBinding::new("ctrl-q", Quit, None)]);
            cx.on_action(|_action: &Quit, cx: &mut App| {
                cx.quit();
            });

            cx.open_window(
                WindowOptions {
                    ..Default::default()
                },
                move |_, cx: &mut App| {
                    let garph = garph.clone();
                    cx.new(|cx| Workspace::new(Some(garph), cx))
                },
            )
            .unwrap();
        });
    Ok(())
}
