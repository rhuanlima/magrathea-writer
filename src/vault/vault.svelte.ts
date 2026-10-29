/**
 * Gerenciador de estado reativo global do cofre (Svelte 5 Runes).
 */

import {
  safeInvoke,
  type CreateVaultResponse,
  type VaultStatusResponse,
  type SaveSceneResponse,
  type IndexOverviewResponse,
  type BacklinkEntry,
  type EntityEntry,
  type RenameRefactorPreviewResponse,
  type ApplyRenameRefactorResponse,
} from '../tauri/bridge';
import { addRecentVault } from './recent-vaults';

class VaultStore {
  isOpen = $state(false);
  isLocked = $state(true);
  vaultId = $state<string | null>(null);
  projectTitle = $state<string | null>(null);
  vaultPath = $state<string | null>(null);
  recoveryKeyToken = $state<string | null>(null);
  isCreating = $state(false);
  isUnlocking = $state(false);
  isSaving = $state(false);
  lastSavedAt = $state<string | null>(null);
  errorMessage = $state<string | null>(null);
  hasKeyringEnabled = $state(false);
  isReencrypting = $state(false);
  isChangingPassphrase = $state(false);
  newRecoveryKeyAfterReencrypt = $state<string | null>(null);
  isSecurityModalOpen = $state(false);

  // Segundo Cérebro (M2-01 .. M2-06)
  indexOverview = $state<IndexOverviewResponse | null>(null);
  activeBacklinks = $state<BacklinkEntry[]>([]);
  isInspectorOpen = $state(false);
  activeInspectorTab = $state<'metadata' | 'backlinks' | 'appearances' | 'gallery' | 'snapshots' | 'context' | 'graph' | 'style'>('metadata');
  isLoadingIndex = $state(false);
  isLoadingBacklinks = $state(false);

  // Refatoração global e rollback (M2-05)
  isRefactorModalOpen = $state(false);
  refactorTarget = $state<string | null>(null);
  lastSnapshotId = $state<string | null>(null);
  isRefactoring = $state(false);
  isRollingBack = $state(false);

  /**
   * Lista de todas as entidades e cenas registradas no cofre.
   */
  get entities(): EntityEntry[] {
    return this.indexOverview?.entities || [];
  }

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

      if (this.isOpen && !this.isLocked) {
        await this.checkKeyringStatus();
      }
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
        dirPath,
        dir_path: dirPath,
        projectTitle: title,
        project_title: title,
        passphrase,
      });

      this.isOpen = true;
      this.isLocked = false;
      this.vaultId = response.vault_id;
      this.projectTitle = response.project_title;
      this.vaultPath = response.vault_path;
      this.recoveryKeyToken = response.recovery_key_token;

      if (response.vault_path) {
        addRecentVault({ path: response.vault_path, title: response.project_title || '' });
      }

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

      if (response.is_open && !response.is_locked && response.vault_path) {
        addRecentVault({ path: response.vault_path, title: response.project_title || '' });
      }

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

      if (response.is_open && !response.is_locked && response.vault_path) {
        addRecentVault({ path: response.vault_path, title: response.project_title || '' });
      }

      return response;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.isUnlocking = false;
    }
  }

  /**
   * Salva o conteúdo de uma cena de forma atômica e cifrada no cofre.
   */
  async saveScene(sceneId: string, content: string): Promise<SaveSceneResponse> {
    if (!this.isOpen || this.isLocked) {
      throw new Error('Cofre não está aberto ou está bloqueado.');
    }

    this.isSaving = true;
    this.errorMessage = null;

    try {
      const response = await safeInvoke<SaveSceneResponse>('save_scene_content', {
        scene_id: sceneId,
        content,
      });

      this.lastSavedAt = response.saved_at;
      return response;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  /**
   * Carrega e decifra o conteúdo de uma cena armazenada no cofre.
   */
  async loadScene(sceneId: string): Promise<string> {
    if (!this.isOpen || this.isLocked) {
      throw new Error('Cofre não está aberto ou está bloqueado.');
    }

    try {
      return await safeInvoke<string>('load_scene_content', {
        scene_id: sceneId,
      });
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Altera a frase-senha mestra do cofre ativo.
   */
  async changePassphrase(currentPass: string, newPass: string): Promise<void> {
    this.isChangingPassphrase = true;
    this.errorMessage = null;

    try {
      await safeInvoke('change_vault_passphrase', {
        current_passphrase: currentPass,
        new_passphrase: newPass,
      });
      await this.checkKeyringStatus();
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.isChangingPassphrase = false;
    }
  }

  /**
   * Recifra o cofre por completo, rotacionando a DEK e gerando uma nova chave de recuperação.
   */
  async reencryptVault(passphrase: string): Promise<string> {
    this.isReencrypting = true;
    this.errorMessage = null;

    try {
      const newRecoveryToken = await safeInvoke<string>('reencrypt_vault', {
        passphrase,
      });
      this.newRecoveryKeyAfterReencrypt = newRecoveryToken;
      return newRecoveryToken;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.isReencrypting = false;
    }
  }

  /**
   * Verifica se o cofre possui credencial salva no chaveiro do SO.
   */
  async checkKeyringStatus(): Promise<boolean> {
    try {
      const status = await safeInvoke<boolean>('get_vault_keyring_status');
      this.hasKeyringEnabled = status;
      return status;
    } catch {
      this.hasKeyringEnabled = false;
      return false;
    }
  }

  /**
   * Habilita e salva a senha do cofre no chaveiro do SO.
   */
  async saveToKeyring(passphrase: string): Promise<void> {
    try {
      await safeInvoke('save_to_keyring', { passphrase });
      this.hasKeyringEnabled = true;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Remove a credencial do cofre do chaveiro do SO.
   */
  async removeFromKeyring(): Promise<void> {
    try {
      await safeInvoke('remove_from_keyring');
      this.hasKeyringEnabled = false;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  /**
   * Desbloqueia o cofre automaticamente via chaveiro do SO.
   */
  async openWithKeyring(targetPath?: string): Promise<VaultStatusResponse> {
    const path = targetPath || this.vaultPath;
    if (!path) {
      throw new Error('Nenhum caminho de cofre definido.');
    }

    this.isUnlocking = true;
    this.errorMessage = null;

    try {
      const response = await safeInvoke<VaultStatusResponse>('open_vault_with_keyring', {
        vault_path: path,
      });

      this.isOpen = response.is_open;
      this.isLocked = response.is_locked;
      this.vaultId = response.vault_id;
      this.projectTitle = response.project_title;
      this.vaultPath = response.vault_path;
      this.hasKeyringEnabled = true;

      if (response.is_open && !response.is_locked && response.vault_path) {
        addRecentVault({ path: response.vault_path, title: response.project_title || '' });
      }

      return response;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.isUnlocking = false;
    }
  }

  /**
   * Alterna a exibição do modal de segurança do cofre.
   */
  toggleSecurityModal(open?: boolean) {
    this.isSecurityModalOpen = open !== undefined ? open : !this.isSecurityModalOpen;
  }

  /**
   * Carrega a visão geral do índice de entidades e conexões em memória.
   */
  async loadVaultIndex(): Promise<IndexOverviewResponse | null> {
    if (!this.isOpen || this.isLocked) return null;
    this.isLoadingIndex = true;
    try {
      const overview = await safeInvoke<IndexOverviewResponse>('get_vault_index');
      this.indexOverview = overview;
      return overview;
    } catch (err) {
      this.errorMessage = String(err);
      return null;
    } finally {
      this.isLoadingIndex = false;
    }
  }

  /**
   * Carrega os backlinks (menções em outros documentos) que apontam para o item informado.
   */
  async loadBacklinks(itemId: string): Promise<BacklinkEntry[]> {
    if (!this.isOpen || this.isLocked || !itemId) {
      this.activeBacklinks = [];
      return [];
    }
    this.isLoadingBacklinks = true;
    try {
      const backlinks = await safeInvoke<BacklinkEntry[]>('get_document_backlinks', {
        item_id: itemId,
      });
      this.activeBacklinks = backlinks;
      return backlinks;
    } catch (err) {
      this.errorMessage = String(err);
      this.activeBacklinks = [];
      return [];
    } finally {
      this.isLoadingBacklinks = false;
    }
  }

  /**
   * Alterna ou define a visibilidade do painel inspetor à direita.
   */
  toggleInspector(open?: boolean) {
    this.isInspectorOpen = open !== undefined ? open : !this.isInspectorOpen;
  }

  /**
   * Define a aba ativa no painel Inspetor ('metadata', 'backlinks', 'appearances', 'gallery', 'snapshots' ou 'context').
   */
  setInspectorTab(tab: 'metadata' | 'backlinks' | 'appearances' | 'gallery' | 'snapshots' | 'context' | 'graph' | 'style') {
    this.activeInspectorTab = tab;
  }

  /**
   * Abre o modal de refatoração global para uma entidade ou cena.
   */
  openRefactorModal(targetName: string) {
    this.refactorTarget = targetName;
    this.isRefactorModalOpen = true;
  }

  /**
   * Fecha o modal de refatoração global.
   */
  closeRefactorModal() {
    this.isRefactorModalOpen = false;
    this.refactorTarget = null;
  }

  /**
   * Gera a pré-visualização de renomeação em massa.
   */
  async previewRename(oldName: string, newName: string): Promise<RenameRefactorPreviewResponse> {
    return await safeInvoke<RenameRefactorPreviewResponse>('preview_rename_entity', {
      old_name: oldName,
      new_name: newName,
    });
  }

  /**
   * Aplica a refatoração atômica em todo o projeto com gravação de snapshot.
   */
  async applyRename(oldName: string, newName: string): Promise<ApplyRenameRefactorResponse> {
    this.isRefactoring = true;
    try {
      const response = await safeInvoke<ApplyRenameRefactorResponse>('apply_rename_refactor', {
        old_name: oldName,
        new_name: newName,
      });
      this.lastSnapshotId = response.snapshot_id;
      await this.loadVaultIndex();
      return response;
    } finally {
      this.isRefactoring = false;
    }
  }

  /**
   * Executa rollback da refatoração restaurando os conteúdos e metadados originais.
   */
  async rollbackRename(snapshotId?: string): Promise<void> {
    const snapId = snapshotId || this.lastSnapshotId;
    if (!snapId) throw new Error('Nenhum snapshot de refatoração para restaurar.');
    this.isRollingBack = true;
    try {
      await safeInvoke('rollback_rename_refactor', { snapshot_id: snapId });
      this.lastSnapshotId = null;
      await this.loadVaultIndex();
    } finally {
      this.isRollingBack = false;
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
      this.indexOverview = null;
      this.activeBacklinks = [];
      this.isInspectorOpen = false;
      this.isRefactorModalOpen = false;
      this.lastSnapshotId = null;
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
      this.indexOverview = null;
      this.activeBacklinks = [];
      this.isInspectorOpen = false;
      this.isRefactorModalOpen = false;
      this.lastSnapshotId = null;
    } catch (err) {
      this.errorMessage = String(err);
    }
  }
}

export const vault = new VaultStore();
