use gpui::{
    Context, EventEmitter, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};

#[derive(Clone)]
pub struct ClickMenuEvent;

#[derive(Clone)]
pub struct DropdownEvent {
    pub is_open: bool,
}
pub struct MenuBar {
    is_dropdown_open: bool,
}

impl EventEmitter<DropdownEvent> for MenuBar {}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            is_dropdown_open: false,
        }
    }

    pub fn is_dropdown_open(&self) -> bool {
        self.is_dropdown_open
    }

    pub fn close_dropdown(&mut self, cx: &mut Context<Self>) {
        if self.is_dropdown_open {
            self.is_dropdown_open = false;
            cx.emit(DropdownEvent { is_open: false });
            cx.notify();
        }
    }
}

impl Render for MenuBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("menu_bar")
            .flex()
            .bg(gpui::rgb(0x1e1e1e))
            .h(px(36.0))
            .items_center()
            .child(
                div()
                    .text_color(gpui::white())
                    .id("file_menu_button")
                    .px(px(16.0))
                    .py(px(8.0))
                    .child("File")
                    .hover(|style| style.bg(gpui::rgb(0x2a2a2a)))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.is_dropdown_open = !this.is_dropdown_open;
                        cx.emit(DropdownEvent {
                            is_open: this.is_dropdown_open,
                        });
                        cx.notify();
                    })),
            )
    }
}
