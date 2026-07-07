import type { ChatMessage, ProviderConfig, Template } from '../types';
import * as aiCommands from '../commands/ai';

let messages = $state<ChatMessage[]>([]);
let selectedProvider = $state<string>('');
let selectedModel = $state<string>('');
let isStreaming = $state(false);
let providers = $state<ProviderConfig[]>([]);
let templates = $state<Template[]>([]);
let streamingContent = $state('');
let error = $state<string | null>(null);

// AI parameter controls
let temperature = $state<number>(0.7);
let maxTokens = $state<number>(2048);
let topP = $state<number>(1.0);

let abortController: AbortController | null = null;

function addMessage(role: ChatMessage['role'], content: string): ChatMessage {
  const msg: ChatMessage = {
    role,
    content,
    id: crypto.randomUUID(),
    timestamp: Date.now(),
    done: true,
  };
  messages = [...messages, msg];
  return msg;
}

async function sendMessage(input: string): Promise<void> {
  if (!input.trim() || isStreaming) return;

  error = null;
  addMessage('user', input);

  const userMessages = messages.map(m => ({
    role: m.role,
    content: m.content,
    id: m.id,
    timestamp: m.timestamp,
    done: m.done,
  }));

  const assistantMsg: ChatMessage = {
    role: 'assistant',
    content: '',
    id: crypto.randomUUID(),
    timestamp: Date.now(),
    done: false,
  };
  messages = [...messages, assistantMsg];
  isStreaming = true;

  try {
    const provider = selectedProvider || providers[0]?.id || '';
    const model = selectedModel || providers[0]?.default_model || '';
    const result = await aiCommands.aiChat(
      userMessages,
      provider,
      model,
      (chunk) => {
        streamingContent += chunk;
        messages = messages.map(m =>
          m.id === assistantMsg.id ? { ...m, content: streamingContent } : m,
        );
      },
      temperature,
      maxTokens,
      topP,
    );

    messages = messages.map(m =>
      m.id === assistantMsg.id ? { ...m, content: result, done: true } : m,
    );
    streamingContent = '';
    isStreaming = false;
  } catch (err) {
    error = String(err);
    messages = messages.map(m =>
      m.id === assistantMsg.id ? { ...m, content: `Error: ${String(err)}`, done: true } : m,
    );
    isStreaming = false;
  }
}

function stopStreaming(): void {
  isStreaming = false;
  if (abortController) {
    abortController.abort();
    abortController = null;
  }
  messages = messages.map(m =>
    !m.done ? { ...m, done: true } : m,
  );
}

function insertResponse(responseIndex: number): void {
  const msg = messages[responseIndex];
  if (!msg || msg.role !== 'assistant') return;

  const event = new CustomEvent('ai:insert', {
    detail: { content: msg.content },
  });
  window.dispatchEvent(event);
}

function replaceSelection(responseIndex: number): void {
  const msg = messages[responseIndex];
  if (!msg || msg.role !== 'assistant') return;

  const event = new CustomEvent('ai:replace', {
    detail: { content: msg.content },
  });
  window.dispatchEvent(event);
}

function retryLastMessage(): void {
  if (messages.length < 2) return;

  const lastUserIdx = [...messages].reverse().findIndex(m => m.role === 'user');
  if (lastUserIdx === -1) return;

  messages = messages.slice(0, messages.length - 1);
  const lastUserMsg = messages[messages.length - 1];
  if (lastUserMsg && lastUserMsg.role === 'user') {
    sendMessage(lastUserMsg.content);
  }
}

function clearConversation(): void {
  messages = [];
  streamingContent = '';
  error = null;
}

function loadTemplates(): void {
  import('../commands/editor').then((m) =>
    m.getTemplates().then((t) => { templates = t; }).catch(() => {})
  );
}

function setProvider(id: string): void {
  selectedProvider = id;
  const prov = providers.find(p => p.id === id);
  if (prov) {
    selectedModel = prov.default_model || prov.models[0] || '';
  }
}

function setModel(model: string): void {
  selectedModel = model;
}

function setProviders(configs: ProviderConfig[]): void {
  providers = configs;
  if (!selectedProvider && configs.length > 0) {
    selectedProvider = configs[0]!.id;
    selectedModel = configs[0]!.default_model || configs[0]!.models[0] || '';
  }
}

function setTemperature(t: number): void {
  temperature = t;
}

function setMaxTokens(t: number): void {
  maxTokens = t;
}

function setTopP(t: number): void {
  topP = t;
}

export function getAIStore() {
  return {
    get messages() { return messages; },
    get selectedProvider() { return selectedProvider; },
    get selectedModel() { return selectedModel; },
    get isStreaming() { return isStreaming; },
    get providers() { return providers; },
    get templates() { return templates; },
    get error() { return error; },
    get temperature() { return temperature; },
    get maxTokens() { return maxTokens; },
    get topP() { return topP; },
    addMessage,
    sendMessage,
    stopStreaming,
    insertResponse,
    replaceSelection,
    retryLastMessage,
    clearConversation,
    loadTemplates,
    setProvider,
    setModel,
    setProviders,
    setTemperature,
    setMaxTokens,
    setTopP,
  };
}
