/**
 * Store reativo em Svelte 5 para o Canvas Visual e Mapas Mentais (PRD E10 / D4 / M8-03).
 */

import {
  loadCanvas,
  saveCanvas,
  listCanvases,
  type CanvasDocument,
  type CanvasNode,
  type CanvasEdge,
  type CanvasSummary,
} from '../tauri/bridge';

class CanvasStore {
  currentCanvas = $state<CanvasDocument | null>(null);
  canvases = $state<CanvasSummary[]>([]);
  isLoading = $state(false);
  isSaving = $state(false);
  saveFeedback = $state(false);
  selectedNodeId = $state<string | null>(null);
  selectedEdgeId = $state<string | null>(null);

  // Câmera do canvas infinito
  panX = $state(0);
  panY = $state(0);
  zoom = $state(1);

  async init(canvasId: string = 'main') {
    this.isLoading = true;
    try {
      this.canvases = await listCanvases();
      this.currentCanvas = await loadCanvas(canvasId);
    } catch (err) {
      console.error('Falha ao inicializar Canvas:', err);
    } finally {
      this.isLoading = false;
    }
  }

  async saveNow() {
    if (!this.currentCanvas) return;
    this.isSaving = true;
    try {
      await saveCanvas(this.currentCanvas);
      this.saveFeedback = true;
      setTimeout(() => {
        this.saveFeedback = false;
      }, 2000);
      this.canvases = await listCanvases();
    } catch (err) {
      console.error('Falha ao salvar Canvas:', err);
    } finally {
      this.isSaving = false;
    }
  }

  addTextNode(x: number = 200, y: number = 200, color: string = '1') {
    if (!this.currentCanvas) return;
    const id = `node-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const newNode: CanvasNode = {
      id,
      type: 'text',
      x,
      y,
      width: 240,
      height: 140,
      color,
      text: 'Nova anotação...',
    };
    this.currentCanvas.nodes = [...this.currentCanvas.nodes, newNode];
    this.selectedNodeId = id;
    void this.saveNow();
  }

  addEntityNode(
    entityId: string,
    title: string,
    kind: string,
    summary?: string | null,
    x: number = 300,
    y: number = 250
  ) {
    if (!this.currentCanvas) return;
    const id = `entity-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const newNode: CanvasNode = {
      id,
      type: 'file',
      x,
      y,
      width: 260,
      height: 130,
      color: '#5E6F45',
      label: title,
      file: entityId,
      entity_kind: kind,
      entity_summary: summary || null,
    };
    this.currentCanvas.nodes = [...this.currentCanvas.nodes, newNode];
    this.selectedNodeId = id;
    void this.saveNow();
  }

  updateNodePosition(nodeId: string, x: number, y: number) {
    if (!this.currentCanvas) return;
    const node = this.currentCanvas.nodes.find((n) => n.id === nodeId);
    if (node) {
      node.x = Math.round(x);
      node.y = Math.round(y);
    }
  }

  updateNodeText(nodeId: string, text: string) {
    if (!this.currentCanvas) return;
    const node = this.currentCanvas.nodes.find((n) => n.id === nodeId);
    if (node) {
      node.text = text;
    }
  }

  updateNodeColor(nodeId: string, color: string) {
    if (!this.currentCanvas) return;
    const node = this.currentCanvas.nodes.find((n) => n.id === nodeId);
    if (node) {
      node.color = color;
      void this.saveNow();
    }
  }

  updateNodeImage(nodeId: string, imageUrl: string | null) {
    if (!this.currentCanvas) return;
    const node = this.currentCanvas.nodes.find((n) => n.id === nodeId);
    if (node) {
      node.image_url = imageUrl;
      if (imageUrl && node.height < 240) {
        node.height = 240;
      }
      void this.saveNow();
    }
  }

  deleteNode(nodeId: string) {
    if (!this.currentCanvas) return;
    this.currentCanvas.nodes = this.currentCanvas.nodes.filter(
      (n) => n.id !== nodeId
    );
    // Remove também arestas conectadas a este nó
    this.currentCanvas.edges = this.currentCanvas.edges.filter(
      (e) => e.fromNode !== nodeId && e.toNode !== nodeId
    );
    if (this.selectedNodeId === nodeId) {
      this.selectedNodeId = null;
    }
    void this.saveNow();
  }

  connectNodes(fromNode: string, toNode: string, label?: string) {
    if (!this.currentCanvas || fromNode === toNode) return;
    const exists = this.currentCanvas.edges.some(
      (e) =>
        (e.fromNode === fromNode && e.toNode === toNode) ||
        (e.fromNode === toNode && e.toNode === fromNode)
    );
    if (exists) return;

    const edge: CanvasEdge = {
      id: `edge-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      fromNode,
      toNode,
      fromEnd: 'none',
      toEnd: 'arrow',
      label: label || undefined,
    };
    this.currentCanvas.edges = [...this.currentCanvas.edges, edge];
    void this.saveNow();
  }

  deleteEdge(edgeId: string) {
    if (!this.currentCanvas) return;
    this.currentCanvas.edges = this.currentCanvas.edges.filter(
      (e) => e.id !== edgeId
    );
    if (this.selectedEdgeId === edgeId) {
      this.selectedEdgeId = null;
    }
    void this.saveNow();
  }

  updateEdgeLabel(edgeId: string, label?: string) {
    if (!this.currentCanvas) return;
    const edge = this.currentCanvas.edges.find((e) => e.id === edgeId);
    if (edge) {
      edge.label = label?.trim() || undefined;
      void this.saveNow();
    }
  }

  resetView() {
    this.panX = 0;
    this.panY = 0;
    this.zoom = 1;
  }
}

export const canvasStore = new CanvasStore();
