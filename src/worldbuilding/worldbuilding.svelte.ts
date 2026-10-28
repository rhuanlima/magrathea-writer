/**
 * Store reativo para o subsistema de Worldbuilding e Enciclopédia Literária (M4-01 a M4-05).
 */

import { safeInvoke, type ManifestItem, type EntityImageMeta, type EntityAppearance } from '../tauri/bridge';
import { manuscript } from '../vault/manuscript.svelte';
import { vault } from '../vault/vault.svelte';
import type { EntityKind } from './templates';

class WorldbuildingStore {
  filterKind = $state<'all' | EntityKind>('all');
  searchQuery = $state<string>('');
  isCreateModalOpen = $state<boolean>(false);
  activeEntityId = $state<string | null>(null);

  // Dados da entidade em visualização/inspeção
  appearances = $state<EntityAppearance[]>([]);
  images = $state<EntityImageMeta[]>([]);
  isLoading = $state<boolean>(false);

  /**
   * Lista todas as entidades cadastradas no cofre (exclui nós estruturais part e chapter).
   */
  get entities(): ManifestItem[] {
    return manuscript.items.filter((item) =>
      ['character', 'location', 'item', 'faction', 'lore', 'entity'].includes(item.kind)
    );
  }

  /**
   * Entidades filtradas por tipo e busca textual.
   */
  get filteredEntities(): ManifestItem[] {
    const q = this.searchQuery.trim().toLowerCase();
    return this.entities.filter((entity) => {
      const matchKind = this.filterKind === 'all' || entity.kind === this.filterKind;
      if (!matchKind) return false;

      if (!q) return true;
      const matchTitle = entity.title.toLowerCase().includes(q);
      const matchSlug = entity.slug.toLowerCase().includes(q);
      const matchTags = entity.tags.some((t) => t.toLowerCase().includes(q));
      return matchTitle || matchSlug || matchTags;
    });
  }

  setFilter(kind: 'all' | EntityKind): void {
    this.filterKind = kind;
  }

  setSearch(query: string): void {
    this.searchQuery = query;
  }

  openCreateModal(): void {
    this.isCreateModalOpen = true;
  }

  closeCreateModal(): void {
    this.isCreateModalOpen = false;
  }

  selectEntity(entityId: string | null): void {
    this.activeEntityId = entityId;
    if (entityId) {
      manuscript.selectScene(entityId);
      void this.loadAppearances(entityId);
      void this.loadImages(entityId);
    } else {
      this.appearances = [];
      this.images = [];
    }
  }

  /**
   * Cria uma nova entidade no cofre com template pré-gerado.
   */
  async createEntity(kind: EntityKind, title: string, tags: string[]): Promise<ManifestItem> {
    this.isLoading = true;
    try {
      const item = await safeInvoke<ManifestItem>('create_world_entity', {
        kind,
        title,
        tags,
      });
      await manuscript.refreshTree();
      await vault.loadVaultIndex();
      this.selectEntity(item.id);
      this.closeCreateModal();
      return item;
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Carrega as aparições da entidade em todas as cenas do manuscrito.
   */
  async loadAppearances(entityId: string): Promise<void> {
    try {
      const list = await safeInvoke<EntityAppearance[]>('get_entity_appearances', {
        entity_id: entityId,
      });
      this.appearances = list;
    } catch (err) {
      console.error('Falha ao carregar aparições da entidade:', err);
      this.appearances = [];
    }
  }

  /**
   * Lista as imagens associadas à entidade.
   */
  async loadImages(entityId: string): Promise<void> {
    try {
      const list = await safeInvoke<EntityImageMeta[]>('list_entity_images', {
        entity_id: entityId,
      });
      this.images = list;
    } catch (err) {
      console.error('Falha ao carregar imagens da entidade:', err);
      this.images = [];
    }
  }

  /**
   * Importa e cifra uma imagem anexando-a à entidade literária.
   */
  async importImage(entityId: string, file: File): Promise<EntityImageMeta> {
    const arrayBuffer = await file.arrayBuffer();
    const bytes = Array.from(new Uint8Array(arrayBuffer));
    const nowIso = new Date().toISOString();

    const meta = await safeInvoke<EntityImageMeta>('import_entity_image', {
      entity_id: entityId,
      file_name: file.name,
      image_bytes: bytes,
      created_at_iso: nowIso,
    });

    await this.loadImages(entityId);
    return meta;
  }

  /**
   * Remove uma imagem do cofre.
   */
  async deleteImage(entityId: string, imageId: string): Promise<void> {
    await safeInvoke('delete_entity_image', {
      entity_id: entityId,
      image_id: imageId,
    });
    await this.loadImages(entityId);
  }
}

export const worldbuilding = new WorldbuildingStore();
