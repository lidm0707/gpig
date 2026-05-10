use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use git2::Repository;
use gpui::prelude::*;
use gpui::{
    AnyElement, Context, EventEmitter, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::path_bar::RepoMode;

const COLOR_HEADING_BG: u32 = 0x252525;
const COLOR_BORDER: u32 = 0x333333;
const COLOR_BG: u32 = 0x1E1E1E;
const COLOR_TEXT: u32 = 0xCCCCCC;
const COLOR_HEAD_BG: u32 = 0x2A3A2A;
const COLOR_HEAD_TEXT: u32 = 0x4AE04A;
const COLOR_HOVER_BG: u32 = 0x2A2A2A;
const COLOR_NO_REPO: u32 = 0x666666;
const COLOR_REMOTE_TEXT: u32 = 0x4A90D9;
const COLOR_CHECKING_TEXT: u32 = 0xF39C12;

#[derive(Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
}

#[derive(Clone, Debug)]
pub struct BranchCheckedOut {
    pub name: String,
}

struct CheckoutResult {
    local_name: String,
}

pub struct BranchPanel {
    repo: Rc<RefCell<Option<Repository>>>,
    repo_path: Option<String>,
    branches: Vec<BranchInfo>,
    mode: RepoMode,
    pending_checkout_rx: Option<Receiver<Result<CheckoutResult, String>>>,
    checking_out: Option<String>,
}

impl EventEmitter<BranchCheckedOut> for BranchPanel {}

impl BranchPanel {
    pub fn new(repo: Rc<RefCell<Option<Repository>>>) -> Self {
        let repo_path = repo.borrow().as_ref().and_then(|r| {
            r.path()
                .parent()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
        });
        let mut panel = Self {
            repo_path,
            repo,
            branches: Vec::new(),
            mode: RepoMode::Local,
            pending_checkout_rx: None,
            checking_out: None,
        };
        panel.reload();
        panel
    }

    pub fn set_mode(&mut self, mode: RepoMode, cx: &mut Context<Self>) {
        self.mode = mode;
        self.reload();
        cx.notify();
    }

    pub fn set_repo_path(&mut self, path: String) {
        self.repo_path = Some(path);
    }

    pub fn reload(&mut self) {
        self.branches.clear();
        let repo = self.repo.borrow();
        let Some(repo) = repo.as_ref() else {
            return;
        };

        let branch_type = match self.mode {
            RepoMode::Local => git2::BranchType::Local,
            RepoMode::Remote => git2::BranchType::Remote,
        };

        let head_name = repo
            .head()
            .ok()
            .and_then(|r| r.shorthand().map(|s| s.to_string()));

        let Ok(branches) = repo.branches(Some(branch_type)) else {
            return;
        };

        for branch in branches.flatten() {
            let (b, _) = branch;
            let name = b.name().ok().flatten().unwrap_or("").to_string();
            let is_remote = matches!(self.mode, RepoMode::Remote);
            let is_head = !is_remote && head_name.as_ref() == Some(&name);
            self.branches.push(BranchInfo {
                name,
                is_head,
                is_remote,
            });
        }
    }

    pub fn checkout(&mut self, name: &str, cx: &mut Context<Self>) {
        if self.checking_out.is_some() {
            return;
        }

        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };

        let is_remote = self.branches.first().map(|b| b.is_remote).unwrap_or(false);

        let branch_name = name.to_string();
        let local_name = if branch_name.contains('/') {
            branch_name
                .split('/')
                .nth(1)
                .unwrap_or(&branch_name)
                .to_string()
        } else {
            branch_name.clone()
        };

        let (tx, rx) = std::sync::mpsc::channel();
        self.pending_checkout_rx = Some(rx);
        self.checking_out = Some(branch_name.clone());

        std::thread::spawn(move || {
            let result = if is_remote {
                do_checkout_remote_bg(&repo_path, &branch_name)
            } else {
                do_checkout_bg(&repo_path, &branch_name)
            };
            let _ = tx.send(result.map(|_| CheckoutResult { local_name }));
        });

        cx.notify();
    }

    fn poll_checkout(&mut self, cx: &mut Context<Self>) {
        let Some(rx) = &self.pending_checkout_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(result)) => {
                self.pending_checkout_rx = None;
                self.checking_out = None;
                self.reload();
                cx.emit(BranchCheckedOut {
                    name: result.local_name,
                });
                cx.notify();
            }
            Ok(Err(msg)) => {
                self.pending_checkout_rx = None;
                self.checking_out = None;
                eprintln!("checkout failed: {}", msg);
                self.reload();
                cx.notify();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                cx.notify();
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.pending_checkout_rx = None;
                self.checking_out = None;
                cx.notify();
            }
        }
    }

    pub fn current_branch(&self) -> Option<&str> {
        self.branches
            .iter()
            .find(|b| b.is_head)
            .map(|b| b.name.as_str())
    }

    pub fn branches(&self) -> &[BranchInfo] {
        &self.branches
    }
}

fn do_checkout_bg(repo_path: &str, name: &str) -> Result<(), String> {
    let repo = Repository::open(repo_path).map_err(|e| e.to_string())?;
    let refname = format!("refs/heads/{}", name);
    repo.set_head(&refname).map_err(|e| e.to_string())?;
    repo.checkout_head(None).map_err(|e| e.to_string())?;
    Ok(())
}

fn do_checkout_remote_bg(repo_path: &str, remote_name: &str) -> Result<(), String> {
    let repo = Repository::open(repo_path).map_err(|e| e.to_string())?;
    let local_name = remote_name
        .split('/')
        .nth(1)
        .ok_or_else(|| "Invalid remote branch name".to_string())?;
    let refname = format!("refs/remotes/{}", remote_name);
    let commit = repo.revparse_single(&refname).map_err(|e| e.to_string())?;
    let refname_local = format!("refs/heads/{}", local_name);
    repo.reference(&refname_local, commit.id(), true, "checkout from remote")
        .map_err(|e| e.to_string())?;
    repo.set_head(&refname_local).map_err(|e| e.to_string())?;
    repo.checkout_head(None).map_err(|e| e.to_string())?;
    Ok(())
}

impl Render for BranchPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_checkout(cx);
        self.render_panel(cx)
    }
}

impl BranchPanel {
    fn render_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let has_repo = self.repo.borrow().is_some();

        if !has_repo {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .bg(gpui::rgb(COLOR_BG))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size_full()
                        .text_color(gpui::rgb(COLOR_NO_REPO))
                        .text_size(px(12.0))
                        .child("No repo"),
                )
                .into_any();
        }

        let heading = match self.mode {
            RepoMode::Local => "Branches",
            RepoMode::Remote => "Remotes",
        };

        let checking = self.checking_out.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(gpui::rgb(COLOR_BG))
            .child(
                div()
                    .w_full()
                    .px(px(10.0))
                    .py(px(6.0))
                    .border_b_1()
                    .border_color(gpui::rgb(COLOR_BORDER))
                    .bg(gpui::rgb(COLOR_HEADING_BG))
                    .text_color(gpui::rgb(COLOR_TEXT))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_size(px(12.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(heading)
                    .when_some(checking, |el, name| {
                        el.child(
                            div()
                                .text_color(gpui::rgb(COLOR_CHECKING_TEXT))
                                .text_size(px(9.0))
                                .font_family("monospace")
                                .child(format!("⏳ {}", name)),
                        )
                    }),
            )
            .child(
                div()
                    .id("branch_list")
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.branches.iter().map(|b| {
                        let name = b.name.clone();
                        let is_head = b.is_head;
                        let is_remote = b.is_remote;
                        let is_busy = self.checking_out.is_some();

                        let bg = if is_head {
                            gpui::rgb(COLOR_HEAD_BG)
                        } else {
                            gpui::rgb(COLOR_BG)
                        };
                        let text_color = if is_head {
                            gpui::rgb(COLOR_HEAD_TEXT)
                        } else if is_remote {
                            gpui::rgb(COLOR_REMOTE_TEXT)
                        } else {
                            gpui::rgb(COLOR_TEXT)
                        };

                        let label = if is_head {
                            format!("* {}", name)
                        } else {
                            format!("  {}", name)
                        };

                        div()
                            .id(SharedString::from(name.clone()))
                            .w_full()
                            .px(px(10.0))
                            .py(px(4.0))
                            .bg(bg)
                            .hover(|s| s.bg(gpui::rgb(COLOR_HOVER_BG)))
                            .cursor_pointer()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_color(text_color)
                                    .text_size(px(12.0))
                                    .font_family("monospace")
                                    .child(label),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _event, _window, cx| {
                                    if !is_head && !is_busy {
                                        this.checkout(&name, cx);
                                    }
                                }),
                            )
                    })),
            )
            .into_any()
    }
}
