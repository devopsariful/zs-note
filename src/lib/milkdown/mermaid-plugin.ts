import { $view } from '@milkdown/utils';
import { codeBlockSchema } from '@milkdown/preset-commonmark';
import type { Node } from '@milkdown/prose/model';
import mermaid from 'mermaid';

let mermaidInitialized = false;

function ensureMermaid() {
  if (mermaidInitialized) return;
  mermaid.initialize({
    startOnLoad: false,
    theme: 'default',
    securityLevel: 'sandbox',
  });
  mermaidInitialized = true;
}

function renderMermaidSvg(id: string, definition: string): Promise<string> {
  ensureMermaid();
  return mermaid.render(id, definition).then((r) => r.svg);
}

export const mermaidPlugin = $view(codeBlockSchema.node, () => {
  return (node: Node) => {
    const attrs = node.attrs as Record<string, unknown> || {};
    const lang = (attrs['language'] as string) || (attrs['lang'] as string) || '';
    if (lang !== 'mermaid') {
      return { dom: document.createElement('div') };
    }

    const container = document.createElement('div');
    container.className = 'mermaid-wrapper';
    container.style.cssText =
      'position:relative;min-height:60px;display:flex;align-items:center;justify-content:center;';

    const loading = document.createElement('div');
    loading.textContent = 'Rendering diagram...';
    loading.style.cssText = 'color:var(--color-text-muted);font-size:13px;';
    container.appendChild(loading);

    const textContent = node.textContent || '';
    const id = `mermaid-${Math.random().toString(36).slice(2, 9)}`;

    renderMermaidSvg(id, textContent)
      .then((svg: string) => {
        container.innerHTML = svg;
        const svgEl = container.querySelector('svg');
        if (svgEl) {
          svgEl.style.cssText = 'max-width:100%;height:auto;';
        }
      })
      .catch(() => {
        container.innerHTML = '';
        const errorMsg = document.createElement('div');
        errorMsg.style.cssText =
          'color:var(--color-error,#e53e3e);font-size:13px;padding:8px;border:1px solid var(--color-error,#e53e3e);border-radius:var(--radius-md,6px);background:var(--color-error-bg,#fff5f5);';
        errorMsg.textContent = 'Failed to render Mermaid diagram';
        container.appendChild(errorMsg);
      });

    return { dom: container };
  };
});
