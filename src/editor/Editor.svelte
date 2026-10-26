<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';

  interface Props {
    initialContent?: string;
    onChange?: (value: string) => void;
  }

  let { initialContent = '', onChange }: Props = $props();

  let editorElement: HTMLDivElement | null = $state(null);
  let view: EditorView | null = null;

  onMount(() => {
    if (!editorElement) {
      return;
    }

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged && onChange) {
        onChange(update.state.doc.toString());
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
    if (view) {
      view.destroy();
      view = null;
    }
  });
</script>

<div class="editor-container" bind:this={editorElement}></div>

<style>
  .editor-container {
    width: 100%;
    height: 100%;
    min-height: 300px;
    box-sizing: border-box;
    background-color: var(--color-paper-base);
  }
</style>
