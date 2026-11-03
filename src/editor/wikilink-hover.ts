/**
 * Hover Preview de entidades e cenas em [[wikilinks]] (M2-04).
 *
 * Exibe popover flutuante contendo tipo, título, resumo/bio e contagem de menções
 * ao passar o mouse sobre o chip ou sintaxe do link.
 */

import { hoverTooltip, type Tooltip } from '@codemirror/view';
import { vault } from '../vault/vault.svelte';
import { manuscript } from '../vault/manuscript.svelte';
import { worldbuilding } from '../worldbuilding/worldbuilding.svelte';
import { getEntityKindLabel } from './wikilink-autocomplete';

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

      // Busca entidade no índice em memória ou no manifesto do manuscrito
      const indexedEntity = vault.entities.find(
        (e) => e.title.toLowerCase() === target.toLowerCase() || e.slug.toLowerCase() === target.toLowerCase()
      );
      const manifestEntity = manuscript.items.find(
        (e) => e.title.toLowerCase() === target.toLowerCase() || e.slug.toLowerCase() === target.toLowerCase()
      );
      const resolvedId = indexedEntity?.id || manifestEntity?.id;
      const resolvedTitle = indexedEntity?.title || manifestEntity?.title || target;
      const resolvedKind = indexedEntity?.kind || manifestEntity?.kind;

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
          title.textContent = resolvedTitle;
          header.appendChild(title);

          if (alias) {
            const aliasBadge = document.createElement('span');
            aliasBadge.className = 'cm-wikilink-hover-alias';
            aliasBadge.textContent = `(como "${alias}")`;
            header.appendChild(aliasBadge);
          }

          const badge = document.createElement('span');
          badge.className = 'cm-wikilink-hover-badge';
          badge.textContent = resolvedKind ? getEntityKindLabel(resolvedKind) : 'referência pendente';
          header.appendChild(badge);

          dom.appendChild(header);

          const body = document.createElement('div');
          body.className = 'cm-wikilink-hover-body';
          body.textContent =
            indexedEntity?.summary ||
            (resolvedId
              ? 'Ficha cadastrada no projeto. Clique abaixo para abrir e consultar.'
              : 'Esta entidade ainda não possui ficha criada no cofre.');
          dom.appendChild(body);

          const footer = document.createElement('div');
          footer.className = 'cm-wikilink-hover-footer';
          footer.style.display = 'flex';
          footer.style.alignItems = 'center';
          footer.style.justifyContent = 'space-between';
          footer.style.gap = '8px';

          const mentions = document.createElement('span');
          mentions.className = 'cm-wikilink-hover-mentions';
          mentions.textContent = indexedEntity
            ? `📌 ${indexedEntity.mention_count} menção(ões)`
            : resolvedId
              ? '✓ Cadastrado'
              : '⚠️ Não criado';
          footer.appendChild(mentions);

          const actionBtn = document.createElement('button');
          actionBtn.type = 'button';
          actionBtn.style.border = '1px solid var(--color-olive, #5E6F45)';
          actionBtn.style.background = 'transparent';
          actionBtn.style.color = 'var(--color-olive, #5E6F45)';
          actionBtn.style.borderRadius = '4px';
          actionBtn.style.padding = '2px 8px';
          actionBtn.style.fontSize = '11px';
          actionBtn.style.fontWeight = '600';
          actionBtn.style.cursor = 'pointer';

          if (resolvedId) {
            actionBtn.textContent = '✏️ Abrir Ficha';
            actionBtn.onclick = (e) => {
              e.preventDefault();
              e.stopPropagation();
              manuscript.selectScene(resolvedId);
            };
          } else {
            actionBtn.textContent = '+ Criar Personagem';
            actionBtn.onclick = (e) => {
              e.preventDefault();
              e.stopPropagation();
              void worldbuilding.createEntity('character', target, []);
            };
          }
          footer.appendChild(actionBtn);

          dom.appendChild(footer);

          return { dom };
        },
      };
    }
  }

  return null;
});
