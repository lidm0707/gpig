use gpui::{App, AppContext, Application, Entity, QuitMode, WindowOptions};
use session::{AppSession, Session};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use workspace::AppState;

use client::user::UserStore;
use db::AppDatabase;
use db::kvp::KeyValueStore;
use editor::Editor;

use language::LanguageRegistry;
use node_runtime::NodeRuntime;
use settings;
use theme::{self, LoadThemes};
use theme_settings;
use workspace::WorkspaceStore;

use reqwest::ReqwestClient;
use watch;

fn main() {
    if let Err(e) = run_app() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_app() -> Result<(), Box<dyn Error>> {
    Application::with_platform(gpui_platform::current_platform(false))
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx: &mut App| {
            // Initialize AppDatabase
            let app_db = AppDatabase::new();

            // Initialize settings properly using settings::init(cx)
            settings::init(cx);

            // Initialize theme system with base theme only
            theme::init(LoadThemes::JustBase, cx);

            // Initialize theme settings provider for UI spacing calculations
            theme_settings::init(LoadThemes::JustBase, cx);

            // Create Session and AppSession following Zed's pattern
            // We need to use app_db before moving it to global
            let session_id = Uuid::new_v4().to_string();
            let session = cx.background_executor().spawn(Session::new(
                session_id.clone(),
                KeyValueStore::from_app_db(&app_db),
            ));

            // Now move app_db to global after we're done using it
            cx.set_global(app_db);
            let session = cx.foreground_executor().block_on(session);
            let app_session = cx.new(|cx| AppSession::new(session, cx));

            let app_state = build_app_state(cx, app_session);
            AppState::set_global(app_state.clone(), cx);

            cx.activate(true);
            workspace::init(app_state.clone(), cx);

            // Open a new workspace with a blank file editor
            workspace::open_new(
                Default::default(),
                app_state.clone(),
                cx,
                |workspace, window, cx| {
                    Editor::new_file(workspace, &Default::default(), window, cx);
                },
            )
            .detach();
        });

    Ok(())
}

fn build_app_state(cx: &mut App, session: Entity<AppSession>) -> Arc<AppState> {
    let http = ReqwestClient::new();
    cx.set_http_client(Arc::new(http));

    let client = client::Client::production(cx);
    let languages = Arc::new(LanguageRegistry::new(cx.background_executor().clone()));
    let user_store = cx.new(|cx| UserStore::new(client.clone(), cx));
    let workspace_store = cx.new(|cx| WorkspaceStore::new(client.clone(), cx));
    let fs = Arc::new(fs::RealFs::new(None, cx.background_executor().clone()));
    let build_window_options = |_uuid: Option<uuid::Uuid>, _cx: &mut App| WindowOptions::default();
    let (_node_binary_options_tx, node_binary_options_rx) = watch::channel(None);
    let node_runtime = NodeRuntime::new(client.http_client(), None, node_binary_options_rx);

    Arc::new(AppState {
        languages,
        client,
        user_store,
        fs,
        build_window_options,
        workspace_store,
        node_runtime,
        session,
    })
}
