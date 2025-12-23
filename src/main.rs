use dark_pig_git::entities::commit::CommitNode;
use dark_pig_git::entities::edge::{Edge, EdgeManager};
use dark_pig_git::entities::garph::Garph;
use dark_pig_git::entities::lane::LaneManager;
use dotenv::dotenv;
use git2::Oid;
use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};
use std::collections::HashMap;
use std::env;
use std::error::Error;
const START_X: f32 = 30.0;
const LANE_WIDTH: f32 = 15.0;
const COMMIT_HEIGHT: f32 = 20.0;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let path_repo = env::var("GIT_REPO_PATH")?;
    let repo = git2::Repository::open(&path_repo)?;
    let mut rewalk = repo.revwalk()?;
    rewalk.push_head()?;
    let mut commits: Vec<CommitNode> = Vec::new();
    let mut map_oid: HashMap<Oid, usize> = HashMap::new();
    let mut lane_manager = LaneManager::new();
    let mut edge_manager = EdgeManager::new();
    // First pass: Collect all commits
    for (index, commit_oid) in rewalk.enumerate() {
        let commit_oid = commit_oid?;
        let commit = repo.find_commit(commit_oid)?;
        let parent_ids: Vec<Oid> = commit.parents().map(|parent| parent.id()).collect();

        let commit_node = CommitNode::new(
            commit.id(),
            commit.message().unwrap_or_default().to_string(),
            commit.author().email().unwrap_or_default().to_string(),
            commit.time(),
            parent_ids.clone(),
            (0.0, 0.0),
        );

        map_oid.insert(commit.id(), index);
        commits.push(commit_node);
    }

    // Sort commits by timestamp to ensure proper chronological order
    commits.sort_by(|a, b| b.timestamp.seconds().cmp(&a.timestamp.seconds()));
    map_oid.clear();
    for (index, commit) in commits.iter().enumerate() {
        map_oid.insert(commit.oid, index);
    }
    // Second pass: Assign lanes and calculate positions
    for (index, commit_node) in commits.iter_mut().enumerate() {
        let lane_id = lane_manager.assign_commit(&commit_node.oid, &commit_node.parents);
        let lane_position = lane_id as f32;

        // Calculate position based on lane and index
        commit_node.position = (
            START_X + (lane_position * LANE_WIDTH),
            COMMIT_HEIGHT * index as f32,
        );
    }

    // Third pass: Create edges between commits
    for commit_node in &commits {
        let from = gpui::Point::new(
            px(commit_node.position.0) + 5.0.into(), // 5.0 = 10/2 size of node
            px(commit_node.position.1),
        );

        for parent_oid in &commit_node.parents {
            if let Some(parent_index) = map_oid.get(parent_oid) {
                let parent = &commits[*parent_index];

                let to =
                    gpui::Point::new(px(parent.position.0) + 5.0.into(), px(parent.position.1)); // 5.0 = 10/2 size of node

                edge_manager.add(from, to);
            }
        }
    }

    println!(
        "Processed {} commits with {} lanes and {} at {:?}",
        commits.len(),
        lane_manager.lanes.len(),
        edge_manager.edges.len(),
        // edge_manager.edges,
        commits[commits.len() - 1]
    );

    let garph = Garph::new(commits, edge_manager);

    Application::new().run(|cx: &mut App| {
        // let bounds = Bounds::centered(None, size(px(1800.), px(800.0)), cx);

        cx.open_window(
            WindowOptions {
                // window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| garph),
        )
        .unwrap();
    });
    Ok(())
}
