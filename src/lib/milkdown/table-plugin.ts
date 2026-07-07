import { $view } from '@milkdown/utils';
import { tableSchema } from '@milkdown/preset-gfm';
import type { Node } from '@milkdown/prose/model';

export const tableTooltipPlugin = $view(tableSchema.node, () => {
  return (node: Node) => {
    const isTable = node.type.name === 'table';
    if (!isTable) {
      return { dom: document.createElement('div') };
    }

    const dom = document.createElement('div');
    dom.className = 'table-wrapper';
    dom.style.cssText = 'position:relative;display:inline-block;';

    const toolbar = document.createElement('div');
    toolbar.className = 'table-toolbar';
    toolbar.style.cssText = [
      'display:flex;gap:4px;padding:4px 8px;',
      'background:var(--color-surface,#f8f9fa);',
      'border:1px solid var(--color-border,#dee2e6);',
      'border-radius:var(--radius-md,6px);',
      'box-shadow:0 2px 8px rgba(0,0,0,0.1);',
      'position:absolute;top:-36px;left:0;z-index:100;',
      'font-size:12px;opacity:0;',
      'transition:opacity 0.15s;',
    ].join('');
    dom.appendChild(toolbar);

    dom.addEventListener('mouseenter', () => {
      toolbar.style.opacity = '1';
      dom.style.outline = '2px solid var(--color-accent,#7c3aed)';
      dom.style.outlineOffset = '2px';
    });
    dom.addEventListener('mouseleave', () => {
      toolbar.style.opacity = '0';
      dom.style.outline = '';
      dom.style.outlineOffset = '';
    });

    const buttons = [
      { label: '⬆', title: 'Row above' },
      { label: '⬇', title: 'Row below' },
      { label: '⬅', title: 'Col left' },
      { label: '➡', title: 'Col right' },
      { label: '✕R', title: 'Delete row' },
      { label: '✕C', title: 'Delete column' },
    ];

    for (const btn of buttons) {
      const el = document.createElement('button');
      el.textContent = btn.label;
      el.title = btn.title;
      el.style.cssText = [
        'padding:2px 6px;border:1px solid var(--color-border,#dee2e6);',
        'border-radius:var(--radius-sm,4px);cursor:pointer;',
        'background:var(--color-bg,white);color:var(--color-text,#1a1a2e);',
        'line-height:1.4;font-size:11px;',
      ].join('');
      el.addEventListener('mouseenter', () => {
        el.style.background = 'var(--color-accent,#7c3aed)';
        el.style.color = 'white';
      });
      el.addEventListener('mouseleave', () => {
        el.style.background = 'var(--color-bg,white)';
        el.style.color = 'var(--color-text,#1a1a2e)';
      });
      toolbar.appendChild(el);
    }

    return { dom };
  };
});
