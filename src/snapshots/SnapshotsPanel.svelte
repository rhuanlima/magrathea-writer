<script lang="ts">
  import { t } from '../i18n';
  import { snapshots } from './snapshots.svelte';
  import { manuscript } from '../vault/manuscript.svelte';

  interface Props {
    currentContent: string;
    onRestoreContent?: (newContent: string) => void;
  }

  let { currentContent, onRestoreContent }: Props = $props();

  let newLabel = $state('');
  let isCreating = $state(false);

  // Recarrega snapshots quando a cena ativa muda
  $effect(() => {
    const activeId = manuscript.activeSceneId;
    if (activeId) {
      void snapshots.loadSnapshots(activeId);
    }
  });

  async function handleCreateSnapshot() {
    if (!manuscript.activeSceneId) return;
    const label = newLabel.trim();
    const created = await snapshots.createSnapshot(
      manuscript.activeSceneId,
      label,
      currentContent
    );
    if (created) {
      newLabel = '';
      isCreating = false;
    }
  }

  async function handleCompare(snapshotId: string) {
    if (!manuscript.activeSceneId) return;
    await snapshots.compareWithCurrent(
      manuscript.activeSceneId,
      snapshotId,
      currentContent
    );
  }

  async function handleRestore(snapshotId: string) {
    if (!manuscript.activeSceneId) return;
    const confirmMsg = t('snapshots.confirmRestore');
    if (!confirm(confirmMsg)) return;

    const restored = await snapshots.restoreSnapshot(
      manuscript.activeSceneId,
      snapshotId,
      currentContent
    );
    if (restored !== null && onRestoreContent) {
      onRestoreContent(restored);
    }
  }

  function formatIso(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(undefined, {
        day: '2-digit',
        month: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return iso;
    }
  }
</script>

<div class="snapshots-container">
  <div class="header-actions">
    <span class="panel-title">{t('snapshots.tabTitle')}</span>
    <button
      type="button"
      class="btn-create"
      onclick={() => (isCreating = !isCreating)}
    >
      + {t('snapshots.createSnapshotButton')}
    </button>
  </div>

  {#if isCreating}
    <div class="create-box">
      <label for="snapshot-label-input" class="create-label">
        {t('snapshots.newSnapshotPrompt')}
      </label>
      <input
        id="snapshot-label-input"
        type="text"
        class="label-input"
        placeholder={t('snapshots.labelPlaceholder')}
        bind:value={newLabel}
        onkeydown={(e) => e.key === 'Enter' && handleCreateSnapshot()}
      />
      <div class="create-buttons">
        <button
          type="button"
          class="btn-submit"
          disabled={snapshots.isSaving}
          onclick={handleCreateSnapshot}
        >
          {snapshots.isSaving ? '...' : t('snapshots.saveSnapshot')}
        </button>
        <button
          type="button"
          class="btn-cancel"
          onclick={() => (isCreating = false)}
        >
          Cancelar
        </button>
      </div>
    </div>
  {/if}

  {#if snapshots.successMessage}
    <div class="success-banner">{snapshots.successMessage}</div>
  {/if}

  {#if snapshots.errorMessage}
    <div class="error-banner">{snapshots.errorMessage}</div>
  {/if}

  <!-- Visualizador de Diff Ativo -->
  {#if snapshots.activeDiff}
    <div class="diff-viewer">
      <div class="diff-header">
        <span class="diff-title">{t('snapshots.diffTitle')}</span>
        <button
          type="button"
          class="btn-close-diff"
          onclick={() => snapshots.closeDiff()}
        >
          ✕ {t('snapshots.closeDiff')}
        </button>
      </div>

      <div class="diff-stats">
        <span>{t('snapshots.wordsCount', { count: snapshots.activeDiff.snapshot_word_count })} (Versão)</span>
        <span>→</span>
        <span>{t('snapshots.wordsCount', { count: snapshots.activeDiff.current_word_count })} (Atual)</span>
      </div>

      <div class="diff-lines">
        {#each snapshots.activeDiff.lines as line}
          <div class="diff-line {line.kind}">
            <span class="diff-gutter">
              {line.kind === 'added' ? '+' : line.kind === 'removed' ? '-' : ' '}
            </span>
            <span class="diff-text">{line.content || ' '}</span>
          </div>
        {/each}
      </div>

      {#if snapshots.selectedSnapshot}
        <button
          type="button"
          class="btn-restore-diff"
          disabled={snapshots.isRestoring}
          onclick={() => handleRestore(snapshots.selectedSnapshot!.id)}
        >
          {snapshots.isRestoring ? '...' : `↺ ${t('snapshots.restoreButton')}`}
        </button>
      {/if}
    </div>
  {/if}

  <!-- Lista de Snapshots -->
  <div class="snapshots-list">
    {#if snapshots.isLoading && snapshots.snapshots.length === 0}
      <div class="empty-state">Carregando versões...</div>
    {:else if snapshots.snapshots.length === 0}
      <div class="empty-state">{t('snapshots.noSnapshotsYet')}</div>
    {:else}
      {#each snapshots.snapshots as snap (snap.id)}
        <div class="snapshot-card" class:active={snapshots.selectedSnapshot?.id === snap.id}>
          <div class="card-top">
            <span class="snap-label">{snap.label}</span>
            <span class="snap-badge {snap.trigger}">
              {snap.trigger === 'auto' ? t('snapshots.autoTag') : t('snapshots.manualTag')}
            </span>
          </div>

          <div class="card-meta">
            <span class="snap-time">{formatIso(snap.timestamp)}</span>
            <span class="snap-words">{t('snapshots.wordsCount', { count: snap.word_count })}</span>
          </div>

          <div class="card-actions">
            <button
              type="button"
              class="btn-action"
              onclick={() => handleCompare(snap.id)}
            >
              🔍 {t('snapshots.compareWithCurrent')}
            </button>
            <button
              type="button"
              class="btn-action btn-restore"
              disabled={snapshots.isRestoring}
              onclick={() => handleRestore(snap.id)}
            >
              ↺ {t('snapshots.restoreButton')}
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .snapshots-container {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .header-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .btn-create {
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 600;
    background-color: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity 150ms ease;
  }

  .btn-create:hover {
    opacity: 0.9;
  }

  .create-box {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
  }

  .create-label {
    font-size: 11px;
    color: var(--color-ink-muted);
  }

  .label-input {
    padding: 6px 8px;
    font-size: 12px;
    background-color: var(--color-paper-base);
    color: var(--color-ink-charcoal);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
  }

  .create-buttons {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }

  .btn-submit {
    padding: 4px 10px;
    font-size: 11px;
    background-color: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .btn-cancel {
    padding: 4px 10px;
    font-size: 11px;
    background: transparent;
    color: var(--color-ink-muted);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .success-banner {
    padding: 6px 10px;
    font-size: 11px;
    background-color: rgba(94, 111, 69, 0.12);
    color: var(--color-olive);
    border-radius: var(--radius-sm);
  }

  .error-banner {
    padding: 6px 10px;
    font-size: 11px;
    background-color: rgba(198, 40, 40, 0.12);
    color: var(--color-error);
    border-radius: var(--radius-sm);
  }

  /* Visualizador de Diff */
  .diff-viewer {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-bronze);
    border-radius: var(--radius-sm);
  }

  .diff-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .diff-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-bronze);
  }

  .btn-close-diff {
    background: transparent;
    border: none;
    font-size: 11px;
    color: var(--color-ink-muted);
    cursor: pointer;
  }

  .diff-stats {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--color-ink-muted);
  }

  .diff-lines {
    max-height: 180px;
    overflow-y: auto;
    font-family: monospace;
    font-size: 11px;
    line-height: 1.4;
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    padding: 4px 0;
  }

  .diff-line {
    display: flex;
    padding: 1px 6px;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .diff-line.added {
    background-color: rgba(94, 111, 69, 0.15);
    color: var(--color-olive);
  }

  .diff-line.removed {
    background-color: rgba(198, 40, 40, 0.12);
    color: var(--color-error);
  }

  .diff-line.equal {
    color: var(--color-ink-muted);
  }

  .diff-gutter {
    width: 14px;
    user-select: none;
    font-weight: bold;
  }

  .btn-restore-diff {
    align-self: flex-start;
    padding: 4px 10px;
    font-size: 11px;
    background-color: var(--color-bronze);
    color: #ffffff;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  /* Lista de Cards de Snapshot */
  .snapshots-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .snapshot-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 10px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    transition: border-color 150ms ease;
  }

  .snapshot-card.active {
    border-color: var(--color-bronze);
  }

  .card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .snap-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .snap-badge {
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 10px;
  }

  .snap-badge.manual {
    background-color: rgba(94, 111, 69, 0.15);
    color: var(--color-olive);
  }

  .snap-badge.auto {
    background-color: rgba(184, 115, 51, 0.15);
    color: var(--color-bronze);
  }

  .card-meta {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--color-ink-muted);
  }

  .card-actions {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }

  .btn-action {
    flex: 1;
    padding: 3px 6px;
    font-size: 10px;
    font-weight: 600;
    background-color: var(--color-paper-base);
    color: var(--color-ink-charcoal);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 150ms ease;
  }

  .btn-action:hover {
    background-color: var(--color-paper-hover);
    border-color: var(--color-olive);
  }

  .btn-action.btn-restore:hover {
    border-color: var(--color-bronze);
    color: var(--color-bronze);
  }

  .empty-state {
    padding: 16px;
    text-align: center;
    font-size: 12px;
    color: var(--color-ink-muted);
    font-style: italic;
  }
</style>
