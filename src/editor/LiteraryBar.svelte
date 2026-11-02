<script lang="ts">
  import { t } from '../i18n';
  import { editorPreferences } from './editor-preferences.svelte';
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
</style>
