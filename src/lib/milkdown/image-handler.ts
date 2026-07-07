import { invoke } from '@tauri-apps/api/core';

export async function handleImagePaste(
  dataUrl: string,
  fileName: string,
  vaultPath?: string
): Promise<string | null> {
  try {
    const markdown = await invoke<string>('import_image', {
      dataUrl,
      fileName,
      vaultPath: vaultPath || null,
    });
    return markdown;
  } catch (err) {
    console.error('Failed to import image:', err);
    return null;
  }
}

export async function handleImageDrop(
  files: FileList,
  vaultPath?: string
): Promise<string[]> {
  const results: string[] = [];

  for (const file of Array.from(files)) {
    if (!file.type.startsWith('image/')) continue;

    const reader = new FileReader();
    const dataUrl = await new Promise<string>((resolve, reject) => {
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });

    const markdown = await handleImagePaste(dataUrl, file.name, vaultPath);
    if (markdown) {
      results.push(markdown);
    }
  }

  return results;
}
