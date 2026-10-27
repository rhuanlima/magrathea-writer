<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { t } from '../i18n';
  import { vault } from '../vault/vault.svelte';
  import { AutosaveManager, type AutosaveStatus } from './autosave';

  interface Props {
    initialContent?: string;
    onChange?: (value: string) => void;
  }

  let { initialContent = '', onChange }: Props = $props();

  let editorElement: HTMLDivElement | null = $state(null);
  let view: EditorView | null = null;
  let currentDoc = $state('');
  let autosaveStatus = $state<AutosaveStatus>('saved');
  let autosaveManager: AutosaveManager | null = null;

  let wordsCount = $derived(
    currentDoc.trim().length === 0 ? 0 : currentDoc.trim().split(/\s+/).length
  );
  let charsCount = $derived(currentDoc.length);

  onMount(() => {
    if (!editorElement) {
      return;
    }
    currentDoc = initialContent;

    autosaveManager = new AutosaveManager({
      delayMs: 800,
      onSave: async (content) => {
        if (vault.isOpen && !vault.isLocked) {
          await vault.saveScene('default_scene', content);
        }
      },
      onStatusChange: (status) => {
        autosaveStatus = status;
      },
    });

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const text = update.state.doc.toString();
        currentDoc = text;
        if (onChange) {
          onChange(text);
        }
        autosaveManager?.notifyChange(text);
      }
    });

    const state = EditorState.create({
      doc: initialContent,
      extensions: [
        basicSetup,
        markdown(),
        updateListener,
        EditorView.theme({
          '&': {
            height: '100%',
            fontSize: '19px',
            lineHeight: '32px',
            fontFamily: 'var(--font-literary)',
            backgroundColor: 'var(--color-paper-base)',
            color: 'var(--color-ink-charcoal)',
          },
          '.cm-scroller': {
            overflow: 'auto',
            padding: 'var(--space-lg) var(--space-md)',
          },
          '.cm-content': {
            maxWidth: '720px',
            margin: '0 auto',
            caretColor: 'var(--color-olive)',
          },
          '.cm-cursor': {
            borderLeftColor: 'var(--color-olive)',
            borderLeftWidth: '2px',
          },
          '.cm-selectionBackground, ::selection': {
            backgroundColor: 'rgba(94, 111, 69, 0.2) !important',
          },
          '.cm-gutters': {
            backgroundColor: 'var(--color-paper-base)',
            borderColor: 'var(--color-paper-border)',
            color: 'var(--color-ink-faded)',
          },
        }),
      ],
    });

    view = new EditorView({
      state,
      parent: editorElement,
    });
  });

  onDestroy(() => {
    if (autosaveManager) {
      void autosaveManager.flush();
      autosaveManager.dispose();
      autosaveManager = null;
    }
    if (view) {
      view.destroy();
      view = null;
    }
  });
</script>

<div class="editor-wrapper">
  <div class="editor-container" bind:this={editorElement}></div>
  <footer class="editor-footer">
    <div class="stats">
      <span>{t('editor.wordsCount', { count: wordsCount })}</span>
      <span class="dot">·</span>
      <span>{t('editor.charactersCount', { count: charsCount })}</span>
    </div>
    <div class="save-status" class:saving={autosaveStatus === 'saving'} class:error={autosaveStatus === 'error'}>
      {#if autosaveStatus === 'saving'}
        <span class="status-indicator saving">●</span>
        <span>{t('editor.saving')}</span>
      {:else if autosaveStatus === 'unsaved'}
        <span class="status-indicator unsaved">○</span>
        <span>{t('editor.unsavedChanges')}</span>
      {:else if autosaveStatus === 'saved'}
        <span class="status-indicator saved">✓</span>
        <span>{t('editor.saved')}</span>
      {:else if autosaveStatus === 'error'}
        <span class="status-indicator error">⚠️</span>
        <span>{vault.errorMessage || 'Erro ao salvar'}</span>
      {/if}
    </div>
  </footer>
</div>

<style>
  .editor-wrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    background-color: var(--color-paper-base);
  }

  .editor-container {
    flex: 1;
    width: 100%;
    overflow: hidden;
    box-sizing: border-box;
    background-color: var(--color-paper-base);
  }

  .editor-footer {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-md);
    background-color: var(--color-paper-surface);
    border-top: 1px solid var(--color-paper-border);
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-ink-muted);
  }

  .stats {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .dot {
    opacity: 0.5;
  }

  .save-status {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .save-status.saving {
    color: var(--color-olive);
  }

  .save-status.error {
    color: var(--color-error);
  }

  .status-indicator {
    font-size: 10px;
  }

  .status-indicator.saving {
    animation: pulse 1s infinite alternate;
  }

  @keyframes pulse {
    from {
      opacity: 0.4;
    }
    to {
      opacity: 1;
    }
  }
</style>
