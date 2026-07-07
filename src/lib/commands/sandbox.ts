import { invoke } from '@tauri-apps/api/core';

export interface SandboxResult {
  stdout: string;
  stderr: string;
  exit_code: number;
}

export async function sandboxExec(
  wasmPath: string,
  input?: string,
): Promise<SandboxResult> {
  return invoke('sandbox_exec', { wasmPath, input });
}

export async function sandboxListSnapshots(): Promise<string[]> {
  return invoke('sandbox_list_snapshots');
}

export async function sandboxCreateSnapshot(name: string): Promise<void> {
  return invoke('sandbox_create_snapshot', { name });
}

export async function sandboxRestoreSnapshot(name: string): Promise<void> {
  return invoke('sandbox_restore_snapshot', { name });
}

export async function sandboxDeleteSnapshot(name: string): Promise<void> {
  return invoke('sandbox_delete_snapshot', { name });
}
