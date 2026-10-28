import { describe, it, expect, beforeEach } from 'vitest';
import { worldbuilding } from './worldbuilding.svelte';
import { ENTITY_TYPE_OPTIONS } from './templates';

describe('M4 — Worldbuilding, Fichas Guiadas e Galeria Cifrada', () => {
  beforeEach(() => {
    worldbuilding.setFilter('all');
    worldbuilding.setSearch('');
    worldbuilding.selectEntity(null);
  });

  describe('M4-01 & M4-02: Templates e Fichas Guiadas', () => {
    it('deve declarar os 5 tipos de entidades literárias do PRD', () => {
      const kinds = ENTITY_TYPE_OPTIONS.map((o) => o.kind);
      expect(kinds).toContain('character');
      expect(kinds).toContain('location');
      expect(kinds).toContain('item');
      expect(kinds).toContain('faction');
      expect(kinds).toContain('lore');
    });

    it('deve criar uma nova entidade no cofre e torná-la ativa', async () => {
      const created = await worldbuilding.createEntity('character', 'Iara', ['xamã', 'litoral']);
      expect(created.title).toBe('Iara');
      expect(created.kind).toBe('character');
      expect(created.tags).toContain('xamã');
      expect(worldbuilding.activeEntityId).toBe(created.id);
    });
  });

  describe('M4-03: Enciclopédia Literária (Filtros e Busca)', () => {
    it('deve filtrar entidades por tipo', () => {
      worldbuilding.setFilter('character');
      const charactersOnly = worldbuilding.filteredEntities;
      expect(charactersOnly.length).toBeGreaterThan(0);
      expect(charactersOnly.every((e) => e.kind === 'character')).toBe(true);

      worldbuilding.setFilter('location');
      const locationsOnly = worldbuilding.filteredEntities;
      expect(locationsOnly.length).toBeGreaterThan(0);
      expect(locationsOnly.every((e) => e.kind === 'location')).toBe(true);
    });

    it('deve buscar entidades por título ou tag', () => {
      worldbuilding.setFilter('all');
      worldbuilding.setSearch('Jaci');
      const searchResults = worldbuilding.filteredEntities;
      expect(searchResults.length).toBe(1);
      expect(searchResults[0].title).toBe('Jaci');

      worldbuilding.setSearch('mistério');
      const tagResults = worldbuilding.filteredEntities;
      expect(tagResults.length).toBe(1);
      expect(tagResults[0].title).toBe('Praia das Conchas');
    });
  });

  describe('M4-04: Galeria de Imagens Cifradas', () => {
    it('deve listar imagens anexadas à entidade', async () => {
      await worldbuilding.loadImages('mock-char-1');
      expect(worldbuilding.images.length).toBe(1);
      expect(worldbuilding.images[0].file_name).toBe('avatar_jaci.png');
      expect(worldbuilding.images[0].mime_type).toBe('image/png');
    });

    it('deve excluir uma imagem e atualizar a galeria', async () => {
      await worldbuilding.loadImages('mock-char-1');
      const imgId = worldbuilding.images[0].id;

      await worldbuilding.deleteImage('mock-char-1', imgId);
      expect(worldbuilding.images.find((i) => i.id === imgId)).toBeUndefined();
    });
  });

  describe('M4-05: Rastreio de Aparições no Manuscrito', () => {
    it('deve carregar as cenas onde a entidade foi mencionada no texto', async () => {
      await worldbuilding.loadAppearances('mock-char-1');
      expect(worldbuilding.appearances.length).toBe(1);
      expect(worldbuilding.appearances[0].scene_id).toBe('mock-scene-1');
      expect(worldbuilding.appearances[0].snippet).toContain('[[Jaci]]');
    });
  });
});
