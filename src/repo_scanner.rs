use std::fs;
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 5;
const GIT_DIR: &str = ".git";

const SKIP_DIRS: &[&str] = &[
    ".cargo",
    ".cache",
    ".config",
    ".local",
    ".rustup",
    ".npm",
    ".nvm",
    ".vscode",
    ".vscode-server",
    ".mozilla",
    ".thunderbird",
    ".gradle",
    ".m2",
    ".android",
    "node_modules",
    "target",
    "build",
    "dist",
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    ".idea",
];

fn default_scan_root() -> PathBuf {
    dirs_home()
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

pub fn scan_git_repos_from(root: &Path) -> Vec<String> {
    let mut repos = Vec::new();
    walk_for_git(root, 0, &mut repos);
    repos.sort();
    repos
}

pub fn scan_git_repos() -> Vec<String> {
    let root = default_scan_root();
    scan_git_repos_from(&root)
}

fn walk_for_git(dir: &Path, depth: usize, out: &mut Vec<String>) {
    if depth > MAX_DEPTH {
        return;
    }
    if dir.join(GIT_DIR).is_dir() {
        if let Some(path_str) = dir.to_str() {
            out.push(path_str.to_string());
        }
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || SKIP_DIRS.contains(&name) {
                continue;
            }
            walk_for_git(&path, depth + 1, out);
        }
    }
}

pub fn short_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
}
