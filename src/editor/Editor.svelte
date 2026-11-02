<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState, Compartment, Transaction, type Extension } from '@codemirror/state';
  import { lineNumbers } from '@codemirror/view';
  import { markdown } from '@codemirror/lang-markdown';
  import { vault } from '../vault/vault.svelte';
  import { manuscript } from '../vault/manuscript.svelte';
  import { AutosaveManager, type AutosaveStatus } from './autosave';
  import { splitFrontmatter, countWords, countChars } from './document';
  import { wikilinkDecoratorPlugin } from './wikilink-decorator';
  import { wikilinkAutocompleteExtension } from './wikilink-autocomplete';
  import { wikilinkHoverExtension } from './wikilink-hover';
  import { externalLinksPlugin } from './external-links';
  import { focusModeExtension, setFocusMode } from './focus-mode';
  import {
    richTextModeExtension,
    richTextKeymap,
    setRichTextMode,
    getActiveFormats,
    getFrontmatterEndLine,
    DEFAULT_ACTIVE_FORMATS,
    type ActiveFormats,
  } from './rich-text-mode';
  import { editorViewState } from './editor-view-state.svelte';
  import { theme } from '../design';
  import { editorThemeCompartment, getEditorThemeExtension } from './theme';
  import { editorPreferences } from './editor-preferences.svelte';
  import { typewriterAudio } from './typewriter-audio';
  import { focusPreferences } from '../preferences/focus-preferences.svelte';
  import SceneHeader from './SceneHeader.svelte';
  import LiteraryBar from './LiteraryBar.svelte';
  import FloatingHud from './FloatingHud.svelte';
  import StyleStatsModal from './StyleStatsModal.svelte';

  interface Props {
    initialContent?: string;
    onChange?: (value: string) => void;
  }

  let { initialContent = '', onChange }: Props = $props();

  let editorElement: HTMLDivElement | null = $state(null);
  let view = $state<EditorView | null>(null);
  let currentDoc = $state('');
  let autosaveStatus = $state<AutosaveStatus>('saved');
  let isStyleModalOpen = $state(false);
  let activeFormats = $state<ActiveFormats>(DEFAULT_ACTIVE_FORMATS);

  function ensureCursorOutsideFrontmatter(targetView: EditorView) {
    if (editorPreferences.editorMode !== 'visual') return;
    const fmEnd = getFrontmatterEndLine(targetView.state);
    if (fmEnd > 0 && fmEnd < targetView.state.doc.lines) {
      const bodyStartPos = targetView.state.doc.line(fmEnd + 1).from;
      if (targetView.state.selection.main.head < bodyStartPos) {
        targetView.dispatch({
          selection: { anchor: bodyStartPos },
        });
      }
    }
  }

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
  let currentEditingSceneId: string | null = null;
  let loadSequence = 0;
  let editorResizeObserver: ResizeObserver | null = null;
  let typewriterRafId: number | null = null;
  let lastSavedWikilinksSig = '';

  /**
   * Extrai uma assinatura leve dos [[wikilinks]] presentes no texto para evitar
   * recarregamentos desnecessários do índice global do cofre a cada autosave.
   */
  function extractWikilinksSignature(text: string): string {
    const matches = text.match(/\[\[([^\]|\n]+)(?:\|[^\]\n]+)?\]\]/g);
    if (!matches || matches.length === 0) return '';
    return Array.from(new Set(matches)).sort().join('|');
  }

  function recenterTypewriter() {
    if (view && editorPreferences.typewriterScrolling) {
      view.dispatch({
        effects: EditorView.scrollIntoView(view.state.selection.main.head, { y: 'center' }),
      });
    }
  }

  function handleWindowResize() {
    if (editorPreferences.typewriterScrolling) {
      requestAnimationFrame(recenterTypewriter);
    }
  }

  let bodyOnly = $derived(splitFrontmatter(currentDoc).body);
  let wordsCount = $derived(countWords(bodyOnly));
  let charsCount = $derived(countChars(bodyOnly));
  let statsModalTab = $state<'style' | 'productivity'>('style');

  onMount(() => {
    if (!editorElement) {
      return;
    }
    currentDoc = initialContent;
    currentEditingSceneId = manuscript.activeSceneId;
    lastSavedWikilinksSig = extractWikilinksSignature(initialContent);

    autosaveManager = new AutosaveManager({
      delayMs: 800,
      onSave: async (content, targetId) => {
        if (vault.isOpen && !vault.isLocked) {
          const effectiveId = targetId || currentEditingSceneId || manuscript.activeSceneId || 'default_scene';
          await vault.saveScene(effectiveId, content);

          const nextWikilinksSig = extractWikilinksSignature(content);
          const wikilinksChanged = nextWikilinksSig !== lastSavedWikilinksSig;
          lastSavedWikilinksSig = nextWikilinksSig;

          if (wikilinksChanged || !vault.indexOverview) {
            await vault.loadVaultIndex();
            if (effectiveId && vault.isInspectorOpen) {
              await vault.loadBacklinks(effectiveId);
            }
          }
        }
      },
      onStatusChange: (status) => {
        autosaveStatus = status;
      },
    });

    vault.registerBeforeLock(async () => {
      if (autosaveManager) {
        await autosaveManager.flush();
      }
      await editorViewState.flushSessionToVault();
    });

    let prevWords = countWords(splitFrontmatter(initialContent).body);
    let lastCursorLine = -1;
    let isPasting = false;
    let pasteTimeout: ReturnType<typeof setTimeout> | null = null;

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged || update.selectionSet) {
        activeFormats = getActiveFormats(update.state);
      }

      if (editorPreferences.typewriterScrolling && (update.docChanged || update.selectionSet)) {
        if (typewriterRafId !== null) {
          cancelAnimationFrame(typewriterRafId);
        }
        typewriterRafId = requestAnimationFrame(() => {
          typewriterRafId = null;
          recenterTypewriter();
        });
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
        autosaveManager?.notifyChange(text, currentEditingSceneId || undefined);

        const newWords = countWords(splitFrontmatter(text).body);
        const wordsDelta = newWords - prevWords;

        // Detecta se a transação do CodeMirror ou o evento de entrada é de colagem (paste) ou arrasto (drop)
        const isPasteOrDrop =
          isPasting ||
          update.transactions.some(
            (tr) =>
              tr.isUserEvent('input.paste') ||
              tr.isUserEvent('input.drop') ||
              tr.annotation(Transaction.userEvent)?.startsWith('input.paste') ||
              tr.annotation(Transaction.userEvent)?.startsWith('input.drop')
          );

        // A meta diária conta apenas a digitação autêntica e contínua do autor.
        // Colagens de blocos de texto (paste/drop ou grandes inserções súbitas >= 15 palavras)
        // atualizam a contagem total do texto, mas são desconsideradas da meta diária da sessão.
        if (wordsDelta > 0 && !isPasteOrDrop && wordsDelta < 15) {
          editorViewState.recordWordsDelta(wordsDelta);
          focusPreferences.checkDailyGoalAchieved(
            editorViewState.productivity.today_words + editorViewState.sessionWords
          );
        }
        prevWords = newWords;
      }
    });

    const typewriterKeyHandler = EditorView.domEventHandlers({
      paste() {
        isPasting = true;
        if (pasteTimeout) clearTimeout(pasteTimeout);
        pasteTimeout = setTimeout(() => {
          isPasting = false;
        }, 150);
        return false;
      },
      drop() {
        isPasting = true;
        if (pasteTimeout) clearTimeout(pasteTimeout);
        pasteTimeout = setTimeout(() => {
          isPasting = false;
        }, 150);
        return false;
      },
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
        EditorView.lineWrapping,
        lineNumbersCompartment.of(getLineNumbersExtension(editorPreferences.relativeLineNumbers)),
        ...((basicSetup as unknown as Extension[]).slice(1)),
        markdown(),
        richTextKeymap,
        richTextModeExtension,
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
    view.dispatch({
      effects: setRichTextMode.of(editorPreferences.editorMode === 'visual'),
    });
    ensureCursorOutsideFrontmatter(view);
    activeFormats = getActiveFormats(view.state);
    editorViewState.applyInitialFocus(view);

    if (editorElement && typeof ResizeObserver !== 'undefined') {
      editorResizeObserver = new ResizeObserver(() => {
        if (editorPreferences.typewriterScrolling) {
          requestAnimationFrame(recenterTypewriter);
        }
      });
      editorResizeObserver.observe(editorElement);
    }
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

  // Recentraliza o typewriter scrolling sempre que o usuário ativar a preferência
  $effect(() => {
    if (editorPreferences.typewriterScrolling && view) {
      requestAnimationFrame(recenterTypewriter);
    }
  });

  // Sincroniza o Modo Foco com a extensão do CodeMirror
  $effect(() => {
    if (view) {
      view.dispatch({
        effects: setFocusMode.of(editorViewState.isFocusMode),
      });
    }
  });

  // Sincroniza o Modo do Editor (Visual Rich Text vs Markdown)
  $effect(() => {
    const isVisual = editorPreferences.editorMode === 'visual';
    if (view) {
      view.dispatch({
        effects: setRichTextMode.of(isVisual),
      });
      if (isVisual) {
        ensureCursorOutsideFrontmatter(view);
      }
    }
  });

  // Reage à mudança de cena ativa selecionada na barra lateral
  $effect(() => {
    const activeId = manuscript.activeSceneId;
    if (activeId && activeId !== currentEditingSceneId && view) {
      const mySeq = ++loadSequence;

      void (async () => {
        // Se houver edições não salvas no documento anterior, força o salvamento com o ID dele
        if (autosaveManager) {
          await autosaveManager.flush();
        }
        if (mySeq !== loadSequence) return;

        try {
          const scene = manuscript.items.find((i) => i.id === activeId);
          const blobId = scene?.blob_id || activeId;
          const content = await vault.loadScene(blobId);

          if (mySeq !== loadSequence) return;

          currentEditingSceneId = activeId;
          currentDoc = content;
          lastSavedWikilinksSig = extractWikilinksSignature(content);
          if (view) {
            view.dispatch({
              changes: {
                from: 0,
                to: view.state.doc.length,
                insert: content,
              },
            });
            ensureCursorOutsideFrontmatter(view);
            activeFormats = getActiveFormats(view.state);
            if (editorPreferences.typewriterScrolling) {
              requestAnimationFrame(recenterTypewriter);
            }
          }
        } catch (err) {
          console.error('Falha ao carregar conteúdo da cena:', err);
        }
      })();
    }
  });

  onDestroy(() => {
    vault.registerBeforeLock(null);
    if (typewriterRafId !== null) {
      cancelAnimationFrame(typewriterRafId);
      typewriterRafId = null;
    }
    if (editorResizeObserver) {
      editorResizeObserver.disconnect();
      editorResizeObserver = null;
    }
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

<svelte:window onkeydown={handleKeydown} onresize={handleWindowResize} />

<div class="editor-wrapper">
  {#if manuscript.activeScene}
    <SceneHeader />
  {/if}

  {#if !editorViewState.isZenMode}
    <LiteraryBar {view} {activeFormats} />
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
    <!-- Canto Esquerdo: Estatísticas em fonte menor como link para o modal unificado -->
    <button
      type="button"
      class="stats-link"
      onclick={() => {
        statsModalTab = 'style';
        isStyleModalOpen = true;
      }}
      title="Clique para ver Estatísticas Estilométricas & Produtividade"
    >
      <span>{wordsCount} {wordsCount === 1 ? 'palavra' : 'palavras'}</span>
      <span class="dot">·</span>
      <span>{charsCount} {charsCount === 1 ? 'caractere' : 'caracteres'}</span>
    </button>

    <!-- Centro: Alternador de Modo (Editor Comum vs Markdown) e Modo Foco -->
    <div class="footer-center">
      <button
        type="button"
        class="focus-pill-btn"
        class:active={editorPreferences.editorMode === 'visual'}
        onclick={() => {
          editorPreferences.toggleEditorMode();
          view?.focus();
        }}
        title={editorPreferences.editorMode === 'visual'
          ? 'Modo Editor Comum ativo (clique para ver código Markdown)'
          : 'Modo Markdown ativo (clique para usar Editor Visual Rich Text)'}
      >
        {editorPreferences.editorMode === 'visual' ? '📝 Editor Comum' : '⌨️ Markdown'}
      </button>

      {#if editorViewState.isFocusMode}
        <button
          type="button"
          class="focus-pill-btn active"
          onclick={() => editorViewState.toggleFocus(view)}
          title="Sair do Modo Foco (Esc)"
        >
          ✕ Sair do Modo Foco (Esc)
        </button>
      {:else}
        <button
          type="button"
          class="focus-pill-btn"
          onclick={() => editorViewState.toggleFocus(view)}
          title="Ativar Modo Foco (Cmd+Alt+F)"
        >
          ✨ Modo Foco
        </button>
      {/if}
    </div>

    <!-- Canto Direito: Status 'salvando' > 'salvo', cadeado de trancar e botão sanduíche do Inspector -->
    <div class="footer-right">
      <div class="save-status-text" title="Status de gravação cifrada no cofre">
        {#if autosaveStatus === 'saving'}
          <span class="save-dot saving">●</span>
          <span>Salvando...</span>
        {:else}
          <span class="save-dot saved">✓</span>
          <span>Salvo</span>
        {/if}
      </div>

      <button
        type="button"
        class="icon-action-btn lock-action"
        onclick={() => vault.lockVault()}
        title="Trancar Mundo (Cmd+L)"
        aria-label="Trancar Mundo"
      >
        🔒
      </button>

      <button
        type="button"
        class="icon-action-btn inspector-toggle"
        class:active={vault.isInspectorOpen}
        onclick={() => vault.toggleInspector()}
        title="Alternar Inspetor Lateral (Cmd+I)"
        aria-label="Alternar Inspetor Lateral"
      >
        ☰
      </button>
    </div>
  </footer>

  <StyleStatsModal
    isOpen={isStyleModalOpen}
    text={currentDoc}
    sceneTitle={manuscript.activeScene?.title}
    initialTab={statsModalTab}
    onClose={() => (isStyleModalOpen = false)}
  />
</div>

<style>
  .editor-wrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    background-color: var(--color-paper-base);
    overflow-x: hidden;
  }

  .editor-container {
    flex: 1;
    width: 100%;
    overflow-x: hidden;
    overflow-y: hidden;
    box-sizing: border-box;
    background-color: var(--color-paper-base);
    position: relative;
  }

  :global(.editor-container .cm-editor) {
    height: 100%;
    outline: none !important;
  }

  :global(.editor-container .cm-scroller) {
    overflow-x: hidden !important;
    overflow-y: auto !important;
  }

  :global(.editor-container:not(.show-line-numbers) .cm-gutters) {
    display: none !important;
  }

  :global(.editor-container.typewriter-scrolling .cm-content) {
    padding-top: 35vh !important;
    padding-bottom: 45vh !important;
  }

  .editor-footer {
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-md);
    background-color: var(--color-paper-surface);
    border-top: 1px solid var(--color-paper-border);
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-ink-muted);
    user-select: none;
    z-index: 10;
  }

  .stats-link {
    display: flex;
    align-items: center;
    gap: 5px;
    background: transparent;
    border: none;
    padding: 3px 6px;
    border-radius: var(--rounded-sm, 4px);
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-ink-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .stats-link:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-olive);
  }

  .dot {
    opacity: 0.5;
  }

  .footer-center {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .focus-pill-btn {
    border: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-base);
    color: var(--color-ink-muted);
    border-radius: 9999px;
    padding: 2px 10px;
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .focus-pill-btn:hover {
    border-color: var(--color-olive);
    color: var(--color-ink-charcoal);
    background-color: var(--color-paper-hover);
  }

  .focus-pill-btn.active {
    background-color: var(--color-paper-hover);
    color: var(--color-olive);
    border-color: var(--color-olive);
    font-weight: 600;
  }

  .footer-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .save-status-text {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--color-ink-faded);
  }

  .save-dot {
    font-size: 8px;
    line-height: 1;
  }

  .save-dot.saving {
    color: var(--color-olive);
    animation: pulse 1s infinite alternate;
  }

  .save-dot.saved {
    color: var(--color-olive);
    font-size: 11px;
  }

  .icon-action-btn {
    background: transparent;
    border: none;
    font-size: 13px;
    cursor: pointer;
    padding: 3px 5px;
    border-radius: 4px;
    color: var(--color-ink-muted);
    transition: all 0.15s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .icon-action-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .icon-action-btn.inspector-toggle.active {
    color: var(--color-olive);
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
