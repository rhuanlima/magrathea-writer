/**
 * Hover Preview de entidades e cenas em [[wikilinks]] (M2-04).
 *
 * Exibe popover flutuante contendo tipo, título, resumo/bio e contagem de menções
 * ao passar o mouse sobre o chip ou sintaxe do link.
 */

import { hoverTooltip, type Tooltip } from '@codemirror/view';
import { vault } from '../vault/vault.svelte';

export const wikilinkHoverExtension = hoverTooltip((view, pos): Tooltip | null => {
  const line = view.state.doc.lineAt(pos);
  const lineText = line.text;
  const col = pos - line.from;

  // Busca todos os [[links]] nesta linha
  const regex = /\[\[([^\]|\n]+)(?:\|([^\]\n]+))?\]\]/g;
  let match: RegExpExecArray | null;

  while ((match = regex.exec(lineText)) !== null) {
    const start = match.index;
    const end = match.index + match[0].length;

    if (col >= start && col <= end) {
      const target = match[1]?.trim() || '';
      const alias = match[2]?.trim();

      // Busca entidade no índice em memória
      const entity = vault.entities.find(
        (e) => e.title.toLowerCase() === target.toLowerCase() || e.slug.toLowerCase() === target.toLowerCase()
      );

      return {
        pos: line.from + start,
        end: line.from + end,
        above: true,
        create() {
          const dom = document.createElement('div');
          dom.className = 'cm-wikilink-hover-card';

          const header = document.createElement('div');
          header.className = 'cm-wikilink-hover-header';

          const title = document.createElement('strong');
          title.textContent = entity ? entity.title : target;
          header.appendChild(title);

          if (alias) {
            const aliasBadge = document.createElement('span');
            aliasBadge.className = 'cm-wikilink-hover-alias';
            aliasBadge.textContent = `(como "${alias}")`;
            header.appendChild(aliasBadge);
          }

          const badge = document.createElement('span');
          badge.className = 'cm-wikilink-hover-badge';
          badge.textContent = entity ? entity.kind : 'referência pendente';
          header.appendChild(badge);

          dom.appendChild(header);

          const body = document.createElement('div');
          body.className = 'cm-wikilink-hover-body';
          body.textContent =
            entity?.summary ||
            (entity
              ? 'Nenhum resumo cadastrado nos metadados desta entidade.'
              : 'Esta entidade ainda não possui nota criada no cofre.');
          dom.appendChild(body);

          const footer = document.createElement('div');
          footer.className = 'cm-wikilink-hover-footer';

          const mentions = document.createElement('span');
          mentions.className = 'cm-wikilink-hover-mentions';
          mentions.textContent = entity
            ? `📌 ${entity.mention_count} menção(ões) no projeto`
            : '⚠️ Não criado';
          footer.appendChild(mentions);

          dom.appendChild(footer);

          return { dom };
        },
      };
    }
  }

  return null;
});
