<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '../i18n';
  import { editorViewState } from '../editor/editor-view-state.svelte';

  let projectTargetInput = $state(50000);
  let dailyTargetInput = $state(1000);
  let deadlineInput = $state('');
  let isSaving = $state(false);
  let saveFeedback = $state(false);

  onMount(() => {
    projectTargetInput = editorViewState.productivity.project_target_words;
    dailyTargetInput = editorViewState.productivity.daily_target_words;
    deadlineInput = editorViewState.productivity.project_deadline || '';
  });

  $effect(() => {
    if (editorViewState.isProductivityModalOpen) {
      projectTargetInput = editorViewState.productivity.project_target_words;
      dailyTargetInput = editorViewState.productivity.daily_target_words;
      deadlineInput = editorViewState.productivity.project_deadline || '';
      saveFeedback = false;
    }
  });

  let totalWords = $derived(editorViewState.productivity.total_manuscript_words);
  let projectPercent = $derived(
    Math.min(
      100,
      Math.round(
        (totalWords / Math.max(1, editorViewState.productivity.project_target_words)) * 100
      )
    )
  );

  let todayWords = $derived(
    editorViewState.productivity.today_words + editorViewState.sessionWords
  );
  let dailyPercent = $derived(
    Math.min(
      100,
      Math.round(
        (todayWords / Math.max(1, editorViewState.productivity.daily_target_words)) * 100
      )
    )
  );

  let remainingWords = $derived(
    Math.max(0, editorViewState.productivity.project_target_words - totalWords)
  );

  let estimatedDays = $derived.by(() => {
    const avg = editorViewState.productivity.seven_days_avg_words;
    if (avg <= 0 || remainingWords <= 0) return null;
    return Math.ceil(remainingWords / avg);
  });

  // Monta histórico dos últimos 7 dias para visualização em barras SVG
  let last7DaysData = $derived.by(() => {
    const days = [];
    const today = new Date();
    for (let i = 6; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const year = d.getFullYear();
      const month = String(d.getMonth() + 1).padStart(2, '0');
      const day = String(d.getDate()).padStart(2, '0');
      const isoDate = `${year}-${month}-${day}`;
      const log = editorViewState.productivity.daily_logs[isoDate];
      let words = log ? log.words_written : 0;
      if (i === 0) {
        words += editorViewState.sessionWords;
      }
      const label = `${day}/${month}`;
      days.push({ isoDate, label, words });
    }
    return days;
  });

  async function handleSaveTargets(e: SubmitEvent) {
    e.preventDefault();
    isSaving = true;
    try {
      await editorViewState.updateTargets(
        projectTargetInput,
        deadlineInput ? deadlineInput : null,
        dailyTargetInput
      );
      saveFeedback = true;
      setTimeout(() => {
        saveFeedback = false;
      }, 3000);
    } catch (err) {
      console.error('Erro ao salvar metas:', err);
    } finally {
      isSaving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && editorViewState.isProductivityModalOpen) {
      editorViewState.closeProductivityDashboard();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if editorViewState.isProductivityModalOpen}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="dash-title">
    <div class="dashboard-dialog">
      <header class="dialog-header">
        <div class="header-branding">
          <span class="header-icon">📊</span>
          <h2 id="dash-title" class="dialog-title">{t('productivity.dashboardTitle')}</h2>
        </div>
        <button
          type="button"
          class="dialog-close-btn"
          onclick={() => editorViewState.closeProductivityDashboard()}
          aria-label={t('common.close')}
        >
          ✕
        </button>
      </header>

      <div class="dialog-content">
        <!-- Grid de métricas analíticas -->
        <div class="metrics-grid">
          <div class="metric-card">
            <span class="card-label">{t('productivity.projectTarget')}</span>
            <div class="card-value font-mono">
              {totalWords.toLocaleString()} / {editorViewState.productivity.project_target_words.toLocaleString()}
            </div>
            <div class="card-sub">{projectPercent}% {t('productivity.completed')}</div>
            <div class="card-progress">
              <div class="progress-fill" style:width="{projectPercent}%"></div>
            </div>
          </div>

          <div class="metric-card">
            <span class="card-label">{t('productivity.todayGoal')}</span>
            <div class="card-value font-mono">
              {todayWords.toLocaleString()} / {editorViewState.productivity.daily_target_words.toLocaleString()}
            </div>
            <div class="card-sub">{dailyPercent}% {t('productivity.completed')}</div>
            <div class="card-progress">
              <div class="progress-fill" style:width="{dailyPercent}%"></div>
            </div>
          </div>

          <div class="metric-card">
            <span class="card-label">{t('productivity.sevenDaysAvg')}</span>
            <div class="card-value font-mono">
              {Math.round(editorViewState.productivity.seven_days_avg_words).toLocaleString()}
            </div>
            <div class="card-sub">{t('productivity.wordsPerDay')}</div>
          </div>

          <div class="metric-card">
            <span class="card-label">{t('productivity.estimatedFinish')}</span>
            <div class="card-value font-mono">
              {#if estimatedDays !== null}
                ≈ {estimatedDays} {t('productivity.daysRemaining')}
              {:else if remainingWords === 0}
                🎉 {t('productivity.goalReached')}
              {:else}
                —
              {/if}
            </div>
            <div class="card-sub">
              {remainingWords.toLocaleString()} {t('productivity.wordsRemaining')}
            </div>
          </div>
        </div>

        <!-- Gráfico dos últimos 7 dias -->
        <section class="chart-section">
          <h3 class="section-title">{t('productivity.last7DaysChart')}</h3>
          <div class="bars-container">
            {#each last7DaysData as day}
              {@const maxScale = Math.max(editorViewState.productivity.daily_target_words * 1.2, 500)}
              {@const barHeight = Math.min(100, Math.round((day.words / maxScale) * 100))}
              {@const metTarget = day.words >= editorViewState.productivity.daily_target_words}
              <div class="chart-column">
                <span class="column-count font-mono">{day.words > 0 ? day.words : '0'}</span>
                <div class="bar-slot">
                  <div
                    class="bar-rect"
                    class:met-target={metTarget}
                    style:height="{Math.max(4, barHeight)}%"
                    title="{day.label}: {day.words} palavras"
                  ></div>
                </div>
                <span class="column-label">{day.label}</span>
              </div>
            {/each}
          </div>
        </section>

        <!-- Formulário de configuração de metas -->
        <section class="targets-form-section">
          <h3 class="section-title">{t('productivity.configureTargets')}</h3>
          <form onsubmit={handleSaveTargets} class="targets-form">
            <div class="form-row">
              <div class="field-group">
                <label for="proj-target-input">{t('productivity.projectTargetWords')}</label>
                <input
                  id="proj-target-input"
                  type="number"
                  min="1000"
                  step="1000"
                  bind:value={projectTargetInput}
                  required
                />
              </div>

              <div class="field-group">
                <label for="daily-target-input">{t('productivity.dailyTargetWords')}</label>
                <input
                  id="daily-target-input"
                  type="number"
                  min="100"
                  step="50"
                  bind:value={dailyTargetInput}
                  required
                />
              </div>

              <div class="field-group">
                <label for="deadline-input">{t('productivity.deadlineOptional')}</label>
                <input
                  id="deadline-input"
                  type="date"
                  bind:value={deadlineInput}
                />
              </div>
            </div>

            <div class="form-actions">
              {#if saveFeedback}
                <span class="feedback-msg">✓ {t('productivity.savedSuccess')}</span>
              {/if}
              <button type="submit" class="save-targets-btn" disabled={isSaving}>
                {isSaving ? t('common.saving') : t('productivity.saveTargetsButton')}
              </button>
            </div>
          </form>
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(30, 35, 29, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: var(--space-md);
  }

  .dashboard-dialog {
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    box-shadow: var(--shadow-modal);
    width: 100%;
    max-width: 680px;
    max-height: 90vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .header-branding {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .header-icon {
    font-size: 18px;
  }

  .dialog-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin: 0;
  }

  .dialog-close-btn {
    width: 32px;
    height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    background: transparent;
    cursor: pointer;
    font-size: 14px;
    color: var(--color-ink-muted, #6B7280);
    border-radius: 8px;
    transition: all 0.15s ease;
  }

  .dialog-close-btn:hover {
    background-color: var(--color-paper-hover, rgba(0, 0, 0, 0.05));
    color: var(--color-ink-charcoal, #111827);
    border-color: var(--color-ink-faded, #9CA3AF);
  }

  :global(.dark) .dialog-close-btn {
    border-color: #383C32;
    color: #A4A89E;
  }

  :global(.dark) .dialog-close-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: #FFFFFF;
    border-color: #555A4E;
  }

  .dialog-content {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-md);
  }

  .metric-card {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .card-label {
    font-size: 12px;
    color: var(--color-ink-muted);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .card-value {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .card-sub {
    font-size: 12px;
    color: var(--color-olive);
    font-weight: 500;
  }

  .card-progress {
    height: 4px;
    background-color: var(--color-paper-border);
    border-radius: var(--rounded-full);
    overflow: hidden;
    margin-top: 4px;
  }

  .progress-fill {
    height: 100%;
    background-color: var(--color-olive);
    border-radius: var(--rounded-full);
    transition: width 200ms ease;
  }

  .section-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 var(--space-sm) 0;
  }

  .chart-section {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    padding: var(--space-md);
  }

  .bars-container {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    height: 140px;
    padding-top: var(--space-sm);
    gap: var(--space-sm);
  }

  .chart-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
    justify-content: flex-end;
  }

  .column-count {
    font-size: 10px;
    color: var(--color-ink-faded);
    margin-bottom: 4px;
  }

  .bar-slot {
    width: 100%;
    max-width: 32px;
    height: 90px;
    display: flex;
    align-items: flex-end;
    background-color: var(--color-paper-base);
    border-radius: var(--rounded-sm);
    overflow: hidden;
  }

  .bar-rect {
    width: 100%;
    background-color: var(--color-ink-faded);
    border-radius: var(--rounded-sm);
    transition: height 250ms ease, background-color 200ms ease;
  }

  .bar-rect.met-target {
    background-color: var(--color-olive);
  }

  .column-label {
    font-size: 11px;
    color: var(--color-ink-muted);
    margin-top: 6px;
  }

  .targets-form-section {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    padding: var(--space-md);
  }

  .form-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-md);
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-group label {
    font-size: 12px;
    color: var(--color-ink-muted);
    font-weight: 500;
  }

  .field-group input {
    padding: 8px 10px;
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    background-color: var(--color-paper-elevated);
    color: var(--color-ink-charcoal);
    font-size: 13px;
    font-family: var(--font-ui);
    box-sizing: border-box;
  }

  .field-group input:focus {
    outline: none;
    border-color: var(--color-olive);
    box-shadow: 0 0 0 2px var(--color-olive-subtle);
  }

  .form-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-md);
    margin-top: var(--space-md);
  }

  .feedback-msg {
    font-size: 13px;
    color: var(--color-olive);
    font-weight: 600;
  }

  .save-targets-btn {
    background-color: var(--color-olive);
    color: var(--color-paper-elevated);
    border: none;
    padding: 8px 16px;
    border-radius: var(--rounded-default);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 150ms ease;
  }

  .save-targets-btn:hover:not(:disabled) {
    background-color: var(--color-olive-hover);
  }

  .save-targets-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .font-mono {
    font-family: var(--font-mono);
  }
</style>
