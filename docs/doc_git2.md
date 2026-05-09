# git2 Crate — Practical Guide

> `git2` is Rust bindings for `libgit2`. It gives you full control over Git operations
> without shelling out to the `git` CLI. Version used in this project: **0.20.3**.

---

## Table of Contents

1. [Setup](#1-setup)
2. [Open a Repository](#2-open-a-repository)
3. [Walk Commits (Revwalk)](#3-walk-commits-revwalk)
4. [Read Commit Info](#4-read-commit-info)
5. [Diff Between Commits](#5-diff-between-commits)
6. [Diff with foreach Callbacks](#6-diff-with-foreach-callbacks)
7. [Tree & Blob — Read File Content](#7-tree--blob--read-file-content)
8. [Detect Binary Files](#8-detect-binary-files)
9. [Get Changed Files per Commit](#9-get-changed-files-per-commit)
10. [Branch Operations](#10-branch-operations)
11. [Create a Commit](#11-create-a-commit)
12. [Remote — Fetch / Push](#12-remote--fetch--push)
13. [Status — Working Directory Changes](#13-status--working-directory-changes)
14. [Stash](#14-stash)
15. [Merge & Reset](#15-merge--reset)
16. [Tags](#16-tags)
17. [Blame](#17-blame)
18. [Config](#18-config)
19. [Signature & Time](#19-signature--time)
20. [Real Patterns from This Project](#20-real-patterns-from-this-project)
21. [Common Pitfalls](#21-common-pitfalls)

---

## 1. Setup

```toml
# Cargo.toml
[dependencies]
git2 = "0.20"
```

```rust
use git2::{Repository, Oid, Commit, Diff, Delta, Time, Signature, Branch, Tag, Stash};
```

---

## 2. Open a Repository

```rust
// Open existing repo (path can be ".git" or the workdir root)
let repo = Repository::open("/path/to/repo")?;

// Auto-discover repo from any subdirectory
let repo = Repository::discover("/path/to/any/subdir")?;

// Open bare repo (no working tree)
let repo = Repository::open_bare("/path/to/bare.git")?;

// Init new repo
let repo = Repository::init("/path/to/new-repo")?;

// Init bare repo
let repo = Repository::init_bare("/path/to/new-bare.git")?;
```

**Check if repo is bare:**

```rust
if repo.is_empty()? {
    println!("No commits yet");
}
```

---

## 3. Walk Commits (Revwalk)

`Revwalk` iterates commit OIDs in topological or time order.

```rust
let repo = Repository::open(".")?;

let mut revwalk = repo.revwalk()?;
revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
revwalk.push_head()?; // start from HEAD

for oid in revwalk.take(50) {
    let oid = oid?;
    let commit = repo.find_commit(oid)?;
    println!("{} {}", &oid.to_string()[..7], commit.message().unwrap_or(""));
}
```

**Push other starting points:**

```rust
revwalk.push_ref("refs/heads/main")?;      // from a branch
revwalk.push(oid)?;                          // from a specific OID
revwalk.push_range("HEAD~10..HEAD")?;        // a range
revwalk.push_glob("refs/tags/v*")?;          // all matching refs
revwalk.hide_ref("refs/heads/dev")?;         // exclude a branch
```

---

## 4. Read Commit Info

```rust
let commit = repo.find_commit(oid)?;

// SHA-1 hash
let id: Oid = commit.id();

// Author & committer
let author: Signature = commit.author();
println!("{} <{}>", author.name().unwrap_or(""), author.email().unwrap_or(""));

let committer: Signature = commit.committer();

// Message
let full: &str = commit.message().unwrap_or("");     // full body
let subject: &str = commit.summary().unwrap_or("");   // first line only

// Timestamp
let time: Time = commit.time();
let ts_seconds: i64 = time.seconds();
let ts_offset: i32 = time.offset_minutes();

// Parents
let parent_count = commit.parent_count();
let parent_ids: ParentIds = commit.parent_ids(); // iterator of Oid
let first_parent: Commit = commit.parent(0)?;

// Tree
let tree: Tree = commit.tree()?;
```

---

## 5. Diff Between Commits

Compare two tree snapshots:

```rust
let old_commit = repo.find_commit(old_oid)?;
let new_commit = repo.find_commit(new_oid)?;

let old_tree = old_commit.tree()?;
let new_tree = new_commit.tree()?;

let diff = repo.diff_tree_to_tree(
    Some(&old_tree),   // old (None = empty tree)
    Some(&new_tree),   // new
    None,              // DiffOptions
)?;
```

**Diff types available:**

```rust
// Working directory vs index
let diff = repo.diff_index_to_workdir(None, None)?;

// Index vs HEAD
let diff = repo.diff_tree_to_index(repo.index()?.as_tree(), None, None)?;

// Single tree (like initial commit)
let diff = repo.diff_tree_to_tree(None, Some(&tree), None)?;
```

**Read diff stats:**

```rust
let stats = diff.stats()?;
println!("files: {}, insertions: {}, deletions: {}",
    stats.files_changed(),
    stats.insertions(),
    stats.deletions()
);
```

---

## 6. Diff with foreach Callbacks

The `foreach` method gives you fine-grained control over diff output.
This is the pattern used heavily in this project.

```rust
diff.foreach(
    // delta callback — called once per file
    &mut |delta: DiffDelta, _progress: f32| -> bool {
        let status: Delta = delta.status();
        let path = delta.new_file().path()
            .or(delta.old_file().path())
            .and_then(|p| p.to_str())
            .unwrap_or("unknown");

        match status {
            Delta::Added    => println!("+++ {}", path),
            Delta::Deleted  => println!("--- {}", path),
            Delta::Modified => println!("M   {}", path),
            Delta::Renamed  => println!("R   {} -> {}", 
                delta.old_file().path().unwrap().display(), path),
            _ => {}
        }
        true // continue
    },
    // hunk callback — called per hunk (optional)
    None,
    // line callback — called per diff line (optional)
    Some(&mut |_delta, _hunk: Option<DiffHunk>, line: DiffLine| -> bool {
        let origin = line.origin(); // '+', '-', ' '
        let content = String::from_utf8_lossy(line.content());
        print!("{}{}", origin, content);
        true
    }),
)?;
```

**Practical limit pattern** (from this project):

```rust
const MAX_TOTAL_LINES: usize = 500;
const MAX_FILES_TO_SHOW: usize = 100;
const MAX_LINES_PER_FILE: usize = 200;

let line_count = RefCell::new(0usize);
let file_count = RefCell::new(0usize);

diff.foreach(
    &mut |delta, _| {
        let mut fc = file_count.borrow_mut();
        if *fc >= MAX_FILES_TO_SHOW { return false; }
        *fc += 1;
        true
    },
    None,
    Some(&mut |_, _, line: DiffLine| {
        let mut lc = line_count.borrow_mut();
        if *lc >= MAX_TOTAL_LINES { return false; }
        let content = String::from_utf8_lossy(line.content());
        println!("{}{}", line.origin(), content.trim_end());
        *lc += 1;
        true
    }),
)?;
```

---

## 7. Tree & Blob — Read File Content

Navigate the git tree to read any file at any commit:

```rust
let commit = repo.find_commit(oid)?;
let tree = commit.tree()?;

// Get entry by path
let entry = tree.get_path(Path::new("src/main.rs"))?;
let object = entry.to_object(&repo)?;
let blob = object.as_blob().unwrap();

let content = blob.content(); // &[u8]
let size = blob.size();       // usize
```

**Iterate all entries in a tree:**

```rust
for entry in tree.iter() {
    let name = entry.name().unwrap_or("");
    let kind = entry.kind();
    println!("{:?} {}", kind, name);
}
```

**Recursively walk a tree:**

```rust
tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
    println!("{}{}", dir, entry.name().unwrap_or("?"));
    git2::TreeWalkResult::Ok
})?;
```

---

## 8. Detect Binary Files

Git's heuristic: check first 8KB for null bytes.

```rust
const MAX_FILE_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const BINARY_CHECK_BYTES: usize = 8000;

fn is_binary_file(repo: &Repository, commit: &Commit, file_path: &str) -> bool {
    let tree = match commit.tree() {
        Ok(t) => t,
        Err(_) => return false,
    };
    let entry = match tree.get_path(Path::new(file_path)) {
        Ok(e) => e,
        Err(_) => return false,
    };
    let object = match entry.to_object(repo) {
        Ok(o) => o,
        Err(_) => return false,
    };
    match object.as_blob() {
        Some(blob) => {
            if blob.size() > MAX_FILE_SIZE_BYTES {
                return true;
            }
            let content = blob.content();
            let check_bytes = std::cmp::min(content.len(), BINARY_CHECK_BYTES);
            content[..check_bytes].contains(&0)
        }
        None => false,
    }
}
```

---

## 9. Get Changed Files per Commit

Extract the list of files changed in a commit with their status:

```rust
use git2::{Delta, Oid};

struct ChangedFile {
    path: String,
    status: Delta,
    old_oid: Option<Oid>,
    new_oid: Option<Oid>,
}

fn get_changed_files(
    repo: &Repository,
    oid: &Oid,
) -> Result<Vec<ChangedFile>, Box<dyn std::error::Error>> {
    let commit = repo.find_commit(*oid)?;
    let parents: Vec<git2::Commit> = commit.parents().collect();
    let mut files = Vec::new();

    if parents.is_empty() {
        let tree = commit.tree()?;
        for entry in tree.iter() {
            if let Some(name) = entry.name() {
                files.push(ChangedFile {
                    path: name.to_string(),
                    status: Delta::Added,
                    old_oid: None,
                    new_oid: Some(*oid),
                });
            }
        }
        return Ok(files);
    }

    let parent_tree = parents[0].tree()?;
    let commit_tree = commit.tree()?;
    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;

    diff.foreach(
        &mut |delta, _| {
            let path = delta.new_file().path()
                .or(delta.old_file().path())
                .and_then(|p| p.to_str())
                .unwrap_or("unknown")
                .to_string();

            let old_oid = {
                let id = delta.old_file().id();
                if id.is_zero() { None } else { Some(id) }
            };
            let new_oid = {
                let id = delta.new_file().id();
                if id.is_zero() { None } else { Some(id) }
            };

            files.push(ChangedFile {
                path,
                status: delta.status(),
                old_oid,
                new_oid,
            });
            true
        },
        None, None, None,
    )?;

    Ok(files)
}
```

---

## 10. Branch Operations

```rust
// List all local branches
let branches = repo.branches(Some(git2::BranchType::Local))?;
for branch in branches {
    let (branch, _bt) = branch?;
    let name = branch.name()?.unwrap_or("");
    let oid = branch.get().target().unwrap();
    println!("{} => {}", name, oid);
}

// Create a new branch at HEAD
let head = repo.head()?.target().unwrap();
let commit = repo.find_commit(head)?;
let branch = repo.branch("feature-x", &commit, false)?;

// Checkout a branch
repo.set_head("refs/heads/feature-x")?;
repo.checkout_head(None)?;

// Delete a branch
let mut branch = repo.find_branch("old-branch", git2::BranchType::Local)?;
branch.delete()?;
```

---

## 11. Create a Commit

```rust
let sig = Signature::now("Name", "email@example.com")?;

// Stage files
let mut index = repo.index()?;
index.add_path(Path::new("src/main.rs"))?;
index.add_path(Path::new("Cargo.toml"))?;
index.write()?;

let tree_id = index.write_tree()?;
let tree = repo.find_tree(tree_id)?;

let head = repo.head()?.target().unwrap();
let parent = repo.find_commit(head)?;

let oid = repo.commit(
    Some("HEAD"),        // update ref
    &sig,                // author
    &sig,                // committer
    "feat: add new feature",
    &tree,               // tree snapshot
    &[&parent],          // parents
)?;

println!("new commit: {}", oid);
```

---

## 12. Remote — Fetch / Push

```rust
// List remotes
let remotes = repo.remotes()?;
for remote in &remotes {
    if let Some(name) = remote {
        println!("remote: {}", name);
    }
}

// Fetch
let mut remote = repo.find_remote("origin")?;
remote.fetch(&["main"], None, None)?;

// Pull = fetch + merge
let fetch_head = repo.find_reference("FETCH_HEAD")?;
let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
let analysis = repo.merge_analysis(&[&fetch_commit])?;

if analysis.0.is_up_to_date() {
    // already up to date
} else if analysis.0.is_fast_forward() {
    let refname = "refs/heads/main";
    repo.reference_delta(refname, &fetch_head, true, None)?;
    repo.set_head(refname)?;
    repo.checkout_head(None)?;
}

// Push
let mut remote = repo.find_remote("origin")?;
remote.push(&["refs/heads/main"], None)?;
```

---

## 13. Status — Working Directory Changes

```rust
let statuses = repo.statuses(None)?;

for entry in statuses.iter() {
    let path = entry.path().unwrap_or("?");
    let status = entry.status();
    let mut label = String::new();

    if status.is_index_new()       { label.push('+'); }
    if status.is_index_modified()  { label.push('M'); }
    if status.is_index_deleted()   { label.push('-'); }
    if status.is_wt_new()          { label.push('?'); }
    if status.is_wt_modified()     { label.push('m'); }
    if status.is_wt_deleted()      { label.push('d'); }
    if status.is_conflicted()      { label.push('C'); }

    println!("{} {}", label, path);
}
```

---

## 14. Stash

```rust
let sig = Signature::now("Name", "email@example.com")?;

// Save stash
repo.stash_save(&sig, "WIP: work in progress", None)?;

// List stashes
repo.stash_foreach(|index, msg, _oid| {
    println!("stash@{{{}}}: {}", index, msg);
    true
})?;

// Apply latest stash
repo.stash_pop(0, None)?;
```

---

## 15. Merge & Reset

```rust
// Merge a branch into HEAD
let fetch_commit = repo.find_commit(other_oid)?;
let annotated = repo.reference_to_annotated_commit(&fetch_commit)?;
let analysis = repo.merge_analysis(&[&annotated])?;

if analysis.0.is_fast_forward() {
    repo.checkout_tree(&fetch_commit.as_object(), None)?;
    repo.head()?.set_target(fetch_commit.id(), "fast-forward")?;
} else {
    repo.merge(&[&annotated], None, None)?;
    // resolve conflicts then commit...
}

// Reset types
repo.reset(commit.as_object(), git2::ResetType::Soft, None)?;   // keep index + workdir
repo.reset(commit.as_object(), git2::ResetType::Mixed, None)?;  // keep workdir only
repo.reset(commit.as_object(), git2::ResetType::Hard, None)?;   // discard all
```

---

## 16. Tags

```rust
// Lightweight tag
repo.reference("refs/tags/v1.0", commit.id(), false, "create tag v1.0")?;

// Annotated tag
let tagger = Signature::now("Name", "email@example.com")?;
repo.tag("v1.0", &commit.as_object(), &tagger, "Release 1.0", false)?;

// List tags
repo.tag_foreach(|oid, name| {
    println!("{} => {}", String::from_utf8_lossy(name), oid);
    true
})?;

// Delete tag
repo.find_reference("refs/tags/v1.0")?.delete()?;
```

---

## 17. Blame

Find which commit last modified each line of a file:

```rust
let blame = repo.blame_file(Path::new("src/main.rs"), None)?;

for hunk in &blame {
    let sig = hunk.final_signature();
    let line_no = hunk.final_start_line();
    let lines = hunk.lines_in_hunk();
    let commit_id = hunk.final_commit_id();
    println!(
        "L{}-{} ({}) {} <{}>",
        line_no,
        line_no + lines - 1,
        &commit_id.to_string()[..7],
        sig.name().unwrap_or(""),
        sig.email().unwrap_or("")
    );
}
```

---

## 18. Config

```rust
// Read config
let config = repo.config()?;
let name = config.get_string("user.name")?;
let email = config.get_string("user.email")?;

// Set config
config.set_string("user.name", "New Name")?;
config.set_string("user.email", "new@email.com")?;
```

---

## 19. Signature & Time

```rust
// Create signature
let sig = Signature::now("Alice", "alice@example.com")?;

// From raw values
let time = Time::new(1700000000, 420); // unix seconds, UTC offset in minutes
let sig = Signature::new("Alice", "alice@example.com", &time)?;

// Read from commit
let commit = repo.find_commit(oid)?;
let author = commit.author();
let t: Time = commit.time();

// Convert to chrono
let dt = chrono::DateTime::from_timestamp(t.seconds(), 0).unwrap();
println!("{}", dt.format("%Y-%m-%d %H:%M"));
```

---

## 20. Real Patterns from This Project

This project (`gpig`) uses `git2` to build a **visual Git commit graph** with GPUI.
Here are the real patterns extracted from `src/garph.rs`.

### Pattern 1: Shared Repository with RefCell

```rust
use std::cell::RefCell;
use std::rc::Rc;

pub struct Garph {
    repo: Rc<RefCell<Option<Repository>>>,
}

pub fn update_repo(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(path)?;
    *self.repo.borrow_mut() = Some(repo);
    Ok(())
}
```

**Why**: `Repository` is `!Send + !Sync`. Wrapping in `Rc<RefCell<>>` lets
multiple parts of the same thread borrow the repo mutably or immutably.

### Pattern 2: Topological Revwalk for Graph Layout

```rust
let mut revwalk = repo.revwalk()?;
revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
revwalk.push_head()?;

for (index, oid) in revwalk.take(LIMIT_ROW).enumerate() {
    let oid = oid?;
    let commit = repo.find_commit(oid)?;
    let parents: Vec<Oid> = commit.parents().map(|p| p.id()).collect();
    // assign lane, draw edges, build CommitNode...
}
```

**Why**: Topological sort keeps branch lines visually separated.
Time sort gives chronological order as tiebreaker.

### Pattern 3: Diff with Limits via RefCell Counters

```rust
const MAX_TOTAL_LINES: usize = 500;
const MAX_FILES_TO_SHOW: usize = 100;
const MAX_LINES_PER_FILE: usize = 200;

let line_count = RefCell::new(0usize);
let file_count = RefCell::new(0usize);

diff.foreach(
    &mut |delta, _| {
        let mut fc = file_count.borrow_mut();
        if *fc >= MAX_FILES_TO_SHOW { return false; }
        *fc += 1;
        true
    },
    None,
    Some(&mut |_, hunk: DiffHunk| {
        diff_lines.borrow_mut().push(
            format!("@@ {} @@", String::from_utf8_lossy(hunk.header()))
        );
        true
    }),
    Some(&mut |_, _, line: DiffLine| {
        let mut lc = line_count.borrow_mut();
        if *lc >= MAX_TOTAL_LINES { return false; }
        let content = String::from_utf8_lossy(line.content());
        let prefix = match line.origin() {
            '+' => "+", '-' => "-", ' ' => " ", _ => " ",
        };
        diff_lines.borrow_mut().push(
            format!("{}{}", prefix, content.trim_end())
        );
        *lc += 1;
        true
    }),
)?;
```

**Why**: Returning `false` from any callback stops iteration early.
`RefCell` lets you mutate counters inside the closures.

### Pattern 4: Single-File Diff Filtering

```rust
let diff = repo.diff_tree_to_tree(
    Some(&parent_tree),
    Some(&commit_tree),
    None,
)?;

let target = "src/main.rs";
let mut found = false;

diff.foreach(
    &mut |delta, _| {
        let path = delta.new_file().path()
            .and_then(|p| p.to_str())
            .unwrap_or("");
        if path == target { found = true; }
        true // keep scanning
    },
    None,
    Some(&mut |_, _, line: DiffLine| {
        if found {
            // collect lines for this file only
        }
        true
    }),
)?;
```

### Pattern 5: Initial Commit Diff (Empty Tree)

```rust
if parents.is_empty() {
    let diff = repo.diff_tree_to_tree(
        None,                // empty tree (nothing before)
        Some(&commit_tree),  // current tree
        None,
    )?;
}
```

---

## 21. Common Pitfalls

| Pitfall | Fix |
|---|---|
| `Repository` is `!Send + !Sync` | Use `Rc<RefCell<Option<Repository>>>` on single thread, or `git2::Repository::open` per thread |
| `find_commit` returns error on non-commit OIDs | Check `repo.find_object(oid, None)?.as_commit()` for flexibility |
| `commit.message()` can return `None` | Always use `.unwrap_or("")` or `.unwrap_or_default()` |
| `Oid` from `delta.new_file().id()` can be zero | Check `oid.is_zero()` before using |
| `diff.foreach` captures mutably | Use `RefCell` for any data you need to mutate in closures |
| File paths can be non-UTF-8 | Use `.to_str()` with fallback, or `Path::display()` for display |
| Credentials for remote ops | Use `RemoteCallbacks` with `credentials` callback |
| Large repos are slow | Set `revwalk.simplify_first_parent()`, use `take(N)`, limit diff lines |
| libgit2 C dependency | May need `libgit2-dev` or `cmake` installed. Use `git2 = { version = "0.20", default-features = false, features = ["vendored"] }` to build from source |
