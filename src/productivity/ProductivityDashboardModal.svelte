<script lang="ts">
  import { t } from '../i18n';
  import { editorViewState } from '../editor/editor-view-state.svelte';
  import ProductivityDashboardContent from './ProductivityDashboardContent.svelte';

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && editorViewState.isProductivityModalOpen) {
      editorViewState.closeProductivityDashboard();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if editorViewState.isProductivityModalOpen}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="dash-title">
    <div class="dashboard-dialog">
      <header class="dialog-header">
        <div class="header-branding">
          <span class="header-icon">📊</span>
          <h2 id="dash-title" class="dialog-title">{t('productivity.dashboardTitle')}</h2>
        </div>
        <button
          type="button"
          class="dialog-close-btn"
          onclick={() => editorViewState.closeProductivityDashboard()}
          aria-label={t('common.close')}
        >
          ✕
        </button>
      </header>

      <div class="dialog-content">
        <ProductivityDashboardContent />
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(30, 35, 29, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: var(--space-md);
  }

  .dashboard-dialog {
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    box-shadow: var(--shadow-modal);
    width: 100%;
    max-width: 680px;
    max-height: 90vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .header-branding {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .header-icon {
    font-size: 18px;
  }

  .dialog-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin: 0;
  }

  .dialog-close-btn {
    background: transparent;
    border: none;
    font-size: 16px;
    color: var(--color-ink-muted);
    cursor: pointer;
    padding: var(--space-xs);
    border-radius: var(--rounded-full);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    transition: background-color var(--transition-fast), color var(--transition-fast);
  }

  .dialog-close-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .dialog-content {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }
</style>
