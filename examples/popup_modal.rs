use std::error::Error;

use gpui::{
    App, AppContext, Application, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Point, Render, Styled, Window, WindowOptions, div,
    point, prelude::FluentBuilder, px,
};

// Modal state: stores which box is open and the click position
#[derive(Clone)]
struct ModalState {
    box_number: usize,
    position: Option<Point<Pixels>>,
}

impl ModalState {
    fn new() -> Self {
        Self {
            box_number: 0,
            position: None,
        }
    }
}

// Workspace is the main UI component
struct Workspace;

impl Workspace {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Use state to track modal state (box number and click position)
        let modal_state = window.use_state(cx, |_, cx| cx.new(|_| ModalState::new()));

        // Main container
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .bg(gpui::rgb(0x1a1a1a))
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(gpui::rgb(0xffffff))
                    .child("Popup Modal Example"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .p_4()
                            .px_8()
                            .bg(gpui::rgb(0x007acc))
                            .text_color(gpui::rgb(0xffffff))
                            .font_weight(gpui::FontWeight::BOLD)
                            .rounded_md()
                            .child("Box 1")
                            .on_mouse_down(MouseButton::Left, {
                                let modal_state = modal_state.clone();
                                cx.listener(move |_this, event: &MouseDownEvent, _window, cx| {
                                    let position = event.position;
                                    modal_state.update(cx, |state, cx| {
                                        *state.as_mut(cx) = ModalState {
                                            box_number: 1,
                                            position: Some(position),
                                        };
                                    });
                                })
                            }),
                    )
                    .child(
                        div()
                            .p_4()
                            .px_8()
                            .bg(gpui::rgb(0x28a745))
                            .text_color(gpui::rgb(0xffffff))
                            .font_weight(gpui::FontWeight::BOLD)
                            .rounded_md()
                            .child("Box 2")
                            .on_mouse_down(MouseButton::Left, {
                                let modal_state = modal_state.clone();
                                cx.listener(move |_this, event: &MouseDownEvent, _window, cx| {
                                    let position = event.position;
                                    modal_state.update(cx, |state, cx| {
                                        *state.as_mut(cx) = ModalState {
                                            box_number: 2,
                                            position: Some(position),
                                        };
                                    });
                                })
                            }),
                    )
                    .child(
                        div()
                            .p_4()
                            .px_8()
                            .bg(gpui::rgb(0xdc3545))
                            .text_color(gpui::rgb(0xffffff))
                            .font_weight(gpui::FontWeight::BOLD)
                            .rounded_md()
                            .child("Box 3")
                            .on_mouse_down(MouseButton::Left, {
                                let modal_state = modal_state.clone();
                                cx.listener(move |_this, event: &MouseDownEvent, _window, cx| {
                                    let position = event.position;
                                    modal_state.update(cx, |state, cx| {
                                        *state.as_mut(cx) = ModalState {
                                            box_number: 3,
                                            position: Some(position),
                                        };
                                    });
                                })
                            }),
                    ),
            )
            // Add modal overlay if a box was clicked
            .when(modal_state.read(cx).read(cx).box_number > 0, |this| {
                let modal_data = modal_state.read(cx).read(cx).clone();
                let box_number = modal_data.box_number;
                let modal_state = modal_state.clone();

                let position = modal_data.position.unwrap_or(point(px(100.0), px(100.0)));

                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(gpui::rgba(0x000000))
                        .flex()
                        .items_start()
                        .justify_start()
                        .child(
                            div()
                                .absolute()
                                .top(position.y + px(10.0))
                                .left(position.x + px(10.0))
                                .p_8()
                                .bg(gpui::rgb(0x2a2a2a))
                                .border_1()
                                .border_color(gpui::rgb(0x444444))
                                .rounded_md()
                                .min_w_64()
                                .child(
                                    div()
                                        .text_xl()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(gpui::rgb(0xffffff))
                                        .child(format!("Box {} Clicked!", box_number)),
                                )
                                .child(
                                    div()
                                        .mt_4()
                                        .p_4()
                                        .px_6()
                                        .bg(gpui::rgb(0x6c757d))
                                        .text_color(gpui::rgb(0xffffff))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .rounded_md()
                                        .child("Close")
                                        .on_mouse_down(MouseButton::Left, {
                                            let modal_state = modal_state.clone();
                                            cx.listener(move |_this, _event, _window, cx| {
                                                cx.stop_propagation();
                                                modal_state.update(cx, |state, cx| {
                                                    *state.as_mut(cx) = ModalState::new();
                                                });
                                            })
                                        }),
                                ),
                        ),
                )
            })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Workspace::new(cx)),
        )
        .unwrap();
    });

    Ok(())
}
