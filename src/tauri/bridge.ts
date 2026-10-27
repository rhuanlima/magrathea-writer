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
      return {
        scene_id: payload?.scene_id || 'default_scene',
        saved_at: new Date().toISOString(),
      } as T;
    }
    if (cmd === 'load_scene_content') {
      return '# Mock Scene Content' as T;
    }
  }

  return await tauriInvoke<T>(cmd, args);
}
