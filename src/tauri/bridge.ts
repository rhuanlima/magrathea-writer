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

export type ManifestItemKind =
  | 'part'
  | 'chapter'
  | 'scene'
  | 'character'
  | 'location'
  | 'item'
  | 'faction'
  | 'lore'
  | 'entity';

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

export interface OccurrencePreview {
  line_number: number;
  raw_match: string;
  old_snippet: string;
  new_snippet: string;
}

export interface DocumentRefactorPreview {
  item_id: string;
  title: string;
  kind: string;
  occurrences: OccurrencePreview[];
}

export interface RenameRefactorPreviewResponse {
  old_name: string;
  new_name: string;
  total_occurrences: number;
  affected_documents: DocumentRefactorPreview[];
}

export interface ApplyRenameRefactorResponse {
  snapshot_id: string;
  modified_docs_count: number;
  total_replacements: number;
  applied_at: string;
}

export interface DailyLogEntry {
  words_written: number;
  session_duration_seconds: number;
}

export interface ProductivityOverview {
  project_target_words: number;
  project_deadline: string | null;
  daily_target_words: number;
  total_manuscript_words: number;
  today_words: number;
  today_seconds: number;
  seven_days_avg_words: number;
  thirty_days_avg_words: number;
  daily_logs: Record<string, DailyLogEntry>;
}

export interface EntityImageMeta {
  id: string;
  entity_id: string;
  file_name: string;
  mime_type: string;
  byte_size: number;
  created_at: string;
}

export interface EntityImageData {
  meta: EntityImageMeta;
  base64_data: string;
}

export interface EntityAppearance {
  scene_id: string;
  scene_title: string;
  line_number: number;
  snippet: string;
  alias: string | null;
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
  {
    id: 'mock-char-1',
    kind: 'character',
    parent_id: null,
    title: 'Jaci',
    slug: 'jaci',
    blob_id: 'mock-char-1',
    order: 0,
    tags: ['protagonista', 'navegadora'],
    status: 'draft',
  },
  {
    id: 'mock-loc-1',
    kind: 'location',
    parent_id: null,
    title: 'Praia das Conchas',
    slug: 'praia-das-conchas',
    blob_id: 'mock-loc-1',
    order: 1,
    tags: ['costa', 'mistério'],
    status: 'draft',
  },
];

let mockTrash: ManifestItem[] = [];
let mockKeyringStatus = false;
const mockSceneBlobs: Record<string, string> = {
  'mock-scene-1': '# Cena 1: A Névoa da Manhã\n\nO vento soprava frio contra a vidraça.',
  'mock-char-1': '---\ntitle: "Jaci"\ntype: character\nrole: Protagonista\n---\n\n## Presença & Descrição Física\nNavegadora destemida dos mares do sul.',
  'mock-loc-1': '---\ntitle: "Praia das Conchas"\ntype: location\n---\n\n## Geografia & Sensações\nAreias alvas e vento salgado.',
};

const mockEntityImages: Record<string, EntityImageMeta[]> = {
  'mock-char-1': [
    {
      id: 'mock-img-1',
      entity_id: 'mock-char-1',
      file_name: 'avatar_jaci.png',
      mime_type: 'image/png',
      byte_size: 1024,
      created_at: '2026-10-27T10:00:00Z',
    },
  ],
};

const mockProductivity: ProductivityOverview = {
  project_target_words: 50000,
  project_deadline: '2026-12-31',
  daily_target_words: 1000,
  total_manuscript_words: 850,
  today_words: 450,
  today_seconds: 1200,
  seven_days_avg_words: 520,
  thirty_days_avg_words: 480,
  daily_logs: {
    '2026-10-27': {
      words_written: 450,
      session_duration_seconds: 1200,
    },
  },
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
    if (cmd === 'preview_rename_entity') {
      const payload = args as unknown as { old_name: string; new_name: string };
      return {
        old_name: payload.old_name,
        new_name: payload.new_name,
        total_occurrences: 1,
        affected_documents: [
          {
            item_id: 'mock-scene-1',
            title: 'Cena 1: A Névoa da Manhã',
            kind: 'scene',
            occurrences: [
              {
                line_number: 1,
                raw_match: `[[${payload.old_name}]]`,
                old_snippet: `Encontrou [[${payload.old_name}]] na praia.`,
                new_snippet: `Encontrou [[${payload.new_name}]] na praia.`,
              },
            ],
          },
        ],
      } as T;
    }
    if (cmd === 'apply_rename_refactor') {
      return {
        snapshot_id: 'mock-snapshot-001',
        modified_docs_count: 1,
        total_replacements: 1,
        applied_at: new Date().toISOString(),
      } as T;
    }
    if (cmd === 'rollback_rename_refactor') {
      return undefined as T;
    }
    if (cmd === 'get_productivity_overview') {
      return { ...mockProductivity } as T;
    }
    if (cmd === 'update_productivity_targets') {
      const payload = args as unknown as {
        project_target: number;
        deadline: string | null;
        daily_target: number;
        today_date_iso: string;
      };
      mockProductivity.project_target_words = payload.project_target;
      mockProductivity.project_deadline = payload.deadline;
      mockProductivity.daily_target_words = payload.daily_target;
      return { ...mockProductivity } as T;
    }
    if (cmd === 'record_productivity_session') {
      const payload = args as unknown as {
        date_iso: string;
        words_delta: number;
        session_seconds: number;
      };
      mockProductivity.today_words += payload.words_delta;
      mockProductivity.today_seconds += payload.session_seconds;
      mockProductivity.total_manuscript_words += payload.words_delta;
      const currentDay = mockProductivity.daily_logs[payload.date_iso] || {
        words_written: 0,
        session_duration_seconds: 0,
      };
      currentDay.words_written += payload.words_delta;
      currentDay.session_duration_seconds += payload.session_seconds;
      mockProductivity.daily_logs[payload.date_iso] = currentDay;
      return { ...mockProductivity } as T;
    }
    if (cmd === 'get_entity_template') {
      const payload = args as unknown as { kind: string; title: string };
      return `---\ntitle: "${payload.title}"\ntype: ${payload.kind}\n---\n\n## Descrição\nTemplate literário.` as T;
    }
    if (cmd === 'create_world_entity') {
      const payload = args as unknown as { kind: string; title: string; tags: string[] };
      const newEntity: ManifestItem = {
        id: `mock-ent-${Date.now()}`,
        kind: payload.kind as ManifestItemKind,
        parent_id: null,
        title: payload.title,
        slug: payload.title.toLowerCase().replace(/\s+/g, '-'),
        blob_id: `mock-blob-${Date.now()}`,
        order: mockItems.length,
        tags: payload.tags || [],
        status: 'draft',
      };
      mockItems.push(newEntity);
      mockSceneBlobs[newEntity.id] = `---\ntitle: "${payload.title}"\ntype: ${payload.kind}\n---\n\n## Detalhes\nConteúdo inicial da entidade.`;
      return newEntity as T;
    }
    if (cmd === 'import_entity_image') {
      const payload = args as unknown as {
        entity_id: string;
        file_name: string;
        image_bytes: number[];
        created_at_iso: string;
      };
      const newMeta: EntityImageMeta = {
        id: `mock-img-${Date.now()}`,
        entity_id: payload.entity_id,
        file_name: payload.file_name,
        mime_type: 'image/png',
        byte_size: payload.image_bytes.length,
        created_at: payload.created_at_iso,
      };
      const list = mockEntityImages[payload.entity_id] || [];
      list.push(newMeta);
      mockEntityImages[payload.entity_id] = list;
      return newMeta as T;
    }
    if (cmd === 'load_entity_image') {
      const payload = args as unknown as { image_id: string };
      let found: EntityImageMeta | undefined;
      for (const list of Object.values(mockEntityImages)) {
        found = list.find((m) => m.id === payload.image_id);
        if (found) break;
      }
      return {
        meta: found || {
          id: payload.image_id,
          entity_id: 'mock-char-1',
          file_name: 'imagem.png',
          mime_type: 'image/png',
          byte_size: 1024,
          created_at: new Date().toISOString(),
        },
        base64_data: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
      } as T;
    }
    if (cmd === 'delete_entity_image') {
      const payload = args as unknown as { entity_id: string; image_id: string };
      const list = mockEntityImages[payload.entity_id];
      if (list) {
        mockEntityImages[payload.entity_id] = list.filter((m) => m.id !== payload.image_id);
      }
      return undefined as T;
    }
    if (cmd === 'list_entity_images') {
      const payload = args as unknown as { entity_id: string };
      return (mockEntityImages[payload.entity_id] || []) as T;
    }
    if (cmd === 'get_entity_appearances') {
      const payload = args as unknown as { entity_id: string };
      if (payload.entity_id === 'mock-char-1') {
        return [
          {
            scene_id: 'mock-scene-1',
            scene_title: 'Cena 1: A Névoa da Manhã',
            line_number: 1,
            snippet: 'O vento soprava frio quando [[Jaci]] olhou o mar.',
            alias: null,
          },
        ] as T;
      }
      return [] as T;
    }
  }

  return await tauriInvoke<T>(cmd, args);
}
