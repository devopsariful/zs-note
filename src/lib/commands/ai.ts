import { invoke } from '@tauri-apps/api/core';
import type { ChatMessage, ProviderConfig, Template } from '../types';
import { listen } from '@tauri-apps/api/event';

export async function aiChat(
  messages: ChatMessage[],
  provider: string,
  model: string,
  onChunk?: (text: string) => void,
  temperature?: number,
  maxTokens?: number,
  topP?: number,
): Promise<string> {
  const args: Record<string, unknown> = {
    messages: messages.map(m => ({
      role: m.role,
      content: m.content,
    })),
    provider,
    model,
    temperature: temperature ?? null,
    maxTokens: maxTokens ?? null,
    topP: topP ?? null,
  };

  if (onChunk) {
    const unlisten = await listen<string>('ai:chunk', (event) => {
      onChunk(event.payload);
    });

    const result = await invoke<string>('ai_chat', args);

    unlisten();
    return result;
  }

  return invoke('ai_chat', args);
}

export async function aiTemplate(
  templateName: string,
  variables: Record<string, string>,
  onChunk?: (text: string) => void,
): Promise<string> {
  if (onChunk) {
    const unlisten = await listen<string>('ai:chunk', (event) => {
      onChunk(event.payload);
    });

    const result = await invoke<string>('ai_template', {
      templateName,
      variables,
    });

    unlisten();
    return result;
  }

  return invoke('ai_template', { templateName, variables });
}

export async function aiListModels(provider: string): Promise<string[]> {
  return invoke('ai_list_models', { provider });
}

export async function testProviderConnection(
  provider: ProviderConfig | string,
): Promise<boolean> {
  return invoke('test_provider_connection', {
    provider: typeof provider === 'string' ? provider : provider.id,
  });
}
