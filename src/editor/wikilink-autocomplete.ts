/**
 * Autocomplete para [[wikilinks]] e links indiretos [alias](alvo) no editor (M2-02 & Ajustes UX).
 *
 * Suporta:
 * 1. Wikilinks diretos: `[[Nome da Entidade]]` ao digitar `[[`
 * 2. Links indiretos markdown: `[alias visível](Nome da Entidade)` ao digitar `[alias](`
 *
 * Consulta o catálogo de entidades e cenas registradas no cofre em memória.
 */

import { autocompletion, type CompletionContext, type CompletionResult } from '@codemirror/autocomplete';
import { vault } from '../vault/vault.svelte';
import { manuscript } from '../vault/manuscript.svelte';

export interface EntityCompletionItem {
  id?: string;
  label: string;
  type: string;
  detail?: string;
  boost?: number;
}

export function getEntityKindLabel(kind: string): string {
  switch (kind) {
    case 'character':
      return '👤 Personagem';
    case 'location':
      return '📍 Local';
    case 'item':
      return '🗝️ Objeto';
    case 'faction':
      return '⚔️ Facção';
    case 'lore':
      return '📜 Lore';
    case 'chapter':
      return '📖 Capítulo';
    case 'scene':
      return '🎬 Cena do Manuscrito';
    default:
      return '📝 Nota / Entidade';
  }
}

function getKindBoost(kind: string, mentionCount = 0): number {
  const base =
    kind === 'character'
      ? 4
      : ['location', 'item', 'faction', 'lore', 'entity'].includes(kind)
        ? 3
        : 1;
  return mentionCount > 0 ? base + 1 : base;
}

/**
 * Coleta todas as entidades (personagens, locais, itens, facções, lore) e cenas/capítulos conhecidos do cofre.
 */
export function getKnownWorldItems(): EntityCompletionItem[] {
  const knownEntities = vault.entities;
  const optionsMap = new Map<string, EntityCompletionItem>();

  // 1. Adiciona todos os itens do manifesto (personagens, locais, itens, facções, lore, capítulos e cenas)
  for (const item of manuscript.items) {
    if (item.kind === 'part') continue;
    optionsMap.set(item.title.toLowerCase(), {
      id: item.id,
      label: item.title,
      type: item.kind,
      detail: getEntityKindLabel(item.kind),
      boost: getKindBoost(item.kind),
    });
  }

  // 2. Enriquece/adiciona entidades do índice em memória (com resumo e contagem de menções)
  for (const ent of knownEntities) {
    if (ent.kind === 'part') continue;
    const kindLabel = getEntityKindLabel(ent.kind);

    optionsMap.set(ent.title.toLowerCase(), {
      id: ent.id,
      label: ent.title,
      type: ent.kind,
      detail: ent.summary ? `${kindLabel} · ${ent.summary}` : kindLabel,
      boost: getKindBoost(ent.kind, ent.mention_count),
    });
  }

  return Array.from(optionsMap.values());
}

/**
 * Resolve sugestões para uma string digitada imediatamente antes do cursor.
 * Útil para testes unitários e lógica pura.
 */
export function getAutocompleteSuggestions(beforeText: string): {
  kind: 'wikilink' | 'indirect';
  query: string;
  alias?: string;
  matchStart: number;
  options: { label: string; type: string; detail?: string; apply: string; boost?: number }[];
} | null {
  // Caso 1: Link indireto no formato `[alias](alvo`
  const indirectRegex = /\[([^\]\n]+)\]\(([^)\n]*)$/;
  const indirectMatch = beforeText.match(indirectRegex);

  if (indirectMatch) {
    const alias = indirectMatch[1];
    const query = indirectMatch[2].trim().toLowerCase();
    const matchStart = beforeText.length - indirectMatch[0].length;
    const allItems = getKnownWorldItems();

    const options = allItems
      .filter((opt) => opt.label.toLowerCase().includes(query))
      .map((opt) => ({
        label: opt.label,
        type: opt.type,
        detail: opt.detail,
        apply: `[${alias}](${opt.label})`,
        boost: opt.boost,
      }));

    return {
      kind: 'indirect',
      query,
      alias,
      matchStart,
      options,
    };
  }

  // Caso 2: Wikilink clássico no formato `[[alvo`
  const wikilinkRegex = /\[\[([^\]\n]*)$/;
  const wikiMatch = beforeText.match(wikilinkRegex);

  if (wikiMatch) {
    const query = wikiMatch[1].trim().toLowerCase();
    const matchStart = beforeText.length - wikiMatch[0].length;
    const allItems = getKnownWorldItems();

    const options = allItems
      .filter((opt) => opt.label.toLowerCase().includes(query))
      .map((opt) => ({
        label: opt.label,
        type: opt.type,
        detail: opt.detail,
        apply: `[[${opt.label}]]`,
        boost: opt.boost,
      }));

    return {
      kind: 'wikilink',
      query,
      matchStart,
      options,
    };
  }

  return null;
}

/**
 * Fonte de autocompletion para o CodeMirror 6.
 * Considera os colchetes/parênteses de fechamento inseridos automaticamente por `closeBrackets()`.
 */
function literaryLinkCompletionSource(context: CompletionContext): CompletionResult | null {
  // Verifica se há link indireto `[alias](...`
  const indirectWord = context.matchBefore(/\[([^\]\n]+)\]\(([^)\n]*)$/);
  if (indirectWord) {
    const res = getAutocompleteSuggestions(indirectWord.text);
    if (!res) return null;
    const after = context.state.sliceDoc(context.pos, context.pos + 1);
    const to = after === ')' ? context.pos + 1 : context.pos;
    return {
      from: indirectWord.from,
      to,
      options: res.options,
      filter: false,
    };
  }

  // Verifica se há wikilink direto `[[...`
  const wikiWord = context.matchBefore(/\[\[([^\]\n]*)$/);
  if (wikiWord) {
    const res = getAutocompleteSuggestions(wikiWord.text);
    if (!res) return null;
    const after = context.state.sliceDoc(context.pos, context.pos + 2);
    const to = after === ']]' ? context.pos + 2 : after.startsWith(']') ? context.pos + 1 : context.pos;
    return {
      from: wikiWord.from,
      to,
      options: res.options,
      filter: false,
    };
  }

  return null;
}

export const wikilinkAutocompleteExtension = autocompletion({
  override: [literaryLinkCompletionSource],
  defaultKeymap: true,
});
