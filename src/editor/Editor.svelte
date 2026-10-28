<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { t } from '../i18n';
  import { vault } from '../vault/vault.svelte';
  import { manuscript } from '../vault/manuscript.svelte';
  import { AutosaveManager, type AutosaveStatus } from './autosave';
  import { splitFrontmatter, countWords, countChars } from './document';
  import { wikilinkDecoratorPlugin } from './wikilink-decorator';
  import { wikilinkAutocompleteExtension } from './wikilink-autocomplete';
  import { wikilinkHoverExtension } from './wikilink-hover';
  import { externalLinksPlugin } from './external-links';
  import { focusModeExtension } from './focus-mode';
  import { editorViewState } from './editor-view-state.svelte';
  import { searchStore } from '../search/search.svelte';
  import { timelineStore } from '../timeline/timeline.svelte';
  import { graphStore } from '../graph/graph.svelte';
  import { compilerStore } from '../compiler/compiler.svelte';
  import { theme } from '../design';
  import { editorThemeCompartment, getEditorThemeExtension } from './theme';
  import LiteraryBar from './LiteraryBar.svelte';
  import SceneHeader from './SceneHeader.svelte';
  import FloatingHud from './FloatingHud.svelte';

  interface Props {
    initialContent?: string;
    onChange?: (value: string) => void;
  }

  let { initialContent = '', onChange }: Props = $props();

  let editorElement: HTMLDivElement | null = $state(null);
  let view = $state<EditorView | null>(null);
  let currentDoc = $state('');
  let autosaveStatus = $state<AutosaveStatus>('saved');
  let autosaveManager: AutosaveManager | null = null;
  let lastLoadedSceneId = $state<string | null>(null);

  let bodyOnly = $derived(splitFrontmatter(currentDoc).body);
  let wordsCount = $derived(countWords(bodyOnly));
  let charsCount = $derived(countChars(bodyOnly));

  onMount(() => {
    if (!editorElement) {
      return;
    }
    currentDoc = initialContent;

    autosaveManager = new AutosaveManager({
      delayMs: 800,
      onSave: async (content) => {
        if (vault.isOpen && !vault.isLocked) {
          const targetSceneId = manuscript.activeSceneId || 'default_scene';
          await vault.saveScene(targetSceneId, content);
          await vault.loadVaultIndex();
          if (manuscript.activeSceneId) {
            await vault.loadBacklinks(manuscript.activeSceneId);
          }
        }
      },
      onStatusChange: (status) => {
        autosaveStatus = status;
      },
    });

    let prevWords = countWords(splitFrontmatter(initialContent).body);

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const text = update.state.doc.toString();
        currentDoc = text;
        if (onChange) {
          onChange(text);
        }
        autosaveManager?.notifyChange(text);

        const newWords = countWords(splitFrontmatter(text).body);
        if (newWords > prevWords) {
          editorViewState.recordWordsDelta(newWords - prevWords);
        }
        prevWords = newWords;
      }
    });

    const state = EditorState.create({
      doc: initialContent,
      extensions: [
        basicSetup,
        markdown(),
        wikilinkDecoratorPlugin,
        wikilinkAutocompleteExtension,
        wikilinkHoverExtension,
        externalLinksPlugin,
        focusModeExtension,
        updateListener,
        editorThemeCompartment.of(getEditorThemeExtension(theme.isDark)),
      ],
    });

    view = new EditorView({
      state,
      parent: editorElement,
    });
    editorViewState.applyInitialFocus(view);
  });

  // Reconfigura instantaneamente o tema e o syntax highlighting do editor ao alternar claro/escuro
  $effect(() => {
    const isDark = theme.isDark;
    if (view) {
      view.dispatch({
        effects: editorThemeCompartment.reconfigure(getEditorThemeExtension(isDark)),
      });
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    // Cmd/Ctrl + Alt + F: Alternar Modo Foco
    if ((e.metaKey || e.ctrlKey) && e.altKey && e.code === 'KeyF') {
      e.preventDefault();
      editorViewState.toggleFocus(view);
    }
    // Cmd/Ctrl + Shift + F: Alternar Modo Zen
    else if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.code === 'KeyF') {
      e.preventDefault();
      editorViewState.toggleZen();
    }
    // Cmd/Ctrl + Shift + M: Alternar Metas / Dashboard
    else if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.code === 'KeyM') {
      e.preventDefault();
      editorViewState.openProductivityDashboard();
    }
  }

  // Reage à mudança de cena ativa selecionada na barra lateral
  $effect(() => {
    const activeId = manuscript.activeSceneId;
    if (activeId && activeId !== lastLoadedSceneId && view) {
      lastLoadedSceneId = activeId;
      void (async () => {
        if (autosaveManager) {
          await autosaveManager.flush();
        }
        try {
          const scene = manuscript.items.find((i) => i.id === activeId);
          const blobId = scene?.blob_id || activeId;
          const content = await vault.loadScene(blobId);
          currentDoc = content;
          if (view) {
            view.dispatch({
              changes: {
                from: 0,
                to: view.state.doc.length,
                insert: content,
              },
            });
          }
        } catch (err) {
          console.error('Falha ao carregar conteúdo da cena:', err);
        }
      })();
    }
  });

  onDestroy(() => {
    if (autosaveManager) {
      void autosaveManager.flush();
      autosaveManager.dispose();
      autosaveManager = null;
    }
    void editorViewState.flushSessionToVault();
    if (view) {
      view.destroy();
      view = null;
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="editor-wrapper">
  <LiteraryBar {view} />

  {#if manuscript.activeScene}
    <SceneHeader />
  {/if}

  <div class="editor-container" bind:this={editorElement}></div>

  <FloatingHud sceneWordsCount={wordsCount} />

  <footer class="editor-footer">
    <div class="stats">
      <span>{t('editor.wordsCount', { count: wordsCount })}</span>
      <span class="dot">·</span>
      <span>{t('editor.charactersCount', { count: charsCount })}</span>
    </div>

    <div class="quick-status-actions" role="toolbar" aria-label="Ações Rápidas">
      <button
        type="button"
        class="quick-action-btn"
        onclick={() => searchStore.openModal()}
        title="Busca Global no Projeto (Cmd+Shift+F)"
      >
        🔍
      </button>

      <button
        type="button"
        class="quick-action-btn"
        onclick={() => timelineStore.openModal()}
        title="Linha do Tempo Literária (Cmd+Shift+T)"
      >
        ⏳
      </button>

      <button
        type="button"
        class="quick-action-btn"
        onclick={() => graphStore.openModal()}
        title="Grafo Narrativo & Canvas (Cmd+Shift+G)"
      >
        🕸️
      </button>

      <button
        type="button"
        class="quick-action-btn"
        onclick={() => editorViewState.openProductivityDashboard()}
        title="Metas de Escrita & Ritmo (Cmd+Shift+P)"
      >
        📊
      </button>

      <button
        type="button"
        class="quick-action-btn"
        onclick={() => compilerStore.openModal('export')}
        title="Exportação Editorial & Publicação (Cmd+Shift+E)"
      >
        📖
      </button>

      <button
        type="button"
        class="quick-action-btn"
        class:active={vault.isInspectorOpen}
        onclick={() => vault.toggleInspector()}
        title="Inspetor de Fichas e Personagens (Cmd+I)"
      >
        ℹ️
      </button>

      <button
        type="button"
        class="quick-action-btn"
        onclick={() => theme.toggleTheme()}
        title="Alternar Tema Claro / Escuro"
      >
        {#if theme.mode === 'light'}
          ☀️
        {:else if theme.mode === 'dark'}
          🌙
        {:else}
          💻
        {/if}
      </button>

      <button
        type="button"
        class="quick-action-btn lock-btn"
        onclick={() => vault.lockVault()}
        title="Trancar Mundo (Cmd+L)"
      >
        🔒
      </button>
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

  .quick-status-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .quick-action-btn {
    background: transparent;
    border: none;
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 11px;
    cursor: pointer;
    color: var(--color-ink-muted);
    transition: background-color 0.15s ease, color 0.15s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    user-select: none;
  }

  .quick-action-btn:hover {
    background-color: var(--color-paper-border);
    color: var(--color-ink-primary);
  }

  .quick-action-btn.active {
    background-color: var(--color-paper-border);
    color: var(--color-terracotta);
    font-weight: 600;
  }

  .quick-action-btn.lock-btn:hover {
    color: #e11d48;
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

  :global(.cm-wikilink-chip) {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background-color: rgba(194, 141, 88, 0.15);
    color: var(--color-bronze, #C28D58);
    border: 1px solid rgba(194, 141, 88, 0.35);
    border-radius: 4px;
    padding: 1px 6px;
    font-family: var(--font-ui, sans-serif);
    font-size: 0.85em;
    font-weight: 500;
    line-height: 1.4;
    vertical-align: middle;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.cm-wikilink-chip:hover) {
    background-color: rgba(194, 141, 88, 0.25);
    border-color: var(--color-bronze, #C28D58);
  }

  :global(.cm-wikilink-icon) {
    font-size: 0.85em;
  }

  :global(.cm-wikilink-hover-card) {
    background-color: var(--color-paper-elevated, #FFFDF9);
    color: var(--color-ink-charcoal, #1E231D);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    padding: 10px 14px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    max-width: 320px;
    font-family: var(--font-ui, sans-serif);
    font-size: 13px;
    line-height: 1.4;
    z-index: 1000;
  }

  :global(.cm-wikilink-hover-header) {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  :global(.cm-wikilink-hover-badge) {
    font-size: 10px;
    text-transform: uppercase;
    padding: 1px 5px;
    border-radius: 3px;
    background-color: var(--color-paper-border, #E5E0D8);
    color: var(--color-ink-faded, #6C7368);
  }

  :global(.cm-wikilink-hover-alias) {
    font-size: 11px;
    color: var(--color-ink-faded, #6C7368);
    font-style: italic;
  }

  :global(.cm-wikilink-hover-body) {
    margin-bottom: 8px;
    color: var(--color-ink-muted, #4A5247);
  }

  :global(.cm-wikilink-hover-footer) {
    font-size: 11px;
    color: var(--color-olive, #5E6F45);
    font-weight: 500;
    border-top: 1px solid var(--color-paper-border, #E5E0D8);
    padding-top: 4px;
  }

  :global(.cm-external-link-chip) {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background-color: var(--color-paper-base, #F8F5EE);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 4px;
    padding: 1px 6px;
    font-family: var(--font-ui, sans-serif);
    font-size: 0.85em;
    color: var(--color-ink-muted, #4A5247);
    vertical-align: middle;
    cursor: default;
    user-select: none;
  }

  :global(.cm-external-link-icon) {
    font-size: 0.85em;
    opacity: 0.7;
  }

  :global(.cm-external-link-text) {
    font-family: var(--font-mono, monospace);
    font-size: 0.9em;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.cm-external-link-copy) {
    background: none;
    border: none;
    padding: 2px 4px;
    cursor: pointer;
    font-size: 11px;
    border-radius: 3px;
    color: var(--color-ink-faded, #6C7368);
    transition: all 0.15s ease;
  }

  :global(.cm-external-link-copy:hover) {
    background-color: var(--color-paper-border, #E5E0D8);
    color: var(--color-ink-charcoal, #1E231D);
  }

  :global(.cm-external-link-copy.copied) {
    color: var(--color-olive, #5E6F45);
    font-weight: bold;
  }
</style>
