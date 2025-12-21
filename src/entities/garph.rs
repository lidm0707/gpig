use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, Window, div, px,
};

use crate::entities::commit::CommitNode;
#[derive(Debug, Clone)]
pub struct Garph {
    pub nodes: Vec<CommitNode>,
    // pub edges: Vec<Edge>,
}

impl Garph {
    pub fn new(nodes: Vec<CommitNode>) -> Self {
        Garph { nodes }
    }

    pub fn create_node(&self, node: CommitNode) -> impl IntoElement {
        div()
            .absolute()
            .bg(gpui::green())
            .border_1()
            .border_color(gpui::black())
            .rounded(px(20.0))
            .size(px(40.0))
            .top(px(node.position.0))
            .left(px(node.position.1))
    }
}

impl Render for Garph {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let meta = self.clone();
        let nodes = self
            .clone()
            .nodes
            .into_iter()
            .map(|node| meta.create_node(node))
            .collect::<Vec<_>>();

        div()
            .size(px(800.0))
            .bg(gpui::rgb(0x282828))
            .id("dag")
            .overflow_scroll()
            .children(nodes)
    }
}
