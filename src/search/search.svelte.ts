/**
//! Store reativa para busca e substituição global no cofre literário (M5-03, M5-04).
//!
//! Varredura em memória de todas as cenas e fichas de entidades com prévia e rollback.
*/

import {
  safeInvoke,
  type GlobalSearchResponse,
  type GlobalReplacePreview,
  type ApplyGlobalReplaceResult,
} from '../tauri/bridge';

class SearchStore {
  isModalOpen = $state(false);
  activeTab = $state<'search' | 'replace'>('search');
  query = $state('');
  replacement = $state('');
  matchCase = $state(false);

  searchResults = $state<GlobalSearchResponse | null>(null);
  replacePreview = $state<GlobalReplacePreview | null>(null);

  isSearching = $state(false);
  isPreviewing = $state(false);
  isApplying = $state(false);
  isRollingBack = $state(false);

  lastRollbackId = $state<string | null>(null);
  errorMessage = $state<string | null>(null);
  successMessage = $state<string | null>(null);

  openModal(tab: 'search' | 'replace' = 'search'): void {
    this.activeTab = tab;
    this.isModalOpen = true;
    this.errorMessage = null;
    this.successMessage = null;
  }

  closeModal(): void {
    this.isModalOpen = false;
  }

  /**
   * Executa busca textual rápida em memória.
   */
  async executeSearch(): Promise<void> {
    if (!this.query.trim()) {
      this.searchResults = null;
      return;
    }

    this.isSearching = true;
    this.errorMessage = null;

    try {
      this.searchResults = await safeInvoke<GlobalSearchResponse>('global_search', {
        query: this.query,
        matchCase: this.matchCase,
        match_case: this.matchCase,
      });
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.isSearching = false;
    }
  }

  /**
   * Gera prévia de substituição em massa.
   */
  async previewReplace(): Promise<void> {
    if (!this.query.trim()) return;

    this.isPreviewing = true;
    this.errorMessage = null;

    try {
      this.replacePreview = await safeInvoke<GlobalReplacePreview>('preview_global_replace', {
        query: this.query,
        replacement: this.replacement,
        matchCase: this.matchCase,
        match_case: this.matchCase,
      });
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.isPreviewing = false;
    }
  }

  /**
   * Aplica a substituição global de forma atômica com criação de snapshot de rollback.
   */
  async applyReplace(): Promise<ApplyGlobalReplaceResult | null> {
    if (!this.query.trim()) return null;

    this.isApplying = true;
    this.errorMessage = null;
    this.successMessage = null;

    try {
      const result = await safeInvoke<ApplyGlobalReplaceResult>('apply_global_replace', {
        query: this.query,
        replacement: this.replacement,
        matchCase: this.matchCase,
        match_case: this.matchCase,
      });

      this.lastRollbackId = result.rollback_snapshot_id;
      this.replacePreview = null;
      this.successMessage = `Substituídas ${result.total_occurrences_replaced} ocorrências em ${result.modified_docs_count} documentos.`;
      // Atualiza resultados de busca
      await this.executeSearch();
      return result;
    } catch (err) {
      this.errorMessage = String(err);
      return null;
    } finally {
      this.isApplying = false;
    }
  }

  /**
   * Reverte uma substituição em massa utilizando o snapshot de rollback gerado.
   */
  async rollback(): Promise<boolean> {
    if (!this.lastRollbackId) return false;

    this.isRollingBack = true;
    this.errorMessage = null;

    try {
      await safeInvoke('rollback_global_replace', {
        snapshotId: this.lastRollbackId,
        snapshot_id: this.lastRollbackId,
      });

      this.lastRollbackId = null;
      this.successMessage = 'Substituição desfeita com sucesso. Documentos restaurados.';
      await this.executeSearch();
      return true;
    } catch (err) {
      this.errorMessage = String(err);
      return false;
    } finally {
      this.isRollingBack = false;
    }
  }
}

export const searchStore = new SearchStore();
