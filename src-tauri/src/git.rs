use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use serde::Serialize;
use tauri::State;

use crate::workspace::WorkspaceState;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitFile {
    pub path: String,
    pub index: String,
    pub worktree: String,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
    pub original_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub repo_root: String,
    pub branch: String,
    pub detached: bool,
    pub ahead: usize,
    pub behind: usize,
    pub clean: bool,
    pub files: Vec<GitFile>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub subject: String,
}

fn workspace_root(state: &WorkspaceState) -> Result<PathBuf, String> {
    state
        .root
        .lock()
        .map_err(|_| "workspace state poisoned".to_string())?
        .clone()
        .ok_or_else(|| "no workspace is open".to_string())
}

fn run_git(cwd: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))
}

fn output_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn error_text(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        stderr
    }
}

fn ensure_ok(output: &Output, action: &str) -> Result<(), String> {
    if output.status.success() {
        Ok(())
    } else {
        let detail = error_text(output);
        Err(if detail.is_empty() {
            format!("git {action} failed")
        } else {
            format!("git {action} failed: {detail}")
        })
    }
}

fn repo_root(root: &Path) -> Result<Option<PathBuf>, String> {
    let output = run_git(root, &["rev-parse", "--show-toplevel"])?;
    if !output.status.success() {
        return Ok(None);
    }
    let raw = output_text(&output);
    if raw.is_empty() {
        return Ok(None);
    }
    let candidate = PathBuf::from(raw)
        .canonicalize()
        .map_err(|e| format!("cannot resolve git repository root: {e}"))?;
    if candidate != root {
        return Err(
            "the open workspace is not the root of its git repository; open the repository root instead"
                .to_string(),
        );
    }
    Ok(Some(candidate))
}

fn require_repo(state: &WorkspaceState) -> Result<PathBuf, String> {
    let root = workspace_root(state)?;
    repo_root(&root)?.ok_or_else(|| "this workspace is not a git repository".to_string())
}

fn checked_paths(paths: Vec<String>) -> Result<Vec<String>, String> {
    if paths.is_empty() {
        return Err("no files were selected".to_string());
    }
    for path in &paths {
        let p = Path::new(path);
        if path.is_empty() || path.contains('\0') || p.is_absolute() {
            return Err("invalid file path".to_string());
        }
        if p.components()
            .any(|part| matches!(part, Component::ParentDir))
        {
            return Err("file path is outside the workspace".to_string());
        }
    }
    Ok(paths)
}

fn parse_porcelain(raw: &str) -> Vec<GitFile> {
    let mut files = Vec::new();
    let mut records = raw.split('\0').filter(|record| !record.is_empty());

    while let Some(record) = records.next() {
        if record.len() < 4 {
            continue;
        }
        let bytes = record.as_bytes();
        let index = bytes[0] as char;
        let worktree = bytes[1] as char;
        let path = record[3..].to_string();
        let untracked = index == '?' && worktree == '?';
        let ignored = index == '!' && worktree == '!';
        if ignored {
            continue;
        }
        let original_path = if matches!(index, 'R' | 'C') || matches!(worktree, 'R' | 'C') {
            records.next().map(str::to_string)
        } else {
            None
        };
        files.push(GitFile {
            path,
            index: index.to_string(),
            worktree: worktree.to_string(),
            staged: !untracked && index != ' ',
            unstaged: untracked || worktree != ' ',
            untracked,
            original_path,
        });
    }

    files
}

#[tauri::command]
pub fn git_status(state: State<'_, WorkspaceState>) -> Result<Option<GitStatus>, String> {
    let root = workspace_root(&state)?;
    let Some(repo) = repo_root(&root)? else {
        return Ok(None);
    };

    let status = run_git(&repo, &["status", "--porcelain=v1", "-z"])?;
    ensure_ok(&status, "status")?;
    let files = parse_porcelain(&String::from_utf8_lossy(&status.stdout));

    let branch_output = run_git(&repo, &["branch", "--show-current"])?;
    let mut branch = output_text(&branch_output);
    let mut detached = false;
    if branch.is_empty() {
        detached = true;
        let hash = run_git(&repo, &["rev-parse", "--short", "HEAD"])?;
        branch = if hash.status.success() {
            output_text(&hash)
        } else {
            "No commits yet".to_string()
        };
    }

    let mut ahead = 0;
    let mut behind = 0;
    let counts = run_git(
        &repo,
        &["rev-list", "--left-right", "--count", "@{upstream}...HEAD"],
    )?;
    if counts.status.success() {
        let text = output_text(&counts);
        let mut parts = text.split_whitespace();
        if let (Some(behind_text), Some(ahead_text)) = (parts.next(), parts.next()) {
            behind = behind_text.parse().unwrap_or(0);
            ahead = ahead_text.parse().unwrap_or(0);
        }
    }

    Ok(Some(GitStatus {
        repo_root: repo.to_string_lossy().replace('\\', "/"),
        branch,
        detached,
        ahead,
        behind,
        clean: files.is_empty(),
        files,
    }))
}

#[tauri::command]
pub fn git_init(state: State<'_, WorkspaceState>) -> Result<GitStatus, String> {
    let root = workspace_root(&state)?;
    let output = run_git(&root, &["init"])?;
    ensure_ok(&output, "init")?;
    git_status(state)?.ok_or_else(|| "git repository was not created".to_string())
}

#[tauri::command]
pub fn git_diff(
    state: State<'_, WorkspaceState>,
    path: String,
    staged: bool,
) -> Result<String, String> {
    let repo = require_repo(&state)?;
    let paths = checked_paths(vec![path])?;
    let path = paths[0].as_str();

    let mut args = vec!["diff", "--no-color", "--no-ext-diff"];
    if staged {
        args.push("--cached");
    }
    args.extend(["--", path]);
    let output = run_git(&repo, &args)?;
    ensure_ok(&output, "diff")?;
    let text = output_text(&output);
    if !text.is_empty() {
        return Ok(text);
    }

    let status = run_git(&repo, &["status", "--porcelain=v1", "--", path])?;
    ensure_ok(&status, "status")?;
    if output_text(&status).starts_with("??") {
        let null = if cfg!(windows) { "NUL" } else { "/dev/null" };
        let untracked = run_git(
            &repo,
            &["diff", "--no-index", "--no-color", "--", null, path],
        )?;
        if untracked.status.code() == Some(1) || untracked.status.success() {
            return Ok(output_text(&untracked));
        }
        return Err(format!("git diff failed: {}", error_text(&untracked)));
    }

    Ok(text)
}

#[tauri::command]
pub fn git_stage(state: State<'_, WorkspaceState>, paths: Vec<String>) -> Result<(), String> {
    let repo = require_repo(&state)?;
    let paths = checked_paths(paths)?;
    let mut args = vec!["add", "--"];
    args.extend(paths.iter().map(String::as_str));
    let output = run_git(&repo, &args)?;
    ensure_ok(&output, "add")
}

#[tauri::command]
pub fn git_unstage(state: State<'_, WorkspaceState>, paths: Vec<String>) -> Result<(), String> {
    let repo = require_repo(&state)?;
    let paths = checked_paths(paths)?;
    let mut args = vec!["restore", "--staged", "--"];
    args.extend(paths.iter().map(String::as_str));
    let output = run_git(&repo, &args)?;
    ensure_ok(&output, "restore --staged")
}

#[tauri::command]
pub fn git_commit(state: State<'_, WorkspaceState>, message: String) -> Result<String, String> {
    let repo = require_repo(&state)?;
    let message = message.trim();
    if message.is_empty() {
        return Err("commit message is empty".to_string());
    }
    let output = run_git(&repo, &["commit", "-m", message])?;
    ensure_ok(&output, "commit")?;
    let hash = run_git(&repo, &["rev-parse", "--short", "HEAD"])?;
    ensure_ok(&hash, "rev-parse")?;
    Ok(output_text(&hash))
}

#[tauri::command]
pub fn git_log(
    state: State<'_, WorkspaceState>,
    limit: Option<usize>,
) -> Result<Vec<GitCommit>, String> {
    let repo = require_repo(&state)?;
    let limit = limit.unwrap_or(20).clamp(1, 100).to_string();
    let output = run_git(
        &repo,
        &[
            "log",
            "-n",
            &limit,
            "--pretty=format:%h%x1f%an%x1f%ar%x1f%s",
        ],
    )?;
    if !output.status.success() {
        return Ok(Vec::new());
    }

    Ok(output_text(&output)
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(4, '\x1f');
            Some(GitCommit {
                hash: parts.next()?.to_string(),
                author: parts.next()?.to_string(),
                date: parts.next()?.to_string(),
                subject: parts.next()?.to_string(),
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modified_and_untracked_files() {
        let files = parse_porcelain(" M note.md\0?? draft.md\0A  src/new.ts\0");
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].path, "note.md");
        assert!(files[0].unstaged);
        assert!(!files[0].staged);
        assert!(files[1].untracked);
        assert!(files[2].staged);
    }

    #[test]
    fn parses_renames_and_keeps_original_path() {
        let files = parse_porcelain("R  new.md\0old.md\0");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "new.md");
        assert_eq!(files[0].original_path.as_deref(), Some("old.md"));
        assert!(files[0].staged);
    }

    #[test]
    fn rejects_paths_outside_the_workspace() {
        assert!(checked_paths(vec!["../secret".to_string()]).is_err());
        assert!(checked_paths(vec!["/etc/passwd".to_string()]).is_err());
        assert!(checked_paths(vec!["src/main.rs".to_string()]).is_ok());
    }
}
