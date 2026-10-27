/**
 * Ponte segura para invocação de comandos Tauri IPC.
 *
 * Fornece mocks automáticos em ambiente de teste ou navegador para que os testes de UI
 * rodem isolados e com determinismo sem necessitar de um backend Tauri em execução.
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export interface CreateVaultPayload {
  dir_path: string;
  project_title: string;
  passphrase: string;
}

export interface CreateVaultResponse {
  vault_id: string;
  project_title: string;
  recovery_key_token: string;
  vault_path: string;
}

export interface OpenVaultPayload {
  vault_path: string;
  passphrase: string;
}

export interface OpenRecoveryPayload {
  vault_path: string;
  recovery_key: string;
}

export interface SaveScenePayload {
  scene_id: string;
  content: string;
}

export interface SaveSceneResponse {
  scene_id: string;
  saved_at: string;
}

export interface LoadScenePayload {
  scene_id: string;
}

export interface VaultStatusResponse {
  is_open: boolean;
  is_locked: boolean;
  vault_id: string | null;
  project_title: string | null;
  vault_path: string | null;
}

export type ManifestItemKind = 'part' | 'chapter' | 'scene';

export interface ManifestItem {
  id: string;
  kind: ManifestItemKind;
  parent_id: string | null;
  title: string;
  slug: string;
  blob_id: string | null;
  order: number;
  tags: string[];
  status: string | null;
}

export interface ManuscriptTreeResponse {
  items: ManifestItem[];
  trash: ManifestItem[];
}

export interface CreateItemPayload {
  kind: ManifestItemKind;
  parent_id?: string | null;
  title: string;
}

export interface RenameItemPayload {
  item_id: string;
  new_title: string;
}

export interface UpdateSceneMetadataPayload {
  scene_id: string;
  status?: string | null;
  tags?: string[];
}

export interface ReorderItemsPayload {
  parent_id?: string | null;
  ordered_ids: string[];
}

export interface ChangePassphrasePayload {
  current_passphrase: string;
  new_passphrase: string;
}

export interface ReencryptVaultPayload {
  passphrase: string;
}

export interface KeyringPassphrasePayload {
  passphrase: string;
}

export interface OpenKeyringPayload {
  vault_path: string;
}

export interface EntityEntry {
  id: string;
  title: string;
  slug: string;
  kind: string;
  tags: string[];
  status: string | null;
  summary: string | null;
  mention_count: number;
}

export interface BacklinkEntry {
  source_id: string;
  source_title: string;
  source_kind: string;
  target_query: string;
  alias: string | null;
  line_number: number;
  byte_offset: number;
  snippet: string;
}

export interface IndexOverviewResponse {
  entities: EntityEntry[];
  total_links_count: number;
  unresolved_links_count: number;
  unresolved_targets: string[];
}

// Armazenamento em memória para testes e ambiente de preview no browser
let mockItems: ManifestItem[] = [
  {
    id: 'mock-part-1',
    kind: 'part',
    parent_id: null,
    title: 'Parte I: A Partida',
    slug: 'parte-i-a-partida',
    blob_id: null,
    order: 0,
    tags: [],
    status: null,
  },
  {
    id: 'mock-chap-1',
    kind: 'chapter',
    parent_id: 'mock-part-1',
    title: 'Capítulo 1: O Despertar',
    slug: 'capitulo-1-o-despertar',
    blob_id: null,
    order: 0,
    tags: [],
    status: null,
  },
  {
    id: 'mock-scene-1',
    kind: 'scene',
    parent_id: 'mock-chap-1',
    title: 'Cena 1: A Névoa da Manhã',
    slug: 'cena-1-a-nevoa-da-manha',
    blob_id: 'mock-scene-1',
    order: 0,
    tags: ['protagonista', 'abertura'],
    status: 'draft',
  },
];

let mockTrash: ManifestItem[] = [];
let mockKeyringStatus = false;
const mockSceneBlobs: Record<string, string> = {
  'mock-scene-1': '# Cena 1: A Névoa da Manhã\n\nO vento soprava frio contra a vidraça.',
};

/**
 * Invoca um comando Tauri ou retorna mock em testes / ambiente sem IPC nativo.
 */
export async function safeInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  const isTauriAvailable =
    typeof window !== 'undefined' &&
    ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);

  if (!isTauriAvailable) {
    if (cmd === 'get_default_vault_dir') {
      return '/home/author/Documents' as T;
    }
    if (cmd === 'get_vault_status') {
      return {
        is_open: false,
        is_locked: true,
        vault_id: null,
        project_title: null,
        vault_path: null,
      } as T;
    }
    if (cmd === 'create_vault') {
      const payload = args as unknown as CreateVaultPayload;
      return {
        vault_id: 'test-vault-uuid-0001',
        project_title: payload?.project_title || 'Romance Teste',
        recovery_key_token:
          '00000000-00000000-00000000-00000000-00000000-00000000-00000000-00000000',
        vault_path: `${payload?.dir_path || '/home/author/Documents'}/${payload?.project_title || 'Romance'}.magrathea`,
      } as T;
    }
    if (cmd === 'open_vault_with_passphrase') {
      const payload = args as unknown as OpenVaultPayload;
      if (payload?.passphrase === 'wrong-passphrase') {
        throw new Error('Senha incorreta ou cofre corrompido.');
      }
      return {
        is_open: true,
        is_locked: false,
        vault_id: 'test-vault-uuid-0001',
        project_title: 'Romance Teste',
        vault_path: payload?.vault_path || '/home/author/Documents/Romance.magrathea',
      } as T;
    }
    if (cmd === 'open_vault_with_recovery_key') {
      const payload = args as unknown as OpenRecoveryPayload;
      if (payload?.recovery_key === 'invalid-key') {
        throw new Error('Chave de recuperação incorreta ou cofre corrompido.');
      }
      return {
        is_open: true,
        is_locked: false,
        vault_id: 'test-vault-uuid-0001',
        project_title: 'Romance Teste',
        vault_path: payload?.vault_path || '/home/author/Documents/Romance.magrathea',
      } as T;
    }
    if (cmd === 'lock_vault' || cmd === 'close_vault') {
      return undefined as T;
    }
    if (cmd === 'save_scene_content') {
      const payload = args as unknown as SaveScenePayload;
      if (payload?.scene_id) {
        mockSceneBlobs[payload.scene_id] = payload.content;
      }
      return {
        scene_id: payload?.scene_id || 'default_scene',
        saved_at: new Date().toISOString(),
      } as T;
    }
    if (cmd === 'load_scene_content') {
      const payload = args as unknown as LoadScenePayload;
      if (payload?.scene_id === 'cena_01') {
        return '# Mock Scene Content' as T;
      }
      const content = (payload?.scene_id && mockSceneBlobs[payload.scene_id]) || '# Mock Scene Content';
      return content as T;
    }
    if (cmd === 'get_manuscript_tree') {
      return {
        items: [...mockItems],
        trash: [...mockTrash],
      } as T;
    }
    if (cmd === 'create_manuscript_item') {
      const payload = args as unknown as CreateItemPayload;
      const id = `mock-item-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
      const title = payload.title.trim() || (payload.kind === 'part' ? 'Nova Parte' : payload.kind === 'chapter' ? 'Novo Capítulo' : 'Nova Cena');
      const newItem: ManifestItem = {
        id,
        kind: payload.kind,
        parent_id: payload.parent_id || null,
        title,
        slug: title.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, ''),
        blob_id: payload.kind === 'scene' ? id : null,
        order: mockItems.filter(i => i.parent_id === (payload.parent_id || null)).length,
        tags: [],
        status: payload.kind === 'scene' ? 'draft' : null,
      };
      if (newItem.blob_id) {
        mockSceneBlobs[newItem.blob_id] = `# ${title}\n\n`;
      }
      mockItems.push(newItem);
      return newItem as T;
    }
    if (cmd === 'rename_manuscript_item') {
      const payload = args as unknown as RenameItemPayload;
      const item = mockItems.find(i => i.id === payload.item_id);
      if (item) {
        item.title = payload.new_title.trim();
      }
      return undefined as T;
    }
    if (cmd === 'update_scene_metadata') {
      const payload = args as unknown as UpdateSceneMetadataPayload;
      const item = mockItems.find(i => i.id === payload.scene_id);
      if (item) {
        if (payload.status !== undefined) {
          item.status = payload.status;
        }
        if (payload.tags !== undefined) {
          item.tags = payload.tags;
        }
      }
      return undefined as T;
    }
    if (cmd === 'move_item_to_trash') {
      const payload = args as unknown as { item_id: string };
      const toMoveIds = new Set<string>();
      function collect(id: string) {
        toMoveIds.add(id);
        for (const child of mockItems.filter(i => i.parent_id === id)) {
          collect(child.id);
        }
      }
      collect(payload.item_id);
      const moving = mockItems.filter(i => toMoveIds.has(i.id));
      mockItems = mockItems.filter(i => !toMoveIds.has(i.id));
      mockTrash.push(...moving);
      return undefined as T;
    }
    if (cmd === 'restore_item_from_trash') {
      const payload = args as unknown as { item_id: string };
      const pos = mockTrash.findIndex(i => i.id === payload.item_id);
      if (pos >= 0) {
        const item = mockTrash.splice(pos, 1)[0];
        if (item.parent_id && !mockItems.some(i => i.id === item.parent_id)) {
          item.parent_id = null;
        }
        mockItems.push(item);
      }
      return undefined as T;
    }
    if (cmd === 'empty_trash') {
      for (const item of mockTrash) {
        if (item.blob_id) {
          delete mockSceneBlobs[item.blob_id];
        }
      }
      mockTrash = [];
      return undefined as T;
    }
    if (cmd === 'reorder_manuscript_items') {
      const payload = args as unknown as ReorderItemsPayload;
      payload.ordered_ids.forEach((id, idx) => {
        const item = mockItems.find(i => i.id === id);
        if (item && item.parent_id === (payload.parent_id || null)) {
          item.order = idx;
        }
      });
      return undefined as T;
    }
    if (cmd === 'change_vault_passphrase') {
      const payload = args as unknown as ChangePassphrasePayload;
      if (payload?.new_passphrase?.length < 10) {
        throw new Error('A nova senha mestra deve conter no mínimo 10 caracteres.');
      }
      return undefined as T;
    }
    if (cmd === 'reencrypt_vault') {
      return '11111111-22222222-33333333-44444444-55555555-66666666-77777777-88888888' as T;
    }
    if (cmd === 'get_vault_keyring_status') {
      return mockKeyringStatus as T;
    }
    if (cmd === 'save_to_keyring') {
      mockKeyringStatus = true;
      return undefined as T;
    }
    if (cmd === 'remove_from_keyring') {
      mockKeyringStatus = false;
      return undefined as T;
    }
    if (cmd === 'open_vault_with_keyring') {
      return {
        is_open: true,
        is_locked: false,
        vault_id: 'test-vault-uuid-0001',
        project_title: 'Romance Teste',
        vault_path: '/home/author/Documents/Romance.magrathea',
      } as T;
    }
    if (cmd === 'get_vault_index' || cmd === 'rebuild_vault_index') {
      const entities: EntityEntry[] = mockItems.map(item => ({
        id: item.id,
        title: item.title,
        slug: item.slug,
        kind: item.kind,
        tags: item.tags,
        status: item.status,
        summary: item.kind === 'scene' ? 'Cena inicial da alvorada.' : null,
        mention_count: item.title.includes('Névoa') ? 1 : 0,
      }));
      return {
        entities,
        total_links_count: 1,
        unresolved_links_count: 0,
        unresolved_targets: [],
      } as T;
    }
    if (cmd === 'get_document_backlinks') {
      const payload = args as unknown as { item_id: string };
      if (payload.item_id === 'mock-scene-1') {
        return [
          {
            source_id: 'mock-scene-2',
            source_title: 'Cena 2: O Chamado',
            source_kind: 'scene',
            target_query: 'A Névoa da Manhã',
            alias: null,
            line_number: 4,
            byte_offset: 120,
            snippet: 'Lembrou-se do que viu em [[Cena 1: A Névoa da Manhã]] e partiu depressa.',
          },
        ] as T;
      }
      return [] as T;
    }
    if (cmd === 'get_entity_preview') {
      const payload = args as unknown as { query: string };
      const q = payload.query.toLowerCase().trim();
      const match = mockItems.find(i => i.title.toLowerCase().includes(q) || i.slug.toLowerCase().includes(q));
      if (match) {
        return {
          id: match.id,
          title: match.title,
          slug: match.slug,
          kind: match.kind,
          tags: match.tags,
          status: match.status,
          summary: 'Resumo da entidade detectada no cofre.',
          mention_count: 1,
        } as T;
      }
      return null as T;
    }
  }

  return await tauriInvoke<T>(cmd, args);
}
