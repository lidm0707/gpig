use git2::Repository;

use crate::branch::{BranchInfo, BranchReloadResult};
use crate::path_bar::RepoMode;
use crate::status_panel::{StatusEntry, StatusKind, StatusReloadResult};

pub struct PanelData {
    pub branches: BranchReloadResult,
    pub status: StatusReloadResult,
    pub branch_name: String,
    pub dirty_count: usize,
}

pub fn load_panel_data_bg(repo_path: &str, mode: &RepoMode) -> Result<PanelData, String> {
    let repo = Repository::open(repo_path).map_err(|e| e.to_string())?;

    let branches = load_branches(&repo, mode)?;
    let status = load_status(&repo)?;
    let branch_name = repo
        .head()
        .ok()
        .and_then(|r| r.shorthand().map(|s| s.to_string()))
        .unwrap_or_default();
    let dirty_count = status.entries.len();

    Ok(PanelData {
        branches,
        status,
        branch_name,
        dirty_count,
    })
}

fn load_branches(repo: &Repository, mode: &RepoMode) -> Result<BranchReloadResult, String> {
    let branch_type = match mode {
        RepoMode::Local => git2::BranchType::Local,
        RepoMode::Remote => git2::BranchType::Remote,
    };

    let head_name = repo
        .head()
        .ok()
        .and_then(|r| r.shorthand().map(|s| s.to_string()));

    let branches_iter = repo
        .branches(Some(branch_type))
        .map_err(|e| e.to_string())?;

    let mut branches = Vec::new();
    for branch in branches_iter.flatten() {
        let (b, _) = branch;
        let name = b.name().ok().flatten().unwrap_or("").to_string();
        let is_remote = matches!(mode, RepoMode::Remote);
        let is_head = !is_remote && head_name.as_ref() == Some(&name);
        branches.push(BranchInfo {
            name,
            is_head,
            is_remote,
        });
    }

    Ok(BranchReloadResult { branches })
}

fn load_status(repo: &Repository) -> Result<StatusReloadResult, String> {
    let statuses = repo.statuses(None).map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("?").to_string();
        let s = entry.status();

        let (staged, kind) = if s.is_conflicted() {
            (false, StatusKind::Conflicted)
        } else if s.is_index_new() {
            (true, StatusKind::New)
        } else if s.is_index_modified() {
            (true, StatusKind::Modified)
        } else if s.is_index_deleted() {
            (true, StatusKind::Deleted)
        } else if s.is_index_renamed() {
            (true, StatusKind::Renamed)
        } else if s.is_wt_new() {
            (false, StatusKind::Untracked)
        } else if s.is_wt_modified() {
            (false, StatusKind::Modified)
        } else if s.is_wt_deleted() {
            (false, StatusKind::Deleted)
        } else {
            continue;
        };

        entries.push(StatusEntry {
            path,
            staged,
            status_kind: kind,
        });
    }

    Ok(StatusReloadResult { entries })
}
