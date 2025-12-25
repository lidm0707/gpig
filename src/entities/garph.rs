use std::collections::HashMap;

use chrono::DateTime;
use git2::{Oid, Repository};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, PathBuilder, Pixels, Point, Render,
    StatefulInteractiveElement, Styled, Window, canvas, div, px,
};

use crate::entities::commit::CommitNode;
use crate::entities::edge::{Edge, EdgeManager};
use crate::entities::lane::LaneManager;

const START_X: f32 = 30.0;
const LANE_WIDTH: f32 = 15.0;
const COMMIT_HEIGHT: f32 = 20.0;
const SIZE: Pixels = px(10.0);
const GAP_ROW: f32 = 40.0;

pub struct Garph {
    repo: Repository,
    nodes: Vec<CommitNode>,
    edges: Vec<Edge>,
    content_height: Pixels,
}

impl Garph {
    pub fn new(repo: Repository) -> Self {
        Self {
            repo,
            nodes: Vec::new(),
            edges: Vec::new(),
            content_height: px(0.0),
        }
    }

    /* ---------------- compute graph (loop เดียว) ---------------- */

    fn recompute(&mut self) {
        self.nodes.clear();
        self.edges.clear();

        let mut revwalk = self.repo.revwalk().unwrap();
        revwalk
            .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
            .unwrap();
        revwalk.push_head().unwrap();

        let mut lane_manager = LaneManager::new();
        let mut edge_manager = EdgeManager::new();
        let mut map_oid: HashMap<Oid, Vec<Point<Pixels>>> = HashMap::new();

        for (index, oid) in revwalk.enumerate() {
            let oid = oid.unwrap();
            let commit = self.repo.find_commit(oid).unwrap();

            let parents: Vec<Oid> = commit.parents().map(|p| p.id()).collect();
            let lane = lane_manager.assign_commit(&oid, &parents) as f32;

            let pos = Point::new(
                (START_X + lane * LANE_WIDTH).into(),
                (COMMIT_HEIGHT * index as f32).into(),
            );

            let edge_anchor = Point::new(pos.x + SIZE / 2.0, pos.y);

            // connect edges
            if let Some(froms) = map_oid.get(&oid) {
                for from in froms {
                    edge_manager.add(from.clone(), edge_anchor);
                }
            }

            for parent in &parents {
                map_oid.entry(*parent).or_default().push(edge_anchor);
            }

            self.nodes.push(CommitNode::new(
                oid,
                commit.message().unwrap_or_default().to_string(),
                commit.author().email().unwrap_or_default().to_string(),
                commit.time(),
                parents,
                pos,
            ));
        }

        self.edges = edge_manager.take_edges();
        self.content_height = px(self.nodes.len() as f32 * COMMIT_HEIGHT + GAP_ROW);
    }

    /* ---------------- view helpers ---------------- */

    fn node_view(&self, node: &CommitNode) -> impl IntoElement {
        div()
            .absolute()
            .left(node.position.x)
            .top(node.position.y)
            .size(SIZE)
            .bg(gpui::green())
            .border_color(gpui::black())
            .rounded(px(5.0))
    }

    fn row_view(&self, node: &CommitNode) -> impl IntoElement {
        div()
            .size(SIZE)
            .absolute()
            .top(node.position.y)
            .left(px(100.0))
            .bg(gpui::rgb(0x383838))
            .min_w(px(600.0))
            .px(px(10.0))
            .py(px(5.0))
            .rounded(px(4.0))
            .h(px(COMMIT_HEIGHT))
            .text_color(gpui::rgb(0x969696))
            .text_size(px(10.0))
            .child(format!(
                "{} — {} — {}",
                node.author,
                DateTime::from_timestamp(node.timestamp.seconds(), 0).unwrap(),
                node.message
            ))
    }
}

impl Render for Garph {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.recompute();

        let nodes = self.nodes.clone();
        let edges = self.edges.clone();
        let height = self.content_height;

        div()
            .size_full()
            .id("garph")
            .overflow_scroll()
            .bg(gpui::rgb(0x282828))
            .relative()
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(height)
                    // edges
                    .child(
                        canvas(
                            move |_, _, _| {},
                            move |bounds, _, window, _| {
                                let offset = bounds.origin;
                                for e in &edges {
                                    let mut path = PathBuilder::stroke(px(1.5));
                                    let size_node = Point::new(px(0.0), px(6.0));
                                    let start = e.from + offset + size_node;
                                    let end = e.to + offset + size_node;

                                    path.move_to(start);
                                    let same_lane = (start.x - end.x).abs() < px(0.5);

                                    if same_lane {
                                        // เส้นตรงยาว
                                        path.line_to(end);
                                    } else if start.x > end.x {
                                        let ctrl1 = Point::new(start.x, end.y);
                                        let ctrl2 = Point::new(start.x, end.y);

                                        path.cubic_bezier_to(end, ctrl1, ctrl2);
                                    } else if start.x < end.x {
                                        let ctrl1 = Point::new(end.x, start.y);
                                        let ctrl2 = Point::new(end.x, start.y);

                                        path.cubic_bezier_to(end, ctrl1, ctrl2);
                                    }
                                    if let Ok(p) = path.build() {
                                        window.paint_path(p, gpui::white());
                                    }
                                }
                            },
                        )
                        .absolute()
                        .size_full(),
                    )
                    // nodes
                    .child(div().children(nodes.iter().map(|n| self.node_view(n))))
                    // rows
                    .child(div().children(nodes.iter().map(|n| self.row_view(n)))),
            )
    }
}
