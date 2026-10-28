<script lang="ts">
  /**
   * Componente de Quadro Infinito (Infinite Canvas) baseado no JSON Canvas 1.0 (PRD E10 / M8-03).
   */

  import { onMount } from 'svelte';
  import { canvasStore } from './canvas.svelte';
  import { manuscript } from '../vault/manuscript.svelte';
  import { t } from '../i18n';
  import type { CanvasNode, CanvasEdge } from '../tauri/bridge';

  interface Props {
    onSelectEntity?: (node: CanvasNode) => void;
  }

  let { onSelectEntity }: Props = $props();

  let containerEl = $state<HTMLDivElement | null>(null);
  let isPanning = $state(false);
  let panStartX = 0;
  let panStartY = 0;

  let connectingFromId = $state<string | null>(null);
  let showAddEntityPicker = $state(false);

  // Cores de post-it
  const POSTIT_COLORS: Record<string, string> = {
    '1': '#FEF9C3', // Amarelo
    '2': '#DCFCE7', // Verde
    '3': '#DBEAFE', // Azul
    '4': '#F3E8FF', // Roxo
    '5': '#FEE2E2', // Vermelho
    '6': '#F1F5F9', // Cinza
  };

  onMount(() => {
    void canvasStore.init();
  });

  function handleContainerMouseDown(e: MouseEvent) {
    if (e.target === containerEl || (e.target as HTMLElement).classList.contains('canvas-infinite-plane')) {
      isPanning = true;
      panStartX = e.clientX - canvasStore.panX;
      panStartY = e.clientY - canvasStore.panY;
      canvasStore.selectedNodeId = null;
    }
  }

  function handleContainerMouseMove(e: MouseEvent) {
    if (isPanning) {
      canvasStore.panX = e.clientX - panStartX;
      canvasStore.panY = e.clientY - panStartY;
    }
  }

  function handleContainerMouseUp() {
    isPanning = false;
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const zoomFactor = e.deltaY < 0 ? 1.08 : 0.92;
    const newZoom = Math.max(0.3, Math.min(canvasStore.zoom * zoomFactor, 2.5));
    canvasStore.zoom = newZoom;
  }

  function handleDoubleClick(e: MouseEvent) {
    if (e.target === containerEl || (e.target as HTMLElement).classList.contains('canvas-infinite-plane')) {
      const rect = containerEl?.getBoundingClientRect();
      if (!rect) return;
      const clickX = (e.clientX - rect.left - canvasStore.panX) / canvasStore.zoom;
      const clickY = (e.clientY - rect.top - canvasStore.panY) / canvasStore.zoom;
      canvasStore.addTextNode(Math.round(clickX - 120), Math.round(clickY - 60));
    }
  }

  // Drag de nó individual
  let draggingNodeId: string | null = null;
  let dragOffset = { x: 0, y: 0 };

  function startDragNode(e: MouseEvent, node: CanvasNode) {
    e.stopPropagation();
    if (connectingFromId) {
      if (connectingFromId !== node.id) {
        canvasStore.connectNodes(connectingFromId, node.id);
      }
      connectingFromId = null;
      return;
    }

    draggingNodeId = node.id;
    canvasStore.selectedNodeId = node.id;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    dragOffset = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };

    const onMove = (ev: MouseEvent) => {
      if (draggingNodeId && containerEl) {
        const cRect = containerEl.getBoundingClientRect();
        const curX = (ev.clientX - cRect.left - canvasStore.panX) / canvasStore.zoom;
        const curY = (ev.clientY - cRect.top - canvasStore.panY) / canvasStore.zoom;
        canvasStore.updateNodePosition(draggingNodeId, curX - dragOffset.x / canvasStore.zoom, curY - dragOffset.y / canvasStore.zoom);
      }
    };

    const onUp = () => {
      draggingNodeId = null;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
      void canvasStore.saveNow();
    };

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  function handleInsertEntity(item: typeof manuscript.items[0]) {
    canvasStore.addEntityNode(
      item.id,
      item.title,
      item.kind,
      null,
      Math.round(-canvasStore.panX / canvasStore.zoom + 250),
      Math.round(-canvasStore.panY / canvasStore.zoom + 200)
    );
    showAddEntityPicker = false;
  }

  // Calcula linha SVG entre nós
  function getEdgeCoords(edge: CanvasEdge) {
    if (!canvasStore.currentCanvas) return null;
    const from = canvasStore.currentCanvas.nodes.find((n) => n.id === edge.fromNode);
    const to = canvasStore.currentCanvas.nodes.find((n) => n.id === edge.toNode);
    if (!from || !to) return null;

    const x1 = from.x + from.width / 2;
    const y1 = from.y + from.height / 2;
    const x2 = to.x + to.width / 2;
    const y2 = to.y + to.height / 2;
    const midX = (x1 + x2) / 2;
    const midY = (y1 + y2) / 2;

    return { x1, y1, x2, y2, midX, midY };
  }
</script>

<div
  class="canvas-wrapper"
  bind:this={containerEl}
  onmousedown={handleContainerMouseDown}
  onmousemove={handleContainerMouseMove}
  onmouseup={handleContainerMouseUp}
  onwheel={handleWheel}
  ondblclick={handleDoubleClick}
  role="region"
  aria-label="Quadro Infinito Canvas"
>
  <!-- Barra de Ferramentas Flutuante do Canvas -->
  <div class="canvas-floating-toolbar">
    <button
      type="button"
      class="toolbar-btn primary"
      onclick={() => canvasStore.addTextNode(300, 200)}
      title="Adicionar post-it"
    >
      📝 {t('canvas.addPostIt')}
    </button>

    <button
      type="button"
      class="toolbar-btn"
      onclick={() => (showAddEntityPicker = !showAddEntityPicker)}
      title="Inserir elemento do mundo"
    >
      🌍 {t('canvas.addEntityCard')}
    </button>

    <div class="toolbar-divider"></div>

    <button
      type="button"
      class="toolbar-btn icon-only"
      onclick={() => (canvasStore.zoom = Math.min(canvasStore.zoom * 1.15, 2.5))}
      title={t('canvas.zoomIn')}
    >
      ➕
    </button>

    <button
      type="button"
      class="toolbar-btn icon-only"
      onclick={() => (canvasStore.zoom = Math.max(canvasStore.zoom * 0.85, 0.3))}
      title={t('canvas.zoomOut')}
    >
      ➖
    </button>

    <button
      type="button"
      class="toolbar-btn"
      onclick={() => canvasStore.resetView()}
      title={t('canvas.fitView')}
    >
      🎯 {Math.round(canvasStore.zoom * 100)}%
    </button>

    <div class="toolbar-divider"></div>

    <button
      type="button"
      class="toolbar-btn"
      onclick={() => canvasStore.saveNow()}
      disabled={canvasStore.isSaving}
    >
      💾 {canvasStore.saveFeedback ? t('canvas.saved') : t('canvas.saveCanvas')}
    </button>
  </div>

  <!-- Dropdown para Escolha de Entidade a Inserir -->
  {#if showAddEntityPicker}
    <div class="entity-picker-dropdown">
      <div class="picker-header">
        <span class="picker-title">{t('canvas.selectEntityPlaceholder')}</span>
        <button type="button" class="btn-close-picker" onclick={() => (showAddEntityPicker = false)}>✕</button>
      </div>
      <div class="picker-list">
        {#each manuscript.items as it}
          <button type="button" class="picker-item" onclick={() => handleInsertEntity(it)}>
            <span class="item-kind-badge">{it.kind}</span>
            <span class="item-title">{it.title}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Mensagem de Conexão Ativa -->
  {#if connectingFromId}
    <div class="connecting-banner">
      <span>🔗 Clique no cartão de destino para criar a conexão direcionada...</span>
      <button type="button" class="btn-cancel-conn" onclick={() => (connectingFromId = null)}>Cancelar</button>
    </div>
  {/if}

  <!-- Plano Infinito Transformável -->
  <div
    class="canvas-infinite-plane"
    style="transform: translate({canvasStore.panX}px, {canvasStore.panY}px) scale({canvasStore.zoom});"
  >
    <!-- Camada SVG de Arestas e Setas -->
    <svg class="canvas-svg-layer" aria-hidden="true">
      <defs>
        <marker
          id="arrow-end"
          viewBox="0 0 10 10"
          refX="10"
          refY="5"
          markerWidth="6"
          markerHeight="6"
          orient="auto-start-reverse"
        >
          <path d="M 0 0 L 10 5 L 0 10 z" fill="#849864" />
        </marker>
      </defs>

      {#if canvasStore.currentCanvas}
        {#each canvasStore.currentCanvas.edges as edge}
          {@const coords = getEdgeCoords(edge)}
          {#if coords}
            <g class="canvas-edge-group">
              <line
                x1={coords.x1}
                y1={coords.y1}
                x2={coords.x2}
                y2={coords.y2}
                stroke="#849864"
                stroke-width="2"
                marker-end="url(#arrow-end)"
              />
              {#if edge.label}
                <text
                  x={coords.midX}
                  y={coords.midY - 8}
                  fill="#5E6F45"
                  font-size="11"
                  font-weight="600"
                  text-anchor="middle"
                  class="edge-label"
                >
                  {edge.label}
                </text>
              {/if}
            </g>
          {/if}
        {/each}
      {/if}
    </svg>

    <!-- Camada de Cartões HTML -->
    {#if canvasStore.currentCanvas}
      {#each canvasStore.currentCanvas.nodes as node (node.id)}
        {#if node.type === 'text'}
          <!-- Cartão Post-it Livre -->
          <div
            class="canvas-card postit-card"
            class:selected={canvasStore.selectedNodeId === node.id}
            style="
              left: {node.x}px;
              top: {node.y}px;
              width: {node.width}px;
              height: {node.height}px;
              background-color: {POSTIT_COLORS[node.color || '1'] || '#FEF9C3'};
            "
            onmousedown={(e) => startDragNode(e, node)}
            role="article"
          >
            <div class="card-top-bar">
              <span class="card-drag-handle">⠿</span>
              <div class="card-controls">
                <button
                  type="button"
                  class="card-action-btn"
                  onclick={() => (connectingFromId = node.id)}
                  title="Conectar a outro nó"
                >
                  🔗
                </button>
                <button
                  type="button"
                  class="card-action-btn"
                  onclick={() => canvasStore.deleteNode(node.id)}
                  title={t('canvas.deleteNode')}
                >
                  ✕
                </button>
              </div>
            </div>

            <textarea
              class="postit-textarea"
              bind:value={node.text}
              oninput={() => canvasStore.saveNow()}
              placeholder="Digite aqui sua ideia..."
            ></textarea>
          </div>
        {:else if node.type === 'file'}
          <!-- Cartão de Entidade do Mundo / Cena -->
          <div
            class="canvas-card entity-card"
            class:selected={canvasStore.selectedNodeId === node.id}
            style="
              left: {node.x}px;
              top: {node.y}px;
              width: {node.width}px;
              height: {node.height}px;
            "
            onmousedown={(e) => startDragNode(e, node)}
            role="article"
          >
            <div class="card-top-bar">
              <span class="entity-kind-badge {node.entity_kind?.toLowerCase()}">
                {node.entity_kind || 'Item'}
              </span>
              <div class="card-controls">
                <button
                  type="button"
                  class="card-action-btn"
                  onclick={() => (connectingFromId = node.id)}
                  title="Conectar a outro nó"
                >
                  🔗
                </button>
                <button
                  type="button"
                  class="card-action-btn"
                  onclick={() => canvasStore.deleteNode(node.id)}
                  title={t('canvas.deleteNode')}
                >
                  ✕
                </button>
              </div>
            </div>

            <div class="entity-card-body">
              <strong class="entity-card-title">{node.label || 'Sem título'}</strong>
              {#if node.entity_summary}
                <p class="entity-card-summary">{node.entity_summary}</p>
              {/if}
            </div>

            <button
              type="button"
              class="btn-inspect-entity"
              onclick={() => onSelectEntity?.(node)}
            >
              📄 {t('canvas.viewDetails')}
            </button>
          </div>
        {/if}
      {/each}
    {/if}
  </div>
</div>

<style>
  .canvas-wrapper {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    background-color: var(--color-paper-base, #FDFCFA);
    background-image: radial-gradient(var(--color-paper-border, #E5E0D8) 1px, transparent 1px);
    background-size: 20px 20px;
    cursor: default;
    user-select: none;
  }

  .canvas-floating-toolbar {
    position: absolute;
    top: 14px;
    left: 14px;
    z-index: 30;
    display: flex;
    align-items: center;
    gap: 8px;
    background-color: rgba(255, 255, 255, 0.9);
    backdrop-filter: blur(8px);
    padding: 6px 10px;
    border-radius: 8px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  }

  .toolbar-btn {
    padding: 6px 10px;
    border-radius: 6px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    background-color: var(--color-paper-surface, #F8F4EC);
    color: var(--color-ink-charcoal, #1E231D);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .toolbar-btn:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
  }

  .toolbar-btn.primary {
    background-color: var(--color-olive, #5E6F45);
    color: #FFFFFF;
    border-color: transparent;
  }

  .toolbar-btn.primary:hover {
    opacity: 0.9;
  }

  .toolbar-btn.icon-only {
    padding: 6px 8px;
  }

  .toolbar-divider {
    width: 1px;
    height: 18px;
    background-color: var(--color-paper-border, #E5E0D8);
    margin: 0 2px;
  }

  .connecting-banner {
    position: absolute;
    top: 64px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 30;
    background-color: var(--color-olive, #5E6F45);
    color: #FFFFFF;
    padding: 8px 16px;
    border-radius: 9999px;
    font-size: 12px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .btn-cancel-conn {
    background: rgba(0, 0, 0, 0.2);
    border: none;
    color: #FFFFFF;
    padding: 2px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
  }

  .entity-picker-dropdown {
    position: absolute;
    top: 56px;
    left: 14px;
    z-index: 35;
    width: 260px;
    max-height: 320px;
    background-color: var(--color-paper-elevated, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .picker-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    font-size: 11px;
    font-weight: 700;
  }

  .btn-close-picker {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-ink-muted, #726E65);
  }

  .picker-list {
    overflow-y: auto;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .picker-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 4px;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    font-size: 12px;
  }

  .picker-item:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
  }

  .item-kind-badge {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 1px 6px;
    border-radius: 4px;
    background-color: var(--color-paper-surface, #F8F4EC);
  }

  .canvas-infinite-plane {
    position: absolute;
    inset: 0;
    transform-origin: 0 0;
    pointer-events: all;
  }

  .canvas-svg-layer {
    position: absolute;
    top: -5000px;
    left: -5000px;
    width: 10000px;
    height: 10000px;
    pointer-events: none;
  }

  .canvas-card {
    position: absolute;
    border-radius: 8px;
    border: 1px solid rgba(0, 0, 0, 0.12);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    cursor: move;
  }

  .canvas-card.selected {
    border-color: var(--color-olive, #5E6F45);
    box-shadow: 0 0 0 2px var(--color-olive, #5E6F45), 0 8px 18px rgba(0, 0, 0, 0.12);
  }

  .card-top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  }

  .card-drag-handle {
    font-size: 12px;
    opacity: 0.4;
  }

  .card-controls {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .card-action-btn {
    background: none;
    border: none;
    font-size: 12px;
    color: var(--color-ink-muted, #726E65);
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
  }

  .card-action-btn:hover {
    background-color: rgba(0, 0, 0, 0.08);
  }

  .postit-textarea {
    flex: 1;
    border: none;
    background: transparent;
    padding: 8px;
    font-family: var(--font-literary, Georgia, serif);
    font-size: 13px;
    line-height: 1.4;
    color: #1E231D;
    resize: none;
    outline: none;
  }

  .entity-card {
    background-color: var(--color-paper-elevated, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E0D8);
  }

  .entity-kind-badge {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 4px;
    background-color: var(--color-paper-surface, #F8F4EC);
    color: var(--color-ink-muted, #726E65);
  }

  .entity-card-body {
    padding: 10px;
    flex: 1;
    overflow: hidden;
  }

  .entity-card-title {
    display: block;
    font-size: 14px;
    color: var(--color-ink-charcoal, #1E231D);
    margin-bottom: 4px;
  }

  .entity-card-summary {
    margin: 0;
    font-size: 11px;
    color: var(--color-ink-muted, #726E65);
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .btn-inspect-entity {
    width: 100%;
    border: none;
    border-top: 1px solid var(--color-paper-border, #E5E0D8);
    background-color: var(--color-paper-surface, #F8F4EC);
    color: var(--color-olive, #5E6F45);
    font-size: 11px;
    font-weight: 600;
    padding: 6px;
    cursor: pointer;
    text-align: center;
  }

  .btn-inspect-entity:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
  }
</style>
