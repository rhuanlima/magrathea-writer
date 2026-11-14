<script lang="ts">
  /**
   * Ajuda rápida do editor: popover com atalhos, sintaxe Markdown e dicas, aberto pelo botão "?"
   * do rodapé ou por Cmd/Ctrl+/. Não bloqueia a escrita: fecha com Esc, clique fora ou no botão.
   */
  import { t } from '../i18n';
  import { HELP_SECTIONS, formatShortcut, type HelpRow } from './editor-help';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen = false, onClose }: Props = $props();

  let closeButton = $state<HTMLButtonElement | null>(null);

  const label = (row: HelpRow) => t(`editorHelp.${row.id}` as const);

  $effect(() => {
    if (isOpen) closeButton?.focus();
  });

  function handleKeydown(e: KeyboardEvent) {
    if (isOpen && e.key === 'Escape') {
      e.stopPropagation();
      onClose();
    }
  }
</script>

<svelte:window onkeydowncapture={handleKeydown} />

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="help-catcher" onclick={onClose}></div>
  <div class="help-popover" role="dialog" aria-labelledby="editor-help-title">
    <header class="help-header">
      <h2 id="editor-help-title">{t('editorHelp.title')}</h2>
      <button
        type="button"
        class="help-close"
        bind:this={closeButton}
        onclick={onClose}
        aria-label={t('common.close')}
      >
        ✕
      </button>
    </header>

    <div class="help-body">
      {#each HELP_SECTIONS as section (section.id)}
        <section class="help-section">
          <h3>{t(`editorHelp.section_${section.id}` as const)}</h3>
          <dl>
            {#each section.rows as row (row.id)}
              <div class="help-row">
                <dt>{label(row)}</dt>
                <dd>
                  {#if row.keys}
                    {#each formatShortcut(row.keys) as part, i (i)}<kbd>{part}</kbd>{/each}
                  {:else}
                    <code>{row.syntax}</code>
                  {/if}
                </dd>
              </div>
            {/each}
          </dl>
        </section>
      {/each}

      <section class="help-section">
        <h3>{t('editorHelp.section_tips')}</h3>
        <ul class="help-tips">
          <li>{t('editorHelp.tipModes')}</li>
          <li>{t('editorHelp.tipMarkers')}</li>
          <li>{t('editorHelp.tipDialogue')}</li>
        </ul>
      </section>
    </div>
  </div>
{/if}

<style>
  .help-catcher {
    position: fixed;
    inset: 0;
    z-index: calc(var(--z-modal, 1000) - 1);
  }

  .help-popover {
    position: fixed;
    right: var(--space-md, 16px);
    bottom: 48px;
    width: min(420px, calc(100vw - 32px));
    max-height: min(70vh, 640px);
    display: flex;
    flex-direction: column;
    z-index: var(--z-modal, 1000);
    background: var(--color-paper-card, #ffffff);
    color: var(--color-ink-charcoal, #111827);
    border: 1px solid var(--color-paper-border, #e5e7eb);
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.22);
    font-family: var(--font-ui);
    overflow: hidden;
  }

  :global(.dark) .help-popover {
    background: #232620;
    color: #f3f4f6;
    border-color: #383c32;
  }

  .help-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border-bottom: 1px solid var(--color-paper-border, #e5e7eb);
  }

  :global(.dark) .help-header {
    border-bottom-color: #383c32;
  }

  .help-header h2 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .help-close {
    border: none;
    background: transparent;
    color: var(--color-ink-muted, #64748b);
    cursor: pointer;
    font-size: 0.85rem;
    border-radius: 4px;
    padding: 2px 6px;
  }

  .help-close:hover {
    color: var(--color-ink-charcoal, #111827);
    background: var(--color-paper-hover, rgba(0, 0, 0, 0.05));
  }

  .help-body {
    overflow-y: auto;
    padding: 4px 14px 12px;
  }

  .help-section h3 {
    margin: 12px 0 4px;
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-olive, #5e6f45);
  }

  :global(.dark) .help-section h3 {
    color: #a3bd84;
  }

  dl {
    margin: 0;
  }

  .help-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 4px 0;
    font-size: 0.8rem;
  }

  dt {
    color: var(--color-ink-muted, #64748b);
  }

  dd {
    margin: 0;
    display: flex;
    gap: 3px;
    flex-shrink: 0;
  }

  kbd,
  code {
    font-family: var(--font-mono, monospace);
    font-size: 0.74rem;
    padding: 1px 6px;
    border-radius: 4px;
    border: 1px solid var(--color-paper-border, #e5e7eb);
    background: var(--color-paper-surface, #f8f9fa);
    color: inherit;
  }

  :global(.dark) kbd,
  :global(.dark) code {
    background: #2b2f27;
    border-color: #383c32;
  }

  .help-tips {
    margin: 0;
    padding-left: 18px;
    font-size: 0.78rem;
    line-height: 1.5;
    color: var(--color-ink-muted, #64748b);
  }

  .help-tips li + li {
    margin-top: 4px;
  }
</style>
