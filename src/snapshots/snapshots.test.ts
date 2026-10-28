import { describe, it, expect, beforeEach } from 'vitest';
import { snapshots } from './snapshots.svelte';

describe('Snapshots & Diff Store (M5-01 & M5-02)', () => {
  beforeEach(() => {
    snapshots.snapshots = [];
    snapshots.activeDiff = null;
    snapshots.selectedSnapshot = null;
    snapshots.errorMessage = null;
    snapshots.successMessage = null;
  });

  it('inicia com lista vazia e sem snapshot selecionado', () => {
    expect(snapshots.snapshots).toEqual([]);
    expect(snapshots.selectedSnapshot).toBeNull();
    expect(snapshots.activeDiff).toBeNull();
    expect(snapshots.errorMessage).toBeNull();
  });

  it('cria um novo snapshot e atualiza a lista', async () => {
    const meta = await snapshots.createSnapshot(
      'scene-1',
      'Versão Inicial do Autor',
      'Primeiro parágrafo do capítulo.'
    );

    expect(meta).toBeDefined();
    expect(meta?.id).toBeDefined();
    expect(meta?.label).toBe('Versão Inicial do Autor');
    expect(snapshots.snapshots.length).toBeGreaterThan(0);
    expect(snapshots.snapshots[0].id).toBe(meta?.id);
  });

  it('compara diff de conteúdo atual contra snapshot selecionado', async () => {
    const meta = await snapshots.createSnapshot(
      'scene-1',
      'Antes da invasão',
      'Era uma vez um castelo antigo.\nTudo era silêncio.'
    );

    expect(meta).toBeDefined();
    if (!meta) return;

    await snapshots.compareWithCurrent(
      'scene-1',
      meta.id,
      'Era uma vez um castelo moderno.\nTudo era silêncio.'
    );

    expect(snapshots.selectedSnapshot?.id).toBe(meta.id);
    expect(snapshots.activeDiff).toBeDefined();
    expect(snapshots.activeDiff?.lines.length).toBeGreaterThan(0);
  });

  it('restaura snapshot com criação de snapshot de segurança', async () => {
    const meta = await snapshots.createSnapshot(
      'scene-1',
      'Original',
      'Texto antes da alteração incorreta.'
    );

    expect(meta).toBeDefined();
    if (!meta) return;

    const restoredContent = await snapshots.restoreSnapshot(
      'scene-1',
      meta.id,
      'Texto incorreto que precisa ser desfeito.'
    );

    expect(restoredContent).toBe('Texto antes da alteração incorreta.');
    expect(snapshots.snapshots.length).toBeGreaterThan(1); // contém o snapshot de segurança gerado
    expect(snapshots.successMessage).toBeDefined();
  });
});
