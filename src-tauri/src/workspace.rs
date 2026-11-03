use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use ignore::WalkBuilder;
use notify::RecursiveMode;
use notify::Watcher;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

pub const CHANGE_EVENT: &str = "workspace-changed";

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".svelte-kit",
    ".cache",
    "vendor",
];

#[derive(Default)]
pub struct WorkspaceState {
    pub root: Mutex<Option<PathBuf>>,
    pub watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeEntry {
    /// Workspace-relative path, always with forward slashes.
    pub path: String,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub path: String,
    pub line: usize,
    pub text: String,
}

pub fn canonical_root(path: &Path) -> Result<PathBuf, String> {
    let canon = path
        .canonicalize()
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    if !canon.is_dir() {
        return Err(format!("{} is not a directory", canon.display()));
    }
    Ok(canon)
}

fn current_root(state: &WorkspaceState) -> Result<PathBuf, String> {
    state
        .root
        .lock()
        .map_err(|_| "workspace state poisoned".to_string())?
        .clone()
        .ok_or_else(|| "no workspace is open".to_string())
}

/// Resolve a workspace-relative (or absolute) path and prove it stays inside the root.
/// Canonicalization also resolves `..` and symlinks, so this cannot be escaped that way.
pub fn resolve_in_root(root: &Path, requested: &str) -> Result<PathBuf, String> {
    let raw = PathBuf::from(requested);
    let candidate = if raw.is_absolute() {
        raw
    } else {
        root.join(raw)
    };
    let canon = candidate
        .canonicalize()
        .map_err(|e| format!("cannot resolve {}: {e}", candidate.display()))?;
    if !canon.starts_with(root) {
        return Err("path is outside the open workspace".to_string());
    }
    Ok(canon)
}

fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_markdown(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("md" | "mdx" | "markdown")
    )
}

/// Extensions we are willing to show in the tree and open in the editor.
/// Markdown gets a rendered preview; everything else opens as plain text.
const TEXT_EXTENSIONS: &[&str] = &[
    "md", "mdx", "markdown", "txt", "text", "rst", "adoc", "asciidoc", "org",
    "json", "jsonc", "json5", "yaml", "yml", "toml", "ini", "cfg", "conf", "config",
    "env", "properties", "csv", "tsv", "log", "lock",
    "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "svelte", "vue", "astro",
    "html", "htm", "css", "scss", "sass", "less",
    "rs", "go", "py", "pyi", "rb", "php", "java", "kt", "kts", "swift", "dart",
    "c", "h", "cc", "cpp", "cxx", "hpp", "hxx", "cs", "fs", "fsx",
    "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd", "sql", "graphql", "gql",
    "lua", "r", "ex", "exs", "erl", "hrl", "hs", "lhs", "clj", "cljs", "cljc", "edn",
    "zig", "nim", "v", "sol", "asm", "s", "xml", "svg", "plist", "gradle", "cmake", "mk",
];

const TEXT_FILENAMES: &[&str] = &[
    ".gitignore", ".gitattributes", ".gitmodules", ".dockerignore", ".editorconfig",
    ".env", ".npmrc", ".prettierrc", ".eslintrc", "dockerfile", "makefile", "justfile",
    "procfile", "license", "readme", "changelog", "gemfile", "rakefile", "go.sum",
];

fn is_text_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    if TEXT_FILENAMES.contains(&name.as_str()) {
        return true;
    }

    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|ext| TEXT_EXTENSIONS.contains(&ext.as_str()))
}

pub(crate) fn is_skill_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.eq_ignore_ascii_case("SKILL.md"))
        .unwrap_or(false)
}

pub fn walk_files(root: &Path) -> impl Iterator<Item = PathBuf> {
    WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .follow_links(false)
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && SKIP_DIRS.iter().any(|d| name.eq_ignore_ascii_case(*d)))
        })
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|t| !t.is_dir()).unwrap_or(false))
        .map(|entry| entry.into_path())
}

pub fn walk_markdown(root: &Path) -> impl Iterator<Item = PathBuf> {
    walk_files(root).filter(|path| is_markdown(path))
}

pub fn walk_text(root: &Path) -> impl Iterator<Item = PathBuf> {
    walk_files(root).filter(|path| is_text_file(path))
}

fn start_watcher(app: &AppHandle, root: &Path, state: &WorkspaceState) -> Result<(), String> {
    let handle = app.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if res.is_ok() {
                let _ = handle.emit(CHANGE_EVENT, ());
            }
        })
        .map_err(|e| format!("cannot start file watcher: {e}"))?;

    watcher
        .watch(root, RecursiveMode::Recursive)
        .map_err(|e| format!("cannot watch {}: {e}", root.display()))?;

    let mut slot = state
        .watcher
        .lock()
        .map_err(|_| "watcher state poisoned".to_string())?;
    *slot = Some(watcher);
    Ok(())
}

#[tauri::command]
pub async fn pick_workspace(
    app: AppHandle,
    state: State<'_, WorkspaceState>,
) -> Result<Option<String>, String> {
    let Some(picked) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|e| format!("invalid folder: {e}"))?;
    open_root(&app, &state, &path).map(Some)
}

#[tauri::command]
pub fn open_workspace(
    app: AppHandle,
    state: State<'_, WorkspaceState>,
    path: String,
) -> Result<String, String> {
    open_root(&app, &state, Path::new(&path))
}

fn open_root(app: &AppHandle, state: &WorkspaceState, path: &Path) -> Result<String, String> {
    let root = canonical_root(path)?;
    start_watcher(app, &root, state)?;
    let display = root.to_string_lossy().to_string();
    let mut slot = state
        .root
        .lock()
        .map_err(|_| "workspace state poisoned".to_string())?;
    *slot = Some(root);
    Ok(display)
}

#[tauri::command]
pub fn close_workspace(state: State<'_, WorkspaceState>) -> Result<(), String> {
    *state
        .root
        .lock()
        .map_err(|_| "workspace state poisoned".to_string())? = None;
    *state
        .watcher
        .lock()
        .map_err(|_| "workspace state poisoned".to_string())? = None;
    Ok(())
}

#[tauri::command]
pub fn workspace_root(state: State<'_, WorkspaceState>) -> Option<String> {
    state
        .root
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .map(|root| root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn scan_tree(state: State<'_, WorkspaceState>) -> Result<Vec<TreeEntry>, String> {
    let root = current_root(&state)?;
    let mut entries = Vec::new();

    for entry in WalkBuilder::new(&root)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .follow_links(false)
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && SKIP_DIRS.iter().any(|d| name.eq_ignore_ascii_case(*d)))
        })
        .build()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == root {
            continue;
        }
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir && !is_text_file(path) {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            entries.push(TreeEntry {
                path: relative_slash(&root, path),
                name: name.to_string(),
                is_dir,
            });
        }
    }

    entries.sort_by(|a, b| {
        a.path
            .to_ascii_lowercase()
            .cmp(&b.path.to_ascii_lowercase())
    });
    Ok(entries)
}

#[tauri::command]
pub fn read_file(state: State<'_, WorkspaceState>, path: String) -> Result<String, String> {
    let root = current_root(&state)?;
    let resolved = resolve_in_root(&root, &path)?;
    if !resolved.is_file() {
        return Err("not a file".to_string());
    }
    fs::read_to_string(&resolved).map_err(|e| format!("cannot read file: {e}"))
}

#[tauri::command]
pub fn save_file(
    state: State<'_, WorkspaceState>,
    path: String,
    content: String,
) -> Result<(), String> {
    let root = current_root(&state)?;
    let resolved = resolve_in_root(&root, &path)?;
    atomic_write(&resolved, content.as_bytes())
}

fn validate_entry_name(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name is empty".to_string());
    }
    if name == "." || name == ".." {
        return Err("that name is reserved".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        return Err("name cannot contain path separators".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn create_entry(
    state: State<'_, WorkspaceState>,
    name: String,
    directory: bool,
) -> Result<String, String> {
    let root = current_root(&state)?;
    let name = name.trim();
    validate_entry_name(name)?;
    let path = root.join(name);
    if path.exists() {
        return Err(format!("{name} already exists"));
    }
    if directory {
        fs::create_dir(&path).map_err(|e| format!("cannot create folder: {e}"))?;
    } else {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| format!("cannot create file: {e}"))?;
    }
    Ok(relative_slash(&root, &path))
}

#[tauri::command]
pub fn search_workspace(
    state: State<'_, WorkspaceState>,
    query: String,
) -> Result<Vec<SearchHit>, String> {
    let root = current_root(&state)?;
    search_root(&root, &query, 250)
}

pub fn search_root(root: &Path, query: &str, limit: usize) -> Result<Vec<SearchHit>, String> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Ok(Vec::new());
    }

    let mut hits = Vec::new();
    for path in walk_text(root) {
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if meta.len() > 2 * 1024 * 1024 {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        if bytes.contains(&0) {
            continue;
        }
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let rel = relative_slash(root, &path);
        for (index, line) in text.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                hits.push(SearchHit {
                    path: rel.clone(),
                    line: index + 1,
                    text: line.trim().chars().take(240).collect(),
                });
                if hits.len() >= limit {
                    return Ok(hits);
                }
            }
        }
    }
    Ok(hits)
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "file has no parent directory".to_string())?;
    let name = path
        .file_name()
        .ok_or_else(|| "file has no name".to_string())?
        .to_string_lossy();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(".{name}.codemd-{}-{stamp}.tmp", std::process::id()));

    let result = (|| -> std::io::Result<()> {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        if let Ok(meta) = fs::metadata(path) {
            fs::set_permissions(&tmp, meta.permissions())?;
        }
        fs::rename(&tmp, path)?;
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result.map_err(|e| format!("cannot save file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("codemd-test-{name}-{stamp}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn atomic_write_replaces_content_and_leaves_no_temp_files() {
        let dir = scratch_dir("atomic");
        let file = dir.join("note.md");
        fs::write(&file, "old").unwrap();

        atomic_write(&file, b"new content\n").unwrap();

        assert_eq!(fs::read_to_string(&file).unwrap(), "new content\n");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp files left behind");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn resolve_in_root_rejects_traversal() {
        let root = scratch_dir("traversal");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs/a.md"), "a").unwrap();

        assert!(resolve_in_root(&root, "docs/a.md").is_ok());
        assert!(resolve_in_root(&root, "../escape.md").is_err());
        assert!(resolve_in_root(&root, "/etc/passwd").is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn entry_names_reject_traversal_and_separators() {
        assert!(validate_entry_name("note.md").is_ok());
        assert!(validate_entry_name("ideas").is_ok());
        assert!(validate_entry_name("").is_err());
        assert!(validate_entry_name("..").is_err());
        assert!(validate_entry_name("docs/note.md").is_err());
        assert!(validate_entry_name("docs\\note.md").is_err());
    }

    #[test]
    fn scan_tree_lists_text_files_and_skips_git() {
        let root = scratch_dir("scan");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join("skills/code-review")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("README.md"), "# hi").unwrap();
        fs::write(
            root.join("skills/code-review/SKILL.md"),
            "---\nname: x\n---",
        )
        .unwrap();
        fs::write(root.join("notes.txt"), "yes").unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("logo.png"), "not text").unwrap();
        fs::write(root.join(".git/HEAD"), "no").unwrap();

        let root_canon = canonical_root(&root).unwrap();
        let entries: Vec<String> = WalkBuilder::new(&root_canon)
            .hidden(false)
            .filter_entry(|entry| {
                let name = entry.file_name().to_string_lossy();
                !(entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                    && SKIP_DIRS.iter().any(|d| name.eq_ignore_ascii_case(*d)))
            })
            .build()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map(|t| !t.is_dir()).unwrap_or(false))
            .filter(|e| is_text_file(e.path()))
            .map(|e| relative_slash(&root_canon, e.path()))
            .collect();

        assert!(entries.contains(&"README.md".to_string()));
        assert!(entries.contains(&"notes.txt".to_string()));
        assert!(entries.contains(&"src/main.rs".to_string()));
        assert!(entries.iter().any(|p| p.ends_with("SKILL.md")));
        assert!(!entries.iter().any(|p| p.contains(".git")));
        assert!(!entries.iter().any(|p| p.ends_with(".png")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn search_is_case_insensitive_and_reports_lines() {
        let root = scratch_dir("search");
        fs::write(root.join("a.md"), "one\nNeedle here\ntwo").unwrap();
        fs::write(root.join("b.md"), "nothing").unwrap();
        fs::write(root.join("c.txt"), "needle in text").unwrap();

        let hits = search_root(&root, "needle", 10).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().any(|h| h.path == "a.md" && h.line == 2));
        assert!(hits.iter().any(|h| h.path == "c.txt" && h.line == 1));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn text_file_detection_covers_code_and_dotfiles() {
        assert!(is_text_file(Path::new("/x/README.MD")));
        assert!(is_text_file(Path::new("/x/notes.txt")));
        assert!(is_text_file(Path::new("/x/main.rs")));
        assert!(is_text_file(Path::new("/x/.gitignore")));
        assert!(is_text_file(Path::new("/x/Dockerfile")));
        assert!(!is_text_file(Path::new("/x/logo.png")));
        assert!(!is_text_file(Path::new("/x/archive.zip")));
    }

    #[test]
    fn skill_file_detection_is_case_insensitive() {
        assert!(is_skill_file(Path::new("/x/skills/foo/skill.md")));
        assert!(!is_skill_file(Path::new("/x/skills/foo/README.md")));
    }
}
