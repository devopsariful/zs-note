import { Editor, rootCtx, defaultValueCtx, editorViewCtx } from '@milkdown/core';
import { commonmark } from '@milkdown/preset-commonmark';
import { gfm } from '@milkdown/preset-gfm';
import { history } from '@milkdown/plugin-history';
import { tooltipFactory } from '@milkdown/plugin-tooltip';
import { listener, listenerCtx } from '@milkdown/plugin-listener';
import { clipboard } from '@milkdown/plugin-clipboard';
import { cursor } from '@milkdown/plugin-cursor';
import { math } from '@milkdown/plugin-math';
import { mermaidPlugin } from './mermaid-plugin';
import { tableTooltipPlugin } from './table-plugin';

export interface MilkdownSetupOptions {
  content: string;
  readOnly?: boolean;
  onChange?: (content: string) => void;
  onSave?: () => void;
  onFormat?: (action: string) => void;
  root: HTMLElement;
}

export function createMilkdownEditor(options: MilkdownSetupOptions) {
  const { content, readOnly, onChange, onSave, onFormat, root } = options;

  const tooltip = tooltipFactory('format');

  const editor = Editor.make()
    .config((ctx) => {
      ctx.set(rootCtx, root);
      ctx.set(defaultValueCtx, content);
    })
    .use(commonmark)
    .use(gfm)
    .use(history)
    .use(tooltip)
    .use(clipboard)
    .use(cursor)
    .use(listener)
    .use(math)
    .use(mermaidPlugin)
    .use(tableTooltipPlugin)
    .config((ctx) => {
      const l = ctx.get(listenerCtx);

      l.markdownUpdated((_ctx, markdown) => {
        onChange?.(markdown);
      });

      l.mounted(() => {
        if (readOnly) {
          const view = ctx.get(editorViewCtx);
          view.dom.setAttribute('contenteditable', 'false');
        }
      });

      document.addEventListener('keydown', (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === 's') {
          e.preventDefault();
          onSave?.();
        }
      });
    });

  return {
    editor,
    async destroy() {
      await editor.destroy();
    },
    async getContent(): Promise<string> {
      try {
        const { getMarkdown } = await import('@milkdown/utils');
        return editor.action(getMarkdown());
      } catch {
        return '';
      }
    },
    async setContent(markdown: string) {
      editor.action((ctx) => {
        ctx.set(defaultValueCtx, markdown);
      });
    },
  };
}
