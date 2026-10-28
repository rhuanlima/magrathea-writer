import { describe, it, expect, beforeEach } from 'vitest';
import { backupStore } from './backup.svelte';

describe('Backup Store (M5-05)', () => {
  beforeEach(() => {
    backupStore.backups = [];
    backupStore.destinationDir = '';
    backupStore.maxRotatedBackups = 5;
    backupStore.errorMessage = null;
    backupStore.successMessage = null;
    backupStore.closeModal();
  });

  it('inicia com campos padrão e modal fechado', () => {
    expect(backupStore.isModalOpen).toBe(false);
    expect(backupStore.backups).toEqual([]);
    expect(backupStore.maxRotatedBackups).toBe(5);
  });

  it('abre e fecha modal de configurações de backup', () => {
    backupStore.openModal();
    expect(backupStore.isModalOpen).toBe(true);
    backupStore.closeModal();
    expect(backupStore.isModalOpen).toBe(false);
  });

  it('executa backup cifrado manual com sucesso', async () => {
    const res = await backupStore.createBackup('/Volumes/ExternalDrive/Backups', 5);

    expect(res).toBeDefined();
    expect(res?.backup_file).toContain('.magrathea_backup');
    expect(backupStore.backups.length).toBeGreaterThan(0);
  });

  it('carrega lista de backups', async () => {
    await backupStore.loadBackups('/home/author/Documents');
    expect(backupStore.backups.length).toBeGreaterThan(0);
  });
});
