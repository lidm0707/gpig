use dark_pig_git::entities::commit::CommitNode;
use dark_pig_git::entities::garph::Garph;
use dark_pig_git::entities::lane::LaneManager;
use dotenv::dotenv;
use git2::Oid;
use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};
use std::collections::HashMap;
use std::env;
use std::error::Error;
const START_X: f32 = 10.0;
const LANE_WIDTH: f32 = 5.0;
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

    // Second pass: Assign lanes and calculate positions
    for (index, commit_node) in commits.iter_mut().enumerate() {
        let lane_id = lane_manager.assign_commit(&commit_node.oid, &commit_node.parents);
        let lane_position = lane_id as f32;

        // Calculate position based on lane and index
        commit_node.position = (
            START_X - (index as f32 * COMMIT_HEIGHT),
            LANE_WIDTH * lane_position,
        );
    }

    println!(
        "Processed {} commits with {} lanes",
        commits.len(),
        lane_manager.lanes.len()
    );

    let garph = Garph::new(commits);

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| garph),
        )
        .unwrap();
    });
    Ok(())
}
