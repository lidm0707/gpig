use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use git2::Repository;
use gpui::prelude::*;
use gpui::{
    AnyElement, Context, EventEmitter, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

#[derive(Clone, Debug)]
pub struct StatusEntry {
    pub path: String,
    pub staged: bool,
    pub status_kind: StatusKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    New,
    Modified,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

#[derive(Clone, Debug)]
pub struct StatusUpdated;

struct StatusReloadResult {
    entries: Vec<StatusEntry>,
}

pub struct StatusPanel {
    repo_path: Option<String>,
    entries: Vec<StatusEntry>,
    pending_reload_rx: Option<Receiver<Result<StatusReloadResult, String>>>,
    loading: bool,
}

impl EventEmitter<StatusUpdated> for StatusPanel {}

const COLOR_LOADING_TEXT: u32 = 0x888888;

impl StatusPanel {
    pub fn new(_repo: Rc<RefCell<Option<Repository>>>) -> Self {
        Self {
            repo_path: None,
            entries: Vec::new(),
            pending_reload_rx: None,
            loading: false,
        }
    }

    pub fn set_repo_path(&mut self, path: String) {
        self.repo_path = Some(path);
    }

    pub fn reload(&mut self) {
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        self.entries.clear();
        self.loading = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.pending_reload_rx = Some(rx);

        std::thread::spawn(move || {
            let result = load_status_bg(&repo_path);
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
                self.entries = result.entries;
                cx.notify();
            }
            Ok(Err(msg)) => {
                self.pending_reload_rx = None;
                self.loading = false;
                eprintln!("status reload failed: {}", msg);
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

    pub fn dirty_count(&self) -> usize {
        self.entries.len()
    }
}

fn load_status_bg(repo_path: &str) -> Result<StatusReloadResult, String> {
    let repo = Repository::open(repo_path).map_err(|e| e.to_string())?;
    let statuses = repo.statuses(None).map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("?").to_string();
        let s = entry.status();

        let (staged, kind) = if s.is_conflicted() {
            (false, StatusKind::Conflicted)
        } else if s.is_index_new() {
            (true, StatusKind::New)
        } else if s.is_index_modified() {
            (true, StatusKind::Modified)
        } else if s.is_index_deleted() {
            (true, StatusKind::Deleted)
        } else if s.is_index_renamed() {
            (true, StatusKind::Renamed)
        } else if s.is_wt_new() {
            (false, StatusKind::Untracked)
        } else if s.is_wt_modified() {
            (false, StatusKind::Modified)
        } else if s.is_wt_deleted() {
            (false, StatusKind::Deleted)
        } else {
            continue;
        };

        entries.push(StatusEntry {
            path,
            staged,
            status_kind: kind,
        });
    }

    Ok(StatusReloadResult { entries })
}

impl Render for StatusPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_reload(cx);
        self.render_panel()
    }
}

impl StatusPanel {
    fn status_label(kind: StatusKind) -> (char, u32) {
        match kind {
            StatusKind::New => ('A', 0x2ECC71),
            StatusKind::Modified => ('M', 0xF39C12),
            StatusKind::Deleted => ('D', 0xE74C3C),
            StatusKind::Renamed => ('R', 0x3498DB),
            StatusKind::Untracked => ('?', 0x888888),
            StatusKind::Conflicted => ('C', 0xFF0000),
        }
    }

    fn render_panel(&self) -> AnyElement {
        let has_repo = self.repo_path.is_some();

        if !has_repo {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .bg(gpui::rgb(0x1E1E1E))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size_full()
                        .text_color(gpui::rgb(0x666666))
                        .text_size(px(12.0))
                        .child("No repo"),
                )
                .into_any();
        }

        if self.loading {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .bg(gpui::rgb(0x1E1E1E))
                .child(
                    div()
                        .w_full()
                        .px(px(10.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_color(gpui::rgb(0x333333))
                        .bg(gpui::rgb(0x252525))
                        .text_color(gpui::rgb(0xCCCCCC))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_size(px(12.0))
                        .child("Changes"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py(px(12.0))
                        .text_color(gpui::rgb(COLOR_LOADING_TEXT))
                        .text_size(px(11.0))
                        .font_family("monospace")
                        .child("Loading status..."),
                )
                .into_any();
        }

        let staged: Vec<&StatusEntry> = self.entries.iter().filter(|e| e.staged).collect();
        let unstaged: Vec<&StatusEntry> = self.entries.iter().filter(|e| !e.staged).collect();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(gpui::rgb(0x1E1E1E))
            .child(
                div()
                    .w_full()
                    .px(px(10.0))
                    .py(px(6.0))
                    .border_b_1()
                    .border_color(gpui::rgb(0x333333))
                    .bg(gpui::rgb(0x252525))
                    .text_color(gpui::rgb(0xCCCCCC))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_size(px(12.0))
                    .child(format!(
                        "Changes ({} staged, {} unstaged)",
                        staged.len(),
                        unstaged.len()
                    )),
            )
            .child(
                div()
                    .id("status_list")
                    .flex_1()
                    .overflow_y_scroll()
                    .children(staged.iter().map(|entry| {
                        let (label, color) = Self::status_label(entry.status_kind);
                        Self::render_entry(&entry.path, label, color, true)
                    }))
                    .children(unstaged.iter().map(|entry| {
                        let (label, color) = Self::status_label(entry.status_kind);
                        Self::render_entry(&entry.path, label, color, false)
                    }))
                    .when(self.entries.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .py(px(12.0))
                                .text_color(gpui::rgb(0x666666))
                                .text_size(px(12.0))
                                .child("Working tree clean"),
                        )
                    }),
            )
            .into_any()
    }

    fn render_entry(path: &str, label: char, color: u32, staged: bool) -> impl IntoElement {
        let section_color = if staged {
            gpui::rgb(0x3A3A2A)
        } else {
            gpui::rgb(0x1E1E1E)
        };

        div()
            .id(SharedString::from(format!(
                "{}-{}",
                if staged { "s" } else { "u" },
                path
            )))
            .w_full()
            .px(px(10.0))
            .py(px(3.0))
            .bg(section_color)
            .hover(|s| s.bg(gpui::rgb(0x2A2A2A)))
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                div()
                    .w(px(20.0))
                    .text_color(gpui::rgb(color))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_size(px(11.0))
                    .font_family("monospace")
                    .child(label.to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .text_color(gpui::rgb(0xCCCCCC))
                    .text_size(px(11.0))
                    .font_family("monospace")
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(path.to_string()),
            )
    }
}
