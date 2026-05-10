use dotenv::dotenv;
use gpig::actions::Quit;
use gpig::garph::Garph;
use gpig::text_input::{
    Backspace, Cut, Delete, End, Home, Left, Paste, Right, SelectAll, SelectLeft, SelectRight,
    ShowCharacterPalette,
};
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

            cx.bind_keys([
                KeyBinding::new("ctrl-q", Quit, None),
                KeyBinding::new("backspace", Backspace, Some("TextInput")),
                KeyBinding::new("delete", Delete, Some("TextInput")),
                KeyBinding::new("left", Left, Some("TextInput")),
                KeyBinding::new("right", Right, Some("TextInput")),
                KeyBinding::new("shift-left", SelectLeft, Some("TextInput")),
                KeyBinding::new("shift-right", SelectRight, Some("TextInput")),
                KeyBinding::new("cmd-a", SelectAll, Some("TextInput")),
                KeyBinding::new("home", Home, Some("TextInput")),
                KeyBinding::new("end", End, Some("TextInput")),
                KeyBinding::new("cmd-v", Paste, Some("TextInput")),
                KeyBinding::new("cmd-c", gpig::text_input::Copy, Some("TextInput")),
                KeyBinding::new("cmd-x", Cut, Some("TextInput")),
                KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, Some("TextInput")),
            ]);
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
