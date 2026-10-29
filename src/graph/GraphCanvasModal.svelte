<script lang="ts">
  /**
   * Modal Interativo Unificado do Grafo e Canvas Visual (PRD E4, E10 / M8-01 e M8-03).
   */

  import { graphStore } from './graph.svelte';
  import { t } from '../i18n';
  import GraphView from './GraphView.svelte';
  import CanvasView from '../canvas/CanvasView.svelte';
  import EntityDrawer from './EntityDrawer.svelte';
  import type { GraphNode, CanvasNode } from '../tauri/bridge';

  let selectedEntityForDrawer = $state<{
    id: string;
    title: string;
    kind: string;
    summary?: string | null;
  } | null>(null);

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (selectedEntityForDrawer) {
        selectedEntityForDrawer = null;
      } else {
        graphStore.closeModal();
      }
    }
  }

  function handleSelectGraphNode(node: GraphNode) {
    selectedEntityForDrawer = {
      id: node.id,
      title: node.title,
      kind: node.kind,
      summary: node.summary,
    };
  }

  function handleSelectCanvasEntity(node: CanvasNode) {
    selectedEntityForDrawer = {
      id: node.file || node.id,
      title: node.label || 'Sem título',
      kind: node.entity_kind || 'item',
      summary: node.entity_summary,
    };
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if graphStore.isOpen}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-labelledby="graph-modal-title"
  >
    <div class="modal-window">
      <!-- Cabeçalho Principal -->
      <header class="modal-header">
        <div class="header-left">
          <div class="brand-badge">🕸️</div>
          <div>
            <h2 id="graph-modal-title" class="header-title">
              {t('graph.modalTitle')}
            </h2>
            {#if graphStore.graphData}
              <div class="header-stats">
                <span>{t('graph.statsNodes', { count: graphStore.filteredNodes.length })}</span>
                <span>•</span>
                <span>{t('graph.statsEdges', { count: graphStore.filteredEdges.length })}</span>
                <span>•</span>
                <span>{t('graph.statsOrphans', { count: graphStore.graphData.orphan_nodes_count })}</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Abas Centrais: Grafo vs. Canvas -->
        <div class="tab-switcher" role="tablist">
          <button
            type="button"
            class="tab-btn"
            class:active={graphStore.activeTab === 'graph'}
            onclick={() => graphStore.setActiveTab('graph')}
            role="tab"
            aria-selected={graphStore.activeTab === 'graph'}
          >
            {t('graph.tabGraph')}
          </button>
          <button
            type="button"
            class="tab-btn"
            class:active={graphStore.activeTab === 'canvas'}
            onclick={() => graphStore.setActiveTab('canvas')}
            role="tab"
            aria-selected={graphStore.activeTab === 'canvas'}
          >
            {t('graph.tabCanvas')}
          </button>
        </div>

        <button
          type="button"
          class="btn-close"
          onclick={() => graphStore.closeModal()}
          aria-label="Fechar"
        >
          ✕
        </button>
      </header>

      <!-- Barra de Filtros e Busca (Exibida no Grafo) -->
      {#if graphStore.activeTab === 'graph'}
        <div class="graph-filters-bar">
          <div class="search-input-wrapper">
            <span class="search-icon">🔍</span>
            <input
              type="text"
              class="search-input"
              placeholder={t('graph.searchPlaceholder')}
              bind:value={graphStore.searchQuery}
            />
          </div>

          <div class="kind-filters">
            {#each ['scene', 'character', 'location', 'item', 'faction', 'lore'] as kind}
              <button
                type="button"
                class="filter-pill {kind}"
                class:active={graphStore.selectedKinds.includes(kind)}
                onclick={() => graphStore.toggleKind(kind)}
              >
                {kind}
              </button>
            {/each}
          </div>

          <button
            type="button"
            class="orphan-toggle-btn"
            class:active={graphStore.includeOrphans}
            onclick={() => graphStore.toggleOrphans()}
          >
            {graphStore.includeOrphans ? t('graph.hideOrphans') : t('graph.showOrphans')}
          </button>
        </div>
      {/if}

      <!-- Área de Visualização Principal -->
      <main class="viewport-area">
        {#if graphStore.activeTab === 'graph'}
          <GraphView onSelectNode={handleSelectGraphNode} />
        {:else if graphStore.activeTab === 'canvas'}
          <CanvasView onSelectEntity={handleSelectCanvasEntity} />
        {/if}

        <!-- Drawer Lateral Retrátil de Entidade (M8-04) -->
        {#if selectedEntityForDrawer}
          <EntityDrawer
            nodeId={selectedEntityForDrawer.id}
            nodeTitle={selectedEntityForDrawer.title}
            nodeKind={selectedEntityForDrawer.kind}
            nodeSummary={selectedEntityForDrawer.summary}
            onClose={() => (selectedEntityForDrawer = null)}
          />
        {/if}
      </main>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background-color: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }

  .modal-window {
    width: 95vw;
    height: 92vh;
    max-width: 1500px;
    background-color: var(--color-paper-base, #FDFCFA);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 12px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background-color: var(--color-paper-elevated, #FFFFFF);
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    height: 56px;
    box-sizing: border-box;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .brand-badge {
    font-size: 20px;
    padding: 6px;
    border-radius: 8px;
    background-color: var(--color-paper-surface, #F8F4EC);
  }

  .header-title {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    color: var(--color-ink-charcoal, #1E231D);
  }

  .header-stats {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--color-ink-muted, #726E65);
    margin-top: 2px;
  }

  .tab-switcher {
    display: flex;
    background-color: var(--color-paper-surface, #F8F4EC);
    padding: 3px;
    border-radius: 8px;
    gap: 4px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
  }

  .tab-btn {
    border: none;
    background: transparent;
    padding: 6px 14px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-muted, #726E65);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab-btn.active {
    background-color: var(--color-paper-elevated, #FFFFFF);
    color: var(--color-ink-charcoal, #1E231D);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.06);
  }

  .btn-close {
    width: 32px;
    height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    font-size: 14px;
    color: var(--color-ink-muted, #726E65);
    cursor: pointer;
    border-radius: 8px;
    transition: all 0.15s ease;
  }

  .btn-close:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
    border-color: var(--color-ink-faded, #9CA3AF);
    color: var(--color-ink-charcoal, #1E231D);
  }

  :global(.dark) .btn-close {
    border-color: #383C32;
    color: #A4A89E;
  }

  :global(.dark) .btn-close:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: #FFFFFF;
    border-color: #555A4E;
  }

  .graph-filters-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 20px;
    background-color: var(--color-paper-surface, #F8F4EC);
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    font-size: 12px;
  }

  .search-input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    width: 220px;
  }

  .search-icon {
    position: absolute;
    left: 8px;
    font-size: 12px;
    color: var(--color-ink-muted, #726E65);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 5px 8px 5px 26px;
    font-size: 12px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    background-color: var(--color-paper-elevated, #FFFFFF);
    outline: none;
  }

  .search-input:focus {
    border-color: var(--color-olive, #5E6F45);
  }

  .kind-filters {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
  }

  .filter-pill {
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 600;
    text-transform: capitalize;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    background-color: var(--color-paper-elevated, #FFFFFF);
    color: var(--color-ink-muted, #726E65);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .filter-pill.active {
    background-color: var(--color-paper-hover, #F4EFE6);
    border-color: var(--color-olive, #5E6F45);
    color: var(--color-ink-charcoal, #1E231D);
  }

  .orphan-toggle-btn {
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    background-color: var(--color-paper-elevated, #FFFFFF);
    color: var(--color-ink-muted, #726E65);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }

  .orphan-toggle-btn.active {
    border-color: var(--color-olive, #5E6F45);
    color: var(--color-olive, #5E6F45);
  }

  .viewport-area {
    flex: 1;
    position: relative;
    overflow: hidden;
  }
</style>
