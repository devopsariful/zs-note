import type { McpServerInfo, McpConfirmationRequest } from '../types';
import { invoke } from '@tauri-apps/api/core';

// ── Confirmation state ──────────────────────────────────────────────────────
let pendingConfirmation = $state<McpConfirmationRequest | null>(null);
let confirmationResolver: ((result: { allow: boolean; always: boolean }) => void) | null = null;
let alwaysAllowed = $state<Map<string, true>>(new Map());

// ── Servers list ────────────────────────────────────────────────────────────
let servers = $state<McpServerInfo[]>([]);
let isLoading = $state(false);
let error = $state<string | null>(null);

// ── Derived ─────────────────────────────────────────────────────────────────
const connectedCount = $derived(servers.filter((s) => s.status === 'connected').length);
const totalTools = $derived(servers.reduce((acc, s) => acc + (s.tools?.length ?? 0), 0));

// ── Helpers ─────────────────────────────────────────────────────────────────

/**
 * Detect sensitive operations from tool name and args.
 */
function detectSensitiveOps(server: string, tool: string, args: Record<string, unknown>): string[] {
  const ops: string[] = [];
  const toolLower = tool.toLowerCase();

  const sensitivePatterns: [RegExp, string][] = [
    [/write|create|edit|modify|update|patch/, 'File write / modify'],
    [/delete|remove|rm|truncate|drop/, 'Delete / remove'],
    [/exec|execute|run|shell|bash|command|spawn/, 'Command execution'],
    [/sql|query|database|db\b/, 'Database query'],
    [/admin|sudo|root|chmod|chown/, 'Privileged operation'],
    [/network|http|fetch|curl|wget|request/, 'Network request'],
    [/email|sms|notify|send/, 'Send notification'],
  ];

  for (const [pattern, label] of sensitivePatterns) {
    if (pattern.test(toolLower)) {
      ops.push(label);
    }
  }

  // Check args for suspicious patterns
  const argsStr = JSON.stringify(args).toLowerCase();
  const argPatterns: [RegExp, string][] = [
    [/(^|\s)(rm|del|wipe|purge)\s/, 'Destructive argument detected'],
    [/sudo\s|chmod\s|chown\s/, 'Privilege escalation in args'],
    [/password|secret|token|api_key|apikey/, 'Credential in arguments'],
  ];

  for (const [pattern, label] of argPatterns) {
    if (pattern.test(argsStr)) {
      if (!ops.includes(label)) {
        ops.push(label);
      }
    }
  }

  return ops;
}

/**
 * Request user confirmation for an MCP tool call.
 * Returns a promise that resolves to true if confirmed, false if denied.
 */
function requestConfirmation(
  server: string,
  tool: string,
  args: Record<string, unknown>,
): Promise<boolean> {
  return new Promise((resolve) => {
    // Check if server/tool is always-allowed
    const key = `${server}/${tool}`;
    if (alwaysAllowed.has(key)) {
      resolve(true);
      return;
    }

    const id = crypto.randomUUID();
    const sensitiveOps = detectSensitiveOps(server, tool, args);

    pendingConfirmation = {
      server,
      tool,
      args,
      id,
      sensitiveOps,
    };

    confirmationResolver = (result: { allow: boolean; always: boolean }) => {
      pendingConfirmation = null;
      confirmationResolver = null;
      if (result.always) {
        alwaysAllowed = new Map(alwaysAllowed).set(key, true);
      }
      resolve(result.allow);
    };
  });
}

function resolveConfirmation(id: string, allow: boolean, always: boolean): void {
  if (pendingConfirmation && pendingConfirmation.id === id && confirmationResolver) {
    confirmationResolver({ allow, always });
  }
}

function dismissConfirmation(): void {
  if (confirmationResolver) {
    confirmationResolver({ allow: false, always: false });
  }
  pendingConfirmation = null;
  confirmationResolver = null;
}

// ── Server management ───────────────────────────────────────────────────────

async function loadServers(): Promise<void> {
  isLoading = true;
  error = null;

  try {
    servers = await invoke<McpServerInfo[]>('mcp_list_servers');
  } catch (err) {
    error = String(err);
  } finally {
    isLoading = false;
  }
}

async function toggleServer(id: string, enabled: boolean): Promise<void> {
  try {
    await invoke('mcp_toggle_server', { id, enabled });
    // Optimistically update
    const current = servers;
    const idx = current.findIndex((s) => s.id === id);
    if (idx !== -1) {
      const copy = current[idx];
      servers = [
        ...current.slice(0, idx),
        Object.assign({}, copy, { enabled }) as McpServerInfo,
        ...current.slice(idx + 1),
      ];
    }
  } catch (err) {
    error = String(err);
    throw err;
  }
}

async function removeServer(id: string): Promise<void> {
  try {
    await invoke('mcp_remove_server', { id });
    servers = servers.filter((s) => s.id !== id);
  } catch (err) {
    error = String(err);
    throw err;
  }
}

async function testConnection(id: string): Promise<boolean> {
  try {
    return await invoke<boolean>('mcp_test_connection', { id });
  } catch (err) {
    error = String(err);
    return false;
  }
}

// ── Export store ────────────────────────────────────────────────────────────

export function getMcpStore() {
  return {
    get pendingConfirmation() { return pendingConfirmation; },
    get servers() { return servers; },
    get isLoading() { return isLoading; },
    get error() { return error; },
    get connectedCount() { return connectedCount; },
    get totalTools() { return totalTools; },
    requestConfirmation,
    resolveConfirmation,
    dismissConfirmation,
    loadServers,
    toggleServer,
    removeServer,
    testConnection,
  };
}
