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
    projectTargetInput = editorViewState.productivity.project_target_words;
    dailyTargetInput = editorViewState.productivity.daily_target_words;
    deadlineInput = editorViewState.productivity.project_deadline || '';
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
</script>

<div class="productivity-content">
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

<style>
  .productivity-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: var(--space-sm);
  }

  .metric-card {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    padding: var(--space-sm) var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .card-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-ink-muted);
  }

  .card-value {
    font-size: 17px;
    font-weight: 700;
    color: var(--color-ink-charcoal);
  }

  .card-sub {
    font-size: 11px;
    color: var(--color-ink-faded);
  }

  .card-progress {
    height: 5px;
    background-color: var(--color-paper-border);
    border-radius: var(--rounded-full);
    overflow: hidden;
    margin-top: 2px;
  }

  .progress-fill {
    height: 100%;
    background-color: var(--color-olive);
    border-radius: var(--rounded-full);
    transition: width var(--transition-normal);
  }

  .chart-section {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .section-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin: 0;
  }

  .bars-container {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    height: 130px;
    gap: var(--space-xs);
    padding-top: var(--space-md);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .chart-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
    gap: 4px;
  }

  .column-count {
    font-size: 10px;
    color: var(--color-ink-faded);
  }

  .bar-slot {
    flex: 1;
    width: 100%;
    max-width: 32px;
    display: flex;
    align-items: flex-end;
  }

  .bar-rect {
    width: 100%;
    background-color: var(--color-paper-border);
    border-radius: var(--rounded-sm) var(--rounded-sm) 0 0;
    transition: height var(--transition-normal), background-color var(--transition-fast);
  }

  .bar-rect.met-target {
    background-color: var(--color-olive);
  }

  .column-label {
    font-size: 10px;
    color: var(--color-ink-muted);
  }

  .targets-form-section {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .targets-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .form-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--space-sm);
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-group label {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-ink-muted);
  }

  .field-group input {
    font-family: var(--font-ui);
    font-size: 13px;
    padding: 6px 10px;
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    background-color: var(--color-paper-base);
    color: var(--color-ink-charcoal);
  }

  .field-group input:focus {
    outline: none;
    border-color: var(--color-olive);
  }

  .form-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-md);
  }

  .feedback-msg {
    font-size: 12px;
    color: var(--color-olive);
    font-weight: 500;
  }

  .save-targets-btn {
    background-color: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--rounded-default);
    padding: 7px 16px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }

  .save-targets-btn:hover:not(:disabled) {
    background-color: var(--color-olive-hover);
  }

  .save-targets-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
