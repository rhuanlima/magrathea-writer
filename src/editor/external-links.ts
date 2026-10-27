/**
 * Extensão CodeMirror 6 para renderização segura de links externos HTTP(S) (M2-06).
 *
 * Em conformidade estrita com o PRD (E4) e SECURITY.md (§3.5):
 * - Links http(s):// NUNCA são abertos pelo aplicativo (zero tráfego de rede e anti-phishing).
 * - Renderizados como chip neutro com botão de 1 clique para "Copiar URL".
 * - O clique no link não navega nem dispara aberturas de janelas externas.
 */

import {
  MatchDecorator,
  WidgetType,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
  Decoration,
  type DecorationSet,
} from '@codemirror/view';

class ExternalLinkWidget extends WidgetType {
  constructor(public readonly url: string) {
    super();
  }

  eq(other: ExternalLinkWidget) {
    return this.url === other.url;
  }

  toDOM() {
    const span = document.createElement('span');
    span.className = 'cm-external-link-chip';
    span.setAttribute('data-url', this.url);
    span.setAttribute('title', `${this.url} (Não clicável por segurança)`);

    // Previne qualquer navegação nativa ao clicar
    span.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
    });

    const icon = document.createElement('span');
    icon.className = 'cm-external-link-icon';
    icon.textContent = '🌐';
    span.appendChild(icon);

    // Formata o texto exibido da URL encurtando se for muito longa
    const urlText = document.createElement('span');
    urlText.className = 'cm-external-link-text';
    const displayUrl = this.url.replace(/^https?:\/\//i, '');
    urlText.textContent = displayUrl.length > 35 ? `${displayUrl.slice(0, 32)}…` : displayUrl;
    span.appendChild(urlText);

    // Botão discreto para copiar a URL para a área de transferência
    const copyBtn = document.createElement('button');
    copyBtn.type = 'button';
    copyBtn.className = 'cm-external-link-copy';
    copyBtn.setAttribute('title', 'Copiar URL');
    copyBtn.setAttribute('aria-label', 'Copiar URL');
    copyBtn.textContent = '📋';

    copyBtn.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      void (async () => {
        try {
          if (navigator.clipboard) {
            await navigator.clipboard.writeText(this.url);
            copyBtn.textContent = '✓';
            copyBtn.classList.add('copied');
            setTimeout(() => {
              copyBtn.textContent = '📋';
              copyBtn.classList.remove('copied');
            }, 1500);
          }
        } catch {
          // Fallback silencioso
        }
      })();
    });

    span.appendChild(copyBtn);

    return span;
  }

  ignoreEvent(event: Event) {
    // Intercepta eventos de clique para não abrir links externos
    if (event.type === 'click' || event.type === 'mousedown') {
      return false;
    }
    return true;
  }
}

const externalLinkMatcher = new MatchDecorator({
  regexp: /https?:\/\/[^\s<>)"]+/g,
  decoration: (match) => {
    const url = match[0];
    return Decoration.replace({
      widget: new ExternalLinkWidget(url),
    });
  },
});

export const externalLinksPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = externalLinkMatcher.createDeco(view);
    }

    update(update: ViewUpdate) {
      this.decorations = externalLinkMatcher.updateDeco(update, this.decorations);
    }
  },
  {
    decorations: (instance) => instance.decorations,
  }
);
