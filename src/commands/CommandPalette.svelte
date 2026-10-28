<script lang="ts">
  /**
   * Paleta de Comandos Global e Quick Open (M9-01).
   *
   * Atalho: Cmd+K / Ctrl+K.
   * Permite navegação rápida por todo o manuscrito e acionamento de qualquer comando.
   */

  import { onMount } from 'svelte';
  import { commandPaletteStore } from './palette.svelte';
  import { manuscript } from '../vault/manuscript.svelte';
  import { vault } from '../vault/vault.svelte';
  import { theme } from '../design';
  import { editorViewState } from '../editor/editor-view-state.svelte';
  import { timelineStore } from '../timeline/timeline.svelte';
  import { graphStore } from '../graph/graph.svelte';
  import { backupStore } from '../backup/backup.svelte';
  import { compilerStore } from '../compiler/compiler.svelte';
  import { searchStore } from '../search/search.svelte';
  import { supportStore } from '../support/support.svelte';

  interface PaletteEntry {
    id: string;
    category: 'navigation' | 'action';
    title: string;
    subtitle?: string;
    icon: string;
    shortcut?: string;
    execute: () => void;
  }

  let searchQuery = $state('');
  let selectedIndex = $state(0);
  let inputElement: HTMLInputElement | null = $state(null);

  // Lista estática de ações gerais disponíveis
  const staticActions = $derived<PaletteEntry[]>([
    {
      id: 'action-search',
      category: 'action',
      title: 'Busca Global no Manuscrito',
      subtitle: 'Pesquisar em todas as cenas e notas do projeto',
      icon: '🔍',
      shortcut: 'Cmd+Shift+F',
      execute: () => {
        searchStore.openModal();
      },
    },
    {
      id: 'action-timeline',
      category: 'action',
      title: 'Linha do Tempo Literária',
      subtitle: 'Visualizar cronologia e eventos da história',
      icon: '⏳',
      shortcut: 'Cmd+Shift+T',
      execute: () => {
        void timelineStore.openModal();
      },
    },
    {
      id: 'action-graph',
      category: 'action',
      title: 'Grafo Narrativo & Canvas',
      subtitle: 'Conexões entre personagens, locais e enredos',
      icon: '🕸️',
      shortcut: 'Cmd+Shift+G',
      execute: () => {
        graphStore.openModal();
      },
    },
    {
      id: 'action-productivity',
      category: 'action',
      title: 'Metas & Ritmo de Escrita',
      subtitle: 'Acompanhar progresso diário e velocidade de escrita',
      icon: '📊',
      shortcut: 'Cmd+Shift+P',
      execute: () => {
        editorViewState.openProductivityDashboard();
      },
    },
    {
      id: 'action-zen',
      category: 'action',
      title: 'Alternar Modo Zen Imersivo',
      subtitle: 'Ocultar toda a interface para foco pleno',
      icon: '🧘',
      shortcut: 'F11',
      execute: () => {
        editorViewState.toggleZen();
      },
    },
    {
      id: 'action-theme',
      category: 'action',
      title: 'Alternar Modo Claro / Escuro',
      subtitle: `Atualmente em modo ${theme.isDark ? 'Escuro' : 'Claro (Branco Puro)'}`,
      icon: theme.isDark ? '☀️' : '🌙',
      execute: () => {
        theme.toggleTheme();
      },
    },
    {
      id: 'action-export-docx',
      category: 'action',
      title: 'Exportar para Word (.docx)',
      subtitle: 'Gerar manuscrito pronto para editoras',
      icon: '📄',
      shortcut: 'Cmd+Shift+E',
      execute: () => {
        void compilerStore.quickExportDirect('docx');
      },
    },
    {
      id: 'action-export-epub',
      category: 'action',
      title: 'Exportar para Livro Digital (.epub)',
      subtitle: 'E-book para Apple Books, Kobo e Kindle',
      icon: '📱',
      execute: () => {
        void compilerStore.quickExportDirect('epub');
      },
    },
    {
      id: 'action-export-pdf',
      category: 'action',
      title: 'Exportar para Leitura / PDF (.html)',
      subtitle: 'Tipografia editorial para impressão ou leitura',
      icon: '🖨️',
      execute: () => {
        void compilerStore.quickExportDirect('pdf');
      },
    },
    {
      id: 'action-export-obsidian',
      category: 'action',
      title: 'Exportar para Cofre Obsidian (.md)',
      subtitle: 'Notas abertas com hiperlinks bidirecionais',
      icon: '💎',
      execute: () => {
        void compilerStore.quickExportDirect('obsidian');
      },
    },
    {
      id: 'action-export-txt',
      category: 'action',
      title: 'Exportar Texto Puro + Dossiê (.txt)',
      subtitle: 'Arquivo consolidado e fichas em texto puro',
      icon: '📝',
      execute: () => {
        void compilerStore.quickExportDirect('txt');
      },
    },
    {
      id: 'action-backup',
      category: 'action',
      title: 'Configurações de Backup Cifrado',
      subtitle: 'Rotatividade automática e snapshots de segurança',
      icon: '💾',
      execute: () => {
        backupStore.openModal();
      },
    },
    {
      id: 'action-support',
      category: 'action',
      title: 'Apoie o Magrathea Writer',
      subtitle: 'Informações de apoio e contribuição voluntária para o projeto livre',
      icon: '☕',
      execute: () => {
        supportStore.open();
      },
    },
    {
      id: 'action-lock',
      category: 'action',
      title: 'Trancar Mundo Literário',
      subtitle: 'Cifrar chaves da memória e proteger o cofre',
      icon: '🔒',
      shortcut: 'Cmd+L',
      execute: () => {
        void vault.lockVault();
      },
    },
  ]);

  // Lista dinâmica de itens do manuscrito (partes, capítulos, cenas, entidades)
  const manuscriptEntries = $derived<PaletteEntry[]>(
    manuscript.items.map((item) => {
      let icon = '🎬';
      let subtitle = 'Cena';
      if (item.kind === 'part') {
        icon = '📚';
        subtitle = 'Parte do Livro';
      } else if (item.kind === 'chapter') {
        icon = '📖';
        subtitle = 'Capítulo';
      } else if (item.kind === 'character') {
        icon = '👤';
        subtitle = 'Personagem';
      } else if (item.kind === 'location') {
        icon = '📍';
        subtitle = 'Local / Cenário';
      } else if (item.kind === 'item') {
        icon = '🗡️';
        subtitle = 'Objeto / Item';
      } else if (item.kind === 'faction') {
        icon = '🛡️';
        subtitle = 'Facção / Grupo';
      } else if (item.kind === 'lore') {
        icon = '📜';
        subtitle = 'Mito / Lore';
      }

      return {
        id: item.id,
        category: 'navigation',
        title: item.title,
        subtitle,
        icon,
        execute: () => {
          manuscript.selectScene(item.id);
        },
      };
    })
  );

  // Filtra e classifica as entradas com base no termo de busca
  const filteredEntries = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    const all = [...manuscriptEntries, ...staticActions];
    if (!q) {
      return all;
    }
    return all.filter((entry) => {
      const matchTitle = entry.title.toLowerCase().includes(q);
      const matchSubtitle = entry.subtitle?.toLowerCase().includes(q);
      return matchTitle || matchSubtitle;
    });
  });

  // Ajusta o índice selecionado caso a lista diminua
  $effect(() => {
    if (selectedIndex >= filteredEntries.length) {
      selectedIndex = Math.max(0, filteredEntries.length - 1);
    }
  });

  // Foco no input ao abrir a paleta
  $effect(() => {
    if (commandPaletteStore.isOpen) {
      searchQuery = '';
      selectedIndex = 0;
      setTimeout(() => {
        inputElement?.focus();
      }, 50);
    }
  });

  function handleSelectEntry(entry: PaletteEntry) {
    commandPaletteStore.close();
    entry.execute();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      commandPaletteStore.close();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (filteredEntries.length > 0) {
        selectedIndex = (selectedIndex + 1) % filteredEntries.length;
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (filteredEntries.length > 0) {
        selectedIndex =
          (selectedIndex - 1 + filteredEntries.length) % filteredEntries.length;
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const current = filteredEntries[selectedIndex];
      if (current) {
        handleSelectEntry(current);
      }
    }
  }

  onMount(() => {
    const listener = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault();
        commandPaletteStore.toggle();
      }
    };
    window.addEventListener('keydown', listener);
    return () => {
      window.removeEventListener('keydown', listener);
    };
  });
</script>

{#if commandPaletteStore.isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="palette-backdrop"
    onclick={() => commandPaletteStore.close()}
  ></div>

  <!-- Container Flutuante Centralizado -->
  <div class="palette-container" role="dialog" aria-modal="true" aria-label="Paleta de Comandos">
    <div class="palette-header">
      <span class="palette-search-icon">🔍</span>
      <input
        bind:this={inputElement}
        type="text"
        class="palette-input"
        placeholder="Navegue para uma cena/personagem ou digite um comando..."
        bind:value={searchQuery}
        onkeydown={handleKeydown}
      />
      <span class="palette-esc-badge">ESC</span>
    </div>

    <div class="palette-list" role="listbox">
      {#if filteredEntries.length === 0}
        <div class="palette-empty">Nenhum comando ou documento encontrado.</div>
      {:else}
        {#each filteredEntries as entry, index}
          {@const isSelected = index === selectedIndex}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="palette-item"
            class:selected={isSelected}
            role="option"
            aria-selected={isSelected}
            tabindex="-1"
            onclick={() => handleSelectEntry(entry)}
            onmouseenter={() => (selectedIndex = index)}
          >
            <span class="entry-icon">{entry.icon}</span>
            <div class="entry-text">
              <span class="entry-title">{entry.title}</span>
              {#if entry.subtitle}
                <span class="entry-subtitle">{entry.subtitle}</span>
              {/if}
            </div>

            {#if entry.shortcut}
              <kbd class="entry-shortcut">{entry.shortcut}</kbd>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    <div class="palette-footer">
      <span>Use <kbd>↑</kbd> <kbd>↓</kbd> para navegar</span>
      <span>•</span>
      <span><kbd>↵</kbd> para selecionar</span>
      <span>•</span>
      <span><kbd>esc</kbd> para fechar</span>
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
    background-color: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(2px);
  }

  .palette-container {
    position: fixed;
    top: 15%;
    left: 50%;
    transform: translateX(-50%);
    width: 90%;
    max-width: 580px;
    z-index: 1000;
    background-color: var(--color-paper-elevated, #ffffff);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-xl, 12px);
    box-shadow: 0 20px 40px -10px rgba(0, 0, 0, 0.25);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    animation: palette-enter 0.15s ease-out;
  }

  @keyframes palette-enter {
    from {
      opacity: 0;
      transform: translate(-50%, -10px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }

  .palette-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-paper-border);
  }

  .palette-search-icon {
    font-size: 16px;
    opacity: 0.6;
  }

  .palette-input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-ink-charcoal);
  }

  .palette-esc-badge {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--color-ink-muted);
    background: var(--color-paper-hover);
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--color-paper-border);
  }

  .palette-list {
    max-height: 340px;
    overflow-y: auto;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .palette-empty {
    padding: 24px;
    text-align: center;
    font-size: 13px;
    color: var(--color-ink-muted);
    font-family: var(--font-ui);
  }

  .palette-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-radius: 8px;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .palette-item.selected {
    background-color: var(--color-olive-subtle, #f0fdf4);
  }

  :global([data-theme="dark"]) .palette-item.selected {
    background-color: rgba(94, 111, 69, 0.25);
  }

  .entry-icon {
    font-size: 16px;
    line-height: 1;
  }

  .entry-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .entry-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-ink-charcoal);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .entry-subtitle {
    font-size: 11px;
    color: var(--color-ink-muted);
  }

  .entry-shortcut {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--color-ink-muted);
    background: var(--color-paper-hover);
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--color-paper-border);
  }

  .palette-footer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px 16px;
    background-color: var(--color-paper-surface);
    border-top: 1px solid var(--color-paper-border);
    font-size: 11px;
    color: var(--color-ink-muted);
    font-family: var(--font-ui);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 4px;
    background: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: 3px;
  }
</style>
