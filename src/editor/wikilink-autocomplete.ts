/**
 * Autocomplete para [[wikilinks]] ao digitar `[[` no editor (M2-02).
 *
 * Consulta o catálogo de entidades e cenas registradas no cofre em memória
 * e insere a sintaxe correta `[[Nome da Entidade]]`.
 */

import { autocompletion, type CompletionContext, type CompletionResult } from '@codemirror/autocomplete';
import { vault } from '../vault/vault.svelte';
import { manuscript } from '../vault/manuscript.svelte';

function wikilinkCompletionSource(context: CompletionContext): CompletionResult | null {
  // Detecta se o cursor está logo após `[[` com possíveis caracteres já digitados
  const word = context.matchBefore(/\[\[([^\]\n]*)$/);
  if (!word) return null;

  const query = word.text.slice(2).trim().toLowerCase();

  // Combina entidades conhecidas do índice e itens da árvore do manuscrito
  const knownEntities = vault.entities;
  const manuscriptScenes = manuscript.items.filter((it) => it.kind === 'scene');

  const optionsMap = new Map<string, { label: string; type: string; detail?: string; boost?: number }>();

  // Adiciona itens do manuscrito
  for (const scene of manuscriptScenes) {
    optionsMap.set(scene.title.toLowerCase(), {
      label: scene.title,
      type: 'scene',
      detail: 'Cena do Manuscrito',
      boost: 1,
    });
  }

  // Adiciona entidades do índice de segundo cérebro
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

  const options = Array.from(optionsMap.values())
    .filter((opt) => opt.label.toLowerCase().includes(query))
    .map((opt) => ({
      label: opt.label,
      type: opt.type,
      detail: opt.detail,
      apply: `[[${opt.label}]]`,
      boost: opt.boost,
    }));

  return {
    from: word.from,
    options,
  };
}

export const wikilinkAutocompleteExtension = autocompletion({
  override: [wikilinkCompletionSource],
  defaultKeymap: true,
});
