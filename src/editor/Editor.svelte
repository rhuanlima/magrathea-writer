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
            fontSize: '16px',
            fontFamily: 'system-ui, -apple-system, sans-serif',
          },
          '.cm-scroller': {
            overflow: 'auto',
            padding: '16px',
          },
          '.cm-content': {
            maxWidth: '720px',
            margin: '0 auto',
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
  }
</style>
