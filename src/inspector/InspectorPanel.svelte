<script lang="ts">
  /**
   * Painel Inspetor do Segundo Cérebro (M2-03).
   *
   * Exibe e edita metadados do frontmatter da cena ativa e lista todas as conexões
   * e citações reversas (backlinks) detectadas no manuscrito pelo índice em memória.
   */

  import { t } from '../i18n';
  import { vault } from '../vault/vault.svelte';
  import { manuscript } from '../vault/manuscript.svelte';
  import { worldbuilding } from '../worldbuilding/worldbuilding.svelte';
  import EntityAppearancesPanel from '../worldbuilding/EntityAppearancesPanel.svelte';
  import EntityGallery from '../worldbuilding/EntityGallery.svelte';
  import EntityContextPanel from '../worldbuilding/EntityContextPanel.svelte';
  import SnapshotsPanel from '../snapshots/SnapshotsPanel.svelte';
  import LocalGraphTab from './LocalGraphTab.svelte';
  import { analyzeProseStyle } from '../editor/text-analysis';

  interface Props {
    currentContent?: string;
    onRestoreContent?: (newContent: string) => void;
  }

  let { currentContent = '', onRestoreContent }: Props = $props();

  let activeItem = $derived(
    manuscript.items.find((it) => it.id === manuscript.activeSceneId) || null
  );

  let backlinks = $derived(vault.activeBacklinks);
  let styleAnalysis = $derived(analyzeProseStyle(currentContent));

  $effect(() => {
    if (activeItem && vault.isInspectorOpen) {
      void vault.loadBacklinks(activeItem.id);
      void worldbuilding.loadAppearances(activeItem.id);
      void worldbuilding.loadImages(activeItem.id);
    }
  });

  function selectBacklinkSource(sourceId: string) {
    manuscript.activeSceneId = sourceId;
  }
</script>

{#if vault.isInspectorOpen}
  <aside class="inspector-panel" aria-label={t('inspector.title')}>
    <!-- Cabeçalho do Inspetor -->
    <div class="inspector-header">
      <div class="header-info">
        <span class="inspector-badge">{activeItem?.kind || t('inspector.defaultKind')}</span>
        <h3 class="inspector-title">{activeItem?.title || t('inspector.noActiveItem')}</h3>
      </div>
      <div class="header-actions">
        {#if activeItem}
          <button
            class="refactor-btn"
            onclick={() => vault.openRefactorModal(activeItem.title)}
            title="Renomear e atualizar referências no cofre"
            aria-label="Renomear e atualizar referências"
          >
            ✎
          </button>
        {/if}
        <button
          class="close-btn"
          onclick={() => vault.toggleInspector(false)}
          title={t('inspector.close')}
          aria-label={t('inspector.close')}
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Abas de Navegação -->
    <div class="inspector-tabs">
      <button
        class="tab-btn"
        class:active={vault.activeInspectorTab === 'context'}
        onclick={() => vault.setInspectorTab('context')}
      >
        ✨ Contexto
      </button>
      <button
        class="tab-btn"
        class:active={vault.activeInspectorTab === 'metadata'}
        onclick={() => vault.setInspectorTab('metadata')}
      >
        📋 {t('inspector.tabMetadata')}
      </button>
      <button
        class="tab-btn"
        class:active={vault.activeInspectorTab === 'backlinks'}
        onclick={() => vault.setInspectorTab('backlinks')}
      >
        🔗 {t('inspector.tabBacklinks')}
        {#if backlinks.length > 0}
          <span class="count-pill">{backlinks.length}</span>
        {/if}
      </button>
      <button
        class="tab-btn"
        class:active={vault.activeInspectorTab === 'appearances'}
        onclick={() => vault.setInspectorTab('appearances')}
      >
        🧭 {t('worldbuilding.tabAppearances')}
        {#if worldbuilding.appearances.length > 0}
          <span class="count-pill">{worldbuilding.appearances.length}</span>
        {/if}
      </button>
      <button
        class="tab-btn"
        class:active={vault.activeInspectorTab === 'gallery'}
        onclick={() => vault.setInspectorTab('gallery')}
      >
        🖼️ {t('worldbuilding.tabGallery')}
        {#if worldbuilding.images.length > 0}
          <span class="count-pill">{worldbuilding.images.length}</span>
        {/if}
      </button>
      <button
        class="tab-btn"
        class:active={vault.activeInspectorTab === 'snapshots'}
        onclick={() => vault.setInspectorTab('snapshots')}
      >
        🕒 {t('snapshots.tabTitle')}
      </button>
      <button
        class="tab-btn"
        class:active={vault.activeInspectorTab === 'graph'}
        onclick={() => vault.setInspectorTab('graph')}
      >
        🕸️ {t('graph.tabLocalGraph')}
      </button>
      <button
        class="tab-btn"
        class:active={vault.activeInspectorTab === 'style'}
        onclick={() => vault.setInspectorTab('style')}
        title="Estilometria, advérbios e verbos de diálogo"
      >
        🔬 Estilo
      </button>
    </div>

    <!-- Conteúdo da Aba Ativa -->
    <div class="inspector-content">
      {#if !activeItem}
        <div class="empty-state">
          <p>{t('inspector.selectItemToInspect')}</p>
        </div>
      {:else if vault.activeInspectorTab === 'metadata'}
        <div class="metadata-view">
          <!-- Status -->
          <div class="meta-field">
            <span class="meta-label">{t('metadata.status')}</span>
            <span class="status-chip status-{activeItem.status || 'draft'}">
              {activeItem.status || 'draft'}
            </span>
          </div>

          <!-- Etiquetas / Tags -->
          <div class="meta-field">
            <span class="meta-label">{t('metadata.tags')}</span>
            <div class="tags-container">
              {#if activeItem.tags && activeItem.tags.length > 0}
                {#each activeItem.tags as tag}
                  <span class="tag-chip">#{tag}</span>
                {/each}
              {:else}
                <span class="empty-hint">{t('metadata.noTags')}</span>
              {/if}
            </div>
          </div>

          <!-- Identificador e Slug -->
          <div class="meta-field">
            <span class="meta-label">{t('metadata.slug')}</span>
            <code class="slug-code">[[{activeItem.slug}]]</code>
          </div>
        </div>
      {:else if vault.activeInspectorTab === 'backlinks'}
        <div class="backlinks-view">
          <div class="backlinks-summary">
            <span>
              {backlinks.length === 1
                ? t('inspector.singleMention')
                : t('inspector.multipleMentions', { count: backlinks.length })}
            </span>
          </div>

          {#if backlinks.length === 0}
            <div class="empty-backlinks">
              <p>{t('inspector.noBacklinksFound')}</p>
              <small>{t('inspector.backlinksHint')}</small>
            </div>
          {:else}
            <div class="backlinks-list">
              {#each backlinks as link}
                <button
                  class="backlink-card"
                  onclick={() => selectBacklinkSource(link.source_id)}
                >
                  <div class="backlink-card-header">
                    <span class="source-icon">📄</span>
                    <strong class="source-title">{link.source_title}</strong>
                    <span class="line-badge">L{link.line_number}</span>
                  </div>
                  <p class="backlink-snippet">"{link.snippet}"</p>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {:else if vault.activeInspectorTab === 'appearances'}
        <EntityAppearancesPanel appearances={worldbuilding.appearances} />
      {:else if vault.activeInspectorTab === 'gallery'}
        <EntityGallery entityId={activeItem.id} />
      {:else if vault.activeInspectorTab === 'snapshots'}
        <SnapshotsPanel {currentContent} {onRestoreContent} />
      {:else if vault.activeInspectorTab === 'context'}
        <EntityContextPanel {currentContent} />
      {:else if vault.activeInspectorTab === 'graph'}
        <LocalGraphTab centerId={activeItem.id} />
      {:else if vault.activeInspectorTab === 'style'}
        <div class="style-tab-content">
          <div class="style-summary-cards">
            <div class="style-mini-card">
              <span class="mini-label">Palavras</span>
              <span class="mini-val">{styleAnalysis.totalWords}</span>
            </div>
            <div class="style-mini-card">
              <span class="mini-label">Riqueza (TTR)</span>
              <span class="mini-val">{styleAnalysis.vocabularyRichnessPercent}%</span>
            </div>
            <div class="style-mini-card">
              <span class="mini-label">Advérbios</span>
              <span class="mini-val">{styleAnalysis.adverbsCount}</span>
            </div>
            <div class="style-mini-card">
              <span class="mini-label">Diálogos</span>
              <span class="mini-val">{styleAnalysis.dialoguePercent}%</span>
            </div>
          </div>

          <!-- Advérbios -->
          <div class="style-block">
            <div class="style-block-title">
              <span>Advérbios ({styleAnalysis.adverbsPercent}%)</span>
              <span class="style-badge">{styleAnalysis.menteAdverbsCount} em -mente</span>
            </div>
            {#if styleAnalysis.topAdverbs.length > 0}
              <div class="style-tags">
                {#each styleAnalysis.topAdverbs as adv}
                  <span class="style-tag" class:mente={adv.word.endsWith('mente')}>
                    {adv.word} <small>{adv.count}×</small>
                  </span>
                {/each}
              </div>
            {:else}
              <p class="style-empty">Nenhum advérbio detectado na cena.</p>
            {/if}
          </div>

          <!-- Verbos de Diálogo -->
          <div class="style-block">
            <div class="style-block-title">
              <span>Verbos de Diálogo</span>
              <span class="style-badge dialogue">{styleAnalysis.dialogueVerbsCount}</span>
            </div>
            {#if styleAnalysis.topDialogueVerbs.length > 0}
              <div class="style-tags">
                {#each styleAnalysis.topDialogueVerbs as v}
                  <span class="style-tag dialogue">
                    {v.word} <small>{v.count}×</small>
                  </span>
                {/each}
              </div>
            {:else}
              <p class="style-empty">Nenhuma tag de diálogo clássica identificada.</p>
            {/if}
          </div>

          <!-- Ritmo Frasal -->
          <div class="style-block">
            <div class="style-block-title">
              <span>Ritmo Frasal</span>
              <span class="style-badge">~{styleAnalysis.avgWordsPerSentence} pal/frase</span>
            </div>
            <div class="style-rhythm-list">
              <div class="rhythm-item">
                <span>Curtas (&lt; 8):</span>
                <strong>{styleAnalysis.shortSentencesCount}</strong>
              </div>
              <div class="rhythm-item">
                <span>Médias (8–20):</span>
                <strong>{styleAnalysis.mediumSentencesCount}</strong>
              </div>
              <div class="rhythm-item">
                <span>Longas (&gt; 20):</span>
                <strong>{styleAnalysis.longSentencesCount}</strong>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </aside>
{/if}

<style>
  .inspector-panel {
    width: 320px;
    height: 100%;
    background-color: var(--color-paper-elevated, #FFFDF9);
    border-left: 1px solid var(--color-paper-border, #E5E0D8);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    font-family: var(--font-ui);
    z-index: 10;
  }

  .inspector-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: var(--space-md);
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
  }

  .header-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow: hidden;
  }

  .inspector-badge {
    font-size: 11px;
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: var(--color-olive, #5E6F45);
  }

  .inspector-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-ink-charcoal, #1E231D);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .refactor-btn {
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    color: var(--color-ink-muted, #6C7368);
    transition: all 0.15s ease;
  }

  .refactor-btn:hover {
    color: var(--color-bronze, #C28D58);
    border-color: var(--color-bronze, #C28D58);
    background-color: rgba(194, 141, 88, 0.08);
  }

  .close-btn {
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    color: var(--color-ink-muted, #6C7368);
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    color: var(--color-ink-charcoal, #111827);
    border-color: var(--color-ink-faded, #9CA3AF);
    background-color: var(--color-paper-hover, rgba(0, 0, 0, 0.05));
  }

  :global(.dark) .refactor-btn,
  :global(.dark) .close-btn {
    border-color: #383C32;
    color: #A4A89E;
  }

  :global(.dark) .refactor-btn:hover {
    border-color: #D4B97F;
    color: #D4B97F;
    background-color: rgba(212, 185, 127, 0.12);
  }

  :global(.dark) .close-btn:hover {
    border-color: #555A4E;
    color: #FFFFFF;
    background-color: rgba(255, 255, 255, 0.1);
  }

  .inspector-tabs {
    display: flex;
    overflow-x: auto;
    scrollbar-width: none;
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-base);
  }

  .inspector-tabs::-webkit-scrollbar {
    display: none;
  }

  .tab-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-ink-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab-btn:hover {
    color: var(--color-ink-charcoal);
  }

  .tab-btn.active {
    color: var(--color-olive);
    border-bottom-color: var(--color-olive);
    font-weight: 600;
    background-color: var(--color-paper-elevated);
  }

  .count-pill {
    background-color: var(--color-bronze, #C28D58);
    color: #fff;
    font-size: 11px;
    padding: 1px 6px;
    border-radius: 10px;
  }

  .inspector-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md);
  }

  .meta-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: var(--space-md);
  }

  .meta-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-faded);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-chip {
    display: inline-block;
    align-self: flex-start;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    text-transform: capitalize;
  }

  .status-draft {
    background-color: rgba(194, 141, 88, 0.15);
    color: var(--color-bronze);
  }

  .status-revised {
    background-color: rgba(94, 111, 69, 0.15);
    color: var(--color-olive);
  }

  .status-final {
    background-color: rgba(46, 125, 50, 0.15);
    color: #2E7D32;
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag-chip {
    background-color: var(--color-paper-border);
    color: var(--color-ink-charcoal);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 12px;
  }

  .empty-hint {
    font-size: 12px;
    color: var(--color-ink-faded);
    font-style: italic;
  }

  .slug-code {
    font-family: var(--font-mono, monospace);
    font-size: 12px;
    color: var(--color-bronze);
    background-color: rgba(194, 141, 88, 0.1);
    padding: 4px 6px;
    border-radius: 4px;
    align-self: flex-start;
  }

  .backlinks-summary {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin-bottom: var(--space-sm);
  }

  .empty-backlinks {
    padding: var(--space-md) 0;
    text-align: center;
    color: var(--color-ink-faded);
  }

  .empty-backlinks p {
    margin: 0 0 6px 0;
    font-size: 13px;
  }

  .empty-backlinks small {
    font-size: 11px;
    line-height: 1.4;
    display: block;
  }

  .backlinks-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .backlink-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: 6px;
    padding: 10px;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .backlink-card:hover {
    border-color: var(--color-bronze);
    background-color: rgba(194, 141, 88, 0.05);
  }

  .backlink-card-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .source-title {
    flex: 1;
    font-size: 13px;
    color: var(--color-ink-charcoal);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .line-badge {
    font-size: 11px;
    color: var(--color-ink-faded);
    background-color: var(--color-paper-border);
    padding: 1px 4px;
    border-radius: 3px;
  }

  .backlink-snippet {
    margin: 0;
    font-size: 12px;
    color: var(--color-ink-muted);
    font-style: italic;
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .style-tab-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .style-summary-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }

  .style-mini-card {
    background: var(--color-paper-card, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E7EB);
    border-radius: 6px;
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
  }

  :global(.dark) .style-mini-card {
    background: #232620;
    border-color: #383C32;
  }

  .mini-label {
    font-size: 10px;
    text-transform: uppercase;
    color: var(--color-ink-faded, #6B7280);
  }

  .mini-val {
    font-size: 14px;
    font-weight: 700;
    color: var(--color-olive, #5E6F45);
  }

  :global(.dark) .mini-val {
    color: #A3BD84;
  }

  .style-block {
    background: var(--color-paper-card, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E7EB);
    border-radius: 6px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  :global(.dark) .style-block {
    background: #232620;
    border-color: #383C32;
  }

  .style-block-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    font-weight: 600;
  }

  .style-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    background: rgba(220, 38, 38, 0.1);
    color: #DC2626;
  }

  :global(.dark) .style-badge {
    color: #F87171;
  }

  .style-badge.dialogue {
    background: rgba(37, 99, 235, 0.1);
    color: #2563EB;
  }

  :global(.dark) .style-badge.dialogue {
    color: #93C5FD;
  }

  .style-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .style-tag {
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.04);
    border: 1px solid rgba(0, 0, 0, 0.06);
  }

  :global(.dark) .style-tag {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.08);
  }

  .style-tag.mente {
    background: rgba(220, 38, 38, 0.1);
    border-color: rgba(220, 38, 38, 0.2);
    color: #DC2626;
  }

  :global(.dark) .style-tag.mente {
    color: #F87171;
  }

  .style-tag.dialogue {
    background: rgba(37, 99, 235, 0.08);
    border-color: rgba(37, 99, 235, 0.15);
    color: #2563EB;
  }

  :global(.dark) .style-tag.dialogue {
    color: #93C5FD;
  }

  .style-rhythm-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 11px;
  }

  .rhythm-item {
    display: flex;
    justify-content: space-between;
    color: var(--color-ink-muted, #4B5563);
  }

  .style-empty {
    font-size: 11px;
    color: var(--color-ink-faded, #6B7280);
    font-style: italic;
    margin: 0;
  }
</style>
