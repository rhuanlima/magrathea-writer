import { describe, it, expect, beforeEach } from 'vitest';
import { searchStore } from './search.svelte';

describe('Global Search & Replace Store (M5-03 & M5-04)', () => {
  beforeEach(() => {
    searchStore.query = '';
    searchStore.replacement = '';
    searchStore.matchCase = false;
    searchStore.searchResults = null;
    searchStore.replacePreview = null;
    searchStore.lastRollbackId = null;
    searchStore.errorMessage = null;
    searchStore.successMessage = null;
    searchStore.closeModal();
  });

  it('inicia com campos zerados e modal fechado', () => {
    expect(searchStore.isModalOpen).toBe(false);
    expect(searchStore.searchResults).toBeNull();
    expect(searchStore.replacePreview).toBeNull();
  });

  it('abre e fecha modal corretamente', () => {
    searchStore.openModal();
    expect(searchStore.isModalOpen).toBe(true);
    searchStore.closeModal();
    expect(searchStore.isModalOpen).toBe(false);
  });

  it('executa busca global e armazena os resultados', async () => {
    searchStore.query = 'Arthur';
    await searchStore.executeSearch();

    expect(searchStore.searchResults).toBeDefined();
    expect(searchStore.searchResults?.total_matches).toBeGreaterThan(0);
    expect(searchStore.searchResults?.documents.length).toBeGreaterThan(0);
  });

  it('gera prévia de substituição global', async () => {
    searchStore.query = 'Arthur';
    searchStore.replacement = 'Ford';
    await searchStore.previewReplace();

    expect(searchStore.replacePreview).toBeDefined();
    expect(searchStore.replacePreview?.total_occurrences).toBeGreaterThan(0);
    expect(searchStore.replacePreview?.items.length).toBeGreaterThan(0);
  });

  it('aplica substituição global e permite reversão (rollback)', async () => {
    searchStore.query = 'Arthur';
    searchStore.replacement = 'Ford';
    const applyRes = await searchStore.applyReplace();

    expect(applyRes).toBeDefined();
    expect(applyRes?.modified_docs_count).toBeGreaterThan(0);
    expect(searchStore.lastRollbackId).toBe(applyRes?.rollback_snapshot_id);

    const rollbackSuccess = await searchStore.rollback();
    expect(rollbackSuccess).toBe(true);
    expect(searchStore.lastRollbackId).toBeNull();
  });
});
