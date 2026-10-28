<script lang="ts">
  /**
   * Aba de Grafo Local no Painel Inspetor (M8-02).
   *
   * Renderiza conexões diretas e indiretas (1 ou 2 graus) do item ativo.
   */

  import { graphStore } from '../graph/graph.svelte';
  import { manuscript } from '../vault/manuscript.svelte';
  import { t } from '../i18n';
  import type { LocalGraphData, GraphNode } from '../tauri/bridge';

  interface Props {
    centerId: string;
  }

  let { centerId }: Props = $props();

  let depth = $state<1 | 2>(1);
  let localData = $state<LocalGraphData | null>(null);
  let isLoading = $state(false);

  $effect(() => {
    if (centerId) {
      void loadLocal();
    }
  });

  async function loadLocal() {
    isLoading = true;
    try {
      localData = await graphStore.fetchLocalGraph(centerId, depth);
    } catch {
      localData = null;
    } finally {
      isLoading = false;
    }
  }

  function handleSelectNode(node: GraphNode) {
    manuscript.activeSceneId = node.id;
  }
</script>

<div class="local-graph-tab">
  <div class="controls-bar">
    <span class="depth-label">{t('graph.localGraphDepth')}</span>
    <div class="depth-buttons">
      <button
        type="button"
        class="depth-btn"
        class:active={depth === 1}
        onclick={() => { depth = 1; void loadLocal(); }}
      >
        1°
      </button>
      <button
        type="button"
        class="depth-btn"
        class:active={depth === 2}
        onclick={() => { depth = 2; void loadLocal(); }}
      >
        2°
      </button>
    </div>
  </div>

  {#if isLoading}
    <div class="loading-state">
      <span>Carregando conexões locais...</span>
    </div>
  {:else if localData && localData.nodes.length > 1}
    <div class="nodes-hierarchy">
      <div class="center-badge">
        <span class="center-icon">🎯</span>
        <strong class="center-title">
          {localData.nodes.find((n) => n.id === centerId)?.title || 'Item Ativo'}
        </strong>
      </div>

      <div class="connections-sublist">
        <span class="connections-count">
          {localData.nodes.length - 1} entidades e cenas conectadas:
        </span>

        <div class="connected-cards">
          {#each localData.nodes as node (node.id)}
            {#if node.id !== centerId}
              <button
                type="button"
                class="node-card"
                onclick={() => handleSelectNode(node)}
                title="Clique para abrir no editor"
              >
                <div class="node-main">
                  <span class="node-kind-badge {node.kind.toLowerCase()}">
                    {node.kind}
                  </span>
                  <span class="node-name">{node.title}</span>
                </div>
                <span class="node-conn-badge">
                  {node.connections_count} links
                </span>
              </button>
            {/if}
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <span class="empty-icon">🕸️</span>
      <p class="empty-text">Nenhuma conexão direta ou citação reversa encontrada para este documento.</p>
    </div>
  {/if}
</div>

<style>
  .local-graph-tab {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    font-size: 12px;
  }

  .controls-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
  }

  .depth-label {
    font-weight: 600;
    color: var(--color-ink-muted, #726E65);
  }

  .depth-buttons {
    display: flex;
    gap: 4px;
  }

  .depth-btn {
    padding: 3px 8px;
    border-radius: 4px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    background-color: var(--color-paper-surface, #F8F4EC);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }

  .depth-btn.active {
    background-color: var(--color-olive, #5E6F45);
    color: #FFFFFF;
    border-color: transparent;
  }

  .loading-state,
  .empty-state {
    padding: 24px 12px;
    text-align: center;
    color: var(--color-ink-muted, #726E65);
  }

  .empty-icon {
    font-size: 24px;
    display: block;
    margin-bottom: 6px;
  }

  .empty-text {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
  }

  .nodes-hierarchy {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .center-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 6px;
    background-color: rgba(94, 111, 69, 0.12);
    border: 1px solid rgba(94, 111, 69, 0.25);
  }

  .center-title {
    color: var(--color-olive, #5E6F45);
    font-size: 13px;
  }

  .connections-count {
    display: block;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-ink-muted, #726E65);
    margin-bottom: 6px;
  }

  .connected-cards {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .node-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border-radius: 6px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    background-color: var(--color-paper-elevated, #FFFFFF);
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .node-card:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
    border-color: var(--color-olive, #5E6F45);
  }

  .node-main {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .node-kind-badge {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 1px 5px;
    border-radius: 3px;
    background-color: var(--color-paper-surface, #F8F4EC);
  }

  .node-name {
    font-weight: 600;
    color: var(--color-ink-charcoal, #1E231D);
  }

  .node-conn-badge {
    font-size: 10px;
    color: var(--color-ink-muted, #726E65);
  }
</style>
