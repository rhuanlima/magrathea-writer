<script lang="ts">
  /**
   * Modal de Análise Estilométrica e Estatísticas de Escrita.
   *
   * Apresenta ao autor métricas avançadas sobre sua prosa:
   * - Densidade e repetições de advérbios (especialmente em '-mente')
   * - Verbos de diálogo e fórmulas de elocução (disse, perguntou, respondeu)
   * - Verbos mais frequentes na narrativa
   * - Riqueza vocabular (TTR) e ritmo de frases (curtas/médias/longas)
   * - Proporção de diálogos vs texto narrativo
   */

  import { analyzeProseStyle, type ProseStyleAnalysis } from './text-analysis';

  interface Props {
    isOpen: boolean;
    text: string;
    sceneTitle?: string;
    onClose: () => void;
  }

  let { isOpen = false, text = '', sceneTitle = '', onClose }: Props = $props();

  let analysis: ProseStyleAnalysis = $derived(analyzeProseStyle(text));

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={onClose}>
    <div
      class="modal-card"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <header class="modal-header">
        <div class="title-group">
          <span class="header-icon">🔬</span>
          <div>
            <h2>Estatísticas Estilométricas & Prosa</h2>
            <p class="subtitle">
              {sceneTitle ? `Análise da cena: ${sceneTitle}` : 'Diagnóstico estilístico em tempo real'}
            </p>
          </div>
        </div>
        <button type="button" class="close-btn" onclick={onClose} title="Fechar (Esc)">✕</button>
      </header>

      <div class="modal-body">
        {#if analysis.totalWords === 0}
          <div class="empty-state">
            <span class="empty-icon">✍️</span>
            <h3>Nenhum texto para analisar ainda</h3>
            <p>Comece a escrever no editor para visualizar as métricas de advérbios, verbos e ritmo.</p>
          </div>
        {:else}
          <!-- Resumo Superior em Cartões -->
          <div class="stats-grid">
            <div class="stat-card">
              <span class="stat-label">Total de Palavras</span>
              <span class="stat-value">{analysis.totalWords}</span>
              <span class="stat-hint">{analysis.totalChars} caracteres</span>
            </div>

            <div class="stat-card">
              <span class="stat-label">Riqueza Lexical</span>
              <span class="stat-value">{analysis.vocabularyRichnessPercent}%</span>
              <span class="stat-hint">{analysis.uniqueWords} vocábulos únicos</span>
            </div>

            <div class="stat-card">
              <span class="stat-label">Ritmo Médio</span>
              <span class="stat-value">{analysis.avgWordsPerSentence}</span>
              <span class="stat-hint">palavras por frase</span>
            </div>

            <div class="stat-card">
              <span class="stat-label">Tempo de Leitura</span>
              <span class="stat-value">~{analysis.estimatedReadingTimeMinutes} min</span>
              <span class="stat-hint">a 200 palavras/min</span>
            </div>
          </div>

          <!-- Seção 1: Advérbios (Show, Don't Tell) -->
          <section class="analysis-section">
            <div class="section-header">
              <div class="section-title">
                <span class="badge badge-adverb">Advérbios</span>
                <h3>Densidade e Advérbios em "-mente"</h3>
              </div>
              <span class="section-counter">
                {analysis.adverbsCount} no total ({analysis.adverbsPercent}%)
              </span>
            </div>

            <p class="section-desc">
              Advérbios modais terminados em <em>-mente</em> (ex: <em>silenciosamente</em>, <em>rapidamente</em>)
              costumam enfraquecer verbos de ação na ficção. Stephen King e a tradição literária recomendam moderação.
            </p>

            <div class="adverbs-summary">
              <div class="metric-pill">
                <strong>{analysis.menteAdverbsCount}</strong> terminados em <em>-mente</em>
              </div>
              <div class="metric-pill">
                <strong>{analysis.adverbsCount - analysis.menteAdverbsCount}</strong> outros advérbios
              </div>
            </div>

            {#if analysis.topAdverbs.length > 0}
              <div class="frequency-list">
                <span class="freq-heading">Advérbios mais recorrentes:</span>
                <div class="tags-cloud">
                  {#each analysis.topAdverbs as adv}
                    <span class="tag-pill" class:tag-mente={adv.word.endsWith('mente')}>
                      <span class="tag-text">{adv.word}</span>
                      <span class="tag-count">{adv.count}×</span>
                    </span>
                  {/each}
                </div>
              </div>
            {/if}
          </section>

          <!-- Seção 2: Verbos de Diálogo e Elocução -->
          <section class="analysis-section">
            <div class="section-header">
              <div class="section-title">
                <span class="badge badge-dialogue">Diálogos</span>
                <h3>Verbos de Elocução (Dialogue Tags)</h3>
              </div>
              <span class="section-counter">
                {analysis.dialogueVerbsCount} verbos de fala
              </span>
            </div>

            <p class="section-desc">
              Verbos como <em>disse</em>, <em>perguntou</em> e <em>respondeu</em> conduzem os diálogos.
              Monitore a repetição para manter as falas dinâmicas e naturais.
            </p>

            {#if analysis.topDialogueVerbs.length > 0}
              <div class="frequency-list">
                <span class="freq-heading">Fórmulas de diálogo identificadas:</span>
                <div class="tags-cloud">
                  {#each analysis.topDialogueVerbs as v}
                    <span class="tag-pill tag-verb">
                      <span class="tag-text">{v.word}</span>
                      <span class="tag-count">{v.count}×</span>
                    </span>
                  {/each}
                </div>
              </div>
            {:else}
              <div class="inline-empty">Nenhum verbo clássico de diálogo detectado nesta cena.</div>
            {/if}
          </section>

          <!-- Seção 3: Verbos Mais Usados na Narrativa -->
          {#if analysis.topVerbs.length > 0}
            <section class="analysis-section">
              <div class="section-header">
                <div class="section-title">
                  <span class="badge badge-action">Ação</span>
                  <h3>Verbos mais Frequentes na Prosa</h3>
                </div>
              </div>
              <div class="tags-cloud">
                {#each analysis.topVerbs as v}
                  <span class="tag-pill tag-action">
                    <span class="tag-text">{v.word}</span>
                    <span class="tag-count">{v.count}×</span>
                  </span>
                {/each}
              </div>
            </section>
          {/if}

          <!-- Seção 4: Proporção Diálogo vs Narrativa & Ritmo de Frases -->
          <div class="two-cols">
            <!-- Proporção Diálogo vs Narrativa -->
            <section class="analysis-section sub-box">
              <div class="section-header">
                <h3>Voz dos Personagens vs Descrição</h3>
              </div>
              <div class="ratio-bar">
                <div
                  class="ratio-segment ratio-dialogue"
                  style="width: {analysis.dialoguePercent}%"
                  title="Diálogo: {analysis.dialoguePercent}%"
                ></div>
                <div
                  class="ratio-segment ratio-narrative"
                  style="width: {analysis.narrativePercent}%"
                  title="Narrativa: {analysis.narrativePercent}%"
                ></div>
              </div>
              <div class="ratio-labels">
                <span>💬 Diálogos: <strong>{analysis.dialoguePercent}%</strong> ({analysis.dialogueWords} palavras)</span>
                <span>📖 Narrativa: <strong>{analysis.narrativePercent}%</strong> ({analysis.narrativeWords} palavras)</span>
              </div>
            </section>

            <!-- Ritmo Frasal -->
            <section class="analysis-section sub-box">
              <div class="section-header">
                <h3>Comprimento de Frases</h3>
              </div>
              <div class="sentence-bars">
                <div class="sentence-row">
                  <span class="s-label">Curtas (&lt; 8 palavras):</span>
                  <span class="s-val">{analysis.shortSentencesCount}</span>
                </div>
                <div class="sentence-row">
                  <span class="s-label">Médias (8–20 palavras):</span>
                  <span class="s-val">{analysis.mediumSentencesCount}</span>
                </div>
                <div class="sentence-row">
                  <span class="s-label">Longas (&gt; 20 palavras):</span>
                  <span class="s-val">{analysis.longSentencesCount}</span>
                </div>
              </div>
            </section>
          </div>
        {/if}
      </div>

      <footer class="modal-footer">
        <button type="button" class="btn btn-secondary" onclick={onClose}>
          Fechar
        </button>
      </footer>
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
    z-index: var(--z-modal, 1000);
    padding: var(--space-md);
  }

  .modal-card {
    background: var(--color-paper-card, #FFFFFF);
    color: var(--color-ink-charcoal, #111827);
    border: 1px solid var(--color-paper-border, #E5E7EB);
    border-radius: var(--radius-lg, 12px);
    width: 100%;
    max-width: 760px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }

  :global(.dark) .modal-card {
    background: #232620;
    color: #F3F4F6;
    border-color: #383C32;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--color-paper-border, #E5E7EB);
  }

  :global(.dark) .modal-header {
    border-bottom-color: #383C32;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .header-icon {
    font-size: 1.5rem;
  }

  .title-group h2 {
    font-size: 1.15rem;
    font-weight: 600;
    margin: 0;
  }

  .subtitle {
    font-size: 0.8rem;
    color: var(--color-ink-faded, #6B7280);
    margin: 2px 0 0;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--color-paper-border, #E5E7EB);
    font-size: 0.95rem;
    cursor: pointer;
    color: var(--color-ink-faded, #6B7280);
    border-radius: 8px;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: var(--color-paper-hover, rgba(0, 0, 0, 0.05));
    border-color: var(--color-ink-faded, #9CA3AF);
    color: var(--color-ink-charcoal, #111827);
  }

  :global(.dark) .close-btn {
    border-color: #383C32;
    color: #A4A89E;
  }

  :global(.dark) .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #FFFFFF;
    border-color: #555A4E;
  }

  .modal-body {
    padding: var(--space-lg);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .empty-state {
    text-align: center;
    padding: var(--space-xl) var(--space-md);
    color: var(--color-ink-faded, #6B7280);
  }

  .empty-icon {
    font-size: 2.5rem;
    display: block;
    margin-bottom: var(--space-sm);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--space-sm);
  }

  .stat-card {
    background: var(--color-paper-subtle, #F9FAFB);
    border: 1px solid var(--color-paper-border, #E5E7EB);
    border-radius: 8px;
    padding: var(--space-sm) var(--space-md);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  :global(.dark) .stat-card {
    background: #1B1D19;
    border-color: #31352B;
  }

  .stat-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-ink-faded, #6B7280);
  }

  .stat-value {
    font-size: 1.35rem;
    font-weight: 700;
    color: var(--color-olive, #5E6F45);
  }

  :global(.dark) .stat-value {
    color: #A3BD84;
  }

  .stat-hint {
    font-size: 0.75rem;
    color: var(--color-ink-faded, #6B7280);
  }

  .analysis-section {
    background: var(--color-paper-subtle, #F9FAFB);
    border: 1px solid var(--color-paper-border, #E5E7EB);
    border-radius: 8px;
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  :global(.dark) .analysis-section {
    background: #1B1D19;
    border-color: #31352B;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-sm);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .section-title h3 {
    font-size: 0.95rem;
    font-weight: 600;
    margin: 0;
  }

  .badge {
    font-size: 0.7rem;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .badge-adverb {
    background: rgba(220, 38, 38, 0.12);
    color: #DC2626;
  }

  .badge-dialogue {
    background: rgba(37, 99, 235, 0.12);
    color: #2563EB;
  }

  .badge-action {
    background: rgba(16, 185, 129, 0.12);
    color: #10B981;
  }

  .section-counter {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-ink-faded, #6B7280);
  }

  .section-desc {
    font-size: 0.8rem;
    color: var(--color-ink-faded, #6B7280);
    line-height: 1.4;
    margin: 0;
  }

  .adverbs-summary {
    display: flex;
    gap: var(--space-sm);
    margin-top: 4px;
  }

  .metric-pill {
    font-size: 0.8rem;
    background: rgba(0, 0, 0, 0.04);
    padding: 4px 10px;
    border-radius: 6px;
  }

  :global(.dark) .metric-pill {
    background: rgba(255, 255, 255, 0.06);
  }

  .frequency-list {
    margin-top: var(--space-xs);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .freq-heading {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-ink-faded, #6B7280);
  }

  .tags-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 0.8rem;
    background: rgba(0, 0, 0, 0.05);
    border: 1px solid rgba(0, 0, 0, 0.08);
  }

  :global(.dark) .tag-pill {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .tag-mente {
    background: rgba(220, 38, 38, 0.1);
    border-color: rgba(220, 38, 38, 0.25);
    color: #DC2626;
  }

  :global(.dark) .tag-mente {
    background: rgba(239, 68, 68, 0.15);
    color: #F87171;
  }

  .tag-verb {
    background: rgba(37, 99, 235, 0.08);
    border-color: rgba(37, 99, 235, 0.2);
    color: #2563EB;
  }

  :global(.dark) .tag-verb {
    background: rgba(59, 130, 246, 0.15);
    color: #93C5FD;
  }

  .tag-action {
    background: rgba(16, 185, 129, 0.08);
    border-color: rgba(16, 185, 129, 0.2);
    color: #059669;
  }

  :global(.dark) .tag-action {
    background: rgba(16, 185, 129, 0.15);
    color: #6EE7B7;
  }

  .tag-count {
    font-size: 0.7rem;
    font-weight: 700;
    opacity: 0.85;
  }

  .two-cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-md);
  }

  @media (max-width: 600px) {
    .two-cols {
      grid-template-columns: 1fr;
    }
  }

  .sub-box {
    margin: 0;
  }

  .ratio-bar {
    height: 10px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.1);
    display: flex;
    overflow: hidden;
    margin: 6px 0;
  }

  .ratio-segment {
    height: 100%;
    transition: width 0.3s ease;
  }

  .ratio-dialogue {
    background: #3B82F6;
  }

  .ratio-narrative {
    background: #10B981;
  }

  .ratio-labels {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.75rem;
    color: var(--color-ink-faded, #6B7280);
  }

  .sentence-bars {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .sentence-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
  }

  .s-label {
    color: var(--color-ink-faded, #6B7280);
  }

  .s-val {
    font-weight: 700;
  }

  .inline-empty {
    font-size: 0.8rem;
    color: var(--color-ink-faded, #6B7280);
    font-style: italic;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: var(--space-sm) var(--space-lg);
    border-top: 1px solid var(--color-paper-border, #E5E7EB);
  }

  :global(.dark) .modal-footer {
    border-top-color: #383C32;
  }

  .btn {
    padding: 6px 16px;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
    border: 1px solid transparent;
    font-weight: 500;
  }

  .btn-secondary {
    background: transparent;
    border-color: var(--color-paper-border, #E5E7EB);
    color: inherit;
  }

  :global(.dark) .btn-secondary {
    border-color: #383C32;
  }

  .btn-secondary:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  :global(.dark) .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.08);
  }
</style>
