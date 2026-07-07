/// <reference types="@sveltejs/vite-plugin-svelte" />
/// <reference types="vite/client" />

declare module '*.svelte' {
  import type { ComponentType } from 'svelte';
  const component: ComponentType;
  export default component;
}

declare module '*.css' {
  const content: string;
  export default content;
}

declare module '*.json' {
  const content: Record<string, unknown>;
  export default content;
}

/* ── Tauri invoke command signatures ── */

interface EditorCommands {
  read_file(path: string): Promise<string>;
  save_file(path: string, content: string): Promise<void>;
  list_files(path: string): Promise<import('./lib/types').FileEntry[]>;
  create_file(path: string): Promise<void>;
  create_folder(path: string): Promise<void>;
  rename_file(old_path: string, new_path: string): Promise<void>;
  move_file(old_path: string, new_path: string): Promise<void>;
  delete_file(path: string): Promise<void>;
  duplicate_file(path: string): Promise<void>;
  get_tags(): Promise<string[]>;
  get_recent_files(): Promise<import('./lib/types').FileEntry[]>;

  ai_chat(
    messages: import('./lib/types').ChatMessage[],
    provider: string,
    model: string,
  ): Promise<string>;
  ai_template(
    template_name: string,
    variables: Record<string, string>,
  ): Promise<string>;
  ai_list_models(provider: string): Promise<string[]>;
  test_provider_connection(provider: string): Promise<boolean>;

  sandbox_exec(
    wasm_path: string,
    input?: string,
  ): Promise<{ stdout: string; stderr: string; exit_code: number }>;
  sandbox_list_snapshots(): Promise<string[]>;
  sandbox_create_snapshot(name: string): Promise<void>;
  sandbox_restore_snapshot(name: string): Promise<void>;
  sandbox_delete_snapshot(name: string): Promise<void>;

  get_config(): Promise<unknown>;
  update_config(new_config: unknown): Promise<void>;
  reset_config(): Promise<void>;
  reload_config(): Promise<unknown>;
  get_editor_settings(): Promise<import('./lib/types').EditorSettings>;
  save_editor_settings(
    settings: import('./lib/types').EditorSettings,
  ): Promise<void>;
  get_providers(): Promise<import('./lib/types').ProviderConfig[]>;
  save_providers(
    providers: import('./lib/types').ProviderConfig[],
  ): Promise<void>;
  get_sandbox_config(): Promise<import('./lib/types').SandboxConfig>;
  save_sandbox_config(
    config: import('./lib/types').SandboxConfig,
  ): Promise<void>;
  get_sync_config(): Promise<import('./lib/types').SyncConfig>;
  save_sync_config(
    config: import('./lib/types').SyncConfig,
  ): Promise<void>;
  get_publish_targets(): Promise<import('./lib/types').PublishTarget[]>;
  save_publish_targets(
    targets: import('./lib/types').PublishTarget[],
  ): Promise<void>;

  git_status(): Promise<import('./lib/types').GitStatus>;
  git_log(file_path?: string): Promise<import('./lib/types').CommitEntry[]>;
  git_diff(
    file_path: string,
    from?: string,
    to?: string,
  ): Promise<import('./lib/types').DiffResult>;
  git_restore(file_path: string, commit: string): Promise<void>;
  git_commit(message: string): Promise<void>;
  git_push(): Promise<void>;
  git_pull(): Promise<void>;

  search_files(query: string, path?: string): Promise<import('./lib/types').SearchResult[]>;
  get_templates(): Promise<import('./lib/types').Template[]>;

  mcp_list_servers(): Promise<import('./lib/types').McpServerInfo[]>;
  mcp_add_server(
    server: import('./lib/types').McpServerInfo,
  ): Promise<void>;
  mcp_remove_server(id: string): Promise<void>;
  mcp_toggle_server(id: string, enabled: boolean): Promise<void>;
  mcp_test_connection(id: string): Promise<boolean>;

  kb_list(): Promise<import('./lib/types').KnowledgeBaseInfo[]>;
  kb_index(id: string): Promise<void>;
  kb_remove(id: string): Promise<void>;
  kb_stats(id: string): Promise<import('./lib/types').IndexStats>;

  publish_now(
    target: import('./lib/types').PublishTarget,
    file_path: string,
    options: Record<string, boolean>,
  ): Promise<void>;
  publish_preview(file_path: string): Promise<string>;

  plugin_list(): Promise<import('./lib/types').PluginConfig[]>;
  plugin_toggle(id: string, enabled: boolean): Promise<void>;
  plugin_install(source: string): Promise<void>;
  plugin_remove(id: string): Promise<void>;
}

type Invoke = <K extends keyof EditorCommands>(
  command: K,
  args?: Parameters<EditorCommands[K]>[0] extends undefined
    ? Record<string, never>
    : Parameters<EditorCommands[K]>[0] extends Record<string, never>
      ? Record<string, never>
      : Parameters<EditorCommands[K]>[0],
) => ReturnType<EditorCommands[K]>;

declare module '@tauri-apps/api/core' {
  export function invoke<T = unknown>(
    cmd: string,
    args?: Record<string, unknown>,
  ): Promise<T>;
}
