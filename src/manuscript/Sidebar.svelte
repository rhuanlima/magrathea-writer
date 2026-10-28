<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '../i18n';
  import { manuscript, type TreeNode } from '../vault/manuscript.svelte';
  import type { ManifestItemKind } from '../tauri/bridge';
  import EncyclopediaView from '../worldbuilding/EncyclopediaView.svelte';

  const DEFAULT_WIDTH = 280;
  const MIN_WIDTH = 220;
  const MAX_WIDTH = 600;

  let sidebarWidth = $state(DEFAULT_WIDTH);
  let isResizing = $state(false);

  onMount(() => {
    try {
      const saved = localStorage.getItem('magrathea_sidebar_width');
      if (saved) {
        const val = parseInt(saved, 10);
        if (val >= MIN_WIDTH && val <= MAX_WIDTH) {
          sidebarWidth = val;
        }
      }
    } catch {
      // Ignorar falha em ambientes sem localStorage
    }
  });

  function startResize(e: PointerEvent) {
    e.preventDefault();
    isResizing = true;
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    function onPointerMove(ev: PointerEvent) {
      const delta = ev.clientX - startX;
      const newWidth = Math.min(Math.max(startWidth + delta, MIN_WIDTH), MAX_WIDTH);
      sidebarWidth = newWidth;
    }

    function onPointerUp() {
      isResizing = false;
      window.removeEventListener('pointermove', onPointerMove);
      window.removeEventListener('pointerup', onPointerUp);
      try {
        localStorage.setItem('magrathea_sidebar_width', sidebarWidth.toString());
      } catch {
        // Ignorar
      }
    }

    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
  }

  let activeSidebarTab = $state<'manuscript' | 'worldbuilding'>('manuscript');
  let editingItemId = $state<string | null>(null);
  let editingTitle = $state('');

  function startEditing(id: string, currentTitle: string) {
    editingItemId = id;
    editingTitle = currentTitle;
  }

  function saveEditing(id: string) {
    if (editingTitle.trim()) {
      void manuscript.renameItem(id, editingTitle.trim());
    }
    editingItemId = null;
  }

  function handleKeyDown(e: KeyboardEvent, id: string) {
    if (e.key === 'Enter') {
      saveEditing(id);
    } else if (e.key === 'Escape') {
      editingItemId = null;
    }
  }

  function handleCreateChild(kind: ManifestItemKind, parentId: string) {
    void manuscript.createItem(kind, parentId);
  }

  function handleCreateRoot(kind: ManifestItemKind) {
    void manuscript.createItem(kind, null);
  }

  function getStatusLabel(status: string | null): string {
    if (status === 'draft') return t('manuscript.status.draft');
    if (status === 'revised') return t('manuscript.status.revised');
    if (status === 'final') return t('manuscript.status.final');
    return status || '';
  }
</script>

{#snippet renderNode(node: TreeNode, depth: number)}
  {@const item = node.item}
  {@const isSelected = item.id === manuscript.activeSceneId}
  {@const isEditing = editingItemId === item.id}

  <div
    class="tree-node depth-{depth} kind-{item.kind}"
    class:selected={isSelected}
  >
    <div
      class="node-main"
      onclick={() => {
        if (item.kind === 'scene') {
          manuscript.selectScene(item.id);
        }
      }}
      ondblclick={() => startEditing(item.id, item.title)}
      onkeydown={(e) => {
        if (e.key === 'Enter' && item.kind === 'scene') {
          manuscript.selectScene(item.id);
        }
      }}
      tabindex="0"
      role="button"
    >
      <span class="kind-icon" title={item.kind}>
        {#if item.kind === 'part'}
          §
        {:else if item.kind === 'chapter'}
          ¶
        {:else}
          •
        {/if}
      </span>

      {#if isEditing}
        <input
          type="text"
          class="inline-edit-input"
          bind:value={editingTitle}
          onblur={() => saveEditing(item.id)}
          onkeydown={(e) => handleKeyDown(e, item.id)}
        />
      {:else}
        <span class="node-title" title={item.title}>
          {item.title}
        </span>
      {/if}

      {#if item.kind === 'scene' && item.status}
        <span
          class="status-dot status-{item.status}"
          title={getStatusLabel(item.status)}
        ></span>
      {/if}
    </div>

    <div class="node-actions">
      {#if item.kind === 'part'}
        <button
          type="button"
          class="action-btn"
          onclick={() => handleCreateChild('chapter', item.id)}
          title={t('manuscript.newChapter')}
        >
          +C
        </button>
      {:else if item.kind === 'chapter'}
        <button
          type="button"
          class="action-btn"
          onclick={() => handleCreateChild('scene', item.id)}
          title={t('manuscript.newScene')}
        >
          +S
        </button>
      {/if}

      <button
        type="button"
        class="action-btn"
        onclick={() => manuscript.moveSibling(item.id, 'up')}
        title={t('manuscript.moveUp')}
      >
        ▲
      </button>

      <button
        type="button"
        class="action-btn"
        onclick={() => manuscript.moveSibling(item.id, 'down')}
        title={t('manuscript.moveDown')}
      >
        ▼
      </button>

      <button
        type="button"
        class="action-btn delete-btn"
        onclick={() => manuscript.moveToTrash(item.id)}
        title={t('manuscript.delete')}
      >
        ×
      </button>
    </div>
  </div>

  {#if node.children.length > 0}
    <div class="children-container">
      {#each node.children as child (child.item.id)}
        {@render renderNode(child, depth + 1)}
      {/each}
    </div>
  {/if}
{/snippet}

<aside
  class="sidebar-container"
  class:collapsed={!manuscript.isSidebarOpen}
  class:is-resizing={isResizing}
  style={manuscript.isSidebarOpen ? `width: ${sidebarWidth}px;` : 'width: 0;'}
>
  <!-- Alternância entre Manuscrito e Worldbuilding -->
  <div class="sidebar-tabs">
    <button
      type="button"
      class="sidebar-tab-btn"
      class:active={activeSidebarTab === 'manuscript'}
      onclick={() => (activeSidebarTab = 'manuscript')}
    >
      📖 {t('manuscript.treeTitle')}
    </button>
    <button
      type="button"
      class="sidebar-tab-btn"
      class:active={activeSidebarTab === 'worldbuilding'}
      onclick={() => (activeSidebarTab = 'worldbuilding')}
    >
      🌐 {t('worldbuilding.tabTitle')}
    </button>
  </div>

  {#if activeSidebarTab === 'manuscript'}
    <header class="sidebar-header">
      <div class="header-left">
        <span class="tree-title">{t('manuscript.treeTitle')}</span>
      </div>
      <div class="header-buttons">
        <button
          type="button"
          class="add-root-btn"
          onclick={() => handleCreateRoot('part')}
          title={t('manuscript.newPart')}
        >
          + {t('manuscript.newPart')}
        </button>
        <button
          type="button"
          class="add-root-btn"
          onclick={() => handleCreateRoot('chapter')}
          title={t('manuscript.newChapter')}
        >
          + {t('manuscript.newChapter')}
        </button>
      </div>
    </header>

    <div class="tree-content">
      {#if manuscript.tree.length === 0}
        <div class="empty-tree-message">
          <p>{t('manuscript.emptyTree')}</p>
        </div>
      {:else}
        <div class="tree-root">
          {#each manuscript.tree as rootNode (rootNode.item.id)}
            {@render renderNode(rootNode, 0)}
          {/each}
        </div>
      {/if}
    </div>

    <footer class="sidebar-footer">
      <button
        type="button"
        class="trash-trigger-btn"
        onclick={() => manuscript.toggleTrash(true)}
        title={t('manuscript.trash')}
      >
        <span class="trash-icon">🗑️</span>
        <span class="trash-label">{t('manuscript.trash')}</span>
        {#if manuscript.trashCount > 0}
          <span class="trash-pill">{manuscript.trashCount}</span>
        {/if}
      </button>
    </footer>
  {:else}
    <EncyclopediaView />
  {/if}

  {#if manuscript.isSidebarOpen}
    <div
      class="sidebar-resizer"
      onpointerdown={startResize}
      role="separator"
      aria-orientation="vertical"
      aria-valuenow={sidebarWidth}
      aria-valuemin={MIN_WIDTH}
      aria-valuemax={MAX_WIDTH}
      tabindex="-1"
    ></div>
  {/if}
</aside>

<style>
  .sidebar-container {
    width: 280px;
    height: 100%;
    position: relative;
    background-color: var(--color-paper-surface);
    border-right: 1px solid var(--color-paper-border);
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    user-select: none;
    overflow: hidden;
    flex-shrink: 0;
  }

  .sidebar-container:not(.is-resizing) {
    transition: width 0.2s ease, transform 0.2s ease;
  }

  .sidebar-container.collapsed {
    width: 0 !important;
    border-right: none;
  }

  .sidebar-resizer {
    position: absolute;
    top: 0;
    right: 0;
    width: 5px;
    height: 100%;
    cursor: col-resize;
    z-index: 50;
    transition: background-color 0.15s ease;
  }

  .sidebar-resizer:hover,
  .sidebar-container.is-resizing .sidebar-resizer {
    background-color: var(--color-olive, #5E6F45);
  }

  .sidebar-tabs {
    display: flex;
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .sidebar-tab-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 8px 6px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    font-size: 11px;
    font-weight: 600;
    font-family: var(--font-ui);
    color: var(--color-ink-muted);
    cursor: pointer;
    transition: all 150ms ease;
  }

  .sidebar-tab-btn:hover {
    color: var(--color-ink-charcoal);
    background-color: var(--color-paper-hover);
  }

  .sidebar-tab-btn.active {
    color: var(--color-olive);
    border-bottom-color: var(--color-olive);
    background-color: var(--color-paper-base);
  }

  .sidebar-header {
    padding: var(--space-sm) var(--space-md);
    background-color: var(--color-paper-base);
    border-bottom: 1px solid var(--color-paper-border);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .header-left {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .tree-title {
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--color-ink-muted);
  }

  .header-buttons {
    display: flex;
    gap: 4px;
  }

  .add-root-btn {
    flex: 1;
    font-family: var(--font-ui);
    font-size: 11px;
    padding: 3px 6px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    color: var(--color-ink-charcoal);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: background-color 0.15s ease;
  }

  .add-root-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-olive);
  }

  .tree-content {
    flex: 1;
    overflow-y: auto;
    padding: 6px 0;
  }

  .empty-tree-message {
    padding: var(--space-md);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-ink-faded);
    text-align: center;
    line-height: 1.4;
  }

  .tree-root {
    display: flex;
    flex-direction: column;
  }

  .tree-node {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    cursor: pointer;
    transition: background-color 0.12s ease;
  }

  .tree-node:hover {
    background-color: var(--color-paper-hover);
  }

  .tree-node.selected {
    background-color: rgba(94, 111, 69, 0.15);
    font-weight: 600;
  }

  .tree-node.depth-0 {
    padding-left: 8px;
  }

  .tree-node.depth-1 {
    padding-left: 18px;
  }

  .tree-node.depth-2 {
    padding-left: 28px;
  }

  .tree-node.kind-part {
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .tree-node.kind-chapter {
    color: var(--color-ink-charcoal);
  }

  .tree-node.kind-scene {
    font-size: 13px;
    color: var(--color-ink-charcoal);
  }

  .node-main {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    overflow: hidden;
    outline: none;
  }

  .kind-icon {
    font-family: var(--font-literary);
    font-size: 13px;
    color: var(--color-ink-faded);
    width: 12px;
    display: inline-block;
    text-align: center;
  }

  .node-title {
    font-family: var(--font-ui);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .inline-edit-input {
    font-family: var(--font-ui);
    font-size: 12px;
    padding: 1px 4px;
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-olive);
    border-radius: var(--radius-sm);
    color: var(--color-ink-charcoal);
    outline: none;
    width: 90%;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-draft {
    background-color: #d97706; /* Âmbar */
  }

  .status-revised {
    background-color: #2563eb; /* Azul */
  }

  .status-final {
    background-color: #16a34a; /* Verde */
  }

  .node-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .tree-node:hover .node-actions {
    opacity: 1;
  }

  .action-btn {
    border: none;
    background: transparent;
    padding: 1px 3px;
    font-size: 10px;
    cursor: pointer;
    color: var(--color-ink-faded);
    border-radius: 2px;
  }

  .action-btn:hover {
    background-color: var(--color-paper-border);
    color: var(--color-ink-charcoal);
  }

  .delete-btn:hover {
    color: var(--color-error);
  }

  .sidebar-footer {
    padding: 8px var(--space-md);
    background-color: var(--color-paper-base);
    border-top: 1px solid var(--color-paper-border);
  }

  .trash-trigger-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 8px;
    border: 1px solid transparent;
    background: transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--color-ink-muted);
    font-family: var(--font-ui);
    font-size: 12px;
    transition: background-color 0.15s ease;
  }

  .trash-trigger-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .trash-icon {
    font-size: 13px;
  }

  .trash-label {
    flex: 1;
    text-align: left;
  }

  .trash-pill {
    background-color: var(--color-paper-border);
    color: var(--color-ink-muted);
    font-size: 10px;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 8px;
  }
</style>
