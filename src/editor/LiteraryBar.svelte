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

<div class="literary-bar" role="toolbar" aria-label="Barra de Formatação e Ferramentas Literárias">
  <div class="bar-inner">
    <!-- 1. Alternador de Modo: Editor Comum (Visual / Rich Text) vs Markdown -->
    <div class="mode-switch" role="radiogroup" aria-label="Modo de Edição">
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
        <span class="mode-icon">📝</span>
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
        <span class="mode-icon">⌨️</span>
        <span>{t('literary.modeMarkdown')}</span>
      </button>
    </div>

    <!-- 2. Ferramentas de Formatação de Editor de Texto Comum (Rich Text) -->
    <div class="btn-group">
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

      <div class="divider"></div>

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

      <div class="divider"></div>

      <button
        type="button"
        class="lit-btn fmt-block"
        class:active={activeFormats.bulletList}
        onclick={() => view && toggleBulletList(view)}
        title={t('literary.bulletList')}
      >
        •≡
      </button>
      <button
        type="button"
        class="lit-btn fmt-block"
        class:active={activeFormats.blockquote}
        onclick={() => view && toggleBlockquote(view)}
        title={t('literary.blockquote')}
      >
        ❝
      </button>
      <button
        type="button"
        class="lit-btn fmt-clear"
        onclick={() => view && clearFormatting(view)}
        title={t('literary.clearFormatting')}
      >
        Tₓ
      </button>
    </div>

    <!-- 3. Ferramentas Tipográficas Literárias -->
    <div class="btn-group">
      <button
        type="button"
        class="lit-btn"
        onclick={() => insertTypography('—')}
        title={t('literary.emDash')}
      >
        —
      </button>
      <button
        type="button"
        class="lit-btn"
        onclick={() => insertTypography('“', '”')}
        title={t('literary.quotes')}
      >
        “ ”
      </button>
      <button
        type="button"
        class="lit-btn"
        onclick={() => insertTypography('«', '»')}
        title={t('literary.guillemets')}
      >
        « »
      </button>
      <button
        type="button"
        class="lit-btn"
        onclick={() => insertTypography('…')}
        title={t('literary.ellipsis')}
      >
        …
      </button>
      <div class="divider"></div>
      <button
        type="button"
        class="lit-btn break-btn"
        onclick={insertSceneBreak}
        title={t('literary.sceneBreak')}
      >
        ***
      </button>
      <div class="divider"></div>

      <!-- Botão e Popover de Referência Rápida [[Personagem / Entidade]] -->
      <div class="ref-dropdown-container">
        <button
          type="button"
          class="lit-btn ref-trigger-btn"
          class:active={isRefMenuOpen}
          onclick={() => {
            isRefMenuOpen = !isRefMenuOpen;
            if (!isRefMenuOpen) refSearch = '';
          }}
          title="Referenciar Personagem, Local, Item ou Cena ([[...]])"
        >
          🔗 [[Referenciar]]
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
</div>

<style>
  .literary-bar {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 5px var(--space-md);
    background-color: var(--color-paper-base);
    border-bottom: 1px solid var(--color-paper-border);
    user-select: none;
  }

  .bar-inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    max-width: 720px;
    width: 100%;
    flex-wrap: wrap;
  }

  .mode-switch {
    display: inline-flex;
    align-items: center;
    background-color: var(--color-paper-surface);
    padding: 2px;
    border-radius: var(--rounded-md, 6px);
    border: 1px solid var(--color-paper-border);
    gap: 2px;
  }

  .mode-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border: none;
    border-radius: var(--rounded-sm, 4px);
    background: transparent;
    color: var(--color-ink-muted);
    font-family: var(--font-ui);
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    line-height: 1.2;
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

  .mode-icon {
    font-size: 12px;
    line-height: 1;
  }

  .btn-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    background-color: var(--color-paper-surface);
    padding: 2px 5px;
    border-radius: var(--rounded-md, 6px);
    border: 1px solid var(--color-paper-border);
  }

  .lit-btn {
    font-family: var(--font-literary);
    font-size: 14px;
    font-weight: 500;
    min-width: 28px;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0 5px;
    border: 1px solid transparent;
    border-radius: var(--rounded-sm, 4px);
    background: transparent;
    color: var(--color-ink-charcoal);
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease, border-color 0.15s ease;
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
    font-family: var(--font-ui);
    font-weight: 700;
    font-size: 12.5px;
  }

  .fmt-italic {
    font-family: var(--font-literary);
    font-style: italic;
    font-weight: 600;
    font-size: 14px;
  }

  .fmt-underline {
    font-family: var(--font-ui);
    text-decoration: underline;
    font-weight: 600;
    font-size: 12.5px;
  }

  .fmt-strike {
    font-family: var(--font-ui);
    text-decoration: line-through;
    font-weight: 600;
    font-size: 12.5px;
  }

  .fmt-heading {
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 700;
  }

  .fmt-block {
    font-family: var(--font-ui);
    font-size: 12px;
  }

  .fmt-clear {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-ink-muted);
  }

  .break-btn {
    font-size: 12px;
    letter-spacing: 1.5px;
  }

  .divider {
    width: 1px;
    height: 16px;
    background-color: var(--color-paper-border);
    margin: 0 2px;
  }

  .ref-dropdown-container {
    position: relative;
    display: inline-flex;
  }

  .ref-trigger-btn {
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 600;
    padding: 0 8px;
    color: var(--color-bronze, #8C733E);
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
