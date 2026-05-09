use gpui::prelude::*;
use gpui::{
    Context, Entity, EventEmitter, IntoElement, MouseButton, ParentElement, Styled, Window, div, px,
};

use crate::text_input::{TextInput, TextInputSubmitted};

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
    repo_input: Entity<TextInput>,
    search_input: Entity<TextInput>,
    error_msg: Option<String>,
}

impl EventEmitter<RepoPathSubmitted> for PathBar {}
impl EventEmitter<SearchPathSubmitted> for PathBar {}
impl EventEmitter<SearchPathCleared> for PathBar {}

impl PathBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let repo_input = cx.new(|cx| TextInput::new("/path/to/repo", cx));
        let search_input = cx.new(|cx| TextInput::new("src/file.rs", cx));

        cx.subscribe(&repo_input, Self::on_repo_submitted).detach();
        cx.subscribe(&search_input, Self::on_search_submitted)
            .detach();

        Self {
            repo_input,
            search_input,
            error_msg: None,
        }
    }

    pub fn set_error(&mut self, msg: Option<String>) {
        self.error_msg = msg;
    }

    pub fn clear_search(&mut self, cx: &mut Context<Self>) {
        self.search_input.update(cx, |input, cx| input.clear(cx));
    }

    fn on_repo_submitted(
        &mut self,
        _input: Entity<TextInput>,
        _event: &TextInputSubmitted,
        cx: &mut Context<Self>,
    ) {
        let path = self.repo_input.read(cx).text().trim().to_string();
        if path.is_empty() {
            return;
        }
        self.error_msg = None;
        cx.emit(RepoPathSubmitted { path });
        cx.notify();
    }

    fn on_search_submitted(
        &mut self,
        _input: Entity<TextInput>,
        _event: &TextInputSubmitted,
        cx: &mut Context<Self>,
    ) {
        let path = self.search_input.read(cx).text().trim().to_string();
        if path.is_empty() {
            cx.emit(SearchPathCleared);
            return;
        }
        cx.emit(SearchPathSubmitted { path });
        cx.notify();
    }

    fn submit_repo(&mut self, cx: &mut Context<Self>) {
        let path = self.repo_input.read(cx).text().trim().to_string();
        if path.is_empty() {
            return;
        }
        self.error_msg = None;
        cx.emit(RepoPathSubmitted { path });
        cx.notify();
    }

    fn submit_search(&mut self, cx: &mut Context<Self>) {
        let path = self.search_input.read(cx).text().trim().to_string();
        if path.is_empty() {
            cx.emit(SearchPathCleared);
            return;
        }
        cx.emit(SearchPathSubmitted { path });
        cx.notify();
    }

    fn emit_clear(&mut self, cx: &mut Context<Self>) {
        self.search_input.update(cx, |input, cx| input.clear(cx));
        cx.emit(SearchPathCleared);
        cx.notify();
    }
}

impl Render for PathBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let search_text = self.search_input.read(cx).text().to_string();
        let error = self.error_msg.clone();
        let repo_input = self.repo_input.clone();
        let search_input = self.search_input.clone();

        div()
            .w_full()
            .h(px(36.0))
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px(px(8.0))
            .py(px(3.0))
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
            .child(
                div()
                    .flex_1()
                    .h(px(26.0))
                    .px(px(6.0))
                    .bg(gpui::rgb(0x1A1A2E))
                    .border_1()
                    .border_color(gpui::rgb(0x444444))
                    .rounded(px(4.0))
                    .overflow_hidden()
                    .text_size(px(11.0))
                    .font_family("monospace")
                    .child(repo_input),
            )
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
            .child(
                div()
                    .flex_1()
                    .h(px(26.0))
                    .px(px(6.0))
                    .bg(gpui::rgb(0x1A1A2E))
                    .border_1()
                    .border_color(gpui::rgb(0x444444))
                    .rounded(px(4.0))
                    .overflow_hidden()
                    .text_size(px(11.0))
                    .font_family("monospace")
                    .child(search_input),
            )
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
            .when(!search_text.is_empty(), |el| {
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
