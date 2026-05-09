use gpui::prelude::*;
use gpui::{
    Context, Entity, EventEmitter, InteractiveElement, IntoElement, MouseButton, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px,
};

use std::sync::mpsc::Receiver;

use crate::repo_scanner::{scan_git_repos, short_name};

const MAX_VISIBLE: usize = 12;
const ITEM_H: f32 = 24.0;
const DROPDOWN_W: f32 = 400.0;
const DROPDOWN_TOP: f32 = 72.0;

#[derive(Clone, Debug)]
pub struct RepoSelected {
    pub path: String,
}

pub struct RepoPicker {
    repos: Vec<String>,
    selected: Option<String>,
    is_open: bool,
    scanning: bool,
    pending_rx: Option<Receiver<Vec<String>>>,
}

impl EventEmitter<RepoSelected> for RepoPicker {}

impl RepoPicker {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let repos = scan_git_repos();
            let _ = tx.send(repos);
        });

        let _ = cx.on_release(|this: &mut RepoPicker, _cx| {
            this.pending_rx = None;
        });

        Self {
            repos: Vec::new(),
            selected: None,
            is_open: false,
            scanning: true,
            pending_rx: Some(rx),
        }
    }

    pub fn repos(&self) -> &[String] {
        &self.repos
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn is_scanning(&self) -> bool {
        self.scanning
    }

    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }

    pub fn select(&mut self, path: String, cx: &mut Context<Self>) {
        self.selected = Some(path.clone());
        self.is_open = false;
        cx.emit(RepoSelected { path });
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        if self.is_open {
            self.is_open = false;
            cx.notify();
        }
    }

    pub fn poll_scan(&mut self, cx: &mut Context<Self>) {
        let Some(rx) = &self.pending_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(repos) => {
                self.repos = repos;
                self.scanning = false;
                self.pending_rx = None;
                cx.notify();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                cx.notify();
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.scanning = false;
                self.pending_rx = None;
            }
        }
    }
}

pub fn render_button(
    picker: &Entity<RepoPicker>,
    cx: &mut Context<crate::path_bar::PathBar>,
) -> impl IntoElement {
    let picker_clone = picker.clone();
    let picker_read = picker.read(cx);
    let label = match &picker_read.selected {
        Some(p) => short_name(p).to_string(),
        None if picker_read.scanning => "Scanning...".to_string(),
        None => "Select repo".to_string(),
    };
    let arrow = if picker_read.is_open { "▲" } else { "▼" };

    div()
        .id("repo_picker_btn")
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .px(px(8.0))
        .h(px(26.0))
        .min_w(px(160.0))
        .bg(gpui::rgb(0x1A1A2E))
        .border_1()
        .border_color(if picker_read.is_open {
            gpui::rgb(0x4A90D9)
        } else {
            gpui::rgb(0x444444)
        })
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(gpui::rgb(0x222244)))
        .on_mouse_down(MouseButton::Left, move |_ev, _win, cx: &mut gpui::App| {
            picker_clone.update(cx, |p: &mut RepoPicker, cx| p.toggle(cx));
            cx.stop_propagation();
        })
        .child(
            div()
                .text_color(gpui::rgb(0xCCCCCC))
                .text_size(px(11.0))
                .font_family("monospace")
                .child(label),
        )
        .child(
            div()
                .ml_auto()
                .text_color(gpui::rgb(0x888888))
                .text_size(px(9.0))
                .child(arrow),
        )
}

pub fn render_dropdown(
    picker: &Entity<RepoPicker>,
    cx: &mut Context<crate::workspace::Workspace>,
) -> Option<impl IntoElement> {
    let p = picker.read(cx);
    if !p.is_open {
        return None;
    }

    let repos = p.repos.clone();
    let scanning = p.scanning;
    let dropdown_h = if repos.is_empty() {
        ITEM_H
    } else {
        (repos.len().min(MAX_VISIBLE) as f32) * ITEM_H
    };

    let picker_clone = picker.clone();

    let el = div()
        .id("repo_dropdown_overlay")
        .absolute()
        .top(px(DROPDOWN_TOP))
        .left(px(40.0))
        .w(px(DROPDOWN_W))
        .max_h(px(dropdown_h + 4.0))
        .overflow_y_scroll()
        .bg(gpui::rgb(0x1a1a1a))
        .border_1()
        .border_color(gpui::rgb(0x444444))
        .rounded(px(4.0))
        .shadow_lg()
        .on_mouse_down(MouseButton::Left, move |_ev, _win, cx: &mut gpui::App| {
            cx.stop_propagation();
        })
        .when(repos.is_empty(), move |el| {
            let msg = if scanning {
                "Scanning..."
            } else {
                "No repos found"
            };
            el.child(
                div()
                    .px(px(10.0))
                    .py(px(6.0))
                    .text_color(gpui::rgb(0x666666))
                    .text_size(px(11.0))
                    .child(msg),
            )
        })
        .children(
            repos
                .into_iter()
                .enumerate()
                .map(|(i, repo): (usize, String)| {
                    let short = short_name(&repo).to_string();
                    let repo_clone = repo.clone();
                    let pc = picker_clone.clone();
                    div()
                        .id(SharedString::from(format!("repo_item_{}", i)))
                        .px(px(10.0))
                        .py(px(3.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .cursor_pointer()
                        .hover(|s| s.bg(gpui::rgb(0x333333)))
                        .on_mouse_down(MouseButton::Left, move |_ev, _win, cx: &mut gpui::App| {
                            pc.update(cx, |p: &mut RepoPicker, cx| {
                                p.select(repo_clone.clone(), cx)
                            });
                        })
                        .child(
                            div()
                                .text_color(gpui::rgb(0x4AE04A))
                                .text_size(px(11.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .child(short),
                        )
                        .child(
                            div()
                                .text_color(gpui::rgb(0x666666))
                                .text_size(px(9.0))
                                .font_family("monospace")
                                .overflow_hidden()
                                .child(repo),
                        )
                }),
        );

    Some(el)
}
