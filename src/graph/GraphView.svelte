<script lang="ts">
  /**
   * Componente Canvas 2D interativo com simulação de forças para o Grafo Narrativo (M8-01).
   */

  import { onMount } from 'svelte';
  import { graphStore } from './graph.svelte';
  import type { GraphNode } from '../tauri/bridge';

  interface Props {
    onSelectNode?: (node: GraphNode) => void;
  }

  let { onSelectNode }: Props = $props();

  let canvasEl = $state<HTMLCanvasElement | null>(null);

  // Nós com posições na simulação física 2D
  interface SimNode extends GraphNode {
    x: number;
    y: number;
    vx: number;
    vy: number;
    radius: number;
  }

  let simNodes = $state<SimNode[]>([]);
  let hoveredNodeId = $state<string | null>(null);
  let draggedNode = $state<SimNode | null>(null);

  // Câmera do grafo
  let cameraX = $state(0);
  let cameraY = $state(0);
  let cameraZoom = $state(1);
  let isPanning = $state(false);
  let panStartX = 0;
  let panStartY = 0;

  const KIND_COLORS: Record<string, string> = {
    scene: '#3B82F6', // Azul
    character: '#D97706', // Âmbar
    location: '#059669', // Esmeralda
    item: '#7C3AED', // Roxo
    faction: '#DC2626', // Carmesim
    lore: '#5E6F45', // Oliva
    note: '#6B7280', // Ardósia
  };

  function getNodeColor(kind: string): string {
    return KIND_COLORS[kind.toLowerCase()] || '#5E6F45';
  }

  // Inicializa posições iniciais dos nós
  $effect(() => {
    const rawNodes = graphStore.filteredNodes;
    if (rawNodes.length === 0) {
      simNodes = [];
      return;
    }

    const existingMap = new Map(simNodes.map((n) => [n.id, n]));
    const width = canvasEl?.width || 800;
    const height = canvasEl?.height || 600;

    simNodes = rawNodes.map((rn, idx) => {
      const existing = existingMap.get(rn.id);
      if (existing) {
        return {
          ...rn,
          x: existing.x,
          y: existing.y,
          vx: existing.vx,
          vy: existing.vy,
          radius: 8 + Math.min(rn.connections_count * 2, 16),
        };
      }

      // Distribuição circular inicial
      const angle = (idx / rawNodes.length) * 2 * Math.PI;
      const radiusDist = 120 + Math.random() * 150;
      return {
        ...rn,
        x: width / 2 + Math.cos(angle) * radiusDist,
        y: height / 2 + Math.sin(angle) * radiusDist,
        vx: 0,
        vy: 0,
        radius: 8 + Math.min(rn.connections_count * 2, 16),
      };
    });
  });

  // Loop da simulação física e renderização no Canvas
  onMount(() => {
    let animId: number;

    const handleResize = () => {
      if (canvasEl && canvasEl.parentElement) {
        canvasEl.width = canvasEl.parentElement.clientWidth;
        canvasEl.height = canvasEl.parentElement.clientHeight;
        if (cameraX === 0 && cameraY === 0) {
          cameraX = canvasEl.width / 2;
          cameraY = canvasEl.height / 2;
        }
      }
    };

    handleResize();
    window.addEventListener('resize', handleResize);

    const step = () => {
      updatePhysics();
      renderCanvas();
      animId = requestAnimationFrame(step);
    };

    animId = requestAnimationFrame(step);

    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener('resize', handleResize);
    };
  });

  function updatePhysics() {
    if (simNodes.length === 0) return;

    const nodeMap = new Map(simNodes.map((n) => [n.id, n]));
    const edges = graphStore.filteredEdges;
    const centerX = canvasEl?.width ? canvasEl.width / 2 : 400;
    const centerY = canvasEl?.height ? canvasEl.height / 2 : 300;

    // 1. Repulsão entre nós (Coulomb)
    for (let i = 0; i < simNodes.length; i++) {
      for (let j = i + 1; j < simNodes.length; j++) {
        const a = simNodes[i];
        const b = simNodes[j];
        const dx = b.x - a.x;
        const dy = b.y - a.y;
        const distSq = dx * dx + dy * dy || 1;
        const dist = Math.sqrt(distSq);

        if (dist < 300) {
          const force = 1200 / distSq;
          const fx = (dx / dist) * force;
          const fy = (dy / dist) * force;

          if (a !== draggedNode) {
            a.vx -= fx;
            a.vy -= fy;
          }
          if (b !== draggedNode) {
            b.vx += fx;
            b.vy += fy;
          }
        }
      }
    }

    // 2. Atração elástica das arestas (Hooke)
    const targetDist = 90;
    for (const edge of edges) {
      const source = nodeMap.get(edge.source);
      const target = nodeMap.get(edge.target);
      if (source && target) {
        const dx = target.x - source.x;
        const dy = target.y - source.y;
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const force = (dist - targetDist) * 0.04;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;

        if (source !== draggedNode) {
          source.vx += fx;
          source.vy += fy;
        }
        if (target !== draggedNode) {
          target.vx -= fx;
          target.vy -= fy;
        }
      }
    }

    // 3. Gravidade central suave & amortecimento
    for (const node of simNodes) {
      if (node === draggedNode) continue;

      const toCenterX = centerX - node.x;
      const toCenterY = centerY - node.y;
      node.vx += toCenterX * 0.005;
      node.vy += toCenterY * 0.005;

      // Amortecimento de velocidade
      node.vx *= 0.88;
      node.vy *= 0.88;

      node.x += node.vx;
      node.y += node.vy;
    }
  }

  function renderCanvas() {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    const width = canvasEl.width;
    const height = canvasEl.height;

    ctx.clearRect(0, 0, width, height);

    ctx.save();
    ctx.translate(cameraX, cameraY);
    ctx.scale(cameraZoom, cameraZoom);
    ctx.translate(-width / 2, -height / 2);

    const edges = graphStore.filteredEdges;
    const nodeMap = new Map(simNodes.map((n) => [n.id, n]));

    // Nós conectados ao hovered para realce
    const hoveredConnected = new Set<string>();
    if (hoveredNodeId) {
      hoveredConnected.add(hoveredNodeId);
      for (const e of edges) {
        if (e.source === hoveredNodeId) hoveredConnected.add(e.target);
        if (e.target === hoveredNodeId) hoveredConnected.add(e.source);
      }
    }

    // 1. Desenha arestas
    for (const edge of edges) {
      const source = nodeMap.get(edge.source);
      const target = nodeMap.get(edge.target);
      if (source && target) {
        const isHighlighted =
          !hoveredNodeId ||
          (hoveredConnected.has(source.id) && hoveredConnected.has(target.id));

        ctx.beginPath();
        ctx.moveTo(source.x, source.y);
        ctx.lineTo(target.x, target.y);
        ctx.strokeStyle = isHighlighted
          ? 'rgba(150, 150, 150, 0.45)'
          : 'rgba(150, 150, 150, 0.1)';
        ctx.lineWidth = isHighlighted ? 1.5 : 1;
        ctx.stroke();
      }
    }

    // 2. Desenha nós
    for (const node of simNodes) {
      const isHighlighted = !hoveredNodeId || hoveredConnected.has(node.id);
      const isSelected = graphStore.selectedNode?.id === node.id;
      const alpha = isHighlighted ? 1 : 0.25;

      ctx.save();
      ctx.globalAlpha = alpha;

      // Círculo base
      ctx.beginPath();
      ctx.arc(node.x, node.y, node.radius, 0, 2 * Math.PI);
      ctx.fillStyle = getNodeColor(node.kind);
      ctx.fill();

      // Borda de seleção
      if (isSelected || node.id === hoveredNodeId) {
        ctx.lineWidth = isSelected ? 3 : 2;
        ctx.strokeStyle = '#FFFFFF';
        ctx.stroke();
      }

      // Rótulo textual
      ctx.font = '11px sans-serif';
      ctx.fillStyle = isHighlighted ? 'var(--color-ink-charcoal, #1E231D)' : '#999999';
      ctx.textAlign = 'center';
      ctx.fillText(node.title, node.x, node.y + node.radius + 14);

      ctx.restore();
    }

    ctx.restore();
  }

  function screenToWorld(sx: number, sy: number) {
    if (!canvasEl) return { x: 0, y: 0 };
    const width = canvasEl.width;
    const height = canvasEl.height;
    const x = (sx - cameraX) / cameraZoom + width / 2;
    const y = (sy - cameraY) / cameraZoom + height / 2;
    return { x, y };
  }

  function handleMouseDown(e: MouseEvent) {
    if (e.button === 1 || e.shiftKey || e.altKey) {
      isPanning = true;
      panStartX = e.clientX - cameraX;
      panStartY = e.clientY - cameraY;
      return;
    }

    const { x, y } = screenToWorld(e.offsetX, e.offsetY);
    for (const node of simNodes) {
      const dist = Math.hypot(node.x - x, node.y - y);
      if (dist <= node.radius + 4) {
        draggedNode = node;
        graphStore.selectNode(node);
        onSelectNode?.(node);
        return;
      }
    }

    // Clicou no fundo: inicia pan
    isPanning = true;
    panStartX = e.clientX - cameraX;
    panStartY = e.clientY - cameraY;
  }

  function handleMouseMove(e: MouseEvent) {
    if (isPanning) {
      cameraX = e.clientX - panStartX;
      cameraY = e.clientY - panStartY;
      return;
    }

    const { x, y } = screenToWorld(e.offsetX, e.offsetY);

    if (draggedNode) {
      draggedNode.x = x;
      draggedNode.y = y;
      draggedNode.vx = 0;
      draggedNode.vy = 0;
      return;
    }

    // Hover detection
    let found: string | null = null;
    for (const node of simNodes) {
      const dist = Math.hypot(node.x - x, node.y - y);
      if (dist <= node.radius + 4) {
        found = node.id;
        break;
      }
    }
    hoveredNodeId = found;
  }

  function handleMouseUp() {
    draggedNode = null;
    isPanning = false;
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
    const newZoom = Math.max(0.2, Math.min(cameraZoom * zoomFactor, 3.5));
    cameraZoom = newZoom;
  }

  export function resetCamera() {
    if (canvasEl) {
      cameraX = canvasEl.width / 2;
      cameraY = canvasEl.height / 2;
      cameraZoom = 1;
    }
  }
</script>

<div class="graph-canvas-container">
  <canvas
    bind:this={canvasEl}
    onmousedown={handleMouseDown}
    onmousemove={handleMouseMove}
    onmouseup={handleMouseUp}
    onwheel={handleWheel}
  ></canvas>

  <div class="canvas-hint-overlay">
    <span>💡 Dica: Role para Zoom • Arraste o fundo para mover • Arraste nós para reorganizar</span>
  </div>
</div>

<style>
  .graph-canvas-container {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    background-color: var(--color-paper-base, #FDFCFA);
    cursor: grab;
  }

  .graph-canvas-container:active {
    cursor: grabbing;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .canvas-hint-overlay {
    position: absolute;
    bottom: 12px;
    left: 12px;
    pointer-events: none;
    font-size: 11px;
    color: var(--color-ink-muted, #726E65);
    background-color: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(4px);
    padding: 4px 8px;
    border-radius: 4px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
  }
</style>
