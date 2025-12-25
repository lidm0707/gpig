use dark_pig_git::entities::garph::Garph;
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
            |_, cx| cx.new(|_| garph),
        )
        .unwrap();
    });
    Ok(())
}
