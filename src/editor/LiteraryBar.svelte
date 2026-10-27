<script lang="ts">
  import { t } from '../i18n';
  import type { EditorView } from 'codemirror';

  interface Props {
    view: EditorView | null;
  }

  let { view }: Props = $props();

  /**
   * Insere ou envolve o texto selecionado com os caracteres literários tipográficos.
   */
  function insertTypography(open: string, close = open) {
    if (!view) return;

    const { state, dispatch } = view;
    const { from, to } = state.selection.main;
    const selectedText = state.sliceDoc(from, to);

    if (from !== to) {
      // Envolve a seleção existente
      const replacement = `${open}${selectedText}${close}`;
      dispatch({
        changes: { from, to, insert: replacement },
        selection: { anchor: from + replacement.length },
      });
    } else {
      // Sem seleção: insere ambos ou caractere único e posiciona cursor
      const insertText = open === close ? open : `${open}${close}`;
      const cursorOffset = open === close ? open.length : open.length;
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

<div class="literary-bar" role="toolbar" aria-label="Ferramentas Literárias">
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

<style>
  .literary-bar {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px var(--space-md);
    background-color: var(--color-paper-surface);
    border-bottom: 1px solid var(--color-paper-border);
    user-select: none;
  }

  .btn-group {
    display: flex;
    align-items: center;
    gap: 4px;
    background-color: var(--color-paper-base);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-paper-border);
  }

  .lit-btn {
    font-family: var(--font-literary);
    font-size: 15px;
    font-weight: 500;
    min-width: 32px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0 6px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-ink-charcoal);
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .lit-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-olive);
  }

  .lit-btn:active {
    background-color: var(--color-paper-border);
  }

  .break-btn {
    font-size: 13px;
    letter-spacing: 2px;
  }

  .divider {
    width: 1px;
    height: 18px;
    background-color: var(--color-paper-border);
    margin: 0 2px;
  }
</style>
