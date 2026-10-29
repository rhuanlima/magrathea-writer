<script lang="ts">
  /**
   * Modal Interativo da Linha do Tempo Literária (M7 / Épico E6).
   *
   * Fornece alternância fluida entre Ordem do Livro (Sjuzhet) e Ordem Cronológica (Fabula),
   * identificação visual de Flashbacks e Flashforwards, filtros por Personagem e Local,
   * gráfico/painel de Pacing por capítulo e navegação direta para o editor.
   */

  import { timelineStore } from './timeline.svelte';
  import { t } from '../i18n';

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      timelineStore.closeModal();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      timelineStore.closeModal();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if timelineStore.isModalOpen}
  <div
    class="modal-backdrop"
    onclick={handleBackdropClick}
    role="presentation"
  >
    <div
      class="timeline-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="timeline-title"
    >
      <!-- Cabeçalho -->
      <header class="modal-header">
        <div class="header-left">
          <span class="header-icon">⏳</span>
          <h2 id="timeline-title" class="modal-title">
            {t('timeline.modalTitle')}
          </h2>
        </div>
        <button
          type="button"
          class="close-button"
          onclick={() => timelineStore.closeModal()}
          aria-label="Fechar"
        >
          ✕
        </button>
      </header>

      <!-- Barra de Estatísticas Literárias -->
      {#if timelineStore.data}
        <div class="stats-bar">
          <div class="stat-pill">
            <span class="stat-num">{timelineStore.data.total_events}</span>
            <span class="stat-label">{t('timeline.totalEvents')}</span>
          </div>
          <div class="stat-pill">
            <span class="stat-num">{timelineStore.data.dated_events_count}</span>
            <span class="stat-label">{t('timeline.datedEvents')}</span>
          </div>
          <div class="stat-pill flashback-stat">
            <span class="stat-num">{timelineStore.data.total_flashbacks}</span>
            <span class="stat-label">{t('timeline.flashbacksCount')}</span>
          </div>
          <div class="stat-pill flashforward-stat">
            <span class="stat-num">{timelineStore.data.total_flashforwards}</span>
            <span class="stat-label">{t('timeline.flashforwardsCount')}</span>
          </div>
        </div>
      {/if}

      <!-- Barra de Controle & Filtros -->
      <div class="controls-bar">
        <!-- Alternador de Visão (M7-03) -->
        <div class="view-mode-group">
          <button
            type="button"
            class="mode-btn"
            class:active={timelineStore.activeViewMode === 'narrative'}
            onclick={() => timelineStore.setViewMode('narrative')}
          >
            {t('timeline.bookOrderTab')}
          </button>
          <button
            type="button"
            class="mode-btn"
            class:active={timelineStore.activeViewMode === 'chronological'}
            onclick={() => timelineStore.setViewMode('chronological')}
          >
            {t('timeline.chronologicalTab')}
          </button>
        </div>

        <!-- Filtros Dinâmicos (M7-04) -->
        <div class="filters-group">
          {#if timelineStore.data && timelineStore.data.all_characters.length > 0}
            <select
              class="filter-select"
              value={timelineStore.selectedCharacter ?? ''}
              onchange={(e) =>
                timelineStore.setCharacterFilter(
                  (e.target as HTMLSelectElement).value || null
                )}
            >
              <option value="">👤 {t('timeline.allCharacters')}</option>
              {#each timelineStore.data.all_characters as char}
                <option value={char}>{char}</option>
              {/each}
            </select>
          {/if}

          {#if timelineStore.data && timelineStore.data.all_locations.length > 0}
            <select
              class="filter-select"
              value={timelineStore.selectedLocation ?? ''}
              onchange={(e) =>
                timelineStore.setLocationFilter(
                  (e.target as HTMLSelectElement).value || null
                )}
            >
              <option value="">📍 {t('timeline.allLocations')}</option>
              {#each timelineStore.data.all_locations as loc}
                <option value={loc}>{loc}</option>
              {/each}
            </select>
          {/if}

          <input
            type="text"
            class="search-input"
            placeholder={t('timeline.searchPlaceholder')}
            value={timelineStore.searchQuery}
            oninput={(e) =>
              timelineStore.setSearchQuery((e.target as HTMLInputElement).value)}
          />

          <!-- Alternar Pacing (M7-05) -->
          <button
            type="button"
            class="pacing-toggle-btn"
            class:active={timelineStore.showPacing}
            onclick={() => timelineStore.togglePacing()}
          >
            📊 {t('timeline.togglePacing')}
          </button>
        </div>
      </div>

      <!-- Conteúdo Principal -->
      <div class="modal-body">
        {#if timelineStore.isLoading}
          <div class="loading-state">
            <span class="spinner">⏳</span>
            <p>Carregando linha do tempo do cofre...</p>
          </div>
        {:else if timelineStore.error}
          <div class="error-banner">
            ⚠️ {timelineStore.error}
          </div>
        {:else}
          <!-- Painel de Pacing e Ritmo por Capítulo (M7-05) -->
          {#if timelineStore.showPacing}
            <section class="pacing-section">
              <h3 class="pacing-title">📊 {t('timeline.pacingPanelTitle')}</h3>
              <div class="pacing-grid">
                {#each timelineStore.chapterPacing as ch}
                  <div class="pacing-card">
                    <div class="pacing-header">
                      <span class="pacing-ch-title">{ch.chapter_title}</span>
                      {#if ch.part_title}
                        <span class="pacing-part-badge">{ch.part_title}</span>
                      {/if}
                    </div>

                    <div class="pacing-meta-row">
                      <span><strong>{ch.scene_count}</strong> {t('timeline.scenesLabel')}</span>
                      <span><strong>{ch.word_count.toLocaleString()}</strong> {t('timeline.wordsLabel')}</span>
                      <span class="density-pill" title="Eventos a cada 1.000 palavras">
                        {ch.event_density} ev/1k
                      </span>
                    </div>

                    {#if ch.start_time || ch.end_time}
                      <div class="pacing-time-range">
                        ⏳ {ch.start_time || '?'} → {ch.end_time || '?'}
                      </div>
                    {/if}

                    {#if ch.pacing_note}
                      <div class="pacing-note">
                        💡 {ch.pacing_note}
                      </div>
                    {/if}

                    {#if ch.characters.length > 0}
                      <div class="pacing-chips">
                        {#each ch.characters as c}
                          <span class="mini-chip char-chip">👤 {c}</span>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            </section>
          {/if}

          <!-- Visualização em Linha do Tempo (M7-02, M7-03, M7-04) -->
          {@const events = timelineStore.filteredEvents}
          {#if events.length === 0}
            <div class="empty-state">
              <span class="empty-icon">🍃</span>
              <p>{t('timeline.noEventsFound')}</p>
            </div>
          {:else}
            <div class="timeline-tree">
              <div class="timeline-axis"></div>

              {#each events as ev}
                <div
                  class="timeline-node-container"
                  class:flashback-node={ev.is_flashback}
                  class:flashforward-node={ev.is_flashforward}
                >
                  <!-- Marcador no eixo central -->
                  <div class="timeline-marker">
                    {#if ev.is_flashback}
                      <span class="marker-icon" title="Flashback">⏪</span>
                    {:else if ev.is_flashforward}
                      <span class="marker-icon" title="Flashforward">⏩</span>
                    {:else if ev.time_type === 'real_date'}
                      <span class="marker-icon">📅</span>
                    {:else if ev.time_type === 'relative'}
                      <span class="marker-icon">⏳</span>
                    {:else}
                      <span class="marker-icon">📝</span>
                    {/if}
                  </div>

                  <!-- Cartão do Evento -->
                  <div class="event-card">
                    <div class="card-header">
                      <div class="time-and-badges">
                        {#if ev.raw_time}
                          <span class="time-badge" class:real-date={ev.time_type === 'real_date'}>
                            {ev.time_type === 'real_date' ? '📅' : '⏳'} {ev.raw_time}
                          </span>
                        {:else}
                          <span class="time-badge undated-badge">
                            {t('timeline.undatedBadge')}
                          </span>
                        {/if}

                        {#if ev.is_flashback}
                          <span class="badge flashback-badge">
                            {t('timeline.flashbackBadge')}
                          </span>
                        {/if}

                        {#if ev.is_flashforward}
                          <span class="badge flashforward-badge">
                            {t('timeline.flashforwardBadge')}
                          </span>
                        {/if}
                      </div>

                      <div class="order-indices">
                        <span class="idx-badge" title="Ordem no Livro">
                          📖 #{ev.book_order + 1}
                        </span>
                        {#if ev.time_type !== 'undated'}
                          <span class="idx-badge chrono-idx" title="Ordem Cronológica da História">
                            ⏳ #{ev.chronological_order + 1}
                          </span>
                        {/if}
                      </div>
                    </div>

                    <!-- Hierarquia (Parte > Capítulo) -->
                    {#if ev.chapter_title || ev.part_title}
                      <div class="hierarchy-breadcrumb">
                        {#if ev.part_title}
                          <span>📚 {ev.part_title}</span>
                          <span>›</span>
                        {/if}
                        {#if ev.chapter_title}
                          <span>📖 {ev.chapter_title}</span>
                        {/if}
                      </div>
                    {/if}

                    <!-- Título da Cena -->
                    <h4 class="scene-title">
                      🎬 {ev.scene_title}
                    </h4>

                    <!-- Sinopse / Resumo -->
                    {#if ev.summary}
                      <p class="scene-summary">
                        {ev.summary}
                      </p>
                    {/if}

                    <!-- Entidades Participantes (M7-04) -->
                    {#if ev.characters.length > 0 || ev.locations.length > 0}
                      <div class="entities-row">
                        {#each ev.characters as char}
                          <span
                            class="entity-chip char-chip"
                            onclick={() => timelineStore.setCharacterFilter(char)}
                            role="button"
                            tabindex="0"
                            onkeydown={(e) => {
                              if (e.key === 'Enter') timelineStore.setCharacterFilter(char);
                            }}
                            title="Filtrar por este personagem"
                          >
                            👤 {char}
                          </span>
                        {/each}

                        {#each ev.locations as loc}
                          <span
                            class="entity-chip loc-chip"
                            onclick={() => timelineStore.setLocationFilter(loc)}
                            role="button"
                            tabindex="0"
                            onkeydown={(e) => {
                              if (e.key === 'Enter') timelineStore.setLocationFilter(loc);
                            }}
                            title="Filtrar por este local"
                          >
                            📍 {loc}
                          </span>
                        {/each}
                      </div>
                    {/if}

                    <!-- Rodapé do Card com Ação de Navegação -->
                    <div class="card-footer">
                      <div class="meta-info">
                        <span class="word-count">
                          {ev.word_count.toLocaleString()} {t('timeline.wordsLabel')}
                        </span>
                        {#if ev.status}
                          <span class="status-pill status-{ev.status}">
                            {ev.status}
                          </span>
                        {/if}
                      </div>

                      <button
                        type="button"
                        class="go-scene-btn"
                        onclick={() => timelineStore.navigateToScene(ev.id)}
                        title="Ir para esta cena no editor"
                      >
                        {t('timeline.goToSceneButton')}
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 1.5rem;
  }

  .timeline-modal {
    background: var(--color-paper-card, #FFFFFF);
    color: var(--color-ink-charcoal, #111827);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 12px;
    width: 100%;
    max-width: 1050px;
    height: 88vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.25);
    overflow: hidden;
  }

  :global(.dark) .timeline-modal {
    background: #1B1D19;
    color: #FFFFFF;
    border-color: #383C32;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    background: var(--color-paper-subtle, #F9FAFB);
  }

  :global(.dark) .modal-header {
    background: #232620;
    border-color: #383C32;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .header-icon {
    font-size: 1.4rem;
  }

  .modal-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: inherit;
  }

  .close-button {
    width: 32px;
    height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    font-size: 0.95rem;
    color: var(--color-ink-muted, #6B7280);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .close-button:hover {
    color: var(--color-ink-charcoal, #111827);
    background: var(--color-paper-hover, rgba(0, 0, 0, 0.05));
    border-color: var(--color-ink-faded, #9CA3AF);
  }

  :global(.dark) .close-button {
    border-color: #383C32;
    color: #A4A89E;
  }

  :global(.dark) .close-button:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #FFFFFF;
    border-color: #555A4E;
  }

  /* Estatísticas */
  .stats-bar {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.6rem 1.5rem;
    background: rgba(94, 111, 69, 0.06);
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    font-size: 0.85rem;
  }

  :global(.dark) .stats-bar {
    background: rgba(138, 166, 106, 0.08);
    border-color: #383C32;
  }

  .stat-pill {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .stat-num {
    font-weight: 700;
    color: var(--color-olive, #5E6F45);
  }

  :global(.dark) .stat-num {
    color: #A3BD84;
  }

  .flashback-stat .stat-num {
    color: var(--color-bronze, #C28D58);
  }

  :global(.dark) .flashback-stat .stat-num {
    color: #D4B97F;
  }

  .flashforward-stat .stat-num {
    color: #2563EB;
  }

  :global(.dark) .flashforward-stat .stat-num {
    color: #60A5FA;
  }

  /* Controles */
  .controls-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 1.5rem;
    background: var(--color-paper-base, #FFFFFF);
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    flex-wrap: wrap;
  }

  :global(.dark) .controls-bar {
    background: #1B1D19;
    border-color: #383C32;
  }

  .view-mode-group {
    display: flex;
    background: var(--color-paper-subtle, #F3F4F6);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    padding: 2px;
  }

  :global(.dark) .view-mode-group {
    background: #232620;
    border-color: #383C32;
  }

  .mode-btn {
    background: transparent;
    border: none;
    color: var(--color-ink-muted, #6B7280);
    padding: 0.45rem 0.9rem;
    font-size: 0.85rem;
    font-weight: 500;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark) .mode-btn {
    color: #A4A89E;
  }

  .mode-btn.active {
    background: var(--color-olive, #5E6F45);
    color: #ffffff;
    font-weight: 600;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.15);
  }

  :global(.dark) .mode-btn.active {
    background: #5E6F45;
    color: #FFFFFF;
  }

  .filters-group {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .filter-select,
  .search-input {
    background: var(--color-paper-card, #FFFFFF);
    color: var(--color-ink-charcoal, #111827);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    padding: 0.4rem 0.75rem;
    font-size: 0.85rem;
    outline: none;
  }

  :global(.dark) .filter-select,
  :global(.dark) .search-input {
    background: #232620;
    color: #FFFFFF;
    border-color: #383C32;
  }

  .filter-select:focus,
  .search-input:focus {
    border-color: var(--color-olive, #5E6F45);
  }

  .pacing-toggle-btn {
    background: var(--color-paper-card, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    color: var(--color-ink-charcoal, #111827);
    border-radius: 6px;
    padding: 0.4rem 0.8rem;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark) .pacing-toggle-btn {
    background: #232620;
    border-color: #383C32;
    color: #FFFFFF;
  }

  .pacing-toggle-btn.active {
    background: rgba(94, 111, 69, 0.15);
    border-color: var(--color-olive, #5E6F45);
    color: var(--color-olive, #5E6F45);
    font-weight: 600;
  }

  :global(.dark) .pacing-toggle-btn.active {
    background: rgba(138, 166, 106, 0.2);
    border-color: #8AA66A;
    color: #A3BD84;
  }

  /* Corpo */
  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    background: var(--color-paper-base, #FFFFFF);
  }

  :global(.dark) .modal-body {
    background: #1B1D19;
  }

  /* Painel de Pacing */
  .pacing-section {
    background: rgba(94, 111, 69, 0.05);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    padding: 1rem 1.25rem;
    margin-bottom: 1.5rem;
  }

  :global(.dark) .pacing-section {
    background: rgba(138, 166, 106, 0.06);
    border-color: #383C32;
  }

  .pacing-title {
    margin: 0 0 0.85rem 0;
    font-size: 1rem;
    color: var(--color-olive, #5E6F45);
    font-weight: 600;
  }

  :global(.dark) .pacing-title {
    color: #A3BD84;
  }

  .pacing-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.85rem;
  }

  .pacing-card {
    background: var(--color-paper-card, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  :global(.dark) .pacing-card {
    background: #232620;
    border-color: #383C32;
  }

  .pacing-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .pacing-ch-title {
    font-weight: 600;
    font-size: 0.9rem;
  }

  .pacing-part-badge {
    font-size: 0.7rem;
    background: rgba(0, 0, 0, 0.05);
    padding: 2px 6px;
    border-radius: 4px;
    color: var(--color-ink-muted, #6B7280);
  }

  :global(.dark) .pacing-part-badge {
    background: rgba(255, 255, 255, 0.08);
    color: #A4A89E;
  }

  .pacing-meta-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: var(--color-ink-muted, #6B7280);
  }

  :global(.dark) .pacing-meta-row {
    color: #A4A89E;
  }

  .density-pill {
    background: rgba(94, 111, 69, 0.1);
    color: var(--color-olive, #5E6F45);
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  :global(.dark) .density-pill {
    background: rgba(138, 166, 106, 0.2);
    color: #A3BD84;
  }

  .pacing-time-range {
    font-size: 0.8rem;
    background: rgba(0, 0, 0, 0.04);
    padding: 2px 6px;
    border-radius: 4px;
  }

  :global(.dark) .pacing-time-range {
    background: rgba(255, 255, 255, 0.06);
  }

  .pacing-note {
    font-size: 0.78rem;
    color: var(--color-bronze, #C28D58);
    font-style: italic;
  }

  :global(.dark) .pacing-note {
    color: #D4B97F;
  }

  .pacing-chips {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
    margin-top: 0.25rem;
  }

  .mini-chip {
    font-size: 0.72rem;
    padding: 1px 5px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.04);
  }

  :global(.dark) .mini-chip {
    background: rgba(255, 255, 255, 0.06);
  }

  /* Linha do Tempo Visual */
  .timeline-tree {
    position: relative;
    padding-left: 2.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .timeline-axis {
    position: absolute;
    left: 1.15rem;
    top: 0.5rem;
    bottom: 0.5rem;
    width: 2px;
    background: var(--color-paper-border, #E5E0D8);
  }

  :global(.dark) .timeline-axis {
    background: #383C32;
  }

  .timeline-node-container {
    position: relative;
    display: flex;
    align-items: flex-start;
  }

  .timeline-marker {
    position: absolute;
    left: -2.5rem;
    top: 0.6rem;
    width: 2.3rem;
    height: 2.3rem;
    border-radius: 50%;
    background: var(--color-paper-card, #FFFFFF);
    border: 2px solid var(--color-paper-border, #E5E0D8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }

  :global(.dark) .timeline-marker {
    background: #232620;
    border-color: #383C32;
  }

  .timeline-node-container:hover .timeline-marker {
    transform: scale(1.1);
    border-color: var(--color-olive, #5E6F45);
  }

  :global(.dark) .timeline-node-container:hover .timeline-marker {
    border-color: #8AA66A;
  }

  .flashback-node .timeline-marker {
    border-color: var(--color-bronze, #C28D58);
    background: rgba(194, 141, 88, 0.1);
  }

  :global(.dark) .flashback-node .timeline-marker {
    border-color: #D4B97F;
    background: rgba(212, 185, 127, 0.15);
  }

  .flashforward-node .timeline-marker {
    border-color: #2563EB;
    background: rgba(37, 99, 235, 0.08);
  }

  :global(.dark) .flashforward-node .timeline-marker {
    border-color: #60A5FA;
    background: rgba(96, 165, 250, 0.15);
  }

  .marker-icon {
    font-size: 0.95rem;
  }

  .event-card {
    background: var(--color-paper-card, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    padding: 1rem;
    width: 100%;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  :global(.dark) .event-card {
    background: #232620;
    border-color: #383C32;
  }

  .event-card:hover {
    border-color: var(--color-olive, #5E6F45);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
  }

  :global(.dark) .event-card:hover {
    border-color: #8AA66A;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .time-and-badges {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .time-badge {
    background: rgba(94, 111, 69, 0.1);
    color: var(--color-olive, #5E6F45);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 600;
  }

  :global(.dark) .time-badge {
    background: rgba(138, 166, 106, 0.15);
    color: #A3BD84;
  }

  .undated-badge {
    background: rgba(0, 0, 0, 0.05);
    color: var(--color-ink-muted, #6B7280);
  }

  :global(.dark) .undated-badge {
    background: rgba(255, 255, 255, 0.06);
    color: #A4A89E;
  }

  .badge {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
  }

  .flashback-badge {
    background: rgba(194, 141, 88, 0.12);
    color: var(--color-bronze, #C28D58);
    border: 1px solid var(--color-bronze, #C28D58);
  }

  :global(.dark) .flashback-badge {
    background: rgba(212, 185, 127, 0.15);
    color: #D4B97F;
    border-color: #D4B97F;
  }

  .flashforward-badge {
    background: rgba(37, 99, 235, 0.1);
    color: #2563EB;
    border: 1px solid #2563EB;
  }

  :global(.dark) .flashforward-badge {
    background: rgba(96, 165, 250, 0.15);
    color: #93C5FD;
    border-color: #60A5FA;
  }

  .order-indices {
    display: flex;
    gap: 0.4rem;
  }

  .idx-badge {
    font-size: 0.72rem;
    color: var(--color-ink-muted, #6B7280);
    background: rgba(0, 0, 0, 0.05);
    padding: 2px 6px;
    border-radius: 4px;
  }

  :global(.dark) .idx-badge {
    background: rgba(255, 255, 255, 0.08);
    color: #A4A89E;
  }

  .chrono-idx {
    color: var(--color-olive, #5E6F45);
    font-weight: 600;
  }

  :global(.dark) .chrono-idx {
    color: #A3BD84;
  }

  .hierarchy-breadcrumb {
    font-size: 0.75rem;
    color: var(--color-ink-muted, #6B7280);
    display: flex;
    gap: 0.35rem;
    margin-bottom: 0.35rem;
  }

  :global(.dark) .hierarchy-breadcrumb {
    color: #A4A89E;
  }

  .scene-title {
    margin: 0 0 0.5rem 0;
    font-size: 1.05rem;
    font-weight: 600;
    color: inherit;
  }

  .scene-summary {
    margin: 0 0 0.75rem 0;
    font-size: 0.88rem;
    line-height: 1.45;
    color: var(--color-ink-muted, #4B5563);
  }

  :global(.dark) .scene-summary {
    color: #D1D5DB;
  }

  .entities-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.75rem;
  }

  .entity-chip {
    font-size: 0.78rem;
    padding: 2px 8px;
    border-radius: 4px;
    cursor: pointer;
    transition: transform 0.1s ease, filter 0.1s ease;
  }

  .entity-chip:hover {
    transform: translateY(-1px);
    filter: brightness(1.1);
  }

  .char-chip {
    background: rgba(94, 111, 69, 0.12);
    color: var(--color-olive, #5E6F45);
    border: 1px solid rgba(94, 111, 69, 0.25);
  }

  :global(.dark) .char-chip {
    background: rgba(138, 166, 106, 0.15);
    color: #A3BD84;
    border-color: rgba(138, 166, 106, 0.3);
  }

  .loc-chip {
    background: rgba(194, 141, 88, 0.12);
    color: var(--color-bronze, #C28D58);
    border: 1px solid rgba(194, 141, 88, 0.25);
  }

  :global(.dark) .loc-chip {
    background: rgba(212, 185, 127, 0.15);
    color: #D4B97F;
    border-color: rgba(212, 185, 127, 0.3);
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid var(--color-paper-border, #E5E0D8);
    padding-top: 0.6rem;
    margin-top: 0.4rem;
  }

  :global(.dark) .card-footer {
    border-color: #383C32;
  }

  .meta-info {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.78rem;
    color: var(--color-ink-muted, #6B7280);
  }

  :global(.dark) .meta-info {
    color: #A4A89E;
  }

  .status-pill {
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 0.72rem;
    text-transform: capitalize;
  }

  .status-draft {
    background: rgba(0, 0, 0, 0.05);
    color: var(--color-ink-muted, #6B7280);
  }

  :global(.dark) .status-draft {
    background: rgba(255, 255, 255, 0.08);
    color: #A4A89E;
  }

  .status-revised {
    background: rgba(37, 99, 235, 0.1);
    color: #2563EB;
  }

  :global(.dark) .status-revised {
    background: rgba(96, 165, 250, 0.15);
    color: #93C5FD;
  }

  .status-final {
    background: rgba(16, 185, 129, 0.12);
    color: #059669;
  }

  :global(.dark) .status-final {
    background: rgba(16, 185, 129, 0.2);
    color: #6EE7B7;
  }

  .go-scene-btn {
    background: var(--color-olive, #5E6F45);
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 0.35rem 0.8rem;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .go-scene-btn:hover {
    background: #4A5836;
  }

  .empty-state,
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 1rem;
    color: var(--color-ink-muted, #6B7280);
    gap: 0.75rem;
  }

  :global(.dark) .empty-state,
  :global(.dark) .loading-state {
    color: #A4A89E;
  }

  .empty-icon,
  .spinner {
    font-size: 2.5rem;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.12);
    border: 1px solid #EF4444;
    color: #DC2626;
    padding: 0.75rem 1rem;
    border-radius: 6px;
  }

  :global(.dark) .error-banner {
    background: rgba(239, 68, 68, 0.2);
    color: #F87171;
  }
</style>
