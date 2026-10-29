<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState, Compartment, type Extension } from '@codemirror/state';
  import { lineNumbers } from '@codemirror/view';
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
  import { commandPaletteStore } from '../commands/palette.svelte';
  import { timelineStore } from '../timeline/timeline.svelte';
  import { graphStore } from '../graph/graph.svelte';
  import { compilerStore } from '../compiler/compiler.svelte';
  import { theme } from '../design';
  import { editorThemeCompartment, getEditorThemeExtension } from './theme';
  import { editorPreferences } from './editor-preferences.svelte';
  import { typewriterAudio } from './typewriter-audio';
  import { preferences } from '../preferences/preferences.svelte';
  import { focusPreferences } from '../preferences/focus-preferences.svelte';
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

  const lineNumbersCompartment = new Compartment();

  function getLineNumbersExtension(relative: boolean): Extension {
    if (!relative) {
      return lineNumbers();
    }
    return lineNumbers({
      formatNumber(lineNo: number, state: EditorState): string {
        const head = state.selection.main.head;
        const cursorLine = state.doc.lineAt(head).number;
        if (lineNo === cursorLine) {
          return String(lineNo);
        }
        return String(Math.abs(lineNo - cursorLine));
      },
    });
  }
  let autosaveManager: AutosaveManager | null = null;
  let lastLoadedSceneId = $state<string | null>(null);

  let bodyOnly = $derived(splitFrontmatter(currentDoc).body);
  let wordsCount = $derived(countWords(bodyOnly));
  let charsCount = $derived(countChars(bodyOnly));
  let isExportMenuOpen = $state(false);
  let isLiteraryMenuOpen = $state(false);

  function insertLiteraryText(snippet: string, cursorOffset = 0) {
    if (!view) return;
    const { from, to } = view.state.selection.main;
    view.dispatch({
      changes: { from, to, insert: snippet },
      selection: { anchor: from + snippet.length + cursorOffset },
    });
    view.focus();
    isLiteraryMenuOpen = false;
  }

  async function handleExportDirect(format: 'docx' | 'epub' | 'pdf' | 'obsidian' | 'txt') {
    isExportMenuOpen = false;
    await compilerStore.quickExportDirect(format);
  }

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
    let lastCursorLine = -1;

    const updateListener = EditorView.updateListener.of((update) => {
      if (editorPreferences.typewriterScrolling && (update.docChanged || update.selectionSet)) {
        setTimeout(() => {
          if (view) {
            view.dispatch({
              effects: EditorView.scrollIntoView(view.state.selection.main.head, { y: 'center' }),
            });
          }
        }, 10);
      }

      if (editorPreferences.relativeLineNumbers && update.selectionSet) {
        const curLine = update.state.doc.lineAt(update.state.selection.main.head).number;
        if (curLine !== lastCursorLine) {
          lastCursorLine = curLine;
          view?.dispatch({
            effects: lineNumbersCompartment.reconfigure(getLineNumbersExtension(true)),
          });
        }
      }

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
          focusPreferences.checkDailyGoalAchieved(
            editorViewState.productivity.today_words + editorViewState.sessionWords
          );
        }
        prevWords = newWords;
      }
    });

    const typewriterKeyHandler = EditorView.domEventHandlers({
      keydown(e) {
        if (!editorPreferences.typewriterSound) return false;

        // Modificadores solitários não disparam som
        if (
          e.key === 'Shift' ||
          e.key === 'Control' ||
          e.key === 'Alt' ||
          e.key === 'Meta' ||
          e.key === 'CapsLock'
        ) {
          return false;
        }

        const vol = editorPreferences.typewriterVolume;
        if (e.key === 'Enter') {
          typewriterAudio.playEnterReturn(vol);
        } else if (e.key === ' ') {
          typewriterAudio.playSpacebar(vol);
        } else if (e.key === 'Backspace' || e.key === 'Delete') {
          typewriterAudio.playBackspace(vol);
        } else {
          // Toda e qualquer outra tecla (letras, números, pontuação, Tab, atalhos de texto)
          typewriterAudio.playKeyClick(vol);
        }
        return false;
      },
    });

    const state = EditorState.create({
      doc: initialContent,
      extensions: [
        lineNumbersCompartment.of(getLineNumbersExtension(editorPreferences.relativeLineNumbers)),
        ...((basicSetup as unknown as Extension[]).slice(1)),
        markdown(),
        wikilinkDecoratorPlugin,
        wikilinkAutocompleteExtension,
        wikilinkHoverExtension,
        externalLinksPlugin,
        focusModeExtension,
        typewriterKeyHandler,
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

  // Reconfigura o modo numérico de linhas (Normal vs Vim relativo)
  $effect(() => {
    const isRel = editorPreferences.relativeLineNumbers;
    if (view) {
      view.dispatch({
        effects: lineNumbersCompartment.reconfigure(getLineNumbersExtension(isRel)),
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
  {#if manuscript.activeScene}
    <SceneHeader />
  {/if}

  <div
    class="editor-container"
    class:show-line-numbers={editorPreferences.showLineNumbers}
    class:typewriter-scrolling={editorPreferences.typewriterScrolling}
    style="--editor-font-size: {editorPreferences.fontSize}px; --editor-font-family: {editorPreferences.fontFamilyCss}; --editor-line-height: {editorPreferences.lineHeight}; --editor-column-max-width: {editorPreferences.columnMaxWidthPx};"
    bind:this={editorElement}
  ></div>

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
        onclick={() => commandPaletteStore.toggle()}
        title="Paleta de Comandos & Quick Open (Cmd+K)"
      >
        ⚡
      </button>

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

      <!-- Menu Popover de Pontuação Literária (Ajuste 5) -->
      <div class="export-popover-wrapper">
        <button
          type="button"
          class="quick-action-btn"
          class:active={isLiteraryMenuOpen}
          onclick={() => (isLiteraryMenuOpen = !isLiteraryMenuOpen)}
          title="Pontuação Literária (—, «», …, ***)"
        >
          <span class="btn-icon">✍️</span>
          <span class="btn-tag">Pontuação</span>
        </button>

        {#if isLiteraryMenuOpen}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="export-backdrop" onclick={() => (isLiteraryMenuOpen = false)}></div>

          <div class="export-dropdown-popover" role="menu">
            <div class="export-dropdown-title">Pontuação Literária</div>
            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => insertLiteraryText('— ')}
            >
              <span class="export-item-icon">—</span>
              <div class="export-item-info">
                <span class="export-item-name">Travessão Literário</span>
                <span class="export-item-ext">— diálogo</span>
              </div>
            </button>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => insertLiteraryText('«»', -1)}
            >
              <span class="export-item-icon">« »</span>
              <div class="export-item-info">
                <span class="export-item-name">Aspas Angulares</span>
                <span class="export-item-ext">«pensamento»</span>
              </div>
            </button>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => insertLiteraryText('“”', -1)}
            >
              <span class="export-item-icon">“ ”</span>
              <div class="export-item-info">
                <span class="export-item-name">Aspas Tipográficas</span>
                <span class="export-item-ext">“citação”</span>
              </div>
            </button>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => insertLiteraryText('…')}
            >
              <span class="export-item-icon">…</span>
              <div class="export-item-info">
                <span class="export-item-name">Reticências Reais</span>
                <span class="export-item-ext">pausa…</span>
              </div>
            </button>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => insertLiteraryText('\n\n* * *\n\n')}
            >
              <span class="export-item-icon">⁂</span>
              <div class="export-item-info">
                <span class="export-item-name">Quebra de Cena</span>
                <span class="export-item-ext">* * *</span>
              </div>
            </button>
          </div>
        {/if}
      </div>

      <div class="export-popover-wrapper">
        <button
          type="button"
          class="quick-action-btn"
          class:active={isExportMenuOpen}
          onclick={() => (isExportMenuOpen = !isExportMenuOpen)}
          title="Exportar Manuscrito (Word, EPUB, PDF...)"
        >
          <span class="btn-icon">📖</span>
          <span class="btn-tag">Exportar</span>
        </button>

        {#if isExportMenuOpen}
          <!-- Backdrop invisível para fechar ao clicar fora -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="export-backdrop" onclick={() => (isExportMenuOpen = false)}></div>

          <div class="export-dropdown-popover" role="menu">
            <div class="export-dropdown-title">Exportação Editorial</div>

            <button
              type="button"
              class="export-dropdown-item primary-highlight"
              onclick={() => {
                isExportMenuOpen = false;
                void compilerStore.openExportWithOptions();
              }}
            >
              <span class="export-item-icon">⚙️</span>
              <div class="export-item-info">
                <span class="export-item-name">Opções de Exportação...</span>
                <span class="export-item-ext">Tudo vs Final • Incluir Fichas</span>
              </div>
            </button>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => {
                isExportMenuOpen = false;
                void compilerStore.openMatterTab();
              }}
            >
              <span class="export-item-icon">✍️</span>
              <div class="export-item-info">
                <span class="export-item-name">Partes do Livro...</span>
                <span class="export-item-ext">Referências, Agradecimentos, Epígrafe</span>
              </div>
            </button>

            <div class="export-dropdown-divider"></div>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => handleExportDirect('docx')}
            >
              <span class="export-item-icon">📄</span>
              <div class="export-item-info">
                <span class="export-item-name">Microsoft Word</span>
                <span class="export-item-ext">.docx</span>
              </div>
            </button>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => handleExportDirect('epub')}
            >
              <span class="export-item-icon">📱</span>
              <div class="export-item-info">
                <span class="export-item-name">Livro Digital</span>
                <span class="export-item-ext">.epub</span>
              </div>
            </button>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => handleExportDirect('pdf')}
            >
              <span class="export-item-icon">🖨️</span>
              <div class="export-item-info">
                <span class="export-item-name">Leitura / Impressão</span>
                <span class="export-item-ext">.html / .pdf</span>
              </div>
            </button>

            <div class="export-dropdown-divider"></div>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => handleExportDirect('obsidian')}
            >
              <span class="export-item-icon">💎</span>
              <div class="export-item-info">
                <span class="export-item-name">Cofre Obsidian</span>
                <span class="export-item-ext">.md</span>
              </div>
            </button>

            <button
              type="button"
              class="export-dropdown-item"
              onclick={() => handleExportDirect('txt')}
            >
              <span class="export-item-icon">📝</span>
              <div class="export-item-info">
                <span class="export-item-name">Texto Puro + Dossiê</span>
                <span class="export-item-ext">.txt</span>
              </div>
            </button>
          </div>
        {/if}
      </div>

      <button
        type="button"
        class="quick-action-btn"
        class:active={editorPreferences.typewriterSound}
        onclick={() => {
          editorPreferences.toggleTypewriterSound();
          if (editorPreferences.typewriterSound) {
            typewriterAudio.playKeyClick(editorPreferences.typewriterVolume);
          }
        }}
        title="Som Mecânico de Máquina de Escrever (Typewriter)"
      >
        <span class="btn-icon">⌨️</span>
        <span class="btn-tag">{editorPreferences.typewriterSound ? 'Sons ON' : 'Sons'}</span>
      </button>

      <button
        type="button"
        class="quick-action-btn"
        class:active={vault.isInspectorOpen}
        onclick={() => vault.toggleInspector()}
        title="Inspetor de Fichas e Personagens (Cmd+I)"
      >
        <span class="btn-icon">ℹ️</span>
        <span class="btn-tag">Inspetor</span>
      </button>

      <button
        type="button"
        class="quick-action-btn"
        onclick={() => preferences.open('editor')}
        title="Preferências do Sistema & Editor (Cmd+,)"
      >
        <span class="btn-icon">⚙️</span>
        <span class="btn-tag">Preferências</span>
      </button>

      <button
        type="button"
        class="quick-action-btn"
        onclick={() => theme.toggleTheme()}
        title="Alternar Tema Claro / Escuro"
      >
        <span class="btn-icon">
          {#if theme.mode === 'light'}
            ☀️
          {:else if theme.mode === 'dark'}
            🌙
          {:else}
            💻
          {/if}
        </span>
        <span class="btn-tag">Tema</span>
      </button>

      <button
        type="button"
        class="quick-action-btn lock-btn"
        onclick={() => vault.lockVault()}
        title="Trancar Mundo (Cmd+L)"
      >
        <span class="btn-icon">🔒</span>
        <span class="btn-tag">Trancar</span>
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

  :global(.editor-container:not(.show-line-numbers) .cm-gutters) {
    display: none !important;
  }

  :global(.editor-container.typewriter-scrolling .cm-content) {
    padding-top: 35vh !important;
    padding-bottom: 45vh !important;
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
    gap: 4px;
  }

  .quick-action-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    padding: 2px 7px;
    font-size: 11px;
    cursor: pointer;
    color: var(--color-ink-muted);
    transition: all 0.15s ease;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    line-height: 1;
    user-select: none;
    font-family: var(--font-ui);
  }

  .quick-action-btn:hover {
    background-color: var(--color-paper-border);
    border-color: var(--color-paper-border);
    color: var(--color-ink-primary);
  }

  .btn-icon {
    font-size: 11px;
    line-height: 1;
  }

  .btn-tag {
    font-size: 10.5px;
    font-weight: 500;
  }

  .quick-action-btn.active {
    background-color: var(--color-paper-border);
    color: var(--color-terracotta);
    font-weight: 600;
  }

  .quick-action-btn.lock-btn:hover {
    color: #e11d48;
  }

  .export-popover-wrapper {
    position: relative;
    display: inline-flex;
  }

  .export-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
    cursor: default;
  }

  .export-dropdown-popover {
    position: absolute;
    bottom: calc(100% + 8px);
    right: 0;
    z-index: 100;
    min-width: 220px;
    background-color: var(--color-paper-elevated, #ffffff);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg, 8px);
    box-shadow: 0 10px 25px -4px rgba(0, 0, 0, 0.12), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .export-dropdown-title {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-ink-faded);
    padding: 4px 8px;
    margin-bottom: 2px;
  }

  .export-dropdown-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    font-family: var(--font-ui);
    color: var(--color-ink-charcoal);
    transition: background-color 0.12s ease;
  }

  .export-dropdown-item:hover {
    background-color: var(--color-paper-hover);
  }

  .export-dropdown-item.primary-highlight {
    background-color: rgba(217, 119, 6, 0.08);
  }

  .export-dropdown-item.primary-highlight:hover {
    background-color: rgba(217, 119, 6, 0.16);
  }

  .export-item-icon {
    font-size: 14px;
    line-height: 1;
  }

  .export-item-info {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    flex: 1;
  }

  .export-item-name {
    font-size: 12px;
    font-weight: 500;
  }

  .export-item-ext {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--color-ink-muted);
  }

  .export-dropdown-divider {
    height: 1px;
    background-color: var(--color-paper-border);
    margin: 4px 0;
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
