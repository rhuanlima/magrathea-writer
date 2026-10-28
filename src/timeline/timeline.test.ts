import { describe, it, expect, beforeEach } from 'vitest';
import { TimelineStore } from './timeline.svelte';

describe('Timeline Store & Chronology (M7)', () => {
  let store: TimelineStore;

  beforeEach(() => {
    store = new TimelineStore();
  });

  it('deve inicializar com modal fechado e modo de leitura do livro ativo', () => {
    expect(store.isModalOpen).toBe(false);
    expect(store.activeViewMode).toBe('narrative');
    expect(store.selectedCharacter).toBeNull();
    expect(store.selectedLocation).toBeNull();
    expect(store.searchQuery).toBe('');
    expect(store.showPacing).toBe(false);
  });

  it('deve abrir o modal e carregar eventos da timeline via mock IPC', async () => {
    await store.openModal();

    expect(store.isModalOpen).toBe(true);
    expect(store.data).not.toBeNull();
    expect(store.data?.total_events).toBe(3);
    expect(store.data?.total_flashbacks).toBe(1);
    expect(store.filteredEvents.length).toBe(3);
  });

  it('deve alternar entre Ordem do Livro e Ordem Cronológica', async () => {
    await store.openModal();

    // 1. Ordem do livro (narrative)
    store.setViewMode('narrative');
    const narrativeEvents = store.filteredEvents;
    expect(narrativeEvents[0].id).toBe('mock-scene-1');
    expect(narrativeEvents[1].id).toBe('mock-scene-2');
    expect(narrativeEvents[2].id).toBe('mock-scene-3');

    // 2. Ordem cronológica (chronological) - Cena 2 em 1998 deve vir primeiro!
    store.setViewMode('chronological');
    const chronoEvents = store.filteredEvents;
    expect(chronoEvents[0].id).toBe('mock-scene-2'); // 1998
    expect(chronoEvents[1].id).toBe('mock-scene-1'); // 2026-10-01
    expect(chronoEvents[2].id).toBe('mock-scene-3'); // 2026-10-02
  });

  it('deve identificar flashback corretamente na cena de 1998', async () => {
    await store.openModal();

    const ev2 = store.data?.events.find((e) => e.id === 'mock-scene-2');
    expect(ev2?.is_flashback).toBe(true);
    expect(ev2?.raw_time).toBe('1998-04-12');
  });

  it('deve filtrar eventos por personagem participante', async () => {
    await store.openModal();

    store.setCharacterFilter('Ford Prefect');
    const filtered = store.filteredEvents;

    // Ford aparece nas cenas 1 e 3
    expect(filtered.length).toBe(2);
    expect(filtered.every((e) => e.characters.includes('Ford Prefect'))).toBe(true);

    // Limpar filtro
    store.setCharacterFilter(null);
    expect(store.filteredEvents.length).toBe(3);
  });

  it('deve filtrar eventos por local', async () => {
    await store.openModal();

    store.setLocationFilter('Inglaterra');
    const filtered = store.filteredEvents;

    expect(filtered.length).toBe(1);
    expect(filtered[0].id).toBe('mock-scene-2');
  });

  it('deve filtrar eventos por busca de texto livre', async () => {
    await store.openModal();

    store.setSearchQuery('vogon');
    const filtered = store.filteredEvents;

    expect(filtered.length).toBe(1);
    expect(filtered[0].scene_title).toContain('Chegada à Vogon');
  });

  it('deve alternar a visibilidade do painel de Pacing/Ritmo', async () => {
    await store.openModal();

    expect(store.showPacing).toBe(false);
    store.togglePacing();
    expect(store.showPacing).toBe(true);

    const pacing = store.chapterPacing;
    expect(pacing.length).toBeGreaterThan(0);
    expect(pacing[0].chapter_title).toBe('Capítulo 1: O Começo');
    expect(pacing[0].event_density).toBeGreaterThan(0);
  });

  it('deve fechar o modal ao navegar para a cena', async () => {
    await store.openModal();
    expect(store.isModalOpen).toBe(true);

    await store.navigateToScene('mock-scene-1');
    expect(store.isModalOpen).toBe(false);
  });
});
