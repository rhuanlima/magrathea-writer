import { describe, it, expect, beforeEach } from 'vitest';
import { vault } from '../vault/vault.svelte';
import { manuscript } from '../vault/manuscript.svelte';
import { getAutocompleteSuggestions } from './wikilink-autocomplete';

describe('Autocompletar Literário: Links Indiretos e Wikilinks', () => {
  beforeEach(async () => {
    vault.isOpen = true;
    vault.isLocked = false;
    vault.vaultPath = '/mock/vault.magrathea';
    manuscript.items = [];
    await vault.loadVaultIndex();
  });

  it('inclui personagens, locais e capítulos do manifesto mesmo antes do índice carregar', () => {
    vault.indexOverview = null;
    manuscript.items = [
      {
        id: 'char-1',
        kind: 'character',
        parent_id: null,
        title: 'Thorin Marreta-Negra',
        slug: 'thorin-marreta-negra',
        blob_id: 'char-1',
        order: 0,
        tags: ['anao'],
        status: null,
      },
      {
        id: 'chap-1',
        kind: 'chapter',
        parent_id: null,
        title: 'Capítulo 1: A Taverna',
        slug: 'capitulo-1-a-taverna',
        blob_id: null,
        order: 1,
        tags: [],
        status: 'draft',
      },
    ];

    const suggestions = getAutocompleteSuggestions('No salão estava [[tho');
    expect(suggestions).not.toBeNull();
    expect(suggestions?.options.some((o) => o.label === 'Thorin Marreta-Negra')).toBe(true);
    expect(suggestions?.options[0].apply).toBe('[[Thorin Marreta-Negra]]');
  });

  it('detecta digitação de link indireto [alias]( e sugere entidades com aplicação limpa', () => {
    const textBeforeCursor = 'A Jaci estava na [casa dela](';
    const suggestions = getAutocompleteSuggestions(textBeforeCursor);

    expect(suggestions).not.toBeNull();
    expect(suggestions?.kind).toBe('indirect');
    expect(suggestions?.alias).toBe('casa dela');
    expect(suggestions?.query).toBe('');
    expect(suggestions?.options.length).toBeGreaterThan(0);

    // O apply deve formatar como [casa dela](Nome da Entidade)
    const first = suggestions!.options[0];
    expect(first.apply).toBe(`[casa dela](${first.label})`);
  });

  it('filtra entidades em links indiretos pelo termo digitado dentro de parênteses', () => {
    const textBeforeCursor = 'Ela pegou a [arma de energia](pis';
    const suggestions = getAutocompleteSuggestions(textBeforeCursor);

    expect(suggestions).not.toBeNull();
    expect(suggestions?.kind).toBe('indirect');
    expect(suggestions?.alias).toBe('arma de energia');
    expect(suggestions?.query).toBe('pis');
  });

  it('mantém funcionamento clássico de wikilink ao digitar [[', () => {
    const textBeforeCursor = 'Capítulo 1 começou em [[';
    const suggestions = getAutocompleteSuggestions(textBeforeCursor);

    expect(suggestions).not.toBeNull();
    expect(suggestions?.kind).toBe('wikilink');
    expect(suggestions?.options.length).toBeGreaterThan(0);

    const first = suggestions!.options[0];
    expect(first.apply).toBe(`[[${first.label}]]`);
  });

  it('retorna null se o texto anterior não corresponder a links ou wikilinks', () => {
    expect(getAutocompleteSuggestions('Texto normal sem links')).toBeNull();
    expect(getAutocompleteSuggestions('Apenas [colchete aberto')).toBeNull();
    expect(getAutocompleteSuggestions('Apenas um (parêntese comum')).toBeNull();
  });
});
