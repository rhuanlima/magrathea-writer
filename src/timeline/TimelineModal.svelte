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
    background: var(--color-bg-base, #1e231d);
    color: var(--color-text-main, #dcdfd8);
    border: 1px solid var(--color-border, #3a4439);
    border-radius: 12px;
    width: 100%;
    max-width: 1050px;
    height: 88vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--color-border, #3a4439);
    background: var(--color-bg-elevated, #242b23);
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
    color: var(--color-text-main, #dcdfd8);
  }

  .close-button {
    background: transparent;
    border: none;
    color: var(--color-text-dim, #8b9987);
    font-size: 1.25rem;
    cursor: pointer;
    padding: 0.35rem 0.65rem;
    border-radius: 6px;
    transition: background 0.15s ease;
  }

  .close-button:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
  }

  /* Estatísticas */
  .stats-bar {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.6rem 1.5rem;
    background: rgba(94, 111, 69, 0.08);
    border-bottom: 1px solid var(--color-border, #3a4439);
    font-size: 0.85rem;
  }

  .stat-pill {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .stat-num {
    font-weight: 700;
    color: var(--color-primary, #8da372);
  }

  .flashback-stat .stat-num {
    color: #d99b50;
  }

  .flashforward-stat .stat-num {
    color: #5bb3d6;
  }

  /* Controles */
  .controls-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 1.5rem;
    background: var(--color-bg-base, #1e231d);
    border-bottom: 1px solid var(--color-border, #3a4439);
    flex-wrap: wrap;
  }

  .view-mode-group {
    display: flex;
    background: var(--color-bg-elevated, #242b23);
    border: 1px solid var(--color-border, #3a4439);
    border-radius: 8px;
    padding: 2px;
  }

  .mode-btn {
    background: transparent;
    border: none;
    color: var(--color-text-dim, #8b9987);
    padding: 0.45rem 0.9rem;
    font-size: 0.85rem;
    font-weight: 500;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mode-btn.active {
    background: var(--color-primary, #5e6f45);
    color: #ffffff;
    font-weight: 600;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .filters-group {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .filter-select,
  .search-input {
    background: var(--color-bg-elevated, #242b23);
    color: var(--color-text-main, #dcdfd8);
    border: 1px solid var(--color-border, #3a4439);
    border-radius: 6px;
    padding: 0.4rem 0.75rem;
    font-size: 0.85rem;
    outline: none;
  }

  .filter-select:focus,
  .search-input:focus {
    border-color: var(--color-primary, #8da372);
  }

  .pacing-toggle-btn {
    background: var(--color-bg-elevated, #242b23);
    border: 1px solid var(--color-border, #3a4439);
    color: var(--color-text-main, #dcdfd8);
    border-radius: 6px;
    padding: 0.4rem 0.8rem;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .pacing-toggle-btn.active {
    background: #425b74;
    border-color: #5bb3d6;
    color: #fff;
  }

  /* Corpo */
  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  /* Painel de Pacing */
  .pacing-section {
    background: rgba(66, 91, 116, 0.12);
    border: 1px solid #425b74;
    border-radius: 8px;
    padding: 1rem 1.25rem;
    margin-bottom: 1.5rem;
  }

  .pacing-title {
    margin: 0 0 0.85rem 0;
    font-size: 1rem;
    color: #8acfe8;
  }

  .pacing-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.85rem;
  }

  .pacing-card {
    background: var(--color-bg-elevated, #242b23);
    border: 1px solid var(--color-border, #3a4439);
    border-radius: 6px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
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
    background: rgba(255, 255, 255, 0.08);
    padding: 2px 6px;
    border-radius: 4px;
    color: var(--color-text-dim, #8b9987);
  }

  .pacing-meta-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: var(--color-text-dim, #8b9987);
  }

  .density-pill {
    background: #394b3e;
    color: #a4c48e;
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .pacing-time-range {
    font-size: 0.8rem;
    color: #dcdfd8;
    background: rgba(0, 0, 0, 0.2);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .pacing-note {
    font-size: 0.78rem;
    color: #f0c674;
    font-style: italic;
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
    background: var(--color-border, #3a4439);
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
    background: var(--color-bg-elevated, #242b23);
    border: 2px solid var(--color-border, #3a4439);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }

  .timeline-node-container:hover .timeline-marker {
    transform: scale(1.1);
    border-color: var(--color-primary, #8da372);
  }

  .flashback-node .timeline-marker {
    border-color: #d99b50;
    background: #362816;
  }

  .flashforward-node .timeline-marker {
    border-color: #5bb3d6;
    background: #172d38;
  }

  .marker-icon {
    font-size: 0.95rem;
  }

  .event-card {
    background: var(--color-bg-elevated, #242b23);
    border: 1px solid var(--color-border, #3a4439);
    border-radius: 8px;
    padding: 1rem;
    width: 100%;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  .event-card:hover {
    border-color: var(--color-primary, #8da372);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
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
    background: rgba(94, 111, 69, 0.2);
    color: var(--color-primary, #a8c287);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .undated-badge {
    background: rgba(255, 255, 255, 0.06);
    color: var(--color-text-dim, #8b9987);
  }

  .badge {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
  }

  .flashback-badge {
    background: rgba(217, 155, 80, 0.2);
    color: #f0b670;
    border: 1px solid #d99b50;
  }

  .flashforward-badge {
    background: rgba(91, 179, 214, 0.2);
    color: #8ad2ee;
    border: 1px solid #5bb3d6;
  }

  .order-indices {
    display: flex;
    gap: 0.4rem;
  }

  .idx-badge {
    font-size: 0.72rem;
    color: var(--color-text-dim, #8b9987);
    background: rgba(0, 0, 0, 0.25);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .chrono-idx {
    color: #a8c287;
  }

  .hierarchy-breadcrumb {
    font-size: 0.75rem;
    color: var(--color-text-dim, #8b9987);
    display: flex;
    gap: 0.35rem;
    margin-bottom: 0.35rem;
  }

  .scene-title {
    margin: 0 0 0.5rem 0;
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--color-text-main, #dcdfd8);
  }

  .scene-summary {
    margin: 0 0 0.75rem 0;
    font-size: 0.88rem;
    line-height: 1.45;
    color: #b5c2b2;
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
    filter: brightness(1.2);
  }

  .char-chip {
    background: rgba(94, 111, 69, 0.25);
    color: #b2cf8e;
    border: 1px solid rgba(94, 111, 69, 0.4);
  }

  .loc-chip {
    background: rgba(184, 137, 72, 0.2);
    color: #e5b978;
    border: 1px solid rgba(184, 137, 72, 0.4);
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    padding-top: 0.6rem;
    margin-top: 0.4rem;
  }

  .meta-info {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.78rem;
    color: var(--color-text-dim, #8b9987);
  }

  .status-pill {
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 0.72rem;
    text-transform: capitalize;
  }

  .status-draft {
    background: rgba(255, 255, 255, 0.08);
    color: #ccc;
  }

  .status-revised {
    background: rgba(91, 179, 214, 0.2);
    color: #8ad2ee;
  }

  .status-final {
    background: rgba(94, 111, 69, 0.3);
    color: #a8c287;
  }

  .go-scene-btn {
    background: var(--color-primary, #5e6f45);
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
    background: var(--color-primary-hover, #6f8352);
  }

  .empty-state,
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 1rem;
    color: var(--color-text-dim, #8b9987);
    gap: 0.75rem;
  }

  .empty-icon,
  .spinner {
    font-size: 2.5rem;
  }

  .error-banner {
    background: rgba(235, 87, 87, 0.15);
    border: 1px solid #eb5757;
    color: #ff8e8e;
    padding: 0.75rem 1rem;
    border-radius: 6px;
  }
</style>
