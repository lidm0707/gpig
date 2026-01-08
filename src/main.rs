use dark_pig_git::garph::Garph;

use dark_pig_git::workspace::Workspace;
use dotenv::dotenv;
use gpui::{App, AppContext, Application, WindowOptions};
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let path_repo = env::var("GIT_REPO_PATH")?;
    let repo = git2::Repository::open(&path_repo)?;
    let garph = Garph::new(repo);
    Application::new().run(|cx: &mut App| {
        // let bounds = Bounds::centered(None, size(px(1800.), px(800.0)), cx);

        cx.open_window(
            WindowOptions {
                // window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                let garph = cx.new(|_| garph);
                cx.new(|_| Workspace::new(Some(garph)))
            },
        )
        .unwrap();
    });
    Ok(())
}
