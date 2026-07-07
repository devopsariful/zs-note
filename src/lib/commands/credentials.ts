import { invoke } from '@tauri-apps/api/core';

export async function storeApiKey(provider: string, apiKey: string): Promise<void> {
  return invoke('store_api_key', { provider, apiKey });
}

export async function getApiKey(provider: string): Promise<string | null> {
  return invoke('get_api_key', { provider });
}

export async function deleteApiKey(provider: string): Promise<void> {
  return invoke('delete_api_key', { provider });
}

export async function listApiKeys(): Promise<string[]> {
  return invoke('list_api_keys');
}
