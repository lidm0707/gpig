use gpui::prelude::*;
use gpui::{
    Context, EventEmitter, InteractiveElement, IntoElement, MouseButton, ParentElement,
    SharedString, Styled, Window, div, px,
};

#[derive(Clone, Debug)]
pub struct RepoPathSubmitted {
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct SearchPathSubmitted {
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct SearchPathCleared;

pub struct PathBar {
    repo_input: String,
    search_input: String,
    error_msg: Option<String>,
}

impl EventEmitter<RepoPathSubmitted> for PathBar {}
impl EventEmitter<SearchPathSubmitted> for PathBar {}
impl EventEmitter<SearchPathCleared> for PathBar {}

impl Default for PathBar {
    fn default() -> Self {
        Self::new()
    }
}

impl PathBar {
    pub fn new() -> Self {
        Self {
            repo_input: String::new(),
            search_input: String::new(),
            error_msg: None,
        }
    }

    pub fn set_error(&mut self, msg: Option<String>) {
        self.error_msg = msg;
    }

    pub fn clear_search(&mut self) {
        self.search_input.clear();
    }

    pub fn set_repo_input(&mut self, path: String) {
        self.repo_input = path;
    }

    fn submit_repo(&mut self, cx: &mut Context<Self>) {
        let path = self.repo_input.trim().to_string();
        if path.is_empty() {
            return;
        }
        self.error_msg = None;
        cx.emit(RepoPathSubmitted { path });
        cx.notify();
    }

    fn submit_search(&mut self, cx: &mut Context<Self>) {
        let path = self.search_input.trim().to_string();
        if path.is_empty() {
            cx.emit(SearchPathCleared);
            return;
        }
        cx.emit(SearchPathSubmitted { path });
        cx.notify();
    }

    fn emit_clear(&mut self, cx: &mut Context<Self>) {
        self.search_input.clear();
        cx.emit(SearchPathCleared);
        cx.notify();
    }

    fn render_input(
        &self,
        id: &str,
        value: &str,
        placeholder: &str,
        border_color: u32,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(id.to_string()))
            .flex_1()
            .h(px(24.0))
            .px(px(8.0))
            .bg(gpui::rgb(0x1A1A2E))
            .border_1()
            .border_color(gpui::rgb(border_color))
            .rounded(px(4.0))
            .flex()
            .items_center()
            .cursor_text()
            .child(if value.is_empty() {
                div()
                    .text_color(gpui::rgb(0x555555))
                    .text_size(px(11.0))
                    .font_family("monospace")
                    .child(placeholder.to_string())
                    .into_any()
            } else {
                div()
                    .text_color(gpui::rgb(0xCCCCCC))
                    .text_size(px(11.0))
                    .font_family("monospace")
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(value.to_string())
                    .into_any()
            })
    }
}

impl Render for PathBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let repo_val = self.repo_input.clone();
        let search_val = self.search_input.clone();
        let error = self.error_msg.clone();

        div()
            .w_full()
            .h(px(32.0))
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px(px(8.0))
            .py(px(2.0))
            .bg(gpui::rgb(0x1E1E1E))
            .border_b_1()
            .border_color(gpui::rgb(0x333333))
            .child(
                div()
                    .text_color(gpui::rgb(0x888888))
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("REPO"),
            )
            .child(self.render_input("repo_input", &repo_val, "/path/to/repo", 0x444444))
            .child(
                div()
                    .id("btn_open_repo")
                    .px(px(8.0))
                    .py(px(2.0))
                    .bg(gpui::rgb(0x2A5A2A))
                    .hover(|s| s.bg(gpui::rgb(0x3A7A3A)))
                    .cursor_pointer()
                    .rounded(px(3.0))
                    .text_color(gpui::rgb(0x4AE04A))
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Open")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _ev, _win, cx| this.submit_repo(cx)),
                    ),
            )
            .child(div().w(px(1.0)).h(px(20.0)).bg(gpui::rgb(0x444444)))
            .child(
                div()
                    .text_color(gpui::rgb(0x888888))
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("PATH"),
            )
            .child(self.render_input("search_input", &search_val, "src/file.rs", 0x444444))
            .child(
                div()
                    .id("btn_filter")
                    .px(px(8.0))
                    .py(px(2.0))
                    .bg(gpui::rgb(0x2A3A5A))
                    .hover(|s| s.bg(gpui::rgb(0x3A5A7A)))
                    .cursor_pointer()
                    .rounded(px(3.0))
                    .text_color(gpui::rgb(0x4A90D9))
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Filter")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _ev, _win, cx| this.submit_search(cx)),
                    ),
            )
            .when(!search_val.is_empty(), |el| {
                el.child(
                    div()
                        .id("btn_clear_search")
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(gpui::rgb(0x5A2A2A))
                        .hover(|s| s.bg(gpui::rgb(0x7A3A3A)))
                        .cursor_pointer()
                        .rounded(px(3.0))
                        .text_color(gpui::rgb(0xE74C3C))
                        .text_size(px(10.0))
                        .child("✕")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _ev, _win, cx| this.emit_clear(cx)),
                        ),
                )
            })
            .when_some(error, |el, msg| {
                el.child(
                    div()
                        .text_color(gpui::rgb(0xE74C3C))
                        .text_size(px(10.0))
                        .child(msg),
                )
            })
    }
}
