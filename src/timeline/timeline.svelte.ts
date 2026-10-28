/**
 * Store reativo para a Linha do Tempo Literária (M7 / Épico E6).
 *
 * Gerencia a alternância entre Ordem do Livro (Discurso) e Ordem Cronológica (Tempo da História),
 * filtros dinâmicos por Personagem e Local, busca de eventos, e navegação direta para cenas.
 */

import {
  getVaultTimeline,
  type TimelineEvent,
  type ChapterPacing,
  type VaultTimelineResponse,
} from '../tauri/bridge';
import { manuscript } from '../vault/manuscript.svelte';

export class TimelineStore {
  data = $state<VaultTimelineResponse | null>(null);
  activeViewMode = $state<'narrative' | 'chronological'>('narrative');
  selectedCharacter = $state<string | null>(null);
  selectedLocation = $state<string | null>(null);
  searchQuery = $state<string>('');
  showPacing = $state<boolean>(false);
  isModalOpen = $state<boolean>(false);
  isLoading = $state<boolean>(false);
  error = $state<string | null>(null);

  /**
   * Carrega os dados mais recentes da Linha do Tempo a partir do cofre criptografado.
   */
  async loadTimeline(): Promise<void> {
    this.isLoading = true;
    this.error = null;
    try {
      this.data = await getVaultTimeline();
    } catch (err: unknown) {
      this.error = err instanceof Error ? err.message : String(err);
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Abre o modal da linha do tempo e atualiza os eventos.
   */
  async openModal(): Promise<void> {
    this.isModalOpen = true;
    await this.loadTimeline();
  }

  /**
   * Fecha o modal da linha do tempo.
   */
  closeModal(): void {
    this.isModalOpen = false;
  }

  /**
   * Alterna entre a Ordem do Livro e a Ordem Cronológica.
   */
  setViewMode(mode: 'narrative' | 'chronological'): void {
    this.activeViewMode = mode;
  }

  /**
   * Filtra por personagem.
   */
  setCharacterFilter(character: string | null): void {
    this.selectedCharacter = character;
  }

  /**
   * Filtra por local.
   */
  setLocationFilter(location: string | null): void {
    this.selectedLocation = location;
  }

  /**
   * Define o termo de busca livre.
   */
  setSearchQuery(query: string): void {
    this.searchQuery = query;
  }

  /**
   * Alterna a visibilidade do painel de Pacing/Ritmo Narrativo.
   */
  togglePacing(): void {
    this.showPacing = !this.showPacing;
  }

  /**
   * Navega diretamente para a cena no editor e fecha o modal.
   */
  async navigateToScene(sceneId: string): Promise<void> {
    await manuscript.selectScene(sceneId);
    this.closeModal();
  }

  /**
   * Retorna os eventos ordenados de acordo com o modo ativo e filtrados.
   */
  get filteredEvents(): TimelineEvent[] {
    if (!this.data) return [];

    const orderList =
      this.activeViewMode === 'narrative'
        ? this.data.narrative_order
        : this.data.chronological_order;

    const eventMap = new Map<string, TimelineEvent>();
    for (const ev of this.data.events) {
      eventMap.set(ev.id, ev);
    }

    const orderedEvents: TimelineEvent[] = [];
    for (const id of orderList) {
      const ev = eventMap.get(id);
      if (ev) orderedEvents.push(ev);
    }

    const q = this.searchQuery.trim().toLowerCase();
    const charFilter = this.selectedCharacter;
    const locFilter = this.selectedLocation;

    return orderedEvents.filter((ev) => {
      // Filtro de personagem
      if (charFilter && !ev.characters.includes(charFilter)) {
        return false;
      }

      // Filtro de local
      if (locFilter && !ev.locations.includes(locFilter)) {
        return false;
      }

      // Filtro de busca textual
      if (q) {
        const matchesTitle = ev.scene_title.toLowerCase().includes(q);
        const matchesChapter = ev.chapter_title?.toLowerCase().includes(q);
        const matchesPart = ev.part_title?.toLowerCase().includes(q);
        const matchesTime = ev.raw_time?.toLowerCase().includes(q);
        const matchesSummary = ev.summary?.toLowerCase().includes(q);
        const matchesChar = ev.characters.some((c) => c.toLowerCase().includes(q));
        const matchesLoc = ev.locations.some((l) => l.toLowerCase().includes(q));

        if (
          !matchesTitle &&
          !matchesChapter &&
          !matchesPart &&
          !matchesTime &&
          !matchesSummary &&
          !matchesChar &&
          !matchesLoc
        ) {
          return false;
        }
      }

      return true;
    });
  }

  /**
   * Retorna os capítulos com dados de pacing.
   */
  get chapterPacing(): ChapterPacing[] {
    return this.data?.chapter_pacing || [];
  }
}

export const timelineStore = new TimelineStore();
