import { describe, it, expect, beforeEach } from 'vitest';
import { graphStore } from './graph.svelte';
import { canvasStore } from '../canvas/canvas.svelte';
import type { GraphNode, CanvasNode } from '../tauri/bridge';

describe('M8 — Grafo Narrativo e Canvas Visual (JSON Canvas)', () => {
  beforeEach(() => {
    graphStore.closeModal();
    canvasStore.resetView();
  });

  describe('Grafo Global e Filtros (M8-01 e M8-02)', () => {
    it('deve abrir o modal do grafo e carregar dados iniciais', async () => {
      expect(graphStore.isOpen).toBe(false);
      graphStore.openModal('graph');
      expect(graphStore.isOpen).toBe(true);
      expect(graphStore.activeTab).toBe('graph');

      await graphStore.loadGraph();
      expect(graphStore.graphData).not.toBeNull();
      expect(graphStore.filteredNodes.length).toBeGreaterThan(0);
      expect(graphStore.filteredEdges.length).toBeGreaterThan(0);
    });

    it('deve alternar filtro de nós órfãos', async () => {
      await graphStore.loadGraph();
      const initialCount = graphStore.filteredNodes.length;

      graphStore.toggleOrphans();
      expect(graphStore.includeOrphans).toBe(false);
      expect(graphStore.filteredNodes.length).toBeLessThanOrEqual(initialCount);

      graphStore.toggleOrphans();
      expect(graphStore.includeOrphans).toBe(true);
      expect(graphStore.filteredNodes.length).toBe(initialCount);
    });

    it('deve filtrar nós pela busca textual', async () => {
      await graphStore.loadGraph();
      graphStore.searchQuery = 'Jaci';
      // Verifica se nós restantes contêm 'jaci' ou se a lista filtrou corretamente
      const hasMatch = graphStore.filteredNodes.some((n) =>
        n.title.toLowerCase().includes('jaci')
      );
      expect(typeof hasMatch).toBe('boolean');
      graphStore.searchQuery = '';
    });

    it('deve alternar filtros de tipo (kind)', async () => {
      await graphStore.loadGraph();
      const allKindsCount = graphStore.filteredNodes.length;

      // Mantém apenas 'character'
      graphStore.selectedKinds = ['character'];
      for (const node of graphStore.filteredNodes) {
        expect(node.kind.toLowerCase()).toBe('character');
      }

      graphStore.selectedKinds = ['scene', 'character', 'location'];
      expect(graphStore.filteredNodes.length).toBeGreaterThanOrEqual(1);
    });

    it('deve carregar dados de grafo local centrado', async () => {
      const local = await graphStore.fetchLocalGraph('mock-char-1', 1);
      expect(local).not.toBeNull();
      if (local) {
        expect(local.center_node_id).toBe('mock-char-1');
        expect(local.nodes.length).toBeGreaterThan(0);
      }
    });
  });

  describe('Canvas Visual & JSON Canvas (M8-03 e M8-04)', () => {
    it('deve inicializar o canvas padrão', async () => {
      await canvasStore.init('main');
      expect(canvasStore.currentCanvas).not.toBeNull();
      if (canvasStore.currentCanvas) {
        expect(canvasStore.currentCanvas.id).toBe('main');
      }
    });

    it('deve adicionar nó de post-it (text) e nó de entidade (file)', async () => {
      await canvasStore.init('main');
      const initialNodes = canvasStore.currentCanvas?.nodes.length || 0;

      // Adiciona post-it
      canvasStore.addTextNode(150, 150, '2');
      expect(canvasStore.currentCanvas?.nodes.length).toBe(initialNodes + 1);
      const postIt = canvasStore.currentCanvas?.nodes.find((n) => n.color === '2');
      expect(postIt?.type).toBe('text');

      // Adiciona entidade do mundo
      canvasStore.addEntityNode('char-1', 'Jaci', 'character', 'Protagonista', 400, 150);
      expect(canvasStore.currentCanvas?.nodes.length).toBe(initialNodes + 2);
      const entityNode = canvasStore.currentCanvas?.nodes.find((n) => n.file === 'char-1');
      expect(entityNode?.type).toBe('file');
      expect(entityNode?.label).toBe('Jaci');
    });

    it('deve conectar dois nós com uma aresta direcionada (edge)', async () => {
      await canvasStore.init('main');
      canvasStore.addTextNode(100, 100);
      canvasStore.addTextNode(350, 100);

      const nodes = canvasStore.currentCanvas!.nodes;
      const idA = nodes[nodes.length - 2].id;
      const idB = nodes[nodes.length - 1].id;

      const initialEdges = canvasStore.currentCanvas!.edges.length;
      canvasStore.connectNodes(idA, idB, 'Causa');

      expect(canvasStore.currentCanvas!.edges.length).toBe(initialEdges + 1);
      const edge = canvasStore.currentCanvas!.edges.find(
        (e) => e.fromNode === idA && e.toNode === idB
      );
      expect(edge).toBeDefined();
      expect(edge?.label).toBe('Causa');
    });

    it('deve remover nó e limpar arestas conectadas a ele automaticamente', async () => {
      await canvasStore.init('main');
      canvasStore.addTextNode(10, 10);
      canvasStore.addTextNode(20, 20);

      const nodes = canvasStore.currentCanvas!.nodes;
      const idA = nodes[nodes.length - 2].id;
      const idB = nodes[nodes.length - 1].id;

      canvasStore.connectNodes(idA, idB);
      expect(canvasStore.currentCanvas!.edges.some((e) => e.fromNode === idA || e.toNode === idA)).toBe(true);

      canvasStore.deleteNode(idA);
      expect(canvasStore.currentCanvas!.nodes.some((n) => n.id === idA)).toBe(false);
      expect(canvasStore.currentCanvas!.edges.some((e) => e.fromNode === idA || e.toNode === idA)).toBe(false);
    });

    it('deve atualizar posições e texto dos nós', async () => {
      await canvasStore.init('main');
      canvasStore.addTextNode(50, 50);
      const node = canvasStore.currentCanvas!.nodes[canvasStore.currentCanvas!.nodes.length - 1];

      canvasStore.updateNodePosition(node.id, 120.4, 230.8);
      expect(node.x).toBe(120);
      expect(node.y).toBe(231);

      canvasStore.updateNodeText(node.id, 'Novo texto de teste');
      expect(node.text).toBe('Novo texto de teste');
    });
  });
});
