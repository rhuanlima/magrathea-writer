/**
 * Store reativo em Svelte 5 para o Grafo Narrativo e Visualização de Conexões (M8-01 e M8-02).
 */

import {
  getVaultGraph,
  getLocalGraph,
  type GraphData,
  type GraphNode,
  type LocalGraphData,
} from '../tauri/bridge';

class GraphStore {
  isOpen = $state(false);
  activeTab = $state<'graph' | 'canvas'>('graph');
  isLoading = $state(false);
  graphData = $state<GraphData | null>(null);
  selectedNode = $state<GraphNode | null>(null);
  searchQuery = $state('');
  includeOrphans = $state(true);
  selectedKinds = $state<string[]>([
    'scene',
    'character',
    'location',
    'item',
    'faction',
    'lore',
    'note',
  ]);
  error = $state<string | null>(null);

  // Nós filtrados reativos
  filteredNodes = $derived.by(() => {
    if (!this.graphData) return [];
    let list = this.graphData.nodes;

    if (this.selectedKinds.length > 0) {
      const set = new Set(this.selectedKinds.map((k) => k.toLowerCase()));
      list = list.filter((n) => set.has(n.kind.toLowerCase()));
    }

    if (!this.includeOrphans) {
      list = list.filter((n) => !n.is_orphan);
    }

    const q = this.searchQuery.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (n) =>
          n.title.toLowerCase().includes(q) ||
          n.kind.toLowerCase().includes(q) ||
          n.tags.some((t) => t.toLowerCase().includes(q))
      );
    }

    return list;
  });

  // Arestas filtradas conectando nós visíveis
  filteredEdges = $derived.by(() => {
    if (!this.graphData) return [];
    const validIds = new Set(this.filteredNodes.map((n) => n.id));
    return this.graphData.edges.filter(
      (e) => validIds.has(e.source) && validIds.has(e.target)
    );
  });

  openModal(tab: 'graph' | 'canvas' = 'graph') {
    this.isOpen = true;
    this.activeTab = tab;
    void this.loadGraph();
  }

  closeModal() {
    this.isOpen = false;
    this.selectedNode = null;
  }

  setActiveTab(tab: 'graph' | 'canvas') {
    this.activeTab = tab;
  }

  async loadGraph() {
    this.isLoading = true;
    this.error = null;
    try {
      const data = await getVaultGraph(undefined, true);
      this.graphData = data;
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
      console.error('Falha ao carregar Grafo:', err);
    } finally {
      this.isLoading = false;
    }
  }

  toggleKind(kind: string) {
    const lower = kind.toLowerCase();
    if (this.selectedKinds.includes(lower)) {
      if (this.selectedKinds.length > 1) {
        this.selectedKinds = this.selectedKinds.filter((k) => k !== lower);
      }
    } else {
      this.selectedKinds = [...this.selectedKinds, lower];
    }
  }

  toggleOrphans() {
    this.includeOrphans = !this.includeOrphans;
  }

  selectNode(node: GraphNode | null) {
    this.selectedNode = node;
  }

  async fetchLocalGraph(
    centerId: string,
    depth: number = 1
  ): Promise<LocalGraphData | null> {
    try {
      return await getLocalGraph(centerId, depth);
    } catch (err) {
      console.error('Falha ao obter grafo local:', err);
      return null;
    }
  }
}

export const graphStore = new GraphStore();
