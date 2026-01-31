use std::error::Error;

use gpui::{
    App, AppContext, Application, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, WindowOptions, div,
};

// Counter is just a state struct - no Render implementation
struct Counter {
    count: usize,
}

// Workspace holds the Counter entity and implements Render
struct Workspace {
    counter: Entity<Counter>,
}

impl Workspace {
    pub fn new(counter: Entity<Counter>, _cx: &mut Context<Self>) -> Self {
        Self { counter }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.counter.read(_cx).count;

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .bg(gpui::rgb(0x1a1a1a))
            .child(
                div()
                    .text_xl()
                    .text_color(gpui::rgb(0xffffff))
                    .child(format!("Count: {}", count)),
            )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                // window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx: &mut App| {
                let counter: Entity<Counter> = cx.new(|_cx| Counter { count: 0 });
                cx.new(|cx| Workspace::new(counter, cx))
            },
        )
        .unwrap();
    });

    Ok(())
}
