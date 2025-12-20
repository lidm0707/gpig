use dark_pig_git::entities::commit::CommitNode;
use dotenv::dotenv;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let path_repo = env::var("GIT_REPO_PATH")?;
    let repo = git2::Repository::open(&path_repo)?;
    let mut rewalk = repo.revwalk()?;
    rewalk.push_head()?;
    let mut commits: Vec<CommitNode> = Vec::new();

    for commit_oid in rewalk {
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
    }

    println!("{:?}", commits[0]);

    Ok(())
}
