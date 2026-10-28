<script lang="ts">
  import { t } from '../i18n';
  import { manuscript } from '../vault/manuscript.svelte';
  import type { EntityAppearance } from '../tauri/bridge';

  interface Props {
    appearances: EntityAppearance[];
  }

  let { appearances = [] }: Props = $props();

  function navigateToScene(sceneId: string) {
    manuscript.selectScene(sceneId);
  }
</script>

<div class="appearances-container">
  <header class="appearances-header">
    <span class="header-icon">🧭</span>
    <h3 class="header-title">{t('worldbuilding.appearancesTitle')}</h3>
    <span class="count-badge font-mono">{appearances.length}</span>
  </header>

  {#if appearances.length === 0}
    <div class="empty-state">
      <p>{t('worldbuilding.noAppearancesYet')}</p>
      <span class="hint">{t('worldbuilding.appearancesHint')}</span>
    </div>
  {:else}
    <ul class="appearances-list">
      {#each appearances as app}
        <li class="appearance-card">
          <button
            type="button"
            class="scene-nav-btn"
            onclick={() => navigateToScene(app.scene_id)}
            title={t('worldbuilding.openScene')}
          >
            <span class="scene-icon">🎬</span>
            <span class="scene-title">{app.scene_title}</span>
            <span class="line-badge font-mono">L{app.line_number}</span>
          </button>

          {#if app.snippet}
            <blockquote class="snippet-quote font-literary">
              "{app.snippet}"
            </blockquote>
          {/if}

          {#if app.alias}
            <div class="alias-pill">
              {t('worldbuilding.citedAs')}: <em>{app.alias}</em>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .appearances-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .appearances-header {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding-bottom: var(--space-xs);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .header-icon {
    font-size: 14px;
  }

  .header-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0;
    flex: 1;
  }

  .count-badge {
    background-color: var(--color-olive-subtle);
    color: var(--color-olive);
    padding: 1px 6px;
    border-radius: var(--rounded-full);
    font-size: 11px;
    font-weight: 600;
  }

  .empty-state {
    padding: var(--space-md) var(--space-sm);
    text-align: center;
    color: var(--color-ink-muted);
    font-size: 13px;
  }

  .hint {
    display: block;
    font-size: 11px;
    color: var(--color-ink-faded);
    margin-top: 4px;
  }

  .appearances-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .appearance-card {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    padding: var(--space-sm);
    display: flex;
    flex-direction: column;
    gap: 6px;
    transition: border-color 150ms ease;
  }

  .appearance-card:hover {
    border-color: var(--color-olive);
  }

  .scene-nav-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    padding: 0;
  }

  .scene-icon {
    font-size: 13px;
  }

  .scene-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    flex: 1;
  }

  .scene-title:hover {
    color: var(--color-olive);
  }

  .line-badge {
    font-size: 10px;
    color: var(--color-ink-faded);
    background-color: var(--color-paper-base);
    padding: 1px 4px;
    border-radius: var(--rounded-sm);
  }

  .snippet-quote {
    margin: 0;
    font-size: 13px;
    color: var(--color-ink-muted);
    font-style: italic;
    line-height: 1.4;
    padding-left: 8px;
    border-left: 2px solid var(--color-bronze);
  }

  .alias-pill {
    font-size: 11px;
    color: var(--color-bronze-text);
  }

  .font-mono {
    font-family: var(--font-mono);
  }

  .font-literary {
    font-family: var(--font-literary);
  }
</style>
