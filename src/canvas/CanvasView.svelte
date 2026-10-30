<script lang="ts">
  /**
   * Componente de Quadro Infinito (Infinite Canvas) baseado no JSON Canvas 1.0 (PRD E10 / M8-03).
   * Suporta post-its com texto e imagens, cartões de entidades vinculados ao manuscrito e conexões direcionadas.
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
  let imageInputEl = $state<HTMLInputElement | null>(null);
  let pendingImageNodeId = $state<string | null>(null);

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

  function handleWindowKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && connectingFromId) {
      connectingFromId = null;
      e.stopPropagation();
    }
  }

  function handleContainerMouseDown(e: MouseEvent) {
    if (
      e.target === containerEl ||
      (e.target as HTMLElement).classList.contains('canvas-infinite-plane')
    ) {
      if (connectingFromId) {
        connectingFromId = null;
      }
      isPanning = true;
      panStartX = e.clientX - canvasStore.panX;
      panStartY = e.clientY - canvasStore.panY;
      canvasStore.selectedNodeId = null;
      canvasStore.selectedEdgeId = null;
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
    if (
      e.target === containerEl ||
      (e.target as HTMLElement).classList.contains('canvas-infinite-plane')
    ) {
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

  function handleCardClick(e: MouseEvent, node: CanvasNode) {
    if (connectingFromId) {
      e.stopPropagation();
      if (connectingFromId !== node.id) {
        canvasStore.connectNodes(connectingFromId, node.id);
      }
      connectingFromId = null;
    }
  }

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
    canvasStore.selectedEdgeId = null;
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
        canvasStore.updateNodePosition(
          draggingNodeId,
          curX - dragOffset.x / canvasStore.zoom,
          curY - dragOffset.y / canvasStore.zoom
        );
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

  function handleInsertEntity(item: (typeof manuscript.items)[0]) {
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

  // --- Upload e Manipulação de Imagens nos Post-its ---
  function promptImageUpload(nodeId: string) {
    pendingImageNodeId = nodeId;
    if (imageInputEl) {
      imageInputEl.value = '';
      imageInputEl.click();
    }
  }

  function handleImageFileSelected(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file && pendingImageNodeId) {
      const reader = new FileReader();
      reader.onload = (ev) => {
        const dataUrl = ev.target?.result as string;
        if (dataUrl && pendingImageNodeId) {
          canvasStore.updateNodeImage(pendingImageNodeId, dataUrl);
          pendingImageNodeId = null;
        }
      };
      reader.readAsDataURL(file);
    }
  }

  function handleCardPaste(e: ClipboardEvent, nodeId: string) {
    const items = e.clipboardData?.items;
    if (!items) return;
    for (const item of items) {
      if (item.type.startsWith('image/')) {
        e.preventDefault();
        const file = item.getAsFile();
        if (file) {
          const reader = new FileReader();
          reader.onload = (ev) => {
            const dataUrl = ev.target?.result as string;
            if (dataUrl) {
              canvasStore.updateNodeImage(nodeId, dataUrl);
            }
          };
          reader.readAsDataURL(file);
        }
        break;
      }
    }
  }

  function handleCardDrop(e: DragEvent, nodeId: string) {
    e.preventDefault();
    e.stopPropagation();
    const file = e.dataTransfer?.files?.[0];
    if (file && file.type.startsWith('image/')) {
      const reader = new FileReader();
      reader.onload = (ev) => {
        const dataUrl = ev.target?.result as string;
        if (dataUrl) {
          canvasStore.updateNodeImage(nodeId, dataUrl);
        }
      };
      reader.readAsDataURL(file);
    }
  }

  function handleOpenEntityInEditor(node: CanvasNode) {
    if (node.file) {
      manuscript.selectScene(node.file);
    }
  }

  // --- Cálculo Matemático de Interseção de Bordas para Arestas SVG ---
  function getNodeBorderIntersection(source: CanvasNode, target: CanvasNode) {
    const scx = source.x + source.width / 2;
    const scy = source.y + source.height / 2;
    const tcx = target.x + target.width / 2;
    const tcy = target.y + target.height / 2;

    const dx = tcx - scx;
    const dy = tcy - scy;

    if (Math.abs(dx) < 0.001 && Math.abs(dy) < 0.001) {
      return { x: scx, y: scy };
    }

    // Compara inclinação do vetor com a diagonal do retângulo
    if (Math.abs(dx) * (source.height / 2) > Math.abs(dy) * (source.width / 2)) {
      // Cruza borda vertical esquerda ou direita
      if (dx > 0) {
        return {
          x: source.x + source.width,
          y: scy + (dy / dx) * (source.width / 2),
        };
      } else {
        return {
          x: source.x,
          y: scy - (dy / dx) * (source.width / 2),
        };
      }
    } else {
      // Cruza borda horizontal superior ou inferior
      if (dy > 0) {
        return {
          x: scx + (dx / dy) * (source.height / 2),
          y: source.y + source.height,
        };
      } else {
        return {
          x: scx - (dx / dy) * (source.height / 2),
          y: source.y,
        };
      }
    }
  }

  function getEdgeCoords(edge: CanvasEdge) {
    if (!canvasStore.currentCanvas) return null;
    const from = canvasStore.currentCanvas.nodes.find((n) => n.id === edge.fromNode);
    const to = canvasStore.currentCanvas.nodes.find((n) => n.id === edge.toNode);
    if (!from || !to) return null;

    const p1 = getNodeBorderIntersection(from, to);
    const p2 = getNodeBorderIntersection(to, from);

    // Recuo sutil na chegada para o marcador de ponta de seta tocar a borda perfeitamente
    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    const len = Math.hypot(dx, dy);
    let endX = p2.x;
    let endY = p2.y;
    if (len > 12) {
      const ux = dx / len;
      const uy = dy / len;
      endX = p2.x - ux * 4;
      endY = p2.y - uy * 4;
    }

    const midX = (p1.x + p2.x) / 2;
    const midY = (p1.y + p2.y) / 2;

    return { x1: p1.x, y1: p1.y, x2: endX, y2: endY, midX, midY };
  }

  function promptEditEdgeLabel(edge: CanvasEdge) {
    const current = edge.label || '';
    const newLabel = window.prompt(t('canvas.edgeLabelPrompt'), current);
    if (newLabel !== null) {
      canvasStore.updateEdgeLabel(edge.id, newLabel);
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeyDown} />

<!-- Input oculto para upload de imagens nos post-its -->
<input
  type="file"
  accept="image/*"
  class="sr-only"
  bind:this={imageInputEl}
  onchange={handleImageFileSelected}
/>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
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
        <button
          type="button"
          class="btn-close-picker"
          onclick={() => (showAddEntityPicker = false)}>✕</button
        >
      </div>
      <div class="picker-list">
        {#each manuscript.items as it}
          <button
            type="button"
            class="picker-item"
            onclick={() => handleInsertEntity(it)}
          >
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
      <span>🔗 {t('canvas.connectingInstruction')}</span>
      <button
        type="button"
        class="btn-cancel-conn"
        onclick={() => (connectingFromId = null)}
      >
        {t('canvas.cancelConnection')} (Esc)
      </button>
    </div>
  {/if}

  <!-- Plano Infinito Transformável -->
  <div
    class="canvas-infinite-plane"
    style="transform: translate({canvasStore.panX}px, {canvasStore.panY}px) scale({canvasStore.zoom});"
  >
    <!-- Camada SVG de Arestas e Setas (com overflow visible e coordenadas relativas ao plano) -->
    <svg class="canvas-svg-layer" aria-hidden="true">
      <defs>
        <marker
          id="canvas-arrow-end"
          viewBox="0 0 10 10"
          refX="9"
          refY="5"
          markerWidth="7"
          markerHeight="7"
          orient="auto-start-reverse"
        >
          <path d="M 0 1 L 9 5 L 0 9 z" fill="#5E6F45" />
        </marker>
        <marker
          id="canvas-arrow-selected"
          viewBox="0 0 10 10"
          refX="9"
          refY="5"
          markerWidth="7"
          markerHeight="7"
          orient="auto-start-reverse"
        >
          <path d="M 0 1 L 9 5 L 0 9 z" fill="#B91C1C" />
        </marker>
      </defs>

      {#if canvasStore.currentCanvas}
        {#each canvasStore.currentCanvas.edges as edge (edge.id)}
          {@const coords = getEdgeCoords(edge)}
          {#if coords}
            <g
              class="canvas-edge-group"
              class:selected={canvasStore.selectedEdgeId === edge.id}
            >
              <!-- Linha de toque invisível para facilitar seleção -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <line
                x1={coords.x1}
                y1={coords.y1}
                x2={coords.x2}
                y2={coords.y2}
                stroke="transparent"
                stroke-width="16"
                class="edge-hitbox"
                onclick={(e) => {
                  e.stopPropagation();
                  canvasStore.selectedEdgeId = edge.id;
                }}
              />

              <!-- Linha visível conectando os cartões -->
              <line
                x1={coords.x1}
                y1={coords.y1}
                x2={coords.x2}
                y2={coords.y2}
                stroke={canvasStore.selectedEdgeId === edge.id ? '#B91C1C' : '#5E6F45'}
                stroke-width={canvasStore.selectedEdgeId === edge.id ? '3' : '2'}
                stroke-linecap="round"
                stroke-dasharray={edge.fromEnd === 'dashed' ? '5,5' : 'none'}
                marker-end={canvasStore.selectedEdgeId === edge.id
                  ? 'url(#canvas-arrow-selected)'
                  : 'url(#canvas-arrow-end)'}
                class="edge-line"
              />

              <!-- Badge central com rótulo e botão de exclusão da conexão -->
              <foreignObject
                x={coords.midX - 70}
                y={coords.midY - 14}
                width="140"
                height="28"
                class="edge-foreign-obj"
              >
                <div
                  class="edge-badge"
                  class:selected={canvasStore.selectedEdgeId === edge.id}
                >
                  <button
                    type="button"
                    class="edge-label-btn"
                    title="Clique duas vezes para renomear"
                    ondblclick={(e) => {
                      e.stopPropagation();
                      promptEditEdgeLabel(edge);
                    }}
                    onclick={(e) => {
                      e.stopPropagation();
                      canvasStore.selectedEdgeId = edge.id;
                    }}
                  >
                    {edge.label || '🔗'}
                  </button>
                  <button
                    type="button"
                    class="edge-delete-btn"
                    title={t('canvas.deleteEdge')}
                    onclick={(e) => {
                      e.stopPropagation();
                      canvasStore.deleteEdge(edge.id);
                    }}
                  >
                    ✕
                  </button>
                </div>
              </foreignObject>
            </g>
          {/if}
        {/each}
      {/if}
    </svg>

    <!-- Camada de Cartões HTML -->
    {#if canvasStore.currentCanvas}
      {#each canvasStore.currentCanvas.nodes as node (node.id)}
        {#if node.type === 'text'}
          <!-- Cartão Post-it Livre com Suporte a Imagens -->
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="canvas-card postit-card"
            class:selected={canvasStore.selectedNodeId === node.id}
            class:connecting-source={connectingFromId === node.id}
            class:connect-target={connectingFromId && connectingFromId !== node.id}
            style="
              left: {node.x}px;
              top: {node.y}px;
              width: {node.width}px;
              min-height: {node.height}px;
              background-color: {POSTIT_COLORS[node.color || '1'] || '#FEF9C3'};
            "
            onmousedown={(e) => startDragNode(e, node)}
            onclick={(e) => handleCardClick(e, node)}
            ondragover={(e) => e.preventDefault()}
            ondrop={(e) => handleCardDrop(e, node.id)}
            role="article"
          >
            <!-- Barra Superior do Post-it -->
            <div class="card-top-bar">
              <span class="card-drag-handle" title="Arraste para reposicionar">⠿</span>
              <div class="card-controls">
                <button
                  type="button"
                  class="card-action-btn"
                  onclick={(e) => {
                    e.stopPropagation();
                    promptImageUpload(node.id);
                  }}
                  title={t('canvas.addImage')}
                >
                  🖼️
                </button>
                <button
                  type="button"
                  class="card-action-btn"
                  onclick={(e) => {
                    e.stopPropagation();
                    connectingFromId = node.id;
                  }}
                  title={t('canvas.connectCard')}
                >
                  🔗
                </button>
                <button
                  type="button"
                  class="card-action-btn"
                  onclick={(e) => {
                    e.stopPropagation();
                    canvasStore.deleteNode(node.id);
                  }}
                  title={t('canvas.deleteNode')}
                >
                  ✕
                </button>
              </div>
            </div>

            <!-- Imagem do Post-it (se houver) -->
            {#if node.image_url}
              <div class="postit-image-container">
                <img
                  src={node.image_url}
                  alt="Imagem da anotação"
                  class="postit-image"
                />
                <button
                  type="button"
                  class="btn-remove-postit-image"
                  onclick={(e) => {
                    e.stopPropagation();
                    canvasStore.updateNodeImage(node.id, null);
                  }}
                  title={t('canvas.removeImage')}
                >
                  ✕
                </button>
              </div>
            {/if}

            <!-- Área de Texto do Post-it -->
            <textarea
              class="postit-textarea"
              bind:value={node.text}
              oninput={() => canvasStore.saveNow()}
              onpaste={(e) => handleCardPaste(e, node.id)}
              placeholder="Digite sua ideia ou cole/solte uma imagem..."
            ></textarea>
          </div>
        {:else if node.type === 'file'}
          <!-- Cartão de Entidade do Mundo / Cena Interligado -->
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="canvas-card entity-card"
            class:selected={canvasStore.selectedNodeId === node.id}
            class:connecting-source={connectingFromId === node.id}
            class:connect-target={connectingFromId && connectingFromId !== node.id}
            style="
              left: {node.x}px;
              top: {node.y}px;
              width: {node.width}px;
              height: {node.height}px;
            "
            onmousedown={(e) => startDragNode(e, node)}
            onclick={(e) => handleCardClick(e, node)}
            ondblclick={(e) => {
              e.stopPropagation();
              handleOpenEntityInEditor(node);
            }}
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
                  onclick={(e) => {
                    e.stopPropagation();
                    connectingFromId = node.id;
                  }}
                  title={t('canvas.connectCard')}
                >
                  🔗
                </button>
                <button
                  type="button"
                  class="card-action-btn"
                  onclick={(e) => {
                    e.stopPropagation();
                    canvasStore.deleteNode(node.id);
                  }}
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

            <div class="entity-card-actions">
              <button
                type="button"
                class="btn-card-action primary"
                onclick={(e) => {
                  e.stopPropagation();
                  handleOpenEntityInEditor(node);
                }}
                title="Abrir no editor"
              >
                📖 {t('graph.openInEditor')}
              </button>
              <button
                type="button"
                class="btn-card-action"
                onclick={(e) => {
                  e.stopPropagation();
                  onSelectEntity?.(node);
                }}
                title="Inspecionar detalhes"
              >
                📄 {t('canvas.viewDetails')}
              </button>
            </div>
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
    background-color: rgba(255, 255, 255, 0.95);
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
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.2);
    animation: fadeInSlide 0.2s ease-out;
  }

  @keyframes fadeInSlide {
    from {
      opacity: 0;
      transform: translate(-50%, -8px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }

  .btn-cancel-conn {
    background: rgba(0, 0, 0, 0.25);
    border: none;
    color: #FFFFFF;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
    transition: background-color 0.15s ease;
  }

  .btn-cancel-conn:hover {
    background: rgba(0, 0, 0, 0.4);
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

  /* Camada SVG: perfeitamente alinhada com as coordenadas do plano infinito */
  .canvas-svg-layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    overflow: visible;
    pointer-events: none;
  }

  .canvas-edge-group {
    pointer-events: stroke;
  }

  .edge-hitbox {
    cursor: pointer;
  }

  .edge-line {
    transition: stroke 0.15s ease, stroke-width 0.15s ease;
  }

  .canvas-edge-group:hover .edge-line {
    stroke: #3F4D2E;
    stroke-width: 3;
  }

  .edge-foreign-obj {
    pointer-events: none;
  }

  .edge-badge {
    pointer-events: all;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    background-color: var(--color-paper-elevated, #FFFFFF);
    border: 1px solid var(--color-olive, #5E6F45);
    border-radius: 9999px;
    padding: 2px 6px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
    font-size: 10px;
    font-weight: 600;
    color: var(--color-olive, #5E6F45);
    max-width: 130px;
    margin: 0 auto;
  }

  .edge-badge.selected {
    border-color: #B91C1C;
    color: #B91C1C;
    box-shadow: 0 0 0 1px #B91C1C;
  }

  .edge-label-btn {
    background: none;
    border: none;
    font-size: 10px;
    font-weight: 600;
    color: inherit;
    cursor: pointer;
    padding: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 90px;
  }

  .edge-delete-btn {
    background: none;
    border: none;
    font-size: 9px;
    color: var(--color-ink-muted, #726E65);
    cursor: pointer;
    padding: 0 2px;
    border-radius: 2px;
  }

  .edge-delete-btn:hover {
    color: #B91C1C;
    background-color: rgba(185, 28, 28, 0.1);
  }

  .canvas-card {
    position: absolute;
    border-radius: 8px;
    border: 1px solid rgba(0, 0, 0, 0.14);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    cursor: move;
    transition: box-shadow 0.15s ease, border-color 0.15s ease;
  }

  .canvas-card.selected {
    border-color: var(--color-olive, #5E6F45);
    box-shadow: 0 0 0 2px var(--color-olive, #5E6F45), 0 8px 18px rgba(0, 0, 0, 0.14);
  }

  .canvas-card.connecting-source {
    border-color: var(--color-olive, #5E6F45);
    box-shadow: 0 0 0 3px var(--color-olive, #5E6F45), 0 0 16px rgba(94, 111, 69, 0.35);
  }

  .canvas-card.connect-target {
    cursor: crosshair;
    border-color: var(--color-olive, #5E6F45);
    border-style: dashed;
    box-shadow: 0 0 0 2px rgba(94, 111, 69, 0.5);
  }

  .card-top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  }

  .card-drag-handle {
    font-size: 12px;
    opacity: 0.5;
    cursor: grab;
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
    transition: background-color 0.12s ease;
  }

  .card-action-btn:hover {
    background-color: rgba(0, 0, 0, 0.08);
  }

  /* Post-it com suporte a imagem */
  .postit-image-container {
    position: relative;
    width: 100%;
    max-height: 140px;
    overflow: hidden;
    background-color: rgba(0, 0, 0, 0.04);
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  }

  .postit-image {
    width: 100%;
    height: 120px;
    object-fit: cover;
    display: block;
  }

  .btn-remove-postit-image {
    position: absolute;
    top: 4px;
    right: 4px;
    background-color: rgba(0, 0, 0, 0.6);
    color: #FFFFFF;
    border: none;
    border-radius: 50%;
    width: 20px;
    height: 20px;
    font-size: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background-color 0.12s ease;
  }

  .btn-remove-postit-image:hover {
    background-color: rgba(185, 28, 28, 0.9);
  }

  .postit-textarea {
    flex: 1;
    min-height: 80px;
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

  .entity-card-actions {
    display: flex;
    border-top: 1px solid var(--color-paper-border, #E5E0D8);
    background-color: var(--color-paper-surface, #F8F4EC);
  }

  .btn-card-action {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--color-ink-charcoal, #1E231D);
    font-size: 11px;
    font-weight: 600;
    padding: 6px;
    cursor: pointer;
    text-align: center;
    transition: background-color 0.12s ease;
  }

  .btn-card-action:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
  }

  .btn-card-action.primary {
    color: var(--color-olive, #5E6F45);
    border-right: 1px solid var(--color-paper-border, #E5E0D8);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
