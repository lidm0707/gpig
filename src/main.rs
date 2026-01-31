use dotenv::dotenv;
use gpig::actions::{OpenFile, Quit};
use gpig::garph::Garph;
use gpig::workspace::Workspace;
use gpui::{App, AppContext, Application, KeyBinding, WindowOptions};
use rfd::FileDialog;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let garph_value = Garph::new(None);

    Application::new().run(move |cx: &mut App| {
        let garph = cx.new(|_| garph_value.clone());

        cx.bind_keys([KeyBinding::new("ctrl-q", Quit, None)]);
        cx.on_action(|_action: &Quit, cx: &mut App| {
            println!("Quit action received");
            cx.quit();
        });

        let garph_for_action = garph.clone();
        cx.on_action(move |_action: &OpenFile, cx: &mut App| {
            println!("OpenFile action handler triggered!");
            let garph_clone = garph_for_action.clone();
            if let Some(path) = FileDialog::new().pick_folder() {
                let path_str = format!("{}", path.display());
                println!("OpenFile action received: {}", path_str);
                cx.update_entity(&garph_clone, |garph, _cx| {
                    if let Err(e) = garph.update_repo(&path_str) {
                        eprintln!("Failed to update repo: {}", e);
                    }
                });
            }
        });

        cx.open_window(
            WindowOptions {
                // window_bounds: Some(WindowBounds::Windowed(bounds)),
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
