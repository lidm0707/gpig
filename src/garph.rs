use chrono::DateTime;
use git2::{Oid, Repository};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, PathBuilder, Pixels, Point, Render,
    StatefulInteractiveElement, Styled, Window, canvas, div, px,
};

use crate::color::ColorManager;
use crate::commit::CommitNode;
use crate::edge::{Edge, EdgeManager};
use crate::history_oid::{HistoryOid, HistoryOidManager};
use crate::lane::LaneManager;

const START_X: f32 = 30.0;
const LANE_WIDTH: f32 = 15.0;
const COMMIT_HEIGHT: f32 = 20.0;
const SIZE: Pixels = px(10.0);
const GAP_ROW: f32 = 40.0;
const LIMIT_ROW: usize = 100;

pub const GIT_RED: u32 = 0xE64D3F;
pub const GIT_YELLOW: u32 = 0xF1C40F;
pub const GIT_GREEN: u32 = 0x2ECC71;
pub const GIT_BLUE: u32 = 0x3498DB;
pub const GIT_PURPLE: u32 = 0x9B59B6;
pub const VEC_COLORS: &[u32] = &[GIT_PURPLE, GIT_BLUE, GIT_RED, GIT_YELLOW, GIT_GREEN];

pub struct Garph {
    repo: Repository,
    nodes: Vec<CommitNode>,
    edges: Vec<Edge>,
    content_height: Pixels,
    max_lane: usize,
}

impl Garph {
    pub fn new(repo: Repository) -> Self {
        Self {
            repo,
            nodes: Vec::new(),
            edges: Vec::new(),
            content_height: px(0.0),
            max_lane: 0,
        }
    }

    /* ---------------- compute graph (loop เดียว) ---------------- */

    fn recompute(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.max_lane = 0;

        let mut revwalk = self.repo.revwalk().unwrap();
        revwalk
            .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
            .unwrap();
        revwalk.push_head().unwrap();

        let mut lane_manager = LaneManager::new();
        let mut edge_manager = EdgeManager::new();
        let mut color_manager = ColorManager::new(VEC_COLORS.to_vec());

        let mut history_oids_manager = HistoryOidManager::new();

        for (index, oid) in revwalk.take(LIMIT_ROW).enumerate() {
            let oid = oid.unwrap();
            let commit = self.repo.find_commit(oid).unwrap();
            let parents: Vec<Oid> = commit.parents().map(|p| p.id()).collect();
            let lane = lane_manager.assign_commit(&oid, &parents);

            let color = color_manager.get_color(&lane);

            let pos = Point::new(
                (START_X + (lane as f32) * LANE_WIDTH).into(),
                (COMMIT_HEIGHT * index as f32).into(),
            );

            // Track maximum lane
            if lane > self.max_lane {
                self.max_lane = lane;
            }

            let current_edge_point = Point::new(pos.x + SIZE / 2.0, pos.y);

            // connect edges
            if let Some(history_oids) = history_oids_manager.get(&oid) {
                for history in history_oids {
                    if history.edge_point.x > current_edge_point.x {
                        edge_manager.add(history.edge_point, current_edge_point, history.color);

                        if history.lane > 0 {
                            color_manager.remove_lane_color(&history.lane);
                        }
                    } else if history.edge_point.x < current_edge_point.x {
                        edge_manager.add(current_edge_point, history.edge_point, color);
                    } else {
                        edge_manager.add(history.edge_point, current_edge_point, history.color);
                    }
                }
            }

            for parent in &parents {
                history_oids_manager
                    .add_history(*parent, HistoryOid::new(current_edge_point, color, lane));
            }

            self.nodes.push(CommitNode::new(
                oid,
                commit.message().unwrap_or_default().to_string(),
                commit.author().email().unwrap_or_default().to_string(),
                commit.time(),
                parents,
                pos,
                color,
            ));
        }

        self.edges = edge_manager.take_edges();
        self.content_height = px(self.nodes.len() as f32 * COMMIT_HEIGHT + GAP_ROW);
    }

    /* ---------------- view helpers ---------------- */

    fn clean_message(message: &str) -> String {
        message.lines().next().unwrap_or(message).to_string()
    }

    fn combined_row_view(&self, node: &CommitNode) -> impl IntoElement {
        let message = Self::clean_message(&node.message);

        // Calculate text position based on max lane to ensure no overlap
        let text_left = START_X + ((self.max_lane + 1) as f32) * LANE_WIDTH + 20.0;

        div()
            .absolute()
            .top(node.position.y)
            .left(px(0.0))
            .right(px(0.0))
            .h(px(COMMIT_HEIGHT))
            .flex()
            .flex_row()
            .items_center()
            .group("commit-row")
            .hover(|style| style.bg(gpui::hsla(0.0, 0.0, 0.22, 0.3)))
            // node
            .child(
                div()
                    .left(node.position.x)
                    .size(SIZE)
                    .bg(gpui::rgb(VEC_COLORS[node.color]))
                    .border_color(gpui::black())
                    .rounded(px(5.0))
                    .group_hover("commit-row", |style| style.size(SIZE + px(5.0))),
            )
            // text
            .child(
                div()
                    .left(px(text_left))
                    .px(px(10.0))
                    .py(px(5.0))
                    .rounded(px(4.0))
                    .text_color(gpui::rgb(0x969696))
                    .text_size(px(10.0))
                    .line_clamp(1)
                    .child(format!(
                        "{}",
                        // "{} — {} — {}",
                        // node.author,
                        // DateTime::from_timestamp(node.timestamp.seconds(), 0).unwrap(),
                        message
                    )),
            )
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
                    .absolute()
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
                                    // straight line
                                    if same_lane {
                                        path.line_to(end);
                                    } else if start.x > end.x {
                                        let ctrl1 = Point::new(start.x, end.y);
                                        let ctrl2 = Point::new(start.x, end.y);
                                        // curve is feak when line too short or long
                                        path.cubic_bezier_to(end, ctrl1, ctrl2);
                                    } else if start.x < end.x {
                                        let ctrl1 = Point::new(end.x, start.y);
                                        let ctrl2 = Point::new(end.x, start.y);
                                        // curve is feak when line too short or long
                                        path.cubic_bezier_to(end, ctrl1, ctrl2);
                                    }
                                    if let Ok(p) = path.build() {
                                        // window.paint_path(p, gpui::white());
                                        window.paint_path(p, gpui::rgb(VEC_COLORS[e.color]));
                                    }
                                }
                            },
                        )
                        .absolute()
                        .size_full(),
                    )
                    // combined rows (node + text)
                    .child(div().children(nodes.iter().map(|n| self.combined_row_view(n)))),
            )
    }
}
