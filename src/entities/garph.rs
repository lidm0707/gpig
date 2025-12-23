use std::mem::offset_of;

use chrono::DateTime;
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, PathBuilder, Render,
    StatefulInteractiveElement, Styled, Window, canvas, div, px,
};

use crate::entities::commit::CommitNode;
use crate::entities::edge::EdgeManager;

#[derive(Debug, Clone)]
pub struct Garph {
    pub nodes: Vec<CommitNode>,
    pub edge_manager: EdgeManager,
}

impl Garph {
    pub fn new(nodes: Vec<CommitNode>, edge_manager: EdgeManager) -> Self {
        Garph {
            nodes,
            edge_manager,
        }
    }

    pub fn create_node(&self, node: CommitNode) -> impl IntoElement {
        // Adjust positioning to match edge coordinates
        let x = node.position.0; // X position (from START_X minus commit height)
        let y = node.position.1; // Y position (based on lane)

        div()
            .absolute()
            .left(px(x)) // Scale lane position for better visibility
            .top(px(y)) // Adjusted Y position (inverted for proper display)
            .w(px(10.0))
            .h(px(10.0))
            .bg(gpui::green())
            .border_color(gpui::black())
            .rounded(px(5.0))
    }

    pub fn create_row_with_node(&self, node: CommitNode, index: usize) -> impl IntoElement {
        // Calculate the Y position to match the node position

        div()
            .absolute()
            .top(px(node.position.1))
            .left(px(1.0)) // Position to the right of the graph
            .flex_row()
            .gap(px(20.0))
            .children([
                // Commit details
                div()
                    .bg(gpui::rgb(0x383838))
                    .min_w(px(600.0))
                    .px(px(10.0))
                    .py(px(5.0))
                    .rounded(px(4.0))
                    .child(
                        div().children([div()
                            .text_color(gpui::rgb(0x969696))
                            .text_size(px(10.0))
                            .child(format!(
                                "{} - {} - {}",
                                node.author,
                                DateTime::from_timestamp(node.timestamp.seconds(), 0).unwrap(),
                                node.message
                            ))]),
                    ),
            ])
    }
}

impl Render for Garph {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let edges = self.edge_manager.edges.clone();
        let content_height = px(self.nodes.len() as f32 * 24.0);

        // Create a container that will handle scrolling for everything
        div()
            .size_full()
            .bg(gpui::rgb(0x282828))
            .id("dag")
            .overflow_scroll()
            .relative()
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(content_height)
                    .child(
                        // canvas สำหรับ edge
                        canvas(
                            move |_, _, _| {},
                            move |bounds, _, window, _| {
                                let offset = bounds.origin;

                                for edge in &edges {
                                    let mut path = PathBuilder::stroke(px(1.5));
                                    path.move_to(edge.from + offset);
                                    path.line_to(edge.to + offset);

                                    if let Ok(p) = path.build() {
                                        window.paint_path(p, gpui::white());
                                    }
                                }
                            },
                        )
                        .absolute()
                        .size_full(),
                    )
                    .child(
                        // nodes
                        div()
                            .absolute()
                            .top(px(0.))
                            .left(px(0.))
                            .children(self.nodes.iter().map(|n| self.create_node(n.clone()))),
                    )
                    .child(
                        // rows
                        div().absolute().top(px(0.)).left(px(100.)).children(
                            self.nodes
                                .iter()
                                .enumerate()
                                .map(|(i, n)| self.create_row_with_node(n.clone(), i)),
                        ),
                    ),
            )
    }
}
