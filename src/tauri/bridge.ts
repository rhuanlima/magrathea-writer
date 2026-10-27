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
          '11223344-55667788-99aabbcc-ddeeff00-11223344-55667788-99aabbcc-ddeeff00',
        vault_path: `${payload?.dir_path || '/home/author/Documents'}/${payload?.project_title || 'Romance'}.magrathea`,
      } as T;
    }
    if (cmd === 'lock_vault') {
      return undefined as T;
    }
  }

  return await tauriInvoke<T>(cmd, args);
}
