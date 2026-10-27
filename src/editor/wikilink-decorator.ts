/**
 * Decoração inline para [[wikilinks]] no CodeMirror 6 (M2-02).
 *
 * Transforma menções `[[Alvo]]` ou `[[Alvo|Apelido]]` em chips bronze estilizados,
 * mantendo a edição do Markdown bruto quando o cursor entra no intervalo do link.
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

class WikilinkWidget extends WidgetType {
  constructor(
    public readonly target: string,
    public readonly label: string,
    public readonly raw: string
  ) {
    super();
  }

  eq(other: WikilinkWidget) {
    return this.target === other.target && this.label === other.label && this.raw === other.raw;
  }

  toDOM() {
    const span = document.createElement('span');
    span.className = 'cm-wikilink-chip';
    span.setAttribute('data-target', this.target);
    span.setAttribute('data-label', this.label);
    span.setAttribute('title', `[[${this.target}]]`);

    const icon = document.createElement('span');
    icon.className = 'cm-wikilink-icon';
    icon.textContent = '🔗';
    span.appendChild(icon);

    const text = document.createElement('span');
    text.className = 'cm-wikilink-text';
    text.textContent = this.label;
    span.appendChild(text);

    return span;
  }

  ignoreEvent() {
    return false;
  }
}

const wikilinkMatcher = new MatchDecorator({
  regexp: /\[\[([^\]|\n]+)(?:\|([^\]\n]+))?\]\]/g,
  decoration: (match) => {
    const raw = match[0];
    const target = match[1]?.trim() || '';
    const label = match[2]?.trim() || target;

    return Decoration.replace({
      widget: new WikilinkWidget(target, label, raw),
    });
  },
});

export const wikilinkDecoratorPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = wikilinkMatcher.createDeco(view);
    }

    update(update: ViewUpdate) {
      this.decorations = wikilinkMatcher.updateDeco(update, this.decorations);
    }
  },
  {
    decorations: (instance) => instance.decorations,
  }
);
