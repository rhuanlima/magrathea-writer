<script lang="ts">
  /**
   * Painel Drawer retrátil para inspeção e edição rápida de fichas no Canvas e Grafo (M8-04).
   */

  import { t } from '../i18n';
  import { manuscript } from '../vault/manuscript.svelte';
  import { worldbuilding } from '../worldbuilding/worldbuilding.svelte';
  import { graphStore } from './graph.svelte';

  interface Props {
    nodeId: string | null;
    nodeTitle: string | null;
    nodeKind: string | null;
    nodeSummary?: string | null;
    onClose: () => void;
  }

  let { nodeId, nodeTitle, nodeKind, nodeSummary, onClose }: Props = $props();

  let appearances = $state<string[]>([]);

  $effect(() => {
    if (nodeId) {
      worldbuilding.loadAppearances(nodeId).then(() => {
        appearances = worldbuilding.appearances.map((a: { scene_title: string }) => a.scene_title);
      }).catch(() => {
        appearances = [];
      });
    }
  });

  function openInEditor() {
    if (!nodeId) return;
    manuscript.activeSceneId = nodeId;
    graphStore.closeModal();
  }
</script>

{#if nodeId}
  <aside class="entity-drawer" aria-label="Detalhes da Entidade">
    <div class="drawer-header">
      <div class="header-main">
        <span class="kind-badge {nodeKind?.toLowerCase()}">
          {nodeKind || 'Item'}
        </span>
        <h3 class="drawer-title">{nodeTitle || 'Sem título'}</h3>
      </div>
      <button type="button" class="close-btn" onclick={onClose} aria-label="Fechar painel">
        ✕
      </button>
    </div>

    <div class="drawer-content">
      {#if nodeSummary}
        <div class="section-block">
          <h4 class="section-label">Resumo Narrativo</h4>
          <p class="summary-text">{nodeSummary}</p>
        </div>
      {/if}

      <div class="section-block">
        <h4 class="section-label">Aparições no Manuscrito ({appearances.length})</h4>
        {#if appearances.length > 0}
          <ul class="appearances-list">
            {#each appearances as sceneTitle}
              <li class="appearance-item">
                <span class="scene-bullet">📖</span>
                <span class="scene-name">{sceneTitle}</span>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="empty-hint">Nenhuma citação direta detectada no texto ainda.</p>
        {/if}
      </div>

      <div class="drawer-actions">
        <button type="button" class="btn-open-editor" onclick={openInEditor}>
          ✍️ {t('graph.openInEditor')}
        </button>
      </div>
    </div>
  </aside>
{/if}

<style>
  .entity-drawer {
    position: absolute;
    top: 56px;
    right: 0;
    bottom: 0;
    width: 320px;
    background-color: var(--color-paper-elevated, #FFFFFF);
    border-left: 1px solid var(--color-paper-border, #E5E0D8);
    box-shadow: -4px 0 16px rgba(0, 0, 0, 0.08);
    display: flex;
    flex-direction: column;
    z-index: 40;
    animation: slideIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }

  .drawer-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
  }

  .kind-badge {
    display: inline-block;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 2px 8px;
    border-radius: 9999px;
    margin-bottom: 4px;
    background-color: var(--color-paper-surface, #F8F4EC);
    color: var(--color-ink-muted, #726E65);
  }

  .kind-badge.character {
    background-color: rgba(217, 119, 6, 0.15);
    color: #D97706;
  }

  .kind-badge.location {
    background-color: rgba(5, 150, 105, 0.15);
    color: #059669;
  }

  .kind-badge.scene {
    background-color: rgba(37, 99, 235, 0.15);
    color: #2563EB;
  }

  .drawer-title {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    color: var(--color-ink-charcoal, #1E231D);
  }

  .close-btn {
    background: transparent;
    border: none;
    font-size: 16px;
    color: var(--color-ink-muted, #726E65);
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
    color: var(--color-ink-charcoal, #1E231D);
  }

  .drawer-content {
    padding: 16px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
    flex: 1;
  }

  .section-label {
    margin: 0 0 6px 0;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-ink-muted, #726E65);
  }

  .summary-text {
    margin: 0;
    font-size: 13px;
    line-height: 1.5;
    color: var(--color-ink-base, #2E332B);
  }

  .appearances-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .appearance-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 6px;
    background-color: var(--color-paper-surface, #F8F4EC);
    font-size: 12px;
  }

  .empty-hint {
    margin: 0;
    font-size: 12px;
    color: var(--color-ink-muted, #726E65);
    font-style: italic;
  }

  .drawer-actions {
    margin-top: auto;
    padding-top: 16px;
  }

  .btn-open-editor {
    width: 100%;
    padding: 10px 14px;
    border-radius: 8px;
    background-color: var(--color-olive, #5E6F45);
    color: #FFFFFF;
    border: none;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .btn-open-editor:hover {
    opacity: 0.9;
  }
</style>
