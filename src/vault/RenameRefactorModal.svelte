<script lang="ts">
  /**
   * Modal de Renomeação com Refatoração Global (M2-05).
   *
   * Permite renomear uma entidade ou cena atualizando todas as citações
   * `[[Nome Antigo]]` e `[[Nome Antigo|Alias]]` do projeto com pré-visualização,
   * gravação atômica de snapshot e suporte a rollback.
   */

  import { t } from '../i18n';
  import { vault } from './vault.svelte';
  import { manuscript } from './manuscript.svelte';
  import type { RenameRefactorPreviewResponse } from '../tauri/bridge';

  let oldName = $derived(vault.refactorTarget || '');
  let newName = $state('');
  let isChecking = $state(false);
  let previewData = $state<RenameRefactorPreviewResponse | null>(null);
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);
  let lastCreatedSnapshot = $state<string | null>(null);

  $effect(() => {
    if (vault.isRefactorModalOpen && vault.refactorTarget) {
      newName = vault.refactorTarget;
      previewData = null;
      errorMessage = null;
      successMessage = null;
      void runPreview(vault.refactorTarget, vault.refactorTarget);
    }
  });

  async function runPreview(targetOld: string, targetNew: string) {
    if (!targetOld.trim() || !targetNew.trim()) {
      previewData = null;
      return;
    }
    isChecking = true;
    errorMessage = null;
    try {
      previewData = await vault.previewRename(targetOld, targetNew);
    } catch (err) {
      errorMessage = String(err);
    } finally {
      isChecking = false;
    }
  }

  function handleNewNameInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    newName = val;
    if (val.trim() && val.trim() !== oldName) {
      void runPreview(oldName, val.trim());
    } else {
      previewData = null;
    }
  }

  async function handleApply() {
    if (!oldName.trim() || !newName.trim()) return;
    errorMessage = null;
    try {
      const res = await vault.applyRename(oldName, newName);
      lastCreatedSnapshot = res.snapshot_id;
      successMessage = t('refactor.applySuccess', {
        replacements: res.total_replacements,
        docs: res.modified_docs_count,
      });
      await manuscript.refreshTree();
    } catch (err) {
      errorMessage = String(err);
    }
  }

  async function handleRollback() {
    if (!lastCreatedSnapshot) return;
    errorMessage = null;
    try {
      await vault.rollbackRename(lastCreatedSnapshot);
      lastCreatedSnapshot = null;
      successMessage = t('refactor.rollbackSuccess');
      await manuscript.refreshTree();
      vault.closeRefactorModal();
    } catch (err) {
      errorMessage = String(err);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && vault.isRefactorModalOpen) {
      vault.closeRefactorModal();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if vault.isRefactorModalOpen}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="refactor-title">
    <div class="modal-card">
      <div class="modal-header">
        <h2 id="refactor-title">🔄 {t('refactor.modalTitle')}</h2>
        <button
          class="close-button"
          onclick={() => vault.closeRefactorModal()}
          aria-label={t('common.close')}
        >
          ✕
        </button>
      </div>

      <div class="modal-body">
        <p class="modal-intro">{t('refactor.explanation')}</p>

        {#if errorMessage}
          <div class="alert error">{errorMessage}</div>
        {/if}

        {#if successMessage}
          <div class="alert success">
            <span>{successMessage}</span>
            {#if lastCreatedSnapshot}
              <button class="rollback-btn" onclick={handleRollback} disabled={vault.isRollingBack}>
                {vault.isRollingBack ? t('refactor.rollingBack') : t('refactor.undoButton')}
              </button>
            {/if}
          </div>
        {/if}

        <div class="form-row">
          <div class="form-group">
            <label for="old-name">{t('refactor.currentNameLabel')}</label>
            <input id="old-name" type="text" value={oldName} disabled class="input-disabled" />
          </div>

          <div class="form-group">
            <label for="new-name">{t('refactor.newNameLabel')}</label>
            <input
              id="new-name"
              type="text"
              value={newName}
              oninput={handleNewNameInput}
              placeholder={t('refactor.newNamePlaceholder')}
              class="input-active"
              disabled={vault.isRefactoring}
            />
          </div>
        </div>

        <!-- Painel de Pré-visualização -->
        <div class="preview-section">
          <div class="preview-header">
            <h4>{t('refactor.previewTitle')}</h4>
            {#if isChecking}
              <span class="loading-hint">⏳ {t('refactor.checking')}</span>
            {:else if previewData}
              <span class="badge-count">
                {t('refactor.occurrencesFound', { count: previewData.total_occurrences })}
              </span>
            {/if}
          </div>

          {#if previewData && previewData.affected_documents.length > 0}
            <div class="preview-list">
              {#each previewData.affected_documents as doc}
                <div class="doc-card">
                  <div class="doc-title">
                    <span>📄</span>
                    <strong>{doc.title}</strong>
                    <small>({doc.occurrences.length} {t('refactor.mentionsLabel')})</small>
                  </div>
                  {#each doc.occurrences as occ}
                    <div class="diff-block">
                      <span class="line-tag">L{occ.line_number}</span>
                      <div class="diff-content">
                        <div class="diff-line old">
                          <span class="diff-sign">-</span>
                          <span>{occ.old_snippet}</span>
                        </div>
                        <div class="diff-line new">
                          <span class="diff-sign">+</span>
                          <span>{occ.new_snippet}</span>
                        </div>
                      </div>
                    </div>
                  {/each}
                </div>
              {/each}
            </div>
          {:else if previewData && previewData.total_occurrences === 0}
            <div class="empty-preview">
              <p>{t('refactor.noOccurrencesFound')}</p>
            </div>
          {/if}
        </div>
      </div>

      <div class="modal-footer">
        <button
          type="button"
          class="btn-secondary"
          onclick={() => vault.closeRefactorModal()}
          disabled={vault.isRefactoring}
        >
          {t('common.cancel')}
        </button>

        <button
          type="button"
          class="btn-primary"
          onclick={handleApply}
          disabled={vault.isRefactoring || !newName.trim() || newName.trim() === oldName}
        >
          {vault.isRefactoring ? t('refactor.applying') : t('refactor.applyButton')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(30, 35, 29, 0.65);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-card {
    background-color: var(--color-paper-elevated, #FFFDF9);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    width: 90%;
    max-width: 640px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.25);
    font-family: var(--font-ui);
    color: var(--color-ink-charcoal);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 16px;
    cursor: pointer;
    color: var(--color-ink-faded);
  }

  .modal-body {
    padding: var(--space-md);
    overflow-y: auto;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .modal-intro {
    margin: 0;
    font-size: 13px;
    color: var(--color-ink-muted);
    line-height: 1.5;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-md);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-group label {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-faded);
    text-transform: uppercase;
  }

  input {
    padding: 8px 12px;
    border: 1px solid var(--color-paper-border);
    border-radius: 6px;
    font-size: 14px;
    background-color: var(--color-paper-base);
    color: var(--color-ink-charcoal);
  }

  .input-disabled {
    opacity: 0.6;
    background-color: var(--color-paper-border);
    cursor: not-allowed;
  }

  .input-active:focus {
    border-color: var(--color-olive);
    outline: none;
  }

  .preview-section {
    border: 1px solid var(--color-paper-border);
    border-radius: 6px;
    padding: var(--space-md);
    background-color: var(--color-paper-base);
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .preview-header h4 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-ink-faded);
  }

  .badge-count {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-olive);
  }

  .preview-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-height: 220px;
    overflow-y: auto;
  }

  .doc-card {
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: 6px;
    padding: 8px 12px;
  }

  .doc-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    margin-bottom: 6px;
  }

  .diff-block {
    display: flex;
    gap: 8px;
    font-size: 12px;
    font-family: var(--font-mono, monospace);
    margin-top: 4px;
    background-color: var(--color-paper-base);
    padding: 4px 8px;
    border-radius: 4px;
  }

  .line-tag {
    color: var(--color-ink-faded);
    flex-shrink: 0;
  }

  .diff-content {
    flex: 1;
    overflow-x: auto;
  }

  .diff-line {
    display: flex;
    gap: 6px;
    line-height: 1.4;
  }

  .diff-line.old {
    color: #C62828;
    text-decoration: line-through;
    opacity: 0.8;
  }

  .diff-line.new {
    color: #2E7D32;
    font-weight: 600;
  }

  .empty-preview {
    text-align: center;
    color: var(--color-ink-faded);
    padding: var(--space-md) 0;
    font-size: 13px;
  }

  .alert {
    padding: 10px 14px;
    border-radius: 6px;
    font-size: 13px;
  }

  .alert.error {
    background-color: rgba(198, 40, 40, 0.1);
    color: #C62828;
    border: 1px solid rgba(198, 40, 40, 0.3);
  }

  .alert.success {
    background-color: rgba(46, 125, 50, 0.1);
    color: #2E7D32;
    border: 1px solid rgba(46, 125, 50, 0.3);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .rollback-btn {
    background-color: #2E7D32;
    color: white;
    border: none;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .rollback-btn:hover {
    background-color: #1B5E20;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
    padding: var(--space-md);
    border-top: 1px solid var(--color-paper-border);
  }

  .btn-secondary {
    background: none;
    border: 1px solid var(--color-paper-border);
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
    color: var(--color-ink-charcoal);
  }

  .btn-primary {
    background-color: var(--color-olive);
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
