<script lang="ts">
  import { t } from '../i18n';
  import { manuscript } from '../vault/manuscript.svelte';

  let newTagInput = $state('');
  let isEditingTitle = $state(false);
  let titleInput = $state('');

  let scene = $derived(manuscript.activeScene);

  $effect(() => {
    if (scene?.id) {
      isEditingTitle = false;
      newTagInput = '';
    }
  });

  function getStatusLabel(status: string | null): string {
    if (status === 'draft') return t('manuscript.status.draft');
    if (status === 'revised') return t('manuscript.status.revised');
    if (status === 'final') return t('manuscript.status.final');
    return status || t('manuscript.status.draft');
  }

  function handleStatusChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    if (scene) {
      void manuscript.updateSceneMetadata(scene.id, target.value, scene.tags);
    }
  }

  function cycleStatus() {
    if (!scene) return;
    const current = scene.status || 'draft';
    const nextStatus =
      current === 'draft' ? 'revised' : current === 'revised' ? 'final' : 'draft';
    void manuscript.updateSceneMetadata(scene.id, nextStatus, scene.tags);
  }

  function handleAddTag(e: KeyboardEvent) {
    if (e.key === 'Enter' && newTagInput.trim() && scene) {
      e.preventDefault();
      const tag = newTagInput.trim().toLowerCase();
      if (!scene.tags.includes(tag)) {
        const nextTags = [...scene.tags, tag];
        void manuscript.updateSceneMetadata(scene.id, scene.status, nextTags);
      }
      newTagInput = '';
    }
  }

  function handleRemoveTag(tagToRemove: string) {
    if (scene) {
      const nextTags = scene.tags.filter((t) => t !== tagToRemove);
      void manuscript.updateSceneMetadata(scene.id, scene.status, nextTags);
    }
  }

  function startTitleEdit() {
    if (scene) {
      titleInput = scene.title;
      isEditingTitle = true;
    }
  }

  function saveTitleEdit() {
    if (scene && titleInput.trim()) {
      void manuscript.renameItem(scene.id, titleInput.trim());
    }
    isEditingTitle = false;
  }

  function getItemKindBadge(kind: string): { icon: string; label: string } {
    switch (kind) {
      case 'part':
        return { icon: '📚', label: 'Parte' };
      case 'chapter':
        return { icon: '📖', label: 'Capítulo' };
      case 'scene':
        return { icon: '🎬', label: 'Cena' };
      case 'character':
        return { icon: '👤', label: 'Personagem' };
      case 'location':
        return { icon: '📍', label: 'Local' };
      case 'item':
        return { icon: '🗡️', label: 'Objeto' };
      case 'faction':
        return { icon: '🛡️', label: 'Facção' };
      case 'lore':
        return { icon: '📜', label: 'Lore' };
      default:
        return { icon: '📄', label: 'Documento' };
    }
  }

  function getItemRoleDescription(kind: string): string {
    switch (kind) {
      case 'part':
        return 'Abertura e texto da Parte no livro compilado';
      case 'chapter':
        return 'Texto do capítulo (compõe antes das cenas ou diretamente como capítulo)';
      case 'scene':
        return 'Cena narrativa sequencial do capítulo';
      case 'character':
        return 'Dossiê, notas de contexto e perfil do personagem';
      case 'location':
        return 'Notas de cenário, atmosfera e geografia';
      case 'item':
        return 'Descrição, história e relevância do objeto na trama';
      case 'faction':
        return 'Ideologia, estrutura e motivações da facção';
      case 'lore':
        return 'História, mitologia e regras do mundo';
      default:
        return 'Documento do manuscrito';
    }
  }

  function handleTitleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      saveTitleEdit();
    } else if (e.key === 'Escape') {
      isEditingTitle = false;
    }
  }
</script>

{#if scene}
  <header class="scene-header">
    <div class="kind-indicator">
      <span class="kind-icon">{getItemKindBadge(scene.kind).icon}</span>
      <span class="kind-label">{getItemKindBadge(scene.kind).label}</span>
      <span class="kind-sep">•</span>
      <span class="kind-role">{getItemRoleDescription(scene.kind)}</span>
    </div>
    <div class="title-row">
      <div class="title-wrapper">
        <button
          type="button"
          class="status-dot-btn"
          onclick={cycleStatus}
          title="Status: {getStatusLabel(scene.status || 'draft')} (clique para alternar)"
          aria-label="{getStatusLabel(scene.status || 'draft')}"
        >
          <span
            class="status-dot status-{scene.status || 'draft'}"
          ></span>
        </button>
        {#if isEditingTitle}
          <input
            type="text"
            class="title-input"
            bind:value={titleInput}
            onblur={saveTitleEdit}
            onkeydown={handleTitleKeyDown}
          />
        {:else}
          <button
            type="button"
            class="scene-title-btn"
            onclick={startTitleEdit}
            title={t('manuscript.rename')}
          >
            <h2 class="scene-title">
              {scene.title}
            </h2>
          </button>
        {/if}
      </div>

      <div class="metadata-controls">
        <label class="status-selector" title="Status literário do manuscrito">
          <span
            class="status-dot-mini status-{scene.status || 'draft'}"
            title={getStatusLabel(scene.status || 'draft')}
          ></span>
          <select value={scene.status || 'draft'} onchange={handleStatusChange}>
            <option value="draft">{t('manuscript.status.draft')}</option>
            <option value="revised">{t('manuscript.status.revised')}</option>
            <option value="final">{t('manuscript.status.final')}</option>
          </select>
        </label>
      </div>
    </div>

    <div class="tags-row">
      <div class="tags-list">
        {#each scene.tags as tag}
          <span class="tag-chip">
            #{tag}
            <button
              type="button"
              class="remove-tag-btn"
              onclick={() => handleRemoveTag(tag)}
              title="Remover"
            >
              ×
            </button>
          </span>
        {/each}
      </div>

      <input
        type="text"
        class="new-tag-input"
        placeholder={t('manuscript.tagPlaceholder')}
        bind:value={newTagInput}
        onkeydown={handleAddTag}
      />
    </div>
  </header>
{/if}

<style>
  .scene-header {
    padding: var(--space-md) var(--space-lg);
    background-color: var(--color-paper-base);
    border-bottom: 1px solid var(--color-paper-border);
    max-width: 720px;
    margin: 0 auto;
    width: 100%;
    box-sizing: border-box;
  }

  .kind-indicator {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-bronze);
    margin-bottom: 4px;
    flex-wrap: wrap;
  }

  .kind-sep {
    color: var(--color-ink-faded);
    font-weight: 400;
  }

  .kind-role {
    font-size: 11px;
    font-weight: 400;
    text-transform: none;
    letter-spacing: normal;
    color: var(--color-ink-muted);
  }

  .title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
  }

  .title-wrapper {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
  }

  .status-dot-btn {
    background: transparent;
    border: none;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    line-height: 0;
    transition: transform 0.15s ease;
  }

  .status-dot-btn:hover {
    transform: scale(1.3);
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
    display: inline-block;
    transition: background-color 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease;
  }

  .status-dot-mini {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    display: inline-block;
  }

  .status-selector {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    padding: 0 8px;
  }

  .status-selector select {
    border: none;
    background: transparent;
    padding: 4px 4px 4px 0;
  }

  .status-draft {
    background-color: #d97706; /* Âmbar */
    box-shadow: 0 0 6px rgba(217, 119, 6, 0.45);
  }

  .status-revised {
    background-color: #2563eb; /* Azul */
    box-shadow: 0 0 6px rgba(37, 99, 235, 0.45);
  }

  .status-final {
    background-color: #16a34a; /* Verde */
    box-shadow: 0 0 6px rgba(22, 163, 74, 0.45);
  }

  .scene-title-btn {
    border: none;
    background: transparent;
    padding: 0;
    margin: 0;
    text-align: left;
    cursor: pointer;
  }

  .scene-title {
    font-family: var(--font-literary);
    font-size: 24px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin: 0;
    cursor: pointer;
    border-bottom: 1px dashed transparent;
    transition: border-color 0.15s ease;
  }

  .scene-title:hover {
    border-bottom-color: var(--color-paper-border);
  }

  .title-input {
    font-family: var(--font-literary);
    font-size: 22px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-olive);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    width: 100%;
    outline: none;
  }

  .metadata-controls select {
    font-family: var(--font-ui);
    font-size: 12px;
    padding: 4px 10px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    color: var(--color-ink-charcoal);
    outline: none;
    cursor: pointer;
  }

  .tags-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
    flex-wrap: wrap;
  }

  .tags-list {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: 12px;
    padding: 2px 8px;
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-ink-faded);
  }

  .remove-tag-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 13px;
    color: var(--color-ink-faded);
    padding: 0;
    line-height: 1;
  }

  .remove-tag-btn:hover {
    color: var(--color-error);
  }

  .new-tag-input {
    font-family: var(--font-ui);
    font-size: 11px;
    padding: 3px 8px;
    background-color: transparent;
    border: 1px dashed var(--color-paper-border);
    border-radius: 12px;
    color: var(--color-ink-charcoal);
    outline: none;
    width: 110px;
  }

  .new-tag-input:focus {
    border-color: var(--color-olive);
    width: 140px;
  }
</style>
