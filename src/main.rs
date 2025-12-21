use dark_pig_git::entities::commit::CommitNode;
use dark_pig_git::entities::lane::Lane;
use dotenv::dotenv;
use git2::Oid;
use std::collections::HashMap;
use std::env;
use std::error::Error;
const START: f32 = 10.0;
const WIDTH: f32 = 5.0;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let path_repo = env::var("GIT_REPO_PATH")?;
    let repo = git2::Repository::open(&path_repo)?;
    let mut rewalk = repo.revwalk()?;
    rewalk.push_head()?;
    let mut commits: Vec<CommitNode> = Vec::new();
    // why make new vec when loop in rewalk can find_commit and process in same time.
    let mut position: HashMap<Oid, f32> = HashMap::new();
    let mut lanes = Lane::new(vec![]);
    for (index, commit_oid) in rewalk.enumerate() {
        let commit_oid = commit_oid?;
        let commit = repo.find_commit(commit_oid)?;
        let commit_node = CommitNode::new(
            commit.id(),
            commit.message().unwrap_or_default().to_string(),
            commit.author().email().unwrap_or_default().to_string(),
            commit.time(),
            commit.parents().map(|parent| parent.id()).collect(),
        );
        commits.push(commit_node);
        if index == 0 {
            // render line && circle
            lanes.commits.push(Some(commit.id()));
            position.insert(commit.id(), START * index as f32);
            continue;
        }

        if lanes.commits.contains(&Some(commit.id())) {
            // render line && circle
            lanes.commits.push(Some(commit.id()));
            position.remove(&commit.id());
            // lane
            position.insert(commit.id(), START * index as f32);
            continue;
        }
    }

    println!("{:?}", commits[0]);

    Ok(())
}
