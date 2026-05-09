use gpui::prelude::*;
use gpui::{
    Context, EventEmitter, InteractiveElement, IntoElement, MouseButton, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use std::sync::mpsc::Receiver;

use crate::repo_scanner::{scan_git_repos, short_name};

const DROPDOWN_MAX_VISIBLE: usize = 12;
const ITEM_HEIGHT: f32 = 24.0;
const DROPDOWN_WIDTH: f32 = 400.0;

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
            eprintln!("[repo_picker] scan thread started");
            let repos = scan_git_repos();
            eprintln!(
                "[repo_picker] scan thread done, sending {} repos",
                repos.len()
            );
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

    pub fn set_selected(&mut self, path: Option<String>) {
        self.selected = path;
    }

    pub fn poll_scan(&mut self, cx: &mut Context<Self>) {
        let Some(rx) = &self.pending_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(repos) => {
                eprintln!("[repo_picker] poll got {} repos", repos.len());
                self.repos = repos;
                self.scanning = false;
                self.pending_rx = None;
                cx.notify();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                eprintln!("[repo_picker] poll empty, will retry");
                cx.notify();
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                eprintln!("[repo_picker] poll disconnected");
                self.scanning = false;
                self.pending_rx = None;
            }
        }
    }

    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }

    fn select(&mut self, path: String, cx: &mut Context<Self>) {
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
}

impl Render for RepoPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_scan(cx);

        let label = match &self.selected {
            Some(p) => short_name(p).to_string(),
            None if self.scanning => "Scanning...".to_string(),
            None => "Select repo".to_string(),
        };

        let arrow = if self.is_open { "▲" } else { "▼" };

        let repos = self.repos.clone();
        let is_open = self.is_open;
        let scanning = self.scanning;

        let dropdown_height = if repos.is_empty() {
            ITEM_HEIGHT
        } else {
            (repos.len().min(DROPDOWN_MAX_VISIBLE) as f32) * ITEM_HEIGHT
        };

        div()
            .relative()
            .child(
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
                    .border_color(if is_open {
                        gpui::rgb(0x4A90D9)
                    } else {
                        gpui::rgb(0x444444)
                    })
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(gpui::rgb(0x222244)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _ev, _win, cx| {
                            this.toggle(cx);
                            cx.stop_propagation();
                        }),
                    )
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
                    ),
            )
            .when(is_open, move |el| {
                el.child(
                    div()
                        .id("repo_dropdown")
                        .absolute()
                        .top(px(30.0))
                        .left(px(0.0))
                        .w(px(DROPDOWN_WIDTH))
                        .max_h(px(dropdown_height + 4.0))
                        .overflow_y_scroll()
                        .bg(gpui::rgb(0x1a1a1a))
                        .border_1()
                        .border_color(gpui::rgb(0x444444))
                        .rounded(px(4.0))
                        .shadow_lg()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|_this, _ev, _win, cx| {
                                cx.stop_propagation();
                            }),
                        )
                        .when(repos.is_empty(), |el| {
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
                        .children(repos.into_iter().enumerate().map(|(i, repo)| {
                            let short = short_name(&repo).to_string();
                            let repo_clone = repo.clone();
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
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _ev, _win, cx| {
                                        this.select(repo_clone.clone(), cx);
                                    }),
                                )
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
                        })),
                )
            })
    }
}
