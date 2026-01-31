use dotenv::dotenv;
use gpig::actions::{OpenFile, Quit};
use gpig::garph::Garph;
use gpig::workspace::Workspace;
use gpui::{App, AppContext, Application, KeyBinding, WindowOptions};
use rfd::FileDialog;
use std::env;
use std::error::Error; // 👈 มาจาก lib.rs

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let path_repo = env::var("GIT_REPO_PATH")?;
    let repo = git2::Repository::open(&path_repo)?;
    let garph = Garph::new(repo);

    Application::new().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("ctrl-q", Quit, None)]);
        cx.on_action(|_action: &Quit, cx: &mut App| {
            println!("Quit action received");
            cx.quit();
        });

        cx.on_action(|_action: &OpenFile, _cx: &mut App| {
            println!("OpenFile action handler triggered!");
            if let Some(path) = FileDialog::new().pick_file() {
                println!("OpenFile action received: {}", path.display());
                // TODO: Actually open the file here
            }
        });
        cx.open_window(
            WindowOptions {
                // window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx: &mut App| {
                let garph = cx.new(|_| garph);

                cx.new(|cx| Workspace::new(Some(garph), cx))
            },
        )
        .unwrap();
    });
    Ok(())
}
