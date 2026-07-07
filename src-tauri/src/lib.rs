pub mod ai;
pub mod commands;
pub mod config;
pub mod git;
pub mod logging;
pub mod mcp;
pub mod sandbox;
pub mod types;
pub mod vector;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::git::GitEngine;
use crate::logging::init_logging;
use crate::sandbox::SandboxEngine;
use crate::types::AppState;
use crate::vector::VectorStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    let config = Arc::new(RwLock::new(Config::default()));
    let sandbox = Arc::new(SandboxEngine::new());
    let vault_path = std::path::PathBuf::from(".");

    let state = AppState {
        config: config.clone(),
        sandbox: sandbox.clone(),
        git: Arc::new(RwLock::new(GitEngine::new(&vault_path))),
        vector: Arc::new(VectorStore::new(&vault_path)),
        vault_path: vault_path.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::editor::read_file,
            commands::editor::save_file,
            commands::editor::list_files,
            commands::editor::create_file,
            commands::editor::create_folder,
            commands::editor::rename_file,
            commands::editor::move_file,
            commands::editor::delete_file,
            commands::editor::duplicate_file,
            commands::editor::get_tags,
            commands::editor::get_recent_files,
            commands::editor::write_file,
            commands::editor::get_temp_dir,
            commands::config::get_config,
            commands::config::update_config,
            commands::config::reset_config,
            commands::config::reload_config,
            commands::search::search_files,
            commands::search::set_embeddings_enabled,
            commands::search::set_embedding_model,
            commands::search::get_embedding_config,
            commands::search::vector_search,
            commands::search::hybrid_search,
            commands::git::git_commit,
            commands::git::git_history,
            commands::git::git_log,
            commands::git::git_diff,
            commands::git::git_restore,
            commands::git::git_status,
            commands::git::git_push,
            commands::git::git_pull,
            commands::ai::ai_chat,
            commands::ai::ai_template,
            commands::ai::ai_list_models,
            commands::ai::test_provider_connection,
            commands::ai::get_templates,
            commands::ingest::ingest_file,
            commands::ingest::ingest_url,
            commands::ingest::list_converters,
            commands::mcp::mcp_list_servers,
            commands::mcp::mcp_add_server,
            commands::mcp::mcp_remove_server,
            commands::mcp::mcp_toggle_server,
            commands::mcp::mcp_test_connection,
            commands::mcp::mcp_call_tool,
            commands::sandbox::sandbox_execute,
            commands::sandbox::sandbox_exec,
            commands::sandbox::sandbox_get_tools,
            commands::sandbox::sandbox_test_tool,
            commands::sandbox::sandbox_list_snapshots,
            commands::sandbox::sandbox_create_snapshot,
            commands::sandbox::sandbox_restore_snapshot,
            commands::sandbox::sandbox_delete_snapshot,
            commands::import::import_image,
            commands::import::import_files,
            commands::credentials::store_api_key,
            commands::credentials::get_api_key,
            commands::credentials::delete_api_key,
            commands::credentials::list_api_keys,
            #[cfg(feature = "voice")]
            commands::voice::voice_start_recording,
            #[cfg(feature = "voice")]
            commands::voice::voice_is_recording,
            #[cfg(feature = "voice")]
            commands::voice::voice_stop_recording,
            #[cfg(feature = "voice")]
            commands::voice::voice_transcribe_file,
            #[cfg(feature = "voice")]
            commands::voice::voice_list_devices,
            #[cfg(feature = "voice")]
            commands::voice::voice_get_audio_info,
            #[cfg(feature = "voice")]
            commands::voice::voice_process_command,
            commands::plugins::plugin_install,
            commands::plugins::plugin_uninstall,
            commands::plugins::plugin_list,
            commands::plugins::plugin_toggle,
            commands::plugins::plugin_get_info,
            commands::plugins::marketplace_fetch,
            commands::plugins::marketplace_install,
            commands::plugins::marketplace_check_updates,
            commands::plugins::marketplace_uninstall,
            commands::publish::publish_now,
            commands::publish::publish_preview,
            commands::publish::upload_image,
            commands::publish::generate_rss,
            commands::publish::list_publications,
            commands::config::get_providers,
            commands::config::save_providers,
            commands::image_gen::generate_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZarishNote");
}
