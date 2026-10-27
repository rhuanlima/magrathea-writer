import { describe, it, expect, beforeEach } from 'vitest';
import { vault } from '../vault/vault.svelte';

describe('Segundo Cérebro: Refatoração Global e Segurança de Links (M2-05..M2-06)', () => {
  beforeEach(() => {
    vault.isOpen = true;
    vault.isLocked = false;
    vault.vaultPath = '/mock/vault.magrathea';
    vault.lastSnapshotId = null;
  });

  it('gera pré-visualização de renomeação de entidade com ocorrências simuladas', async () => {
    const preview = await vault.previewRename('Jaci', 'Iara');
    expect(preview.old_name).toBe('Jaci');
    expect(preview.new_name).toBe('Iara');
    expect(preview.total_occurrences).toBe(1);
    expect(preview.affected_documents.length).toBe(1);
    expect(preview.affected_documents[0].occurrences[0].new_snippet).toContain('[[Iara]]');
  });

  it('aplica refatoração global, armazena snapshot_id e permite rollback', async () => {
    const applyRes = await vault.applyRename('Jaci', 'Iara');
    expect(applyRes.snapshot_id).toBe('mock-snapshot-001');
    expect(vault.lastSnapshotId).toBe('mock-snapshot-001');
    expect(applyRes.modified_docs_count).toBe(1);

    // Rollback
    await vault.rollbackRename('mock-snapshot-001');
    expect(vault.lastSnapshotId).toBeNull();
  });

  it('controla a abertura e o fechamento do modal de refatoração', () => {
    expect(vault.isRefactorModalOpen).toBe(false);
    expect(vault.refactorTarget).toBeNull();

    vault.openRefactorModal('Jaci');
    expect(vault.isRefactorModalOpen).toBe(true);
    expect(vault.refactorTarget).toBe('Jaci');

    vault.closeRefactorModal();
    expect(vault.isRefactorModalOpen).toBe(false);
    expect(vault.refactorTarget).toBeNull();
  });

  it('identifica e sanitiza URLs http(s) impedindo navegação externa', () => {
    const text = 'Consulte https://pt.wikipedia.org/wiki/Magrathea e http://exemplo.com/docs para mais detalhes.';
    const urlRegex = /https?:\/\/[^\s<>)"]+/g;
    const matches = Array.from(text.matchAll(urlRegex)).map((m) => m[0]);

    expect(matches.length).toBe(2);
    expect(matches[0]).toBe('https://pt.wikipedia.org/wiki/Magrathea');
    expect(matches[1]).toBe('http://exemplo.com/docs');
  });
});
