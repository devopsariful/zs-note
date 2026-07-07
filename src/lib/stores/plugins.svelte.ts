import type { PluginInfo, MarketplaceServerInfo } from '../types';
import * as pluginCommands from '../commands/plugins';

let plugins = $state<PluginInfo[]>([]);
let marketplaceServers = $state<MarketplaceServerInfo[]>([]);
let isLoading = $state(false);
let marketplaceLoading = $state(false);
let error = $state<string | null>(null);

const installedCount = $derived(plugins.length);
const enabledCount = $derived(plugins.filter((p) => p.enabled).length);

async function loadPlugins(): Promise<void> {
  isLoading = true;
  error = null;

  try {
    plugins = await pluginCommands.pluginList();
  } catch (err) {
    error = String(err);
  } finally {
    isLoading = false;
  }
}

async function installPlugin(path: string): Promise<PluginInfo> {
  error = null;
  try {
    const info = await pluginCommands.pluginInstall(path);
    await loadPlugins();
    return info;
  } catch (err) {
    error = String(err);
    throw err;
  }
}

async function uninstallPlugin(id: string): Promise<void> {
  error = null;
  try {
    await pluginCommands.pluginUninstall(id);
    await loadPlugins();
  } catch (err) {
    error = String(err);
    throw err;
  }
}

async function togglePlugin(id: string, enabled: boolean): Promise<void> {
  error = null;
  try {
    await pluginCommands.pluginToggle(id, enabled);
    // Update local state optimistically
    const idx = plugins.findIndex((p) => p.id === id);
    if (idx !== -1) {
      const updated = [...plugins];
      updated[idx] = { ...(plugins[idx] as PluginInfo), enabled };
      plugins = updated;
    }
  } catch (err) {
    error = String(err);
    throw err;
  }
}

async function fetchMarketplace(registryUrl?: string): Promise<void> {
  marketplaceLoading = true;
  error = null;

  try {
    marketplaceServers = await pluginCommands.marketplaceFetch(registryUrl);
  } catch (err) {
    error = String(err);
  } finally {
    marketplaceLoading = false;
  }
}

async function installFromMarketplace(serverId: string, registryUrl?: string): Promise<void> {
  error = null;
  try {
    await pluginCommands.marketplaceInstall(serverId, registryUrl);
    await fetchMarketplace(registryUrl);
  } catch (err) {
    error = String(err);
    throw err;
  }
}

export function getPluginsStore() {
  return {
    get plugins() { return plugins; },
    set plugins(v: PluginInfo[]) { plugins = v; },
    get marketplaceServers() { return marketplaceServers; },
    set marketplaceServers(v: MarketplaceServerInfo[]) { marketplaceServers = v; },
    get isLoading() { return isLoading; },
    get marketplaceLoading() { return marketplaceLoading; },
    get error() { return error; },
    get installedCount() { return installedCount; },
    get enabledCount() { return enabledCount; },
    loadPlugins,
    installPlugin,
    uninstallPlugin,
    togglePlugin,
    fetchMarketplace,
    installFromMarketplace,
  };
}
