use std::cell::RefCell;
use std::rc::Rc;

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

pub struct StatusPanel {
    repo: Rc<RefCell<Option<Repository>>>,
    entries: Vec<StatusEntry>,
}

impl EventEmitter<StatusUpdated> for StatusPanel {}

impl StatusPanel {
    pub fn new(repo: Rc<RefCell<Option<Repository>>>) -> Self {
        let mut panel = Self {
            repo,
            entries: Vec::new(),
        };
        panel.reload();
        panel
    }

    pub fn reload(&mut self) {
        self.entries.clear();
        let repo = self.repo.borrow();
        let Some(repo) = repo.as_ref() else {
            return;
        };

        let Ok(statuses) = repo.statuses(None) else {
            return;
        };

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

            self.entries.push(StatusEntry {
                path,
                staged,
                status_kind: kind,
            });
        }
    }

    pub fn dirty_count(&self) -> usize {
        self.entries.len()
    }
}

impl Render for StatusPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
        let has_repo = self.repo.borrow().is_some();

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
