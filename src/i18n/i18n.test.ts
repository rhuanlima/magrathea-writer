import { describe, it, expect, beforeEach } from 'vitest';
import { i18n, t } from './index';
import { enUS } from './locales/en-US';
import { ptBR } from './locales/pt-BR';

/**
 * Extracts all leaf keys from a nested translation dictionary into dot-notation strings.
 */
function getAllLeafKeys(obj: Record<string, unknown>, prefix = ''): string[] {
  let keys: string[] = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullPath = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      keys = keys.concat(getAllLeafKeys(value as Record<string, unknown>, fullPath));
    } else {
      keys.push(fullPath);
    }
  }
  return keys;
}

describe('i18n Localization Foundation', () => {
  beforeEach(() => {
    i18n.setLocale('en-US');
  });

  it('translates simple keys in English and Portuguese', () => {
    i18n.setLocale('en-US');
    expect(t('app.title')).toBe('Magrathea Writer');
    expect(t('common.cancel')).toBe('Cancel');

    i18n.setLocale('pt-BR');
    expect(t('app.title')).toBe('Magrathea Writer');
    expect(t('common.cancel')).toBe('Cancelar');
  });

  it('correctly interpolates dynamic parameters', () => {
    i18n.setLocale('en-US');
    expect(t('editor.wordsCount', { count: 1250 })).toBe('1250 words');
    expect(t('editor.charactersCount', { count: 7400 })).toBe('7400 characters');

    i18n.setLocale('pt-BR');
    expect(t('editor.wordsCount', { count: 1250 })).toBe('1250 palavras');
    expect(t('editor.charactersCount', { count: 7400 })).toBe('7400 caracteres');
  });

  it('preserves placeholder syntax if parameter is missing', () => {
    i18n.setLocale('en-US');
    expect(t('editor.wordsCount')).toBe('{count} words');
  });

  it('enforces 100% key parity between en-US and pt-BR catalogues', () => {
    const enKeys = getAllLeafKeys(enUS as unknown as Record<string, unknown>).sort();
    const ptKeys = getAllLeafKeys(ptBR as unknown as Record<string, unknown>).sort();

    const missingInPt = enKeys.filter((k) => !ptKeys.includes(k));
    const missingInEn = ptKeys.filter((k) => !enKeys.includes(k));

    expect(missingInPt, `Keys present in en-US but missing in pt-BR: ${missingInPt.join(', ')}`).toEqual([]);
    expect(missingInEn, `Keys present in pt-BR but missing in en-US: ${missingInEn.join(', ')}`).toEqual([]);
    expect(enKeys).toEqual(ptKeys);
  });
});
