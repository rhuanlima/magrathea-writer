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
  label: string;
  type: string;
  detail?: string;
  boost?: number;
}

/**
 * Coleta todas as entidades e cenas conhecidas do cofre.
 */
export function getKnownWorldItems(): EntityCompletionItem[] {
  const knownEntities = vault.entities;
  const manuscriptScenes = manuscript.items.filter((it) => it.kind === 'scene');
  const optionsMap = new Map<string, EntityCompletionItem>();

  // Adiciona itens do manuscrito
  for (const scene of manuscriptScenes) {
    optionsMap.set(scene.title.toLowerCase(), {
      label: scene.title,
      type: 'scene',
      detail: 'Cena do Manuscrito',
      boost: 1,
    });
  }

  // Adiciona entidades da enciclopédia / índice
  for (const ent of knownEntities) {
    const kindLabel =
      ent.kind === 'character'
        ? '👤 Personagem'
        : ent.kind === 'location'
          ? '📍 Local'
          : ent.kind === 'item'
            ? '🗝️ Objeto'
            : ent.kind === 'scene'
              ? '📄 Cena'
              : '📝 Nota';

    optionsMap.set(ent.title.toLowerCase(), {
      label: ent.title,
      type: ent.kind,
      detail: ent.summary || kindLabel,
      boost: ent.mention_count > 0 ? 2 : 1,
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
 */
function literaryLinkCompletionSource(context: CompletionContext): CompletionResult | null {
  // Verifica se há link indireto `[alias](...`
  const indirectWord = context.matchBefore(/\[([^\]\n]+)\]\(([^)\n]*)$/);
  if (indirectWord) {
    const res = getAutocompleteSuggestions(indirectWord.text);
    if (!res) return null;
    return {
      from: indirectWord.from,
      options: res.options,
    };
  }

  // Verifica se há wikilink direto `[[...`
  const wikiWord = context.matchBefore(/\[\[([^\]\n]*)$/);
  if (wikiWord) {
    const res = getAutocompleteSuggestions(wikiWord.text);
    if (!res) return null;
    return {
      from: wikiWord.from,
      options: res.options,
    };
  }

  return null;
}

export const wikilinkAutocompleteExtension = autocompletion({
  override: [literaryLinkCompletionSource],
  defaultKeymap: true,
});
