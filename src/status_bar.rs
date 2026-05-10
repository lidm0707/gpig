use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use git2::Repository;
use gpui::prelude::*;
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};

struct StatusBarReloadResult {
    branch_name: String,
    commit_count: usize,
    dirty_count: usize,
}

pub struct StatusBar {
    repo_path: Option<String>,
    branch_name: String,
    commit_count: usize,
    dirty_count: usize,
    pending_reload_rx: Option<Receiver<Result<StatusBarReloadResult, String>>>,
    loading: bool,
}

impl StatusBar {
    pub fn new(_repo: Rc<RefCell<Option<Repository>>>) -> Self {
        Self {
            repo_path: None,
            branch_name: String::new(),
            commit_count: 0,
            dirty_count: 0,
            pending_reload_rx: None,
            loading: false,
        }
    }

    pub fn set_repo_path(&mut self, path: String) {
        self.repo_path = Some(path);
    }

    pub fn refresh(&mut self) {
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        self.loading = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.pending_reload_rx = Some(rx);

        std::thread::spawn(move || {
            let result = load_status_bar_bg(&repo_path);
            let _ = tx.send(result);
        });
    }

    fn poll_reload(&mut self, cx: &mut Context<Self>) {
        let Some(rx) = &self.pending_reload_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(result)) => {
                self.pending_reload_rx = None;
                self.loading = false;
                self.branch_name = result.branch_name;
                self.commit_count = result.commit_count;
                self.dirty_count = result.dirty_count;
                cx.notify();
            }
            Ok(Err(msg)) => {
                self.pending_reload_rx = None;
                self.loading = false;
                eprintln!("status bar reload failed: {}", msg);
                cx.notify();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                cx.notify();
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.pending_reload_rx = None;
                self.loading = false;
                cx.notify();
            }
        }
    }

    pub fn set_dirty_count(&mut self, count: usize) {
        self.dirty_count = count;
    }
}

fn load_status_bar_bg(repo_path: &str) -> Result<StatusBarReloadResult, String> {
    let repo = Repository::open(repo_path).map_err(|e| e.to_string())?;

    let branch_name = repo
        .head()
        .ok()
        .and_then(|r| r.shorthand().map(|s| s.to_string()))
        .unwrap_or_default();

    let commit_count = repo
        .revwalk()
        .and_then(|mut rw| {
            rw.push_head()?;
            Ok(rw.count())
        })
        .unwrap_or(0);

    let dirty_count = repo.statuses(None).map(|s| s.iter().count()).unwrap_or(0);

    Ok(StatusBarReloadResult {
        branch_name,
        commit_count,
        dirty_count,
    })
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_reload(cx);

        let branch = if self.branch_name.is_empty() {
            if self.loading {
                "loading...".to_string()
            } else {
                "no repo".to_string()
            }
        } else {
            self.branch_name.clone()
        };

        div()
            .w_full()
            .h(px(24.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .bg(gpui::rgb(0x1A1A2E))
            .border_t_1()
            .border_color(gpui::rgb(0x333333))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_color(gpui::rgb(0x4AE04A))
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .font_family("monospace")
                            .child(format!(" {}", branch)),
                    )
                    .child(
                        div()
                            .text_color(gpui::rgb(0x888888))
                            .text_size(px(11.0))
                            .font_family("monospace")
                            .child(format!("{} commits", self.commit_count)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .when(self.dirty_count > 0, |el| {
                        el.child(
                            div()
                                .text_color(gpui::rgb(0xF39C12))
                                .text_size(px(11.0))
                                .font_family("monospace")
                                .child(format!("{} dirty", self.dirty_count)),
                        )
                    })
                    .when(
                        self.dirty_count == 0 && !self.branch_name.is_empty(),
                        |el| {
                            el.child(
                                div()
                                    .text_color(gpui::rgb(0x4AE04A))
                                    .text_size(px(11.0))
                                    .font_family("monospace")
                                    .child("clean"),
                            )
                        },
                    ),
            )
    }
}
