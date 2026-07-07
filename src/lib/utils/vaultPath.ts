/**
 * Normalize command path arguments to the vault-relative format expected by Tauri.
 * The backend owns vault resolution; the frontend should pass portable slash paths.
 */
export function toVaultRelativePath(path: string): string {
  return path.replace(/\\/g, '/').replace(/^\/+/, '');
}
