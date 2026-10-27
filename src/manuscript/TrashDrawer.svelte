<script lang="ts">
  import { t } from '../i18n';
  import { manuscript } from '../vault/manuscript.svelte';

  function handleEmptyTrash() {
    if (confirm(t('manuscript.confirmEmptyTrash'))) {
      void manuscript.emptyTrash();
    }
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
        {#if manuscript.trash.length === 0}
          <div class="empty-state">
            <p>{t('manuscript.noTrashItems')}</p>
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
          <button
            type="button"
            class="empty-trash-btn"
            onclick={handleEmptyTrash}
          >
            {t('manuscript.emptyTrash')}
          </button>
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
    width: 360px;
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
    gap: 8px;
  }

  .title-group h3 {
    margin: 0;
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .count-badge {
    background-color: var(--color-paper-border);
    color: var(--color-ink-muted);
    font-size: 11px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 10px;
  }

  .close-btn {
    border: none;
    background: transparent;
    font-size: 20px;
    line-height: 1;
    cursor: pointer;
    color: var(--color-ink-muted);
  }

  .close-btn:hover {
    color: var(--color-ink-charcoal);
  }

  .trash-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md);
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--color-ink-muted);
    font-family: var(--font-ui);
    font-size: 13px;
    text-align: center;
  }

  .trash-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .trash-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    gap: 8px;
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
    transition: background-color 0.15s ease;
  }

  .empty-trash-btn:hover {
    background-color: rgba(184, 51, 42, 0.2);
  }
</style>
