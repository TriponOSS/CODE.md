mod git;
mod skills;
mod workspace;

use workspace::WorkspaceState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(WorkspaceState::default())
        .invoke_handler(tauri::generate_handler![
            workspace::pick_workspace,
            workspace::open_workspace,
            workspace::close_workspace,
            workspace::workspace_root,
            workspace::scan_tree,
            workspace::read_file,
            workspace::save_file,
            workspace::create_entry,
            workspace::search_workspace,
            skills::scan_skills,
            git::git_status,
            git::git_init,
            git::git_diff,
            git::git_stage,
            git::git_unstage,
            git::git_commit,
            git::git_push,
            git::git_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
