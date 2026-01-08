use gpui::{
    AnyElement, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};

use crate::garph::Garph;

pub struct Dock;
pub struct Pane;
pub struct Workspace {
    dock: Option<Entity<Garph>>,
    // pane: Vec<Entity<AnyElement>>,
}

impl Workspace {
    pub fn new(dock: Option<Entity<Garph>>) -> Self {
        Self { dock }
    }

    // pub fn add_pane(&mut self, pane: Entity<AnyElement>) {
    //     self.pane.push(pane);
    // }

    // pub fn remove_pane(&mut self, index: usize) {
    //     self.pane.remove(index);
    // }

    // pub fn remove_all_panes(&mut self) {
    //     self.pane.clear();
    // }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let dock = self.dock.clone().unwrap();
        let result =
            div()
                .size_full()
                .relative()
                .flex()
                .child(div().w(gpui::px(300.0)).h_full().child(dock))
                .child(div().flex_1().bg(gpui::white()).text_2xl().child(
                    "test Workspace \n test Workspace \n test Workspace \n test Workspace \n ",
                ));

        result
    }
}
