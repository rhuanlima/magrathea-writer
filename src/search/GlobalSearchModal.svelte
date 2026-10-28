<script lang="ts">
  import { t } from '../i18n';
  import { searchStore } from './search.svelte';
  import { manuscript } from '../vault/manuscript.svelte';

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && searchStore.isModalOpen) {
      searchStore.closeModal();
    }
  }

  function handleSelectDocument(itemId: string) {
    manuscript.selectScene(itemId);
    searchStore.closeModal();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if searchStore.isModalOpen}
  <div class="modal-backdrop">
    <div
      class="modal-card"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="global-search-title"
    >
      <header class="modal-header">
        <h2 id="global-search-title" class="modal-title">
          {t('search.modalTitle')}
        </h2>
        <button
          type="button"
          class="btn-close"
          onclick={() => searchStore.closeModal()}
          aria-label="Fechar"
        >
          ✕
        </button>
      </header>

      <!-- Abas de Busca vs Substituição -->
      <div class="tabs-bar">
        <button
          type="button"
          class="tab-btn"
          class:active={searchStore.activeTab === 'search'}
          onclick={() => (searchStore.activeTab = 'search')}
        >
          🔍 {t('search.searchTab')}
        </button>
        <button
          type="button"
          class="tab-btn"
          class:active={searchStore.activeTab === 'replace'}
          onclick={() => (searchStore.activeTab = 'replace')}
        >
          ⚡ {t('search.replaceTab')}
        </button>
      </div>

      <div class="modal-body">
        <!-- Inputs de Consulta -->
        <div class="form-row">
          <input
            type="text"
            class="input-query"
            placeholder={t('search.queryPlaceholder')}
            bind:value={searchStore.query}
            onkeydown={(e) => {
              if (e.key === 'Enter') {
                if (searchStore.activeTab === 'search') {
                  void searchStore.executeSearch();
                } else {
                  void searchStore.previewReplace();
                }
              }
            }}
          />
          <button
            type="button"
            class="btn-primary"
            disabled={searchStore.isSearching || searchStore.isPreviewing}
            onclick={() => {
              if (searchStore.activeTab === 'search') {
                void searchStore.executeSearch();
              } else {
                void searchStore.previewReplace();
              }
            }}
          >
            {searchStore.isSearching || searchStore.isPreviewing
              ? '...'
              : searchStore.activeTab === 'search'
                ? t('search.searchButton')
                : t('search.previewButton')}
          </button>
        </div>

        {#if searchStore.activeTab === 'replace'}
          <div class="form-row">
            <input
              type="text"
              class="input-query"
              placeholder={t('search.replacementPlaceholder')}
              bind:value={searchStore.replacement}
              onkeydown={(e) => e.key === 'Enter' && void searchStore.previewReplace()}
            />
          </div>
        {/if}

        <div class="options-bar">
          <label class="checkbox-label">
            <input
              type="checkbox"
              bind:checked={searchStore.matchCase}
              onchange={() => {
                if (searchStore.activeTab === 'search') {
                  void searchStore.executeSearch();
                } else {
                  void searchStore.previewReplace();
                }
              }}
            />
            <span>{t('search.matchCaseLabel')}</span>
          </label>

          {#if searchStore.lastRollbackId}
            <button
              type="button"
              class="btn-undo"
              disabled={searchStore.isRollingBack}
              onclick={() => void searchStore.rollback()}
            >
              ↺ {searchStore.isRollingBack ? '...' : t('search.undoButton')}
            </button>
          {/if}
        </div>

        {#if searchStore.successMessage}
          <div class="banner-success">{searchStore.successMessage}</div>
        {/if}

        {#if searchStore.errorMessage}
          <div class="banner-error">{searchStore.errorMessage}</div>
        {/if}

        <!-- Seção de Prévia de Substituição -->
        {#if searchStore.activeTab === 'replace' && searchStore.replacePreview}
          <div class="replace-preview-container">
            <div class="preview-summary">
              <span>
                {t('search.resultsCount', { count: searchStore.replacePreview.total_occurrences })}
                em {searchStore.replacePreview.total_affected_docs} documento(s)
              </span>
              <button
                type="button"
                class="btn-apply-replace"
                disabled={searchStore.isApplying || searchStore.replacePreview.total_occurrences === 0}
                onclick={() => void searchStore.applyReplace()}
              >
                {searchStore.isApplying ? '...' : t('search.replaceAllButton')}
              </button>
            </div>

            <div class="preview-list">
              {#each searchStore.replacePreview.items as item}
                <div class="preview-card">
                  <div class="preview-card-header">
                    <span class="doc-title">{item.title}</span>
                    <span class="count-pill">
                      {t('search.occurrencesCount', { count: item.occurrences_count })}
                    </span>
                  </div>
                  <div class="diff-snippet before">
                    <span class="diff-gutter">-</span>
                    <span>{item.before_snippet}</span>
                  </div>
                  <div class="diff-snippet after">
                    <span class="diff-gutter">+</span>
                    <span>{item.after_snippet}</span>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Seção de Resultados de Busca -->
        {#if searchStore.activeTab === 'search'}
          <div class="results-container">
            {#if searchStore.searchResults}
              <div class="results-header">
                <span>
                  {t('search.resultsCount', { count: searchStore.searchResults.total_matches })}
                </span>
              </div>

              {#if searchStore.searchResults.documents.length === 0}
                <div class="empty-results">{t('search.noResults')}</div>
              {:else}
                <div class="documents-list">
                  {#each searchStore.searchResults.documents as doc}
                    <div class="document-group">
                      <button
                        type="button"
                        class="doc-header-btn"
                        onclick={() => handleSelectDocument(doc.item_id)}
                      >
                        <span class="doc-icon">📖</span>
                        <span class="doc-title">{doc.title}</span>
                        <span class="count-pill">
                          {t('search.occurrencesCount', { count: doc.match_count })}
                        </span>
                      </button>

                      <div class="occurrences-list">
                        {#each doc.occurrences as occ}
                          <button
                            type="button"
                            class="occurrence-btn"
                            onclick={() => handleSelectDocument(doc.item_id)}
                          >
                            <span class="line-badge">{t('search.linePrefix', { line: occ.line_number })}</span>
                            <span class="snippet-text">{occ.snippet}</span>
                          </button>
                        {/each}
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-card {
    width: 680px;
    max-width: 95vw;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-md);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .modal-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin: 0;
  }

  .btn-close {
    background: transparent;
    border: none;
    font-size: 14px;
    color: var(--color-ink-muted);
    cursor: pointer;
  }

  .tabs-bar {
    display: flex;
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .tab-btn {
    flex: 1;
    padding: 8px 12px;
    font-size: 12px;
    font-weight: 600;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--color-ink-muted);
    cursor: pointer;
  }

  .tab-btn.active {
    color: var(--color-olive);
    border-bottom-color: var(--color-olive);
    background-color: var(--color-paper-base);
  }

  .modal-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
  }

  .form-row {
    display: flex;
    gap: 8px;
  }

  .input-query {
    flex: 1;
    padding: 8px 12px;
    font-size: 13px;
    background-color: var(--color-paper-surface);
    color: var(--color-ink-charcoal);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
  }

  .btn-primary {
    padding: 8px 16px;
    font-size: 12px;
    font-weight: 600;
    background-color: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .options-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    color: var(--color-ink-muted);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  .btn-undo {
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 600;
    background-color: rgba(184, 115, 51, 0.15);
    color: var(--color-bronze);
    border: 1px solid var(--color-bronze);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .banner-success {
    padding: 6px 10px;
    font-size: 12px;
    background-color: rgba(94, 111, 69, 0.12);
    color: var(--color-olive);
    border-radius: var(--radius-sm);
  }

  .banner-error {
    padding: 6px 10px;
    font-size: 12px;
    background-color: rgba(198, 40, 40, 0.12);
    color: var(--color-error);
    border-radius: var(--radius-sm);
  }

  /* Resultados */
  .results-container,
  .replace-preview-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .results-header,
  .preview-summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-muted);
  }

  .btn-apply-replace {
    padding: 4px 12px;
    font-size: 11px;
    font-weight: 600;
    background-color: var(--color-bronze);
    color: #ffffff;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .documents-list,
  .preview-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .document-group,
  .preview-card {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .doc-header-btn {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--color-paper-border);
    cursor: pointer;
    text-align: left;
  }

  .doc-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    flex: 1;
  }

  .count-pill {
    font-size: 10px;
    padding: 1px 6px;
    background-color: var(--color-paper-base);
    color: var(--color-ink-muted);
    border-radius: 10px;
  }

  .occurrences-list {
    display: flex;
    flex-direction: column;
  }

  .occurrence-btn {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 6px 10px;
    background: transparent;
    border: none;
    border-bottom: 1px solid rgba(0, 0, 0, 0.04);
    cursor: pointer;
    text-align: left;
    transition: background-color 150ms ease;
  }

  .occurrence-btn:hover {
    background-color: var(--color-paper-hover);
  }

  .line-badge {
    font-size: 10px;
    font-family: monospace;
    color: var(--color-olive);
    flex-shrink: 0;
  }

  .snippet-text {
    font-size: 12px;
    color: var(--color-ink-charcoal);
  }

  .diff-snippet {
    display: flex;
    align-items: baseline;
    gap: 6px;
    padding: 4px 10px;
    font-size: 11px;
    font-family: monospace;
  }

  .diff-snippet.before {
    background-color: rgba(198, 40, 40, 0.08);
    color: var(--color-error);
  }

  .diff-snippet.after {
    background-color: rgba(94, 111, 69, 0.1);
    color: var(--color-olive);
  }

  .diff-gutter {
    font-weight: bold;
    user-select: none;
  }

  .empty-results {
    padding: 24px;
    text-align: center;
    font-size: 12px;
    color: var(--color-ink-muted);
    font-style: italic;
  }
</style>
