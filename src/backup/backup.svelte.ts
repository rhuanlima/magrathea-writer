/**
//! Store reativa para backup cifrado rotativo do cofre literário (M5-05).
//!
//! Controla cópia de segurança com timestamp, rotação automática e aviso de mesmo disco.
*/

import { safeInvoke, type BackupInfo, type BackupResult } from '../tauri/bridge';

class BackupStore {
  isModalOpen = $state(false);
  destinationDir = $state('');
  maxRotatedBackups = $state(5);
  backups = $state<BackupInfo[]>([]);
  lastBackup = $state<BackupResult | null>(null);

  isLoading = $state(false);
  isCreating = $state(false);
  errorMessage = $state<string | null>(null);
  successMessage = $state<string | null>(null);

  openModal(): void {
    this.isModalOpen = true;
    this.errorMessage = null;
    this.successMessage = null;
    if (this.destinationDir.trim()) {
      void this.loadBackups(this.destinationDir);
    }
  }

  closeModal(): void {
    this.isModalOpen = false;
  }

  /**
   * Lista backups existentes no destino configurado.
   */
  async loadBackups(destinationDir: string): Promise<void> {
    if (!destinationDir.trim()) return;
    this.isLoading = true;
    this.errorMessage = null;

    try {
      this.backups = await safeInvoke<BackupInfo[]>('list_vault_backups', {
        destinationDir,
        destination_dir: destinationDir,
      });
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Cria uma cópia integral cifrada do cofre literário.
   */
  async createBackup(
    destinationDir: string,
    maxBackups = 5
  ): Promise<BackupResult | null> {
    if (!destinationDir.trim()) {
      this.errorMessage = 'Defina a pasta de destino para salvar o backup.';
      return null;
    }

    this.isCreating = true;
    this.errorMessage = null;
    this.successMessage = null;

    try {
      const result = await safeInvoke<BackupResult>('create_vault_backup', {
        destinationDir,
        destination_dir: destinationDir,
        maxBackups,
        max_backups: maxBackups,
      });

      this.lastBackup = result;
      this.successMessage = `Backup cifrado criado com sucesso (${(result.byte_size / 1024).toFixed(1)} KB).`;
      await this.loadBackups(destinationDir);
      return result;
    } catch (err) {
      this.errorMessage = String(err);
      return null;
    } finally {
      this.isCreating = false;
    }
  }
}

export const backupStore = new BackupStore();
