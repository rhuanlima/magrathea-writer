<script lang="ts">
  import { t } from '../i18n';
  import { backupStore } from './backup.svelte';

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && backupStore.isModalOpen) {
      backupStore.closeModal();
    }
  }

  async function handleExecuteBackup() {
    await backupStore.createBackup(
      backupStore.destinationDir,
      backupStore.maxRotatedBackups
    );
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatIso(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(undefined, {
        day: '2-digit',
        month: '2-digit',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return iso;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if backupStore.isModalOpen}
  <div class="modal-backdrop">
    <div
      class="modal-card"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="backup-modal-title"
    >
      <header class="modal-header">
        <h2 id="backup-modal-title" class="modal-title">
          🛡️ {t('backup.modalTitle')}
        </h2>
        <button
          type="button"
          class="btn-close"
          onclick={() => backupStore.closeModal()}
          aria-label="Fechar"
        >
          ✕
        </button>
      </header>

      <div class="modal-body">
        <div class="form-group">
          <label for="backup-dest-input" class="field-label">
            {t('backup.destinationLabel')}
          </label>
          <div class="input-with-button">
            <input
              id="backup-dest-input"
              type="text"
              class="dest-input"
              placeholder={t('backup.destinationPlaceholder')}
              bind:value={backupStore.destinationDir}
              onchange={() => void backupStore.loadBackups(backupStore.destinationDir)}
            />
          </div>
        </div>

        <div class="form-group">
          <label for="backup-rotation-select" class="field-label">
            {t('backup.rotationLabel')}
          </label>
          <select
            id="backup-rotation-select"
            class="rotation-select"
            bind:value={backupStore.maxRotatedBackups}
          >
            <option value={3}>3 backups recentes</option>
            <option value={5}>5 backups recentes (recomendado)</option>
            <option value={10}>10 backups recentes</option>
          </select>
        </div>

        <!-- Alerta de mesmo disco (redundância) -->
        {#if backupStore.lastBackup?.is_same_disk || (backupStore.backups.length > 0 && backupStore.backups[0].is_same_disk)}
          <div class="disk-warning-card">
            <span class="warning-icon">⚠️</span>
            <div class="warning-text">
              <strong>{t('backup.sameDiskWarning')}</strong>
              <p>{t('backup.sameDiskTip')}</p>
            </div>
          </div>
        {/if}

        {#if backupStore.successMessage}
          <div class="banner-success">{backupStore.successMessage}</div>
        {/if}

        {#if backupStore.errorMessage}
          <div class="banner-error">{backupStore.errorMessage}</div>
        {/if}

        <button
          type="button"
          class="btn-run-backup"
          disabled={backupStore.isCreating || !backupStore.destinationDir.trim()}
          onclick={handleExecuteBackup}
        >
          {backupStore.isCreating ? 'Copiando e protegendo...' : t('backup.runBackupButton')}
        </button>

        <!-- Lista de Backups Recentes -->
        <div class="recent-backups-section">
          <h3 class="section-title">{t('backup.recentBackupsTitle')}</h3>

          {#if backupStore.isLoading}
            <div class="empty-state">Verificando backups no destino...</div>
          {:else if backupStore.backups.length === 0}
            <div class="empty-state">{t('backup.noBackupsYet')}</div>
          {:else}
            <div class="backups-list">
              {#each backupStore.backups as backup}
                <div class="backup-item">
                  <div class="backup-info">
                    <span class="backup-file">{backup.file_name}</span>
                    <span class="backup-date">{formatIso(backup.timestamp)}</span>
                  </div>
                  <span class="backup-size">{formatBytes(backup.byte_size)}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-card {
    width: 580px;
    max-width: 95vw;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-md);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .modal-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin: 0;
  }

  .btn-close {
    background: transparent;
    border: none;
    font-size: 14px;
    color: var(--color-ink-muted);
    cursor: pointer;
  }

  .modal-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-ink-muted);
  }

  .dest-input,
  .rotation-select {
    padding: 8px 10px;
    font-size: 12px;
    background-color: var(--color-paper-surface);
    color: var(--color-ink-charcoal);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
  }

  .disk-warning-card {
    display: flex;
    gap: 10px;
    padding: 10px;
    background-color: rgba(184, 115, 51, 0.12);
    border: 1px solid var(--color-bronze);
    border-radius: var(--radius-sm);
    color: var(--color-ink-charcoal);
  }

  .warning-icon {
    font-size: 18px;
    flex-shrink: 0;
  }

  .warning-text strong {
    font-size: 12px;
    color: var(--color-bronze);
  }

  .warning-text p {
    font-size: 11px;
    color: var(--color-ink-muted);
    margin: 2px 0 0 0;
  }

  .btn-run-backup {
    padding: 10px;
    font-size: 12px;
    font-weight: 600;
    background-color: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity 150ms ease;
  }

  .btn-run-backup:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-run-backup:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .banner-success {
    padding: 8px;
    font-size: 12px;
    background-color: rgba(94, 111, 69, 0.12);
    color: var(--color-olive);
    border-radius: var(--radius-sm);
  }

  .banner-error {
    padding: 8px;
    font-size: 12px;
    background-color: rgba(198, 40, 40, 0.12);
    color: var(--color-error);
    border-radius: var(--radius-sm);
  }

  .recent-backups-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 8px;
    border-top: 1px solid var(--color-paper-border);
    padding-top: 10px;
  }

  .section-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-muted);
    margin: 0;
  }

  .backups-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .backup-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
  }

  .backup-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .backup-file {
    font-size: 11px;
    font-family: monospace;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .backup-date {
    font-size: 10px;
    color: var(--color-ink-muted);
  }

  .backup-size {
    font-size: 11px;
    color: var(--color-olive);
    font-weight: 600;
  }

  .empty-state {
    padding: 16px;
    text-align: center;
    font-size: 11px;
    color: var(--color-ink-muted);
    font-style: italic;
  }
</style>
