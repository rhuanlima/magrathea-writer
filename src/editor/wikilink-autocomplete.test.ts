import { describe, it, expect, beforeEach } from 'vitest';
import { vault } from '../vault/vault.svelte';
import { getAutocompleteSuggestions } from './wikilink-autocomplete';

describe('Autocompletar Literário: Links Indiretos e Wikilinks', () => {
  beforeEach(async () => {
    vault.isOpen = true;
    vault.isLocked = false;
    vault.vaultPath = '/mock/vault.magrathea';
    await vault.loadVaultIndex();
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
