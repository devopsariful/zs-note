import { invoke } from '@tauri-apps/api/core';
import type { PluginInfo, MarketplaceServerInfo } from '../types';

/**
 * Install a WASM plugin from a .wasm file path.
 * The file must be accompanied by a plugin.toml manifest in the same directory.
 */
export async function pluginInstall(path: string): Promise<PluginInfo> {
  return invoke('plugin_install', { path });
}

/**
 * Uninstall a plugin by its id.
 */
export async function pluginUninstall(id: string): Promise<void> {
  return invoke('plugin_uninstall', { id });
}

/**
 * List all installed plugins.
 */
export async function pluginList(): Promise<PluginInfo[]> {
  return invoke('plugin_list');
}

/**
 * Enable or disable a plugin by its id.
 */
export async function pluginToggle(id: string, enabled: boolean): Promise<void> {
  return invoke('plugin_toggle', { id, enabled });
}

/**
 * Get detailed information about a single plugin.
 */
export async function pluginGetInfo(id: string): Promise<PluginInfo> {
  return invoke('plugin_get_info', { id });
}

/**
 * Fetch available MCP servers from the marketplace registry.
 */
export async function marketplaceFetch(registryUrl?: string): Promise<MarketplaceServerInfo[]> {
  return invoke('marketplace_fetch', { registryUrl: registryUrl ?? null });
}

/**
 * Install an MCP server from the marketplace.
 */
export async function marketplaceInstall(serverId: string, registryUrl?: string): Promise<void> {
  return invoke('marketplace_install', { serverId, registryUrl: registryUrl ?? null });
}

/**
 * Check for updates to installed marketplace servers.
 */
export async function marketplaceCheckUpdates(registryUrl?: string): Promise<Array<{
  server_id: string;
  name: string;
  installed_version: string;
  latest_version: string;
}>> {
  return invoke('marketplace_check_updates', { registryUrl: registryUrl ?? null });
}

/**
 * Uninstall an MCP server that was installed from the marketplace.
 */
export async function marketplaceUninstall(serverId: string): Promise<void> {
  return invoke('marketplace_uninstall', { serverId });
}
