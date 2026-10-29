<script lang="ts">
  import { t } from '../i18n';
  import { manuscript } from '../vault/manuscript.svelte';

  let isConfirming = $state(false);
  let isClearing = $state(false);
  let errorMessage = $state<string | null>(null);

  async function handleConfirmEmpty() {
    isClearing = true;
    errorMessage = null;
    try {
      await manuscript.emptyTrash();
      isConfirming = false;
    } catch (err) {
      errorMessage = String(err);
    } finally {
      isClearing = false;
    }
  }

  function handleCancel() {
    isConfirming = false;
    errorMessage = null;
  }
</script>

{#if manuscript.isTrashOpen}
  <div
    class="trash-backdrop"
    onclick={() => manuscript.toggleTrash(false)}
    onkeydown={(e) => e.key === 'Escape' && manuscript.toggleTrash(false)}
    tabindex="0"
    role="button"
  >
    <div
      class="trash-drawer"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      tabindex="-1"
      role="dialog"
      aria-label={t('manuscript.trash')}
    >
      <header class="trash-header">
        <div class="title-group">
          <span class="trash-icon">🗑️</span>
          <h3>{t('manuscript.trash')}</h3>
          <span class="count-badge">{manuscript.trashCount}</span>
        </div>
        <button
          type="button"
          class="close-btn"
          onclick={() => manuscript.toggleTrash(false)}
          title={t('common.close')}
        >
          ×
        </button>
      </header>

      <div class="trash-body">
        {#if errorMessage}
          <div class="trash-error">⚠️ {errorMessage}</div>
        {/if}

        {#if manuscript.trash.length === 0}
          <div class="empty-state">
            <span class="empty-icon">🍃</span>
            <p>{t('manuscript.noTrashItems')}</p>
            <span class="empty-hint">Nenhum capítulo ou cena descartada no momento.</span>
          </div>
        {:else}
          <ul class="trash-list">
            {#each manuscript.trash as item}
              <li class="trash-item">
                <div class="item-meta">
                  <span class="item-kind">{item.kind.toUpperCase()}</span>
                  <span class="item-title">{item.title}</span>
                </div>
                <button
                  type="button"
                  class="restore-btn"
                  onclick={() => manuscript.restoreFromTrash(item.id)}
                  title={t('manuscript.restore')}
                >
                  ↩ {t('manuscript.restore')}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      {#if manuscript.trash.length > 0}
        <footer class="trash-footer">
          {#if isConfirming}
            <div class="confirm-box">
              <span class="confirm-text">⚠️ Deseja apagar permanentemente todos os {manuscript.trashCount} itens da lixeira?</span>
              <div class="confirm-actions">
                <button
                  type="button"
                  class="confirm-danger-btn"
                  onclick={handleConfirmEmpty}
                  disabled={isClearing}
                >
                  {isClearing ? 'Esvaziando...' : 'Sim, Esvaziar Tudo'}
                </button>
                <button
                  type="button"
                  class="cancel-btn"
                  onclick={handleCancel}
                  disabled={isClearing}
                >
                  Cancelar
                </button>
              </div>
            </div>
          {:else}
            <button
              type="button"
              class="empty-trash-btn"
              onclick={() => (isConfirming = true)}
            >
              🗑️ {t('manuscript.emptyTrash')}
            </button>
          {/if}
        </footer>
      {/if}
    </div>
  </div>
{/if}

<style>
  .trash-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(2px);
    z-index: 1000;
    display: flex;
    justify-content: flex-end;
  }

  .trash-drawer {
    width: 380px;
    max-width: 90vw;
    height: 100%;
    background-color: var(--color-paper-base);
    border-left: 1px solid var(--color-paper-border);
    display: flex;
    flex-direction: column;
    box-shadow: -4px 0 16px rgba(0, 0, 0, 0.15);
    animation: slideIn 0.2s ease-out;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }

  .trash-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md);
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .trash-icon {
    font-size: 16px;
  }

  .trash-header h3 {
    margin: 0;
    font-size: 14px;
    font-family: var(--font-ui);
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .count-badge {
    background-color: var(--color-paper-border);
    color: var(--color-ink-faded);
    font-size: 11px;
    font-family: var(--font-mono);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
  }

  .close-btn {
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    background: transparent;
    font-size: 13px;
    color: var(--color-ink-muted, #6B7280);
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    color: var(--color-ink-charcoal, #111827);
    background-color: var(--color-paper-hover, rgba(0, 0, 0, 0.05));
    border-color: var(--color-ink-faded, #9CA3AF);
  }

  :global(.dark) .close-btn {
    border-color: #383C32;
    color: #A4A89E;
  }

  :global(.dark) .close-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: #FFFFFF;
    border-color: #555A4E;
  }

  .trash-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-sm);
  }

  .trash-error {
    background: #fee2e2;
    color: #991b1b;
    padding: 0.6rem 0.8rem;
    border-radius: 4px;
    font-size: 0.8rem;
    margin-bottom: 0.75rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 60%;
    color: var(--color-ink-muted);
    font-family: var(--font-ui);
    font-size: 13px;
    text-align: center;
    gap: 0.5rem;
  }

  .empty-icon {
    font-size: 2rem;
  }

  .empty-hint {
    font-size: 0.78rem;
    color: var(--color-ink-faded);
  }

  .trash-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .trash-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    gap: var(--space-xs);
  }

  .item-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .item-kind {
    font-family: var(--font-ui);
    font-size: 9px;
    font-weight: 700;
    color: var(--color-ink-faded);
    letter-spacing: 0.5px;
  }

  .item-title {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-ink-charcoal);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .restore-btn {
    font-family: var(--font-ui);
    font-size: 11px;
    padding: 4px 8px;
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    background-color: var(--color-paper-base);
    color: var(--color-ink-charcoal);
    cursor: pointer;
    white-space: nowrap;
    transition: background-color 0.15s ease;
  }

  .restore-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-olive);
  }

  .trash-footer {
    padding: var(--space-md);
    border-top: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .empty-trash-btn {
    width: 100%;
    padding: 8px;
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    background-color: rgba(184, 51, 42, 0.1);
    color: var(--color-error);
    border: 1px solid rgba(184, 51, 42, 0.2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .empty-trash-btn:hover {
    background-color: rgba(184, 51, 42, 0.2);
  }

  .confirm-box {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 0.5rem;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: 6px;
  }

  .confirm-text {
    font-size: 0.78rem;
    color: var(--color-error);
    font-family: var(--font-ui);
    line-height: 1.35;
  }

  .confirm-actions {
    display: flex;
    gap: 0.4rem;
  }

  .confirm-danger-btn {
    flex: 1;
    padding: 6px;
    font-family: var(--font-ui);
    font-size: 0.8rem;
    font-weight: 600;
    background-color: #b8332a;
    color: #ffffff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .confirm-danger-btn:hover {
    background-color: #991b1b;
  }

  .cancel-btn {
    padding: 6px 12px;
    font-family: var(--font-ui);
    font-size: 0.8rem;
    background: transparent;
    color: var(--color-ink-muted);
    border: 1px solid var(--color-paper-border);
    border-radius: 4px;
    cursor: pointer;
  }

  .cancel-btn:hover {
    background: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }
</style>
