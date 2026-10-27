/**
 * Gerenciador de estado reativo global do cofre (Svelte 5 Runes).
 */

import { safeInvoke, type CreateVaultResponse, type VaultStatusResponse } from '../tauri/bridge';

class VaultStore {
  isOpen = $state(false);
  isLocked = $state(true);
  vaultId = $state<string | null>(null);
  projectTitle = $state<string | null>(null);
  vaultPath = $state<string | null>(null);
  recoveryKeyToken = $state<string | null>(null);
  isCreating = $state(false);
  isUnlocking = $state(false);
  errorMessage = $state<string | null>(null);

  /**
   * Sincroniza o estado em memória com o backend Tauri.
   */
  async refreshStatus() {
    try {
      const status = await safeInvoke<VaultStatusResponse>('get_vault_status');
      this.isOpen = status.is_open;
      this.isLocked = status.is_locked;
      this.vaultId = status.vault_id;
      this.projectTitle = status.project_title;
      this.vaultPath = status.vault_path;
    } catch (err) {
      this.errorMessage = String(err);
    }
  }

  /**
   * Invoca o comando de criação atômica de cofre no backend.
   */
  async createVault(
    dirPath: string,
    title: string,
    passphrase: string
  ): Promise<CreateVaultResponse> {
    this.isCreating = true;
    this.errorMessage = null;

    try {
      const response = await safeInvoke<CreateVaultResponse>('create_vault', {
        dir_path: dirPath,
        project_title: title,
        passphrase,
      });

      this.isOpen = true;
      this.isLocked = false;
      this.vaultId = response.vault_id;
      this.projectTitle = response.project_title;
      this.vaultPath = response.vault_path;
      this.recoveryKeyToken = response.recovery_key_token;

      return response;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.isCreating = false;
    }
  }

  /**
   * Desbloqueia o cofre atual utilizando a frase-senha mestra.
   */
  async openWithPassphrase(passphrase: string, targetPath?: string): Promise<VaultStatusResponse> {
    const path = targetPath || this.vaultPath;
    if (!path) {
      throw new Error('Nenhum caminho de cofre definido.');
    }

    this.isUnlocking = true;
    this.errorMessage = null;

    try {
      const response = await safeInvoke<VaultStatusResponse>('open_vault_with_passphrase', {
        vault_path: path,
        passphrase,
      });

      this.isOpen = response.is_open;
      this.isLocked = response.is_locked;
      this.vaultId = response.vault_id;
      this.projectTitle = response.project_title;
      this.vaultPath = response.vault_path;

      return response;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.isUnlocking = false;
    }
  }

  /**
   * Desbloqueia o cofre atual utilizando a chave de recuperação de emergência.
   */
  async openWithRecoveryKey(recoveryKey: string, targetPath?: string): Promise<VaultStatusResponse> {
    const path = targetPath || this.vaultPath;
    if (!path) {
      throw new Error('Nenhum caminho de cofre definido.');
    }

    this.isUnlocking = true;
    this.errorMessage = null;

    try {
      const response = await safeInvoke<VaultStatusResponse>('open_vault_with_recovery_key', {
        vault_path: path,
        recovery_key: recoveryKey,
      });

      this.isOpen = response.is_open;
      this.isLocked = response.is_locked;
      this.vaultId = response.vault_id;
      this.projectTitle = response.project_title;
      this.vaultPath = response.vault_path;

      return response;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.isUnlocking = false;
    }
  }

  /**
   * Bloqueia o cofre atual em memória.
   */
  async lockVault() {
    try {
      await safeInvoke('lock_vault');
      this.isLocked = true;
      this.recoveryKeyToken = null;
    } catch (err) {
      this.errorMessage = String(err);
    }
  }

  /**
   * Fecha e desvincula o cofre ativo, limpando a sessão recente.
   */
  async closeVault() {
    try {
      await safeInvoke('close_vault');
      this.isOpen = false;
      this.isLocked = true;
      this.vaultId = null;
      this.projectTitle = null;
      this.vaultPath = null;
      this.recoveryKeyToken = null;
      this.errorMessage = null;
    } catch (err) {
      this.errorMessage = String(err);
    }
  }
}

export const vault = new VaultStore();
