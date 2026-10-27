import { describe, it, expect, beforeEach } from 'vitest';
import { manuscript } from './manuscript.svelte';

describe('Manuscript Studio Tree, Metadata & Trash Flow (M1-06..M1-11)', () => {
  beforeEach(async () => {
    await manuscript.refreshTree();
  });

  it('carrega a árvore inicial com itens mock e seleciona a primeira cena ativa', () => {
    expect(manuscript.items.length).toBeGreaterThan(0);
    expect(manuscript.activeSceneId).toBe('mock-scene-1');
    expect(manuscript.activeScene?.title).toBe('Cena 1: A Névoa da Manhã');
    expect(manuscript.trashCount).toBe(0);
  });

  it('cria uma nova parte, capítulo e cena na hierarquia', async () => {
    const part = await manuscript.createItem('part', null, 'Parte II: A Viagem');
    expect(part.kind).toBe('part');
    expect(part.title).toBe('Parte II: A Viagem');

    const chap = await manuscript.createItem('chapter', part.id, 'Capítulo 2: No Espaço');
    expect(chap.kind).toBe('chapter');
    expect(chap.parent_id).toBe(part.id);

    const scene = await manuscript.createItem('scene', chap.id, 'Cena 2: O Hiperespaço');
    expect(scene.kind).toBe('scene');
    expect(scene.parent_id).toBe(chap.id);
    expect(manuscript.activeSceneId).toBe(scene.id);
  });

  it('renomeia um item da árvore com sucesso', async () => {
    await manuscript.renameItem('mock-scene-1', 'Cena 1: Renomeada');
    const updated = manuscript.items.find((i) => i.id === 'mock-scene-1');
    expect(updated?.title).toBe('Cena 1: Renomeada');
  });

  it('atualiza status literário e etiquetas (tags) de uma cena', async () => {
    await manuscript.updateSceneMetadata('mock-scene-1', 'final', ['clímax', 'revisado']);
    const updated = manuscript.items.find((i) => i.id === 'mock-scene-1');
    expect(updated?.status).toBe('final');
    expect(updated?.tags).toEqual(['clímax', 'revisado']);
  });

  it('move nó para a lixeira interna cifrada e restaura com sucesso', async () => {
    const sceneToTrash = await manuscript.createItem('scene', null, 'Cena Temporária');
    expect(manuscript.items.some((i) => i.id === sceneToTrash.id)).toBe(true);

    await manuscript.moveToTrash(sceneToTrash.id);
    expect(manuscript.items.some((i) => i.id === sceneToTrash.id)).toBe(false);
    expect(manuscript.trash.some((i) => i.id === sceneToTrash.id)).toBe(true);
    expect(manuscript.trashCount).toBeGreaterThan(0);

    await manuscript.restoreFromTrash(sceneToTrash.id);
    expect(manuscript.items.some((i) => i.id === sceneToTrash.id)).toBe(true);
    expect(manuscript.trash.some((i) => i.id === sceneToTrash.id)).toBe(false);
  });

  it('esvazia definitivamente a lixeira interna', async () => {
    const tempScene = await manuscript.createItem('scene', null, 'Cena Descartada');
    await manuscript.moveToTrash(tempScene.id);
    expect(manuscript.trashCount).toBeGreaterThan(0);

    await manuscript.emptyTrash();
    expect(manuscript.trashCount).toBe(0);
    expect(manuscript.trash).toHaveLength(0);
  });

  it('reordena nós irmãos para cima e para baixo', async () => {
    const itemA = await manuscript.createItem('part', null, 'Parte A');
    const itemB = await manuscript.createItem('part', null, 'Parte B');

    expect(itemA.order).toBeLessThanOrEqual(itemB.order);

    await manuscript.moveSibling(itemB.id, 'up');
    const refreshedB = manuscript.items.find((i) => i.id === itemB.id);
    const refreshedA = manuscript.items.find((i) => i.id === itemA.id);
    expect(refreshedB?.order).toBe(refreshedA?.order ? refreshedA.order - 1 : 0);
  });

  it('alterna visibilidade da barra lateral e da gaveta de lixeira', () => {
    expect(manuscript.isSidebarOpen).toBe(true);
    manuscript.toggleSidebar();
    expect(manuscript.isSidebarOpen).toBe(false);
    manuscript.toggleSidebar(true);
    expect(manuscript.isSidebarOpen).toBe(true);

    expect(manuscript.isTrashOpen).toBe(false);
    manuscript.toggleTrash(true);
    expect(manuscript.isTrashOpen).toBe(true);
    manuscript.toggleTrash(false);
    expect(manuscript.isTrashOpen).toBe(false);
  });
});
