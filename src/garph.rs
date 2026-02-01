use git2::{Oid, Repository};
use gpui::prelude::FluentBuilder;
use gpui::{
    Context, EventEmitter, InteractiveElement, IntoElement, MouseButton, ParentElement,
    PathBuilder, Pixels, Point, Render, StatefulInteractiveElement, Styled, Window, canvas, div,
    px,
};

use crate::color::ColorManager;
use crate::commit::CommitNode;
use crate::edge::{Edge, EdgeManager};
use crate::history_oid::{HistoryOid, HistoryOidManager};
use crate::lane::LaneManager;
use std::cell::RefCell;
use std::rc::Rc;

const START_X: f32 = 30.0;
const LANE_WIDTH: f32 = 15.0;
const COMMIT_HEIGHT: f32 = 20.0;
const SIZE: Pixels = px(10.0);
const GAP_ROW: f32 = 40.0;
const LIMIT_ROW: usize = 100;

// Diff limits to prevent memory exhaustion
const MAX_FILES_TO_SHOW: usize = 10;
const MAX_LINES_PER_FILE: usize = 25;
const MAX_TOTAL_LINES: usize = 100;

pub const GIT_RED: u32 = 0xE64D3F;
pub const GIT_YELLOW: u32 = 0xF1C40F;
pub const GIT_GREEN: u32 = 0x2ECC71;
pub const GIT_BLUE: u32 = 0x3498DB;
pub const GIT_PURPLE: u32 = 0x9B59B6;
pub const VEC_COLORS: &[u32] = &[GIT_PURPLE, GIT_BLUE, GIT_RED, GIT_YELLOW, GIT_GREEN];

#[derive(Clone)]
pub struct CommitSelected {
    pub oid: Oid,
    pub message: String,
    pub author: String,
    pub timestamp: git2::Time,
    pub parents: Vec<Oid>,
}

#[derive(Clone, Debug)]
pub struct ChangedFile {
    pub path: String,
    pub status: git2::Delta,
    pub old_oid: Option<git2::Oid>,
    pub new_oid: Option<git2::Oid>,
}

pub struct RepoPathChanged {
    pub path: String,
}

#[derive(Clone)]
pub struct Garph {
    repo: Rc<RefCell<Option<Repository>>>,
    nodes: Vec<CommitNode>,
    edges: Vec<Edge>,
    content_height: Pixels,
    max_lane: usize,
}

impl Garph {
    pub fn new(repo: Option<Repository>) -> Self {
        Self {
            repo: Rc::new(RefCell::new(repo)),
            nodes: Vec::new(),
            edges: Vec::new(),
            content_height: px(0.0),
            max_lane: 0,
        }
    }

    pub fn update_repo(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let repo = git2::Repository::open(path)?;
        *self.repo.borrow_mut() = Some(repo);
        Ok(())
    }

    pub fn compute_commit_diff(
        &self,
        oid: &git2::Oid,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let repo = self.repo.borrow();
        let repo = repo.as_ref().ok_or("No repository loaded")?;

        let commit = repo.find_commit(*oid)?;
        let parents: Vec<git2::Commit> = commit.parents().collect();

        if parents.is_empty() {
            // Initial commit - show all files as added
            let tree = commit.tree()?;
            let mut diff_lines = vec![format!("Initial commit - {} files added", tree.len())];
            diff_lines.push("".to_string());

            for entry in tree.iter() {
                diff_lines.push(format!("+++ a/{}", entry.name().unwrap_or_default()));
            }

            return Ok(diff_lines.join("\n"));
        }

        // For now, just show diff with the first parent
        // For merge commits, we could show diff with all parents
        let parent = &parents[0];

        // Get the tree for this commit and its parent
        let commit_tree = commit.tree()?;
        let parent_tree = parent.tree()?;

        // Compute the diff
        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;

        // Format the diff as a string with limits to prevent memory exhaustion
        let file_count = std::cell::RefCell::new(0usize);
        let line_count = std::cell::RefCell::new(0usize);
        let current_file_lines = std::cell::RefCell::new(0usize);
        let diff_lines = RefCell::new(Vec::new());
        let mut diff_stats = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                let mut file_count = file_count.borrow_mut();
                if *file_count >= MAX_FILES_TO_SHOW {
                    return false;
                }

                let file_path = delta
                    .new_file()
                    .path()
                    .or(delta.old_file().path())
                    .and_then(|p| p.to_str())
                    .unwrap_or("unknown");

                match delta.status() {
                    git2::Delta::Added => diff_stats.push(format!("+++ a/{}", file_path)),
                    git2::Delta::Deleted => diff_stats.push(format!("--- a/{}", file_path)),
                    git2::Delta::Modified => {
                        diff_stats.push(format!("--- a/{}", file_path));
                        diff_stats.push(format!("+++ b/{}", file_path));
                    }
                    git2::Delta::Renamed => {
                        diff_stats.push(format!(
                            "--- a/{}",
                            delta
                                .old_file()
                                .path()
                                .and_then(|p| p.to_str())
                                .unwrap_or("unknown")
                        ));
                        diff_stats.push(format!("+++ b/{}", file_path));
                    }
                    git2::Delta::Copied => {
                        diff_stats.push(format!(
                            "--- a/{}",
                            delta
                                .old_file()
                                .path()
                                .and_then(|p| p.to_str())
                                .unwrap_or("unknown")
                        ));
                        diff_stats.push(format!("+++ b/{}", file_path));
                    }
                    _ => {}
                }
                *file_count += 1;
                *current_file_lines.borrow_mut() = 0;
                true
            },
            None,
            Some(&mut |_: git2::DiffDelta<'_>, hunk: git2::DiffHunk<'_>| {
                let mut line_count = line_count.borrow_mut();
                if *line_count >= MAX_TOTAL_LINES {
                    return false;
                }
                *line_count += 1;
                diff_lines
                    .borrow_mut()
                    .push(format!("@@ {} @@", String::from_utf8_lossy(hunk.header())));
                true
            }),
            Some(&mut |_: git2::DiffDelta<'_>,
                       _: Option<git2::DiffHunk<'_>>,
                       line: git2::DiffLine<'_>| {
                let mut line_count = line_count.borrow_mut();
                let mut current_file_lines = current_file_lines.borrow_mut();

                if *line_count >= MAX_TOTAL_LINES || *current_file_lines >= MAX_LINES_PER_FILE {
                    return false;
                }

                let line_content = String::from_utf8_lossy(line.content());
                let origin = line.origin();
                let prefix = match origin {
                    '+' => "+",
                    '-' => "-",
                    ' ' => " ",
                    _ => " ",
                };
                diff_lines
                    .borrow_mut()
                    .push(format!("{}{}", prefix, line_content.trim_end()));

                *line_count += 1;
                *current_file_lines += 1;
                true
            }),
        )?;

        // Combine stats and lines
        let mut result = Vec::new();
        result.extend(diff_stats);
        if !result.is_empty() {
            result.push("".to_string());
        }

        let final_lines = diff_lines.into_inner();
        result.extend(final_lines);

        // Add truncation message if limits were hit
        let file_count = file_count.into_inner();
        let line_count = line_count.into_inner();

        if file_count >= MAX_FILES_TO_SHOW {
            result.push("".to_string());
            result.push(format!(
                "... (showing first {} files, diff truncated)",
                MAX_FILES_TO_SHOW
            ));
        }

        if line_count >= MAX_TOTAL_LINES {
            result.push("".to_string());
            result.push(format!(
                "... (showing first {} lines, diff truncated)",
                MAX_TOTAL_LINES
            ));
        }

        Ok(result.join("\n"))
    }

    pub fn compute_diff_between_commits(
        &self,
        old_oid: &git2::Oid,
        new_oid: &git2::Oid,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let repo = self.repo.borrow();
        let repo = repo.as_ref().ok_or("No repository loaded")?;

        let old_commit = repo.find_commit(*old_oid)?;
        let new_commit = repo.find_commit(*new_oid)?;

        let old_tree = old_commit.tree()?;
        let new_tree = new_commit.tree()?;

        let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?;

        // Format the diff as a string with limits to prevent memory exhaustion
        let file_count = std::cell::RefCell::new(0usize);
        let line_count = std::cell::RefCell::new(0usize);
        let current_file_lines = std::cell::RefCell::new(0usize);
        let diff_lines = RefCell::new(Vec::new());
        let mut diff_stats = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                let mut file_count = file_count.borrow_mut();
                if *file_count >= MAX_FILES_TO_SHOW {
                    return false;
                }

                let file_path = delta
                    .new_file()
                    .path()
                    .or(delta.old_file().path())
                    .and_then(|p| p.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                match delta.status() {
                    git2::Delta::Added => diff_stats.push(format!("+++ a/{}", file_path)),
                    git2::Delta::Deleted => diff_stats.push(format!("--- a/{}", file_path)),
                    git2::Delta::Modified => {
                        diff_stats.push(format!("--- a/{}", file_path));
                        diff_stats.push(format!("+++ b/{}", file_path));
                    }
                    git2::Delta::Renamed => {
                        diff_stats.push(format!(
                            "--- a/{}",
                            delta
                                .old_file()
                                .path()
                                .and_then(|p| p.to_str())
                                .unwrap_or("unknown")
                        ));
                        diff_stats.push(format!("+++ b/{}", file_path));
                    }
                    git2::Delta::Copied => {
                        diff_stats.push(format!(
                            "--- a/{}",
                            delta
                                .old_file()
                                .path()
                                .and_then(|p| p.to_str())
                                .unwrap_or("unknown")
                        ));
                        diff_stats.push(format!("+++ b/{}", file_path));
                    }
                    _ => {}
                }
                *file_count += 1;
                *current_file_lines.borrow_mut() = 0;
                true
            },
            None,
            Some(&mut |_: git2::DiffDelta<'_>, hunk: git2::DiffHunk<'_>| {
                let mut line_count = line_count.borrow_mut();
                if *line_count >= MAX_TOTAL_LINES {
                    return false;
                }
                *line_count += 1;
                diff_lines
                    .borrow_mut()
                    .push(format!("@@ {} @@", String::from_utf8_lossy(hunk.header())));
                true
            }),
            Some(&mut |_: git2::DiffDelta<'_>,
                       _: Option<git2::DiffHunk<'_>>,
                       line: git2::DiffLine<'_>| {
                let mut line_count = line_count.borrow_mut();
                let mut current_file_lines = current_file_lines.borrow_mut();

                if *line_count >= MAX_TOTAL_LINES || *current_file_lines >= MAX_LINES_PER_FILE {
                    return false;
                }

                let line_content = String::from_utf8_lossy(line.content());
                let origin = line.origin();
                let prefix = match origin {
                    '+' => "+",
                    '-' => "-",
                    ' ' => " ",
                    _ => " ",
                };
                diff_lines
                    .borrow_mut()
                    .push(format!("{}{}", prefix, line_content.trim_end()));

                *line_count += 1;
                *current_file_lines += 1;
                true
            }),
        )?;

        // Combine stats and lines
        let mut result = Vec::new();
        result.extend(diff_stats);
        if !result.is_empty() {
            result.push("".to_string());
        }

        let final_lines = diff_lines.into_inner();
        result.extend(final_lines);

        // Add truncation message if limits were hit
        let file_count = file_count.into_inner();
        let line_count = line_count.into_inner();

        if file_count >= MAX_FILES_TO_SHOW {
            result.push("".to_string());
            result.push(format!(
                "... (showing first {} files, diff truncated)",
                MAX_FILES_TO_SHOW
            ));
        }

        if line_count >= MAX_TOTAL_LINES {
            result.push("".to_string());
            result.push(format!(
                "... (showing first {} lines, diff truncated)",
                MAX_TOTAL_LINES
            ));
        }

        Ok(result.join("\n"))
    }

    pub fn get_changed_files(
        &self,
        oid: &git2::Oid,
    ) -> Result<Vec<ChangedFile>, Box<dyn std::error::Error>> {
        let repo = self.repo.borrow();
        let repo = repo.as_ref().ok_or("No repository loaded")?;

        let commit = repo.find_commit(*oid)?;
        let parents: Vec<git2::Commit> = commit.parents().collect();

        let mut changed_files = Vec::new();

        if parents.is_empty() {
            // Initial commit - get all files as added
            let tree = commit.tree()?;
            for entry in tree.iter() {
                if let Some(name) = entry.name() {
                    changed_files.push(ChangedFile {
                        path: name.to_string(),
                        status: git2::Delta::Added,
                        old_oid: None,
                        new_oid: Some(oid.clone()),
                    });
                }
            }
        } else {
            // Get diff with first parent
            let parent = &parents[0];
            let commit_tree = commit.tree()?;
            let parent_tree = parent.tree()?;
            let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;

            // Log before processing diff
            eprintln!("Processing diff for commit: {}", oid);

            let diff_result = diff.foreach(
                &mut |delta, _| {
                    // Log delta status
                    eprintln!("Processing delta with status: {:?}", delta.status());

                    // Safely extract file path with detailed logging
                    let file_path = match delta.new_file().path().or(delta.old_file().path()) {
                        Some(path) => match path.to_str() {
                            Some(s) => {
                                eprintln!("Extracted file path: {}", s);
                                s.to_string()
                            }
                            None => {
                                eprintln!(
                                    "Warning: File path contains invalid UTF-8, using 'unknown'"
                                );
                                "unknown".to_string()
                            }
                        },
                        None => {
                            eprintln!("Warning: No file path available, using 'unknown'");
                            "unknown".to_string()
                        }
                    };

                    // Safely extract old OID
                    let old_oid = {
                        let old_file = delta.old_file();
                        let oid = old_file.id();
                        if oid.is_zero() {
                            eprintln!("Old OID is zero");
                            None
                        } else {
                            eprintln!("Old OID: {}", oid);
                            Some(oid)
                        }
                    };

                    // Safely extract new OID
                    let new_oid = {
                        let new_file = delta.new_file();
                        let oid = new_file.id();
                        if oid.is_zero() {
                            eprintln!("New OID is zero");
                            None
                        } else {
                            eprintln!("New OID: {}", oid);
                            Some(oid)
                        }
                    };

                    eprintln!(
                        "Adding changed file: {} (status: {:?})",
                        file_path,
                        delta.status()
                    );

                    changed_files.push(ChangedFile {
                        path: file_path,
                        status: delta.status(),
                        old_oid,
                        new_oid,
                    });
                    true
                },
                None,
                None,
                None,
            );

            match diff_result {
                Ok(_) => {
                    eprintln!(
                        "Successfully processed diff, found {} changed files",
                        changed_files.len()
                    );
                }
                Err(e) => {
                    eprintln!("Error processing diff: {}", e);
                    return Err(format!("Failed to process diff: {}", e).into());
                }
            }
        }

        Ok(changed_files)
    }

    pub fn compute_file_diff(
        &self,
        commit_oid: &git2::Oid,
        file_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        const MAX_FILE_DIFF_LINES: usize = 200;

        let repo = self.repo.borrow();
        let repo = repo.as_ref().ok_or("No repository loaded")?;

        let commit = repo.find_commit(*commit_oid)?;
        let commit_tree = commit.tree()?;

        let diff = match commit.parent(0) {
            Ok(parent) => {
                // Has parent - compute diff with parent
                let parent_tree = parent.tree()?;
                repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?
            }
            Err(_) => {
                // Initial commit - compute diff with empty tree
                repo.diff_tree_to_tree(None, Some(&commit_tree), None)?
            }
        };

        let diff_lines = RefCell::new(Vec::new());
        let line_count = RefCell::new(0usize);

        let target_path = file_path.to_string();

        // Process diff with error handling
        let result = diff.foreach(
            &mut |delta, _| {
                let current_path = match delta.new_file().path().or(delta.old_file().path()) {
                    Some(path) => match path.to_str() {
                        Some(s) => s.to_string(),
                        None => return true, // Skip files with invalid UTF-8 paths
                    },
                    None => return true, // Skip files with no path
                };

                // Only process diff if this is the target file
                if current_path != target_path {
                    return true;
                }

                match delta.status() {
                    git2::Delta::Added => diff_lines
                        .borrow_mut()
                        .push(format!("+++ a/{}", current_path)),
                    git2::Delta::Deleted => diff_lines
                        .borrow_mut()
                        .push(format!("--- a/{}", current_path)),
                    git2::Delta::Modified => {
                        diff_lines
                            .borrow_mut()
                            .push(format!("--- a/{}", current_path));
                        diff_lines
                            .borrow_mut()
                            .push(format!("+++ b/{}", current_path));
                    }
                    git2::Delta::Renamed => {
                        let old_path = match delta.old_file().path() {
                            Some(path) => match path.to_str() {
                                Some(s) => s.to_string(),
                                None => current_path.clone(), // Fallback to current path
                            },
                            None => current_path.clone(), // Fallback to current path
                        };
                        diff_lines.borrow_mut().push(format!("--- a/{}", old_path));
                        diff_lines
                            .borrow_mut()
                            .push(format!("+++ b/{}", current_path));
                    }
                    _ => {}
                }
                true
            },
            None,
            Some(&mut |_: git2::DiffDelta<'_>, hunk: git2::DiffHunk<'_>| {
                let line_count = line_count.borrow_mut();
                if *line_count >= MAX_FILE_DIFF_LINES {
                    return false;
                }
                diff_lines
                    .borrow_mut()
                    .push(format!("@@ {} @@", String::from_utf8_lossy(hunk.header())));
                true
            }),
            Some(&mut |_: git2::DiffDelta<'_>,
                       _: Option<git2::DiffHunk<'_>>,
                       line: git2::DiffLine<'_>| {
                let mut line_count = line_count.borrow_mut();
                if *line_count >= MAX_FILE_DIFF_LINES {
                    return false;
                }

                let line_content = String::from_utf8_lossy(line.content());
                let origin = line.origin();
                let prefix = match origin {
                    '+' => "+",
                    '-' => "-",
                    ' ' => " ",
                    _ => " ",
                };
                diff_lines
                    .borrow_mut()
                    .push(format!("{}{}", prefix, line_content.trim_end()));

                *line_count += 1;
                true
            }),
        );

        // Handle any errors from diff.foreach()
        match result {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error processing diff for file {}: {}", file_path, e);
                return Err(format!("Failed to compute diff for file {}: {}", file_path, e).into());
            }
        }

        let result = diff_lines.into_inner();

        // Add truncation message if limit was hit
        let final_result = if line_count.into_inner() >= MAX_FILE_DIFF_LINES {
            let mut truncated = result;
            truncated.push("".to_string());
            truncated.push(format!(
                "... (showing first {} lines, diff truncated)",
                MAX_FILE_DIFF_LINES
            ));
            truncated
        } else {
            result
        };

        Ok(final_result.join("\n"))
    }

    /* ---------------- compute graph (loop เดียว) ---------------- */

    fn recompute(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.max_lane = 0;

        let repo = self.repo.borrow();
        let Some(repo) = repo.as_ref() else {
            return;
        };
        let mut revwalk = repo.revwalk().unwrap();
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
            let commit = repo.find_commit(oid).unwrap();
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

            let current_edge_point = Point::new(pos.x + SIZE / 2.0, pos.y + SIZE / 2.0);

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
}

impl EventEmitter<CommitSelected> for Garph {}

impl EventEmitter<RepoPathChanged> for Garph {}

impl Render for Garph {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.recompute();

        let has_repo = self.repo.borrow().is_some();
        let nodes = self.nodes.clone();
        let edges = self.edges.clone();
        let height = self.content_height;
        let max_lane = self.max_lane;

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .id("garph")
            .overflow_scroll()
            .bg(gpui::rgb(0x282828))
            .when(!has_repo, |div1| {
                div1.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_4()
                        .child(
                            div()
                                .text_color(gpui::rgb(0x969696))
                                .text_size(px(14.0))
                                .child("No repository loaded"),
                        )
                        .child(
                            div()
                                .px(px(20.0))
                                .py(px(10.0))
                                .bg(gpui::rgb(0x4A90D9))
                                .rounded(px(6.0))
                                .text_color(gpui::white())
                                .text_size(px(14.0))
                                .cursor_pointer()
                                .hover(|style| style.bg(gpui::rgb(0x357ABD)))
                                .child("Set Path File")
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |_this, _event, window, cx| {
                                        window.dispatch_action(
                                            Box::new(crate::actions::OpenFile),
                                            cx,
                                        );
                                    }),
                                ),
                        ),
                )
            })
            // .absolute()
            // .relative()
            .child(
                div()
                    .absolute()
                    .w_full()
                    // can't use h_full because container isn't overflowed.
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
                    .child(div().children(nodes.iter().map(|n| {
                        let message = Self::clean_message(&n.message);
                        let oid = n.oid;
                        let message_text = n.message.clone();
                        let author_text = n.author.clone();
                        let timestamp = n.timestamp;
                        let parents = n.parents.clone();

                        // Calculate text position based on max lane to ensure no overlap
                        let container_text_left = START_X + (max_lane as f32) * LANE_WIDTH;

                        div()
                            .absolute()
                            .top(n.position.y)
                            .left(px(0.0))
                            .right(px(0.0))
                            .h(px(COMMIT_HEIGHT))
                            .flex()
                            .flex_row()
                            .items_center()
                            .group("commit-row")
                            .hover(|style| style.bg(gpui::hsla(0.0, 0.0, 0.22, 0.3)))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _event, _window, cx| {
                                    cx.emit(CommitSelected {
                                        oid,
                                        message: message_text.clone(),
                                        author: author_text.clone(),
                                        timestamp,
                                        parents: parents.clone(),
                                    });
                                }),
                            )
                            // node
                            .child(
                                div()
                                    .left(n.position.x)
                                    .size(SIZE)
                                    .bg(gpui::rgb(VEC_COLORS[n.color]))
                                    .border_color(gpui::black())
                                    .rounded(px(5.0))
                                    .group_hover("commit-row", |style| style.size(SIZE + px(20.0))),
                            )
                            // text
                            .child(
                                div()
                                    .left(px(container_text_left))
                                    .px(px(10.0))
                                    .py(px(5.0))
                                    .rounded(px(4.0))
                                    .text_color(gpui::rgb(0x969696))
                                    .text_size(px(10.0))
                                    .line_clamp(1)
                                    .child(format!("{}", message)),
                            )
                    }))),
            )
    }
}
