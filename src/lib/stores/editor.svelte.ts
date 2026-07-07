import type { FileEntry, IngestProgress } from '../types';
import * as editorCommands from '../commands/editor';
import { toVaultRelativePath } from '../utils/vaultPath';

let activeFilePath = $state<string>('');
let content = $state<string>('');
let previousContent = $state<string>('');
let editorMode = $state<'wysiwyg' | 'source' | 'split'>('wysiwyg');
let cursorPosition = $state<{ line: number; col: number }>({ line: 1, col: 1 });
let isLoading = $state(false);
let error = $state<string | null>(null);

// ── Ingestion progress ──────────────────────────────────────────────────────
let ingestProgress = $state<IngestProgress[]>([]);

const isDirty = $derived(content !== previousContent);

function openFile(path: string): void {
  const relativePath = toVaultRelativePath(path);
  isLoading = true;
  error = null;

  editorCommands.readFile(relativePath)
    .then((fileContent) => {
      content = fileContent;
      previousContent = fileContent;
      activeFilePath = relativePath;
      isLoading = false;
    })
    .catch((err) => {
      error = String(err);
      isLoading = false;
    });
}

function saveFile(): Promise<void> {
  if (!activeFilePath) return Promise.resolve();
  error = null;

  return editorCommands.saveFile(toVaultRelativePath(activeFilePath), content)
    .then(() => {
      previousContent = content;
    })
    .catch((err) => {
      error = String(err);
      throw err;
    });
}

function setMode(mode: 'wysiwyg' | 'source' | 'split'): void {
  editorMode = mode;
}

function updateContent(newContent: string): void {
  content = newContent;
}

function setCursorPosition(line: number, col: number): void {
  cursorPosition = { line, col };
}

function closeFile(): void {
  activeFilePath = '';
  content = '';
  previousContent = '';
}

// ── Ingestion helpers ───────────────────────────────────────────────────────

function addIngestItem(item: IngestProgress): void {
  ingestProgress = [...ingestProgress, item];
}

function updateIngestProgress(fileName: string, percent: number, status: IngestProgress['status'], errorMsg?: string): void {
  ingestProgress = ingestProgress.map((item) =>
    item.fileName === fileName
      ? { ...item, percent, status, error: errorMsg ?? item.error }
      : item,
  );
}

function clearIngestProgress(): void {
  ingestProgress = [];
}

function handleDrop(_files: File[]): void {
  // The actual drop handling is done in the Editor component's drag-and-drop event handler.
  // This function is available for programmatic use from other parts of the app.
  // It clears previous progress for a new batch.
  clearIngestProgress();
}

export function getEditorStore() {
  return {
    get activeFilePath() { return activeFilePath; },
    get content() { return content; },
    get editorMode() { return editorMode; },
    get cursorPosition() { return cursorPosition; },
    get isDirty() { return isDirty; },
    get isLoading() { return isLoading; },
    get error() { return error; },
    set error(v: string | null) { error = v; },
    get ingestProgress() { return ingestProgress; },
    openFile,
    saveFile,
    setMode,
    updateContent,
    setCursorPosition,
    closeFile,
    addIngestItem,
    updateIngestProgress,
    clearIngestProgress,
    handleDrop,
  };
}
