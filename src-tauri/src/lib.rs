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
            workspace::search_workspace,
            skills::scan_skills,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
