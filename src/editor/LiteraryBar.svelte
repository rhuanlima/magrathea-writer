<script lang="ts">
  import { t } from '../i18n';
  import { editorPreferences } from './editor-preferences.svelte';
  import { manuscript } from '../vault/manuscript.svelte';
  import { worldbuilding } from '../worldbuilding/worldbuilding.svelte';
  import { getEntityKindLabel } from './wikilink-autocomplete';
  import {
    toggleBold,
    toggleItalic,
    toggleUnderline,
    toggleStrikethrough,
    toggleHeading,
    toggleBulletList,
    toggleBlockquote,
    clearFormatting,
    DEFAULT_ACTIVE_FORMATS,
    type ActiveFormats,
  } from './rich-text-mode';
  import type { EditorView } from 'codemirror';

  interface Props {
    view: EditorView | null;
    activeFormats?: ActiveFormats;
  }

  let { view, activeFormats = DEFAULT_ACTIVE_FORMATS }: Props = $props();

  let isRefMenuOpen = $state(false);
  let refSearch = $state('');

  let referencableItems = $derived.by(() => {
    const q = refSearch.trim().toLowerCase();
    return manuscript.items
      .filter((it) => it.kind !== 'part' && it.id !== manuscript.activeSceneId)
      .filter((it) => !q || it.title.toLowerCase().includes(q) || it.tags.some((tag) => tag.toLowerCase().includes(q)))
      .sort((a, b) => {
        const score = (k: string) => (k === 'character' ? 0 : ['location', 'item', 'faction', 'lore', 'entity'].includes(k) ? 1 : 2);
        return score(a.kind) - score(b.kind) || a.title.localeCompare(b.title);
      });
  });

  function insertEntityReference(targetTitle: string) {
    if (!view) return;
    const { state, dispatch } = view;
    const { from, to } = state.selection.main;
    const selectedText = state.sliceDoc(from, to).trim();

    const snippet =
      selectedText && selectedText.toLowerCase() !== targetTitle.toLowerCase()
        ? `[[${targetTitle}|${selectedText}]]`
        : `[[${targetTitle}]]`;

    dispatch({
      changes: { from, to, insert: snippet },
      selection: { anchor: from + snippet.length },
    });
    isRefMenuOpen = false;
    refSearch = '';
    view.focus();
  }

  /**
   * Insere ou envolve o texto selecionado com os caracteres literários tipográficos.
   */
  function insertTypography(open: string, close = open) {
    if (!view) return;

    const { state, dispatch } = view;
    const { from, to } = state.selection.main;
    const selectedText = state.sliceDoc(from, to);

    if (from !== to) {
      const replacement = `${open}${selectedText}${close}`;
      dispatch({
        changes: { from, to, insert: replacement },
        selection: { anchor: from + replacement.length },
      });
    } else {
      const insertText = open === close ? open : `${open}${close}`;
      const cursorOffset = open.length;
      dispatch({
        changes: { from, insert: insertText },
        selection: { anchor: from + cursorOffset },
      });
    }

    view.focus();
  }

  function insertSceneBreak() {
    if (!view) return;
    const { state, dispatch } = view;
    const { from } = state.selection.main;
    const breakSnippet = '\n\n***\n\n';
    dispatch({
      changes: { from, insert: breakSnippet },
      selection: { anchor: from + breakSnippet.length },
    });
    view.focus();
  }
</script>

{#snippet iconList()}
  <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
    <circle cx="2.5" cy="4" r="0.9" fill="currentColor" stroke="none" /><circle cx="2.5" cy="8" r="0.9" fill="currentColor" stroke="none" /><circle cx="2.5" cy="12" r="0.9" fill="currentColor" stroke="none" />
    <path d="M6 4h8M6 8h8M6 12h8" />
  </svg>
{/snippet}

{#snippet iconQuote()}
  <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
    <path d="M2.5 3v10" /><path d="M6 5h8M6 8h8M6 11h5" />
  </svg>
{/snippet}

{#snippet iconClear()}
  <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M3 3.5h8M7 3.5v8" /><path d="M10 10l3.5 3.5M13.5 10L10 13.5" />
  </svg>
{/snippet}

{#snippet iconVisual()}
  <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M3 13l1-3.5L11.5 2l2.5 2.5L6.5 12z" /><path d="M10 3.5l2.5 2.5" />
  </svg>
{/snippet}

{#snippet iconCode()}
  <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M5.5 4L1.5 8l4 4M10.5 4l4 4-4 4" />
  </svg>
{/snippet}

{#snippet iconLink()}
  <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M6.5 9.5a3 3 0 004.2 0l2-2a3 3 0 00-4.2-4.2l-.8.8" /><path d="M9.5 6.5a3 3 0 00-4.2 0l-2 2a3 3 0 004.2 4.2l.8-.8" />
  </svg>
{/snippet}

<div class="literary-bar" role="toolbar" aria-label="Barra de Formatação e Ferramentas Literárias">
  <div class="bar-inner">
    <!-- 1. Modo do editor: Texto Comum (Visual / Rich Text) vs Markdown -->
    <div class="tool-group mode-switch" role="radiogroup" aria-label={t('literary.modeGroup')}>
      <button
        type="button"
        class="mode-btn"
        class:active={editorPreferences.editorMode === 'visual'}
        onclick={() => {
          editorPreferences.setEditorMode('visual');
          view?.focus();
        }}
        title={t('literary.modeVisualHint')}
        role="radio"
        aria-checked={editorPreferences.editorMode === 'visual'}
      >
        {@render iconVisual()}
        <span>{t('literary.modeVisual')}</span>
      </button>
      <button
        type="button"
        class="mode-btn"
        class:active={editorPreferences.editorMode === 'markdown'}
        onclick={() => {
          editorPreferences.setEditorMode('markdown');
          view?.focus();
        }}
        title={t('literary.modeMarkdownHint')}
        role="radio"
        aria-checked={editorPreferences.editorMode === 'markdown'}
      >
        {@render iconCode()}
        <span>{t('literary.modeMarkdown')}</span>
      </button>
    </div>

    <!-- 2. Formatação de texto -->
    <div class="tool-group" role="group" aria-label={t('literary.groupFormat')}>
      <button
        type="button"
        class="lit-btn fmt-bold"
        class:active={activeFormats.bold}
        onclick={() => view && toggleBold(view)}
        title={t('literary.bold')}
      >
        B
      </button>
      <button
        type="button"
        class="lit-btn fmt-italic"
        class:active={activeFormats.italic}
        onclick={() => view && toggleItalic(view)}
        title={t('literary.italic')}
      >
        I
      </button>
      <button
        type="button"
        class="lit-btn fmt-underline"
        class:active={activeFormats.underline}
        onclick={() => view && toggleUnderline(view)}
        title={t('literary.underline')}
      >
        U
      </button>
      <button
        type="button"
        class="lit-btn fmt-strike"
        class:active={activeFormats.strikethrough}
        onclick={() => view && toggleStrikethrough(view)}
        title={t('literary.strikethrough')}
      >
        S
      </button>
      <span class="divider" aria-hidden="true"></span>
      <button
        type="button"
        class="lit-btn fmt-heading"
        class:active={activeFormats.heading === 1}
        onclick={() => view && toggleHeading(view, 1)}
        title={t('literary.heading1')}
      >
        H1
      </button>
      <button
        type="button"
        class="lit-btn fmt-heading"
        class:active={activeFormats.heading === 2}
        onclick={() => view && toggleHeading(view, 2)}
        title={t('literary.heading2')}
      >
        H2
      </button>
      <button
        type="button"
        class="lit-btn fmt-heading"
        class:active={activeFormats.heading === 3}
        onclick={() => view && toggleHeading(view, 3)}
        title={t('literary.heading3')}
      >
        H3
      </button>
      <span class="divider" aria-hidden="true"></span>
      <button
        type="button"
        class="lit-btn "
        class:active={activeFormats.bulletList}
        onclick={() => view && toggleBulletList(view)}
        title={t('literary.bulletList')}
      >
        {@render iconList()}
      </button>
      <button
        type="button"
        class="lit-btn "
        class:active={activeFormats.blockquote}
        onclick={() => view && toggleBlockquote(view)}
        title={t('literary.blockquote')}
      >
        {@render iconQuote()}
      </button>
      <button
        type="button"
        class="lit-btn "
        onclick={() => view && clearFormatting(view)}
        title={t('literary.clearFormatting')}
      >
        {@render iconClear()}
      </button>
    </div>

    <!-- 3. Pontuação literária -->
    <div class="tool-group" role="group" aria-label={t('literary.groupInsert')}>
      <button
        type="button"
        class="lit-btn "
        onclick={() => insertTypography('—')}
        title={t('literary.emDash')}
      >
        —
      </button>
      <button
        type="button"
        class="lit-btn "
        onclick={() => insertTypography('“', '”')}
        title={t('literary.quotes')}
      >
        “ ”
      </button>
      <button
        type="button"
        class="lit-btn "
        onclick={() => insertTypography('«', '»')}
        title={t('literary.guillemets')}
      >
        « »
      </button>
      <button
        type="button"
        class="lit-btn "
        onclick={() => insertTypography('…')}
        title={t('literary.ellipsis')}
      >
        …
      </button>
      <span class="divider" aria-hidden="true"></span>
      <button
        type="button"
        class="lit-btn break-btn"
        onclick={insertSceneBreak}
        title={t('literary.sceneBreak')}
      >
        * * *
      </button>
    </div>

    <!-- 4. Referência rápida [[Personagem / Entidade]] -->
    <div class="ref-dropdown-container">
      <button
        type="button"
        class="ref-trigger-btn"
        class:active={isRefMenuOpen}
        onclick={() => {
          isRefMenuOpen = !isRefMenuOpen;
          if (!isRefMenuOpen) refSearch = '';
        }}
        title={t('literary.referenceHint')}
        aria-haspopup="dialog"
        aria-expanded={isRefMenuOpen}
      >
        {@render iconLink()}
        <span>{t('literary.reference')}</span>
      </button>

        {#if isRefMenuOpen}
          <div class="ref-popover" role="dialog" aria-label="Referenciar Entidade">
            <div class="ref-popover-header">
              <input
                type="text"
                class="ref-search-input"
                placeholder="Buscar personagem, local, item..."
                bind:value={refSearch}
                onkeydown={(e) => {
                  if (e.key === 'Escape') {
                    isRefMenuOpen = false;
                    view?.focus();
                  } else if (e.key === 'Enter') {
                    e.preventDefault();
                    if (referencableItems.length > 0) {
                      insertEntityReference(referencableItems[0].title);
                    } else if (refSearch.trim()) {
                      insertEntityReference(refSearch.trim());
                    }
                  }
                }}
              />
            </div>

            <div class="ref-list">
              {#if referencableItems.length > 0}
                {#each referencableItems as item (item.id)}
                  <button
                    type="button"
                    class="ref-option"
                    onclick={() => insertEntityReference(item.title)}
                  >
                    <span class="ref-option-title">{item.title}</span>
                    <span class="ref-option-badge">{getEntityKindLabel(item.kind)}</span>
                  </button>
                {/each}
              {:else if refSearch.trim()}
                <button
                  type="button"
                  class="ref-option"
                  onclick={() => insertEntityReference(refSearch.trim())}
                >
                  <span class="ref-option-title">Inserir <strong>[[{refSearch.trim()}]]</strong></span>
                  <span class="ref-option-badge">✨ Novo link</span>
                </button>
              {:else}
                <div class="ref-empty">
                  Nenhum personagem ou local cadastrado ainda.
                </div>
              {/if}
            </div>

            <div class="ref-popover-footer">
              <button
                type="button"
                class="ref-create-btn"
                onclick={() => {
                  isRefMenuOpen = false;
                  worldbuilding.openCreateModal();
                }}
              >
                + Criar Personagem / Entidade
              </button>
            </div>
          </div>
        {/if}
    </div>
  </div>
</div>

<style>
  /* Todos os grupos compartilham a mesma altura (34px), o mesmo raio e o mesmo fundo,
     para que os controles fiquem alinhados na mesma linha de base. */
  .literary-bar {
    display: flex;
    justify-content: center;
    padding: 6px var(--space-md);
    background-color: var(--color-paper-base);
    border-bottom: 1px solid var(--color-paper-border);
    user-select: none;
  }

  .bar-inner {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-wrap: wrap;
    gap: 6px 10px;
    width: 100%;
    max-width: 960px;
  }

  .tool-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    height: 34px;
    padding: 3px;
    box-sizing: border-box;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md, 6px);
  }

  .mode-btn,
  .lit-btn,
  .ref-trigger-btn {
    box-sizing: border-box;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    border-radius: var(--rounded-sm, 4px);
    background: transparent;
    color: var(--color-ink-charcoal);
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }

  .mode-btn {
    gap: 6px;
    padding: 0 10px;
    color: var(--color-ink-muted);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    line-height: 1;
  }

  .mode-btn:hover {
    color: var(--color-ink-charcoal);
    background-color: var(--color-paper-hover);
  }

  .mode-btn.active {
    background-color: var(--color-olive);
    color: #ffffff;
    font-weight: 600;
  }

  .lit-btn {
    min-width: 28px;
    padding: 0 6px;
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 500;
    line-height: 1;
  }

  .lit-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-olive);
  }

  .lit-btn.active {
    background-color: var(--color-olive-subtle);
    color: var(--color-olive);
    border-color: var(--color-olive);
    font-weight: 700;
  }

  .fmt-bold {
    font-weight: 700;
  }

  .fmt-italic {
    font-family: var(--font-literary);
    font-style: italic;
    font-weight: 600;
    font-size: 15px;
  }

  .fmt-underline {
    text-decoration: underline;
    text-underline-offset: 2px;
    font-weight: 600;
  }

  .fmt-strike {
    text-decoration: line-through;
    font-weight: 600;
  }

  .fmt-heading {
    font-size: 11.5px;
    font-weight: 700;
    min-width: 30px;
  }

  /* Pontuação literária: glifos tipográficos no mesmo tamanho e fonte */
  .tool-group:not(.mode-switch) .lit-btn:not(.fmt-bold):not(.fmt-italic):not(.fmt-underline):not(.fmt-strike):not(.fmt-heading) {
    font-family: var(--font-literary);
    font-size: 15px;
  }

  .break-btn {
    letter-spacing: 1px;
    font-size: 13px;
  }

  .divider {
    width: 1px;
    height: 16px;
    margin: 0 3px;
    background-color: var(--color-paper-border);
  }

  .ref-dropdown-container {
    position: relative;
    display: inline-flex;
  }

  .ref-trigger-btn {
    gap: 6px;
    height: 34px;
    padding: 0 12px;
    border-color: var(--color-bronze, #8C733E);
    color: var(--color-bronze, #8C733E);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
  }

  .ref-trigger-btn:hover,
  .ref-trigger-btn.active {
    background-color: var(--color-paper-hover);
  }

  .ref-popover {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    width: 280px;
    background-color: var(--color-paper-elevated, #FFFFFF);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md, 8px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.16);
    z-index: 200;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .ref-popover-header {
    padding: 8px;
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .ref-search-input {
    width: 100%;
    padding: 5px 8px;
    font-family: var(--font-ui);
    font-size: 12px;
    border: 1px solid var(--color-paper-border);
    border-radius: 4px;
    background-color: var(--color-paper-base);
    color: var(--color-ink-charcoal);
    outline: none;
    box-sizing: border-box;
  }

  .ref-search-input:focus {
    border-color: var(--color-olive);
  }

  .ref-list {
    max-height: 200px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 4px;
  }

  .ref-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--color-ink-charcoal);
    font-family: var(--font-ui);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.12s ease;
  }

  .ref-option:hover {
    background-color: var(--color-paper-hover);
  }

  .ref-option-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  }

  .ref-option-badge {
    font-size: 10px;
    color: var(--color-ink-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .ref-empty {
    padding: 12px 8px;
    font-family: var(--font-ui);
    font-size: 11.5px;
    color: var(--color-ink-muted);
    text-align: center;
  }

  .ref-popover-footer {
    padding: 6px 8px;
    border-top: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .ref-create-btn {
    width: 100%;
    padding: 5px 8px;
    border: 1px dashed var(--color-olive);
    border-radius: 4px;
    background: transparent;
    color: var(--color-olive);
    font-family: var(--font-ui);
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .ref-create-btn:hover {
    background-color: var(--color-olive-subtle);
  }
</style>
