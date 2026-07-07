import { invoke } from '@tauri-apps/api/core';

export async function importFiles(sourcePaths: string[], targetDir: string): Promise<string[]> {
  return invoke('import_files', { sourcePaths, targetDir });
}

export async function importImage(dataUrl: string, fileName: string): Promise<string> {
  return invoke('import_image', { dataUrl, fileName });
}
