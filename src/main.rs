use dark_pig_git::entities::commit::CommitNode;
use dark_pig_git::entities::edge::EdgeManager;
use dark_pig_git::entities::garph::Garph;
use dark_pig_git::entities::lane::LaneManager;
use dotenv::dotenv;
use git2::Oid;
use gpui::{App, AppContext, Application, Pixels, Point, WindowOptions};
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
    rewalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
    rewalk.push_head()?;
    let mut commits: Vec<CommitNode> = Vec::new();
    // let mut map_oid: HashMap<Oid, usize> = HashMap::new();
    let mut map_oid: HashMap<Oid, Vec<Point<Pixels>>> = HashMap::new();

    // from
    let mut lane_manager = LaneManager::new();
    let mut edge_manager = EdgeManager::new();
    // First pass: Collect all commits
    for (index, commit_oid) in rewalk.enumerate() {
        let commit_oid = commit_oid?;
        let commit = repo.find_commit(commit_oid)?;
        let parent_ids: Vec<Oid> = commit.parents().map(|parent| parent.id()).collect();
        let lane_position = lane_manager.assign_commit(&commit_oid, &parent_ids) as f32;
        //
        let current_position: Point<Pixels> = Point::new(
            (START_X + (lane_position * LANE_WIDTH)).into(),
            (COMMIT_HEIGHT * index as f32).into(),
        );
        let current_edge = Point::new(current_position.x + 5.0.into(), current_position.y);
        if let Some(pixels) = map_oid.get(&commit_oid) {
            for px in pixels {
                edge_manager.add(px.clone(), current_edge);
            }
        }
        for parent_oid in &parent_ids {
            match map_oid.get_mut(&parent_oid) {
                Some(pixels) => {
                    pixels.push(current_position);
                }
                None => {
                    map_oid.insert(parent_oid.clone(), vec![current_edge]);
                }
            }
        }
        let commit_node = CommitNode::new(
            commit.id(),
            commit.message().unwrap_or_default().to_string(),
            commit.author().email().unwrap_or_default().to_string(),
            commit.time(),
            parent_ids.clone(),
            current_position,
        );
        commits.push(commit_node.clone());
    }

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
