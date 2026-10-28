<script lang="ts">
  import { t } from '../i18n';
  import { editorViewState } from './editor-view-state.svelte';

  interface Props {
    sceneWordsCount?: number;
  }

  let { sceneWordsCount = 0 }: Props = $props();

  let todayAccumulated = $derived(
    editorViewState.productivity.today_words + editorViewState.sessionWords
  );
  let dailyTarget = $derived(editorViewState.productivity.daily_target_words || 1000);
  let progressPercent = $derived(
    Math.min(100, Math.round((todayAccumulated / Math.max(1, dailyTarget)) * 100))
  );

  function formatTime(totalSeconds: number): string {
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    if (minutes < 60) {
      return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
    }
    const hours = Math.floor(minutes / 60);
    const remMinutes = minutes % 60;
    return `${hours}h ${String(remMinutes).padStart(2, '0')}m`;
  }
</script>

{#if editorViewState.isHudVisible}
  <aside
    class="floating-hud"
    class:minimized={editorViewState.isHudMinimized}
    aria-label={t('hud.title')}
  >
    {#if editorViewState.isHudMinimized}
      <button
        type="button"
        class="minimized-pill"
        onclick={() => editorViewState.toggleHudMinimized()}
        title={t('hud.expand')}
      >
        <span class="pill-metric">✍️ {sceneWordsCount}</span>
        <span class="pill-dot">·</span>
        <span class="pill-progress">{progressPercent}%</span>
        <span class="pill-icon">▲</span>
      </button>
    {:else}
      <div class="hud-card">
        <header class="hud-header">
          <div class="hud-title">
            <span class="hud-mark">◈</span>
            <span>{t('hud.title')}</span>
          </div>
          <div class="hud-actions">
            <button
              type="button"
              class="hud-icon-btn"
              onclick={() => editorViewState.openProductivityDashboard()}
              title={t('productivity.dashboardTitle')}
            >
              📊
            </button>
            <button
              type="button"
              class="hud-icon-btn"
              onclick={() => editorViewState.toggleHudMinimized()}
              title={t('hud.minimize')}
            >
              −
            </button>
          </div>
        </header>

        <div class="hud-body">
          <div class="metric-row">
            <span class="metric-label">{t('hud.sceneWords')}</span>
            <span class="metric-value font-mono">{sceneWordsCount.toLocaleString()}</span>
          </div>

          <div class="metric-row">
            <span class="metric-label">{t('hud.sessionWords')}</span>
            <span class="metric-value font-mono">+{editorViewState.sessionWords.toLocaleString()}</span>
          </div>

          <div class="metric-row">
            <span class="metric-label">{t('hud.sessionTime')}</span>
            <span class="metric-value font-mono">{formatTime(editorViewState.sessionSeconds)}</span>
          </div>

          <div class="hud-progress-block">
            <div class="progress-labels">
              <span class="progress-title">{t('hud.dailyGoal')}</span>
              <span class="progress-ratio font-mono">
                {todayAccumulated} / {dailyTarget} ({progressPercent}%)
              </span>
            </div>
            <div class="progress-bar-track">
              <div
                class="progress-bar-fill"
                style:width="{progressPercent}%"
              ></div>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </aside>
{/if}

<style>
  .floating-hud {
    position: fixed;
    right: var(--space-lg);
    bottom: var(--space-lg);
    z-index: 90;
    user-select: none;
    font-family: var(--font-ui);
    transition: all var(--transition-focus);
  }

  .minimized-pill {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-full);
    padding: 6px 14px;
    box-shadow: var(--shadow-hud);
    color: var(--color-ink-charcoal);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 150ms ease;
  }

  .minimized-pill:hover {
    background-color: var(--color-paper-hover);
    border-color: var(--color-olive);
  }

  .pill-metric {
    font-family: var(--font-mono);
  }

  .pill-dot {
    color: var(--color-ink-faded);
  }

  .pill-progress {
    color: var(--color-olive);
    font-weight: 600;
  }

  .pill-icon {
    font-size: 10px;
    color: var(--color-ink-faded);
    margin-left: 2px;
  }

  .hud-card {
    width: 250px;
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    box-shadow: var(--shadow-hud);
    backdrop-filter: blur(8px);
    overflow: hidden;
  }

  .hud-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    background-color: var(--color-paper-surface);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .hud-title {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-olive);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .hud-mark {
    font-size: 11px;
  }

  .hud-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .hud-icon-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 13px;
    line-height: 1;
    padding: 3px 6px;
    border-radius: var(--rounded-sm);
    color: var(--color-ink-muted);
  }

  .hud-icon-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .hud-body {
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .metric-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
  }

  .metric-label {
    color: var(--color-ink-muted);
  }

  .metric-value {
    color: var(--color-ink-charcoal);
    font-weight: 600;
    font-size: 13px;
  }

  .hud-progress-block {
    margin-top: var(--space-xs);
    padding-top: var(--space-xs);
    border-top: 1px solid var(--color-paper-border);
  }

  .progress-labels {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    margin-bottom: 4px;
  }

  .progress-title {
    color: var(--color-ink-muted);
    font-weight: 500;
  }

  .progress-ratio {
    color: var(--color-olive);
    font-weight: 600;
  }

  .progress-bar-track {
    width: 100%;
    height: 6px;
    background-color: var(--color-paper-surface);
    border-radius: var(--rounded-full);
    overflow: hidden;
    border: 1px solid var(--color-paper-border);
  }

  .progress-bar-fill {
    height: 100%;
    background-color: var(--color-olive);
    border-radius: var(--rounded-full);
    transition: width 250ms ease;
  }

  .font-mono {
    font-family: var(--font-mono);
  }
</style>
