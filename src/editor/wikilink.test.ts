import { describe, it, expect, beforeEach } from 'vitest';
import { vault } from '../vault/vault.svelte';

describe('Segundo Cérebro: Wikilinks e Índice em Memória (M2-01..M2-04)', () => {
  beforeEach(async () => {
    vault.isOpen = true;
    vault.isLocked = false;
    vault.vaultPath = '/mock/vault.magrathea';
  });

  it('carrega o resumo do índice de entidades do cofre', async () => {
    const overview = await vault.loadVaultIndex();
    expect(overview).not.toBeNull();
    expect(overview?.entities.length).toBeGreaterThan(0);
    expect(vault.entities.length).toBeGreaterThan(0);

    const firstEntity = vault.entities[0];
    expect(firstEntity.id).toBeDefined();
    expect(firstEntity.title).toBeDefined();
    expect(firstEntity.kind).toBeDefined();
  });

  it('carrega os backlinks para um documento específico', async () => {
    const backlinks = await vault.loadBacklinks('mock-scene-1');
    expect(backlinks.length).toBe(1);
    expect(backlinks[0].source_title).toBe('Cena 2: O Chamado');
    expect(backlinks[0].target_query).toBe('A Névoa da Manhã');
    expect(backlinks[0].snippet).toContain('Cena 1: A Névoa da Manhã');
  });

  it('controla a abertura e as abas do painel inspetor', () => {
    expect(vault.isInspectorOpen).toBe(false);
    vault.toggleInspector(true);
    expect(vault.isInspectorOpen).toBe(true);

    expect(vault.activeInspectorTab).toBe('metadata');
    vault.setInspectorTab('backlinks');
    expect(vault.activeInspectorTab).toBe('backlinks');

    vault.toggleInspector(false);
    expect(vault.isInspectorOpen).toBe(false);
  });

  it('limpa o índice e os backlinks ao bloquear o cofre', async () => {
    await vault.loadVaultIndex();
    await vault.loadBacklinks('mock-scene-1');
    vault.toggleInspector(true);

    expect(vault.indexOverview).not.toBeNull();
    expect(vault.activeBacklinks.length).toBeGreaterThan(0);

    await vault.lockVault();

    expect(vault.isLocked).toBe(true);
    expect(vault.indexOverview).toBeNull();
    expect(vault.activeBacklinks.length).toBe(0);
    expect(vault.isInspectorOpen).toBe(false);
  });
});
