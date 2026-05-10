use gpui::prelude::*;
use gpui::{
    Context, Entity, EventEmitter, InteractiveElement, IntoElement, MouseButton, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::repo_picker::{self, RepoPicker, RepoSelected};
use crate::suggest::SuggestState;
use crate::text_input::{TextInput, TextInputSubmitted};

const SUGGEST_ITEM_H: f32 = 22.0;
const SUGGEST_DROPDOWN_W: f32 = 300.0;

const COLOR_BG: u32 = 0x1E1E1E;
const COLOR_BORDER: u32 = 0x333333;
const COLOR_INPUT_BG: u32 = 0x1A1A2E;
const COLOR_LABEL: u32 = 0x888888;
const COLOR_SEPARATOR: u32 = 0x444444;
const COLOR_ERROR: u32 = 0xE74C3C;
const COLOR_MODE_ACTIVE_BG: u32 = 0x2A3A5A;
const COLOR_MODE_ACTIVE_TEXT: u32 = 0x4A90D9;
const COLOR_MODE_INACTIVE_BG: u32 = 0x252525;
const COLOR_MODE_INACTIVE_TEXT: u32 = 0x666666;

#[derive(Clone, Debug, PartialEq)]
pub enum RepoMode {
    Local,
    Remote,
}

#[derive(Clone, Debug)]
pub struct ViewModeChanged {
    pub mode: RepoMode,
}

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
    repo_picker: Entity<RepoPicker>,
    search_input: Entity<TextInput>,
    error_msg: Option<String>,
    suggest: SuggestState,
    suggest_open: bool,
    mode: RepoMode,
}

impl EventEmitter<ViewModeChanged> for PathBar {}
impl EventEmitter<RepoPathSubmitted> for PathBar {}
impl EventEmitter<SearchPathSubmitted> for PathBar {}
impl EventEmitter<SearchPathCleared> for PathBar {}

impl PathBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let repo_picker = cx.new(RepoPicker::new);
        let search_input = cx.new(|cx| TextInput::new("src/file.rs", cx));

        cx.subscribe(&repo_picker, Self::on_repo_selected).detach();
        cx.subscribe(&search_input, Self::on_search_submitted)
            .detach();

        Self {
            repo_picker,
            search_input,
            error_msg: None,
            suggest: SuggestState::new(),
            suggest_open: false,
            mode: RepoMode::Local,
        }
    }

    pub fn repo_picker(&self) -> &Entity<RepoPicker> {
        &self.repo_picker
    }

    pub fn search_input(&self) -> &Entity<TextInput> {
        &self.search_input
    }

    pub fn mode(&self) -> &RepoMode {
        &self.mode
    }

    pub fn set_error(&mut self, msg: Option<String>) {
        self.error_msg = msg;
    }

    pub fn set_suggest_paths(&mut self, paths: Vec<String>) {
        self.suggest.set_paths(paths);
    }

    pub fn suggest_open(&self) -> bool {
        self.suggest_open
    }

    pub fn suggest_items(&self, cx: &gpui::App) -> Vec<String> {
        let text = self.search_input.read(cx).text().trim().to_string();
        if text.is_empty() {
            return Vec::new();
        }
        self.suggest
            .filter(&text)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn select_suggestion(&mut self, path: String, cx: &mut Context<Self>) {
        self.search_input
            .update(cx, |input, cx| input.set_text(&path, cx));
        self.suggest_open = false;
        cx.emit(SearchPathSubmitted { path });
        cx.notify();
    }

    pub fn close_suggest(&mut self) {
        self.suggest_open = false;
    }

    pub fn clear_search(&mut self, cx: &mut Context<Self>) {
        self.search_input.update(cx, |input, cx| input.clear(cx));
        self.suggest_open = false;
    }

    pub fn close_repo_dropdown(&mut self, cx: &mut Context<Self>) {
        self.repo_picker.update(cx, |picker, cx| picker.close(cx));
    }

    fn set_mode(&mut self, mode: RepoMode, cx: &mut Context<Self>) {
        if self.mode == mode {
            return;
        }
        self.mode = mode.clone();
        cx.emit(ViewModeChanged { mode });
        cx.notify();
    }

    fn on_repo_selected(
        &mut self,
        _picker: Entity<RepoPicker>,
        event: &RepoSelected,
        cx: &mut Context<Self>,
    ) {
        self.error_msg = None;
        cx.emit(RepoPathSubmitted {
            path: event.path.clone(),
        });
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
        self.suggest_open = false;
        cx.emit(SearchPathSubmitted { path });
        cx.notify();
    }

    fn submit_search(&mut self, cx: &mut Context<Self>) {
        let path = self.search_input.read(cx).text().trim().to_string();
        if path.is_empty() {
            cx.emit(SearchPathCleared);
            return;
        }
        self.suggest_open = false;
        cx.emit(SearchPathSubmitted { path });
        cx.notify();
    }

    fn emit_clear(&mut self, cx: &mut Context<Self>) {
        self.search_input.update(cx, |input, cx| input.clear(cx));
        self.suggest_open = false;
        cx.emit(SearchPathCleared);
        cx.notify();
    }

    fn render_mode_toggle(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let local_active = self.mode == RepoMode::Local;
        let remote_active = self.mode == RepoMode::Remote;

        div()
            .flex()
            .flex_row()
            .rounded(px(4.0))
            .border_1()
            .border_color(gpui::rgb(COLOR_SEPARATOR))
            .overflow_hidden()
            .child(
                div()
                    .id("mode_local")
                    .px(px(6.0))
                    .py(px(2.0))
                    .cursor_pointer()
                    .bg(if local_active {
                        gpui::rgb(COLOR_MODE_ACTIVE_BG)
                    } else {
                        gpui::rgb(COLOR_MODE_INACTIVE_BG)
                    })
                    .text_color(if local_active {
                        gpui::rgb(COLOR_MODE_ACTIVE_TEXT)
                    } else {
                        gpui::rgb(COLOR_MODE_INACTIVE_TEXT)
                    })
                    .text_size(px(9.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("LOCAL")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _ev, _win, cx| this.set_mode(RepoMode::Local, cx)),
                    ),
            )
            .child(
                div()
                    .id("mode_remote")
                    .px(px(6.0))
                    .py(px(2.0))
                    .cursor_pointer()
                    .bg(if remote_active {
                        gpui::rgb(COLOR_MODE_ACTIVE_BG)
                    } else {
                        gpui::rgb(COLOR_MODE_INACTIVE_BG)
                    })
                    .text_color(if remote_active {
                        gpui::rgb(COLOR_MODE_ACTIVE_TEXT)
                    } else {
                        gpui::rgb(COLOR_MODE_INACTIVE_TEXT)
                    })
                    .text_size(px(9.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("REMOTE")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _ev, _win, cx| this.set_mode(RepoMode::Remote, cx)),
                    ),
            )
            .into_any()
    }
}

pub fn render_suggest_dropdown(
    path_bar: &Entity<PathBar>,
    cx: &mut Context<crate::workspace::Workspace>,
) -> Option<gpui::AnyElement> {
    let items = path_bar.read(cx).suggest_items(cx);
    if items.is_empty() {
        return None;
    }

    let dropdown_h = items.len() as f32 * SUGGEST_ITEM_H;
    let pb = path_bar.clone();

    Some(
        div()
            .id("suggest_dropdown_overlay")
            .absolute()
            .top(px(72.0))
            .left(px(300.0))
            .w(px(SUGGEST_DROPDOWN_W))
            .max_h(px(dropdown_h + 4.0))
            .overflow_y_scroll()
            .bg(gpui::rgb(0x1a1a1a))
            .border_1()
            .border_color(gpui::rgb(0x444444))
            .rounded(px(4.0))
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, move |_ev, _win, _cx: &mut gpui::App| {
                _cx.stop_propagation();
            })
            .children(
                items
                    .into_iter()
                    .enumerate()
                    .map(|(i, path): (usize, String)| {
                        let path_clone = path.clone();
                        let pb2 = pb.clone();
                        div()
                            .id(SharedString::from(format!("suggest_{}", i)))
                            .px(px(8.0))
                            .py(px(2.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(gpui::rgb(0x333333)))
                            .on_mouse_down(
                                MouseButton::Left,
                                move |_ev, _win, cx: &mut gpui::App| {
                                    pb2.update(cx, |pb, cx| {
                                        pb.select_suggestion(path_clone.clone(), cx);
                                    });
                                    cx.stop_propagation();
                                },
                            )
                            .child(
                                div()
                                    .text_color(gpui::rgb(0xCCCCCC))
                                    .text_size(px(11.0))
                                    .font_family("monospace")
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .child(path),
                            )
                            .into_any()
                    }),
            )
            .into_any_element(),
    )
}

impl Render for PathBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.repo_picker.update(cx, |p, cx| p.poll_scan(cx));

        let error = self.error_msg.clone();
        let mode_toggle = self.render_mode_toggle(cx);

        let search_text = self.search_input.read(cx).text().to_string();
        let suggestions = self.suggest.filter(search_text.trim());
        self.suggest_open = !suggestions.is_empty() && !search_text.trim().is_empty();

        let repo_picker = self.repo_picker.clone();
        let search_input = self.search_input.clone();

        div()
            .id("path_bar")
            .w_full()
            .h(px(36.0))
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px(px(8.0))
            .py(px(3.0))
            .bg(gpui::rgb(COLOR_BG))
            .border_b_1()
            .border_color(gpui::rgb(COLOR_BORDER))
            .child(
                div()
                    .text_color(gpui::rgb(COLOR_LABEL))
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("REPO"),
            )
            .child(repo_picker::render_button(&repo_picker, cx))
            .child(mode_toggle)
            .child(div().w(px(1.0)).h(px(20.0)).bg(gpui::rgb(COLOR_SEPARATOR)))
            .child(
                div()
                    .text_color(gpui::rgb(COLOR_LABEL))
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("PATH"),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h(px(26.0))
                    .px(px(6.0))
                    .bg(gpui::rgb(COLOR_INPUT_BG))
                    .border_1()
                    .border_color(gpui::rgb(COLOR_SEPARATOR))
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
                        .text_color(gpui::rgb(COLOR_ERROR))
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
                        .text_color(gpui::rgb(COLOR_ERROR))
                        .text_size(px(10.0))
                        .child(msg),
                )
            })
    }
}
