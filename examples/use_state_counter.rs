use std::error::Error;

use gpui::{
    App, AppContext, Application, Context, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, Styled, Window, WindowOptions, div,
};

// Workspace is the main UI component
// No separate Counter entity needed - state will be managed with use_state
struct Workspace;

impl Workspace {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // use_state creates state that persists as long as this render method
        // is called in consecutive frames. It automatically generates a key
        // based on the caller's code location.
        //
        // The state is returned as an Entity<usize> that we can update and read.
        let counter = window.use_state(cx, |_, cx| cx.new(|_| 0usize));
        let mut number = 0usize;
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap_4()
            .bg(gpui::rgb(0x1a1a1a))
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(gpui::rgb(0xffffff))
                    .child("window.use_state Example"),
            )
            .child(
                div()
                    .p_8()
                    .bg(gpui::rgb(0x2a2a2a))
                    .border_1()
                    .border_color(gpui::rgb(0x444444))
                    .rounded_md()
                    .text_color(gpui::rgb(0xffffff))
                    .text_xl()
                    .child(format!("Count: {}", counter.read(cx).read(cx)))
                    .child(format!("Number: {}", number)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(
                        div()
                            .p_4()
                            .px_6()
                            .bg(gpui::rgb(0xdc3545))
                            .text_color(gpui::rgb(0xffffff))
                            .font_weight(gpui::FontWeight::BOLD)
                            .rounded_md()
                            .child("-")
                            .on_mouse_down(
                                MouseButton::Left,
                                {
                                    number += 1;
                                    let counter = counter.clone();
                                    cx.listener(move |_this, _event, _window, cx| {
                                        counter.update(cx, |count, cx| {
                                            if *count.read(cx) > 0 {

                                                *count.as_mut(cx) = *count.read(cx) - 1;
                                            }
                                        });
                                    })
                                },
                            ),
                    )
                    .child(
                        div()
                            .p_4()
                            .px_6()
                            .bg(gpui::rgb(0x007acc))
                            .text_color(gpui::rgb(0xffffff))
                            .font_weight(gpui::FontWeight::BOLD)
                            .rounded_md()
                            .child("+")
                            .on_mouse_down(
                                MouseButton::Left,
                                {
                                    let counter = counter.clone();
                                    cx.listener(move |_this, _event, _window, cx| {
                                        counter.update(cx, |count, cx| {
                                            *count.as_mut(cx) += 1;
                                            // cx.notify(); // Trigger re-render
                                        });
                                    })
                                },
                            ),
                    )
                    .child(
                        div()
                            .p_4()
                            .px_6()
                            .bg(gpui::rgb(0x6c757d))
                            .text_color(gpui::rgb(0xffffff))
                            .font_weight(gpui::FontWeight::BOLD)
                            .rounded_md()
                            .child("Reset")
                            .on_mouse_down(
                                MouseButton::Left,
                                {
                                    let counter = counter.clone();
                                    cx.listener(move |_this, _event, _window, cx| {
                                        counter.update(cx, |count, cx| {
                                            *count.as_mut(cx) = 0;
                                            // cx.notify();
                                        });
                                    })
                                },
                            ),
                    )
            )
            .child(
                div()
                    .max_w_96()
                    .text_sm()
                    .text_color(gpui::rgb(0x888888))
                    .child(
                        "This example demonstrates window.use_state(), which creates state \
                         that persists as long as the render method is called in consecutive frames. \
                         The state is automatically keyed to the location in code where use_state is called.",
                    ),
            )
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
