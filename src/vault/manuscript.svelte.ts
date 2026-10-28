/**
 * Gerenciador reativo da estrutura hierárquica do manuscrito e lixeira interna.
 *
 * Utiliza Svelte 5 Runes ($state, $derived) para manter a árvore de Partes,
 * Capítulos e Cenas perfeitamente sincronizada com o manifesto criptografado.
 */

import {
  safeInvoke,
  type ManifestItem,
  type ManifestItemKind,
  type ManuscriptTreeResponse,
} from '../tauri/bridge';

export interface TreeNode {
  item: ManifestItem;
  children: TreeNode[];
}

class ManuscriptStore {
  items = $state<ManifestItem[]>([]);
  trash = $state<ManifestItem[]>([]);
  activeSceneId = $state<string | null>(null);
  isSidebarOpen = $state<boolean>(true);
  isTrashOpen = $state<boolean>(false);
  isLoading = $state<boolean>(false);
  errorMessage = $state<string | null>(null);

  /**
   * Item atualmente selecionado para edição (cena, capítulo, parte ou ficha).
   */
  activeItem = $derived(
    this.items.find((it) => it.id === this.activeSceneId) || null
  );

  /**
   * Mantém retrocompatibilidade para componentes que consomem activeScene.
   */
  activeScene = $derived(this.activeItem);

  /**
   * Quantidade de itens descartados na lixeira interna.
   */
  trashCount = $derived(this.trash.length);

  /**
   * Monta a árvore hierárquica respeitando a ordenação monotônica dos nós da narrativa (part, chapter, scene).
   * Fichas de worldbuilding (character, location, item, faction, lore) são isoladas na aba própria.
   */
  tree = $derived.by(() => {
    const narrativeItems = this.items.filter((it) =>
      ['part', 'chapter', 'scene'].includes(it.kind)
    );
    const buildNodes = (parentId: string | null): TreeNode[] => {
      return narrativeItems
        .filter((it) => it.parent_id === parentId)
        .sort((a, b) => a.order - b.order)
        .map((item) => ({
          item,
          children: buildNodes(item.id),
        }));
    };
    return buildNodes(null);
  });

  /**
   * Atualiza a árvore e lixeira a partir do manifesto do cofre ativo.
   */
  async refreshTree(): Promise<void> {
    this.isLoading = true;
    this.errorMessage = null;

    try {
      const response = await safeInvoke<ManuscriptTreeResponse>('get_manuscript_tree');
      this.items = response.items;
      this.trash = response.trash;

      // Se não há cena ativa selecionada, seleciona a primeira cena disponível
      if (!this.activeSceneId || !this.items.some((i) => i.id === this.activeSceneId)) {
        const firstScene = this.items.find((i) => i.kind === 'scene');
        this.activeSceneId = firstScene ? firstScene.id : null;
      }
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Cria um novo nó na árvore (parte, capítulo ou cena).
   */
  async createItem(
    kind: ManifestItemKind,
    parentId?: string | null,
    title = ''
  ): Promise<ManifestItem> {
    try {
      const newItem = await safeInvoke<ManifestItem>('create_manuscript_item', {
        kind,
        parent_id: parentId || null,
        title,
      });

      await this.refreshTree();

      // Se foi criada uma cena, torna-a imediatamente a cena ativa
      if (newItem.kind === 'scene') {
        this.activeSceneId = newItem.id;
      }

      return newItem;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Renomeia o título de um nó.
   */
  async renameItem(itemId: string, newTitle: string): Promise<void> {
    try {
      await safeInvoke('rename_manuscript_item', {
        item_id: itemId,
        new_title: newTitle,
      });
      await this.refreshTree();
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Atualiza metadados literários de uma cena (status e tags).
   */
  async updateSceneMetadata(
    sceneId: string,
    status?: string | null,
    tags?: string[]
  ): Promise<void> {
    try {
      await safeInvoke('update_scene_metadata', {
        scene_id: sceneId,
        status: status ?? null,
        tags: tags ?? [],
      });
      await this.refreshTree();
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Move um nó para a lixeira interna cifrada.
   */
  async moveToTrash(itemId: string): Promise<void> {
    try {
      await safeInvoke('move_item_to_trash', {
        item_id: itemId,
      });

      if (this.activeSceneId === itemId) {
        this.activeSceneId = null;
      }

      await this.refreshTree();
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Restaura um nó da lixeira para a árvore ativa.
   */
  async restoreFromTrash(itemId: string): Promise<void> {
    try {
      await safeInvoke('restore_item_from_trash', {
        item_id: itemId,
      });
      await this.refreshTree();
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Esvazia definitivamente a lixeira interna e exclui os blobs correspondentes no disco.
   */
  async emptyTrash(): Promise<void> {
    try {
      await safeInvoke('empty_trash');
      await this.refreshTree();
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Reordena nós irmãos sob um mesmo nó pai.
   */
  async reorderSiblings(parentId: string | null, orderedIds: string[]): Promise<void> {
    try {
      await safeInvoke('reorder_manuscript_items', {
        parent_id: parentId,
        ordered_ids: orderedIds,
      });
      await this.refreshTree();
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Move um nó irmão para cima ou para baixo na ordenação.
   */
  async moveSibling(itemId: string, direction: 'up' | 'down'): Promise<void> {
    const item = this.items.find((i) => i.id === itemId);
    if (!item) return;

    const siblings = this.items
      .filter((i) => i.parent_id === item.parent_id)
      .sort((a, b) => a.order - b.order);

    const currentIndex = siblings.findIndex((i) => i.id === itemId);
    if (currentIndex < 0) return;

    const targetIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1;
    if (targetIndex < 0 || targetIndex >= siblings.length) return;

    // Inverte posições no array de IDs
    const reordered = [...siblings];
    const [moved] = reordered.splice(currentIndex, 1);
    reordered.splice(targetIndex, 0, moved);

    await this.reorderSiblings(
      item.parent_id,
      reordered.map((i) => i.id)
    );
  }

  /**
   * Move um nó para um novo pai hierárquico e define sua ordem (Drag & Drop).
   */
  async moveItem(
    itemId: string,
    targetParentId: string | null,
    targetOrder?: number
  ): Promise<void> {
    try {
      await safeInvoke('move_manuscript_item', {
        item_id: itemId,
        target_parent_id: targetParentId,
        target_order: targetOrder,
      });
      await this.refreshTree();
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Seleciona uma cena como ativa para edição no CodeMirror.
   */
  selectScene(sceneId: string): void {
    this.activeSceneId = sceneId;
  }

  /**
   * Alterna a visibilidade da barra lateral retrátil.
   */
  toggleSidebar(open?: boolean): void {
    this.isSidebarOpen = open !== undefined ? open : !this.isSidebarOpen;
  }

  /**
   * Alterna a visualização da gaveta/modal da lixeira.
   */
  toggleTrash(open?: boolean): void {
    this.isTrashOpen = open !== undefined ? open : !this.isTrashOpen;
  }
}

export const manuscript = new ManuscriptStore();
