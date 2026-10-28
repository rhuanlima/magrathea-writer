import { describe, it, expect, beforeEach } from 'vitest';
import { preferences } from './preferences.svelte';
import { editorPreferences } from '../editor/editor-preferences.svelte';

describe('PreferencesStore (Ajustes 3 e 3.1)', () => {
  beforeEach(() => {
    preferences.close();
  });

  it('inicia fechado por padrão', () => {
    expect(preferences.isOpen).toBe(false);
  });

  it('abre na aba especificada e sincroniza com editorPreferences', () => {
    preferences.open('security');
    expect(preferences.isOpen).toBe(true);
    expect(preferences.activeTab).toBe('security');
    expect(editorPreferences.isModalOpen).toBe(true);

    preferences.setTab('editor');
    expect(preferences.activeTab).toBe('editor');
  });

  it('fecha o modal e sincroniza o encerramento', () => {
    preferences.open('focus');
    expect(preferences.isOpen).toBe(true);

    preferences.close();
    expect(preferences.isOpen).toBe(false);
    expect(editorPreferences.isModalOpen).toBe(false);
  });
});
