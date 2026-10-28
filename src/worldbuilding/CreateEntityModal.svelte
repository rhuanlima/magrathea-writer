<script lang="ts">
  import { t } from '../i18n';
  import { worldbuilding } from './worldbuilding.svelte';
  import { ENTITY_TYPE_OPTIONS, type EntityKind } from './templates';

  let selectedKind = $state<EntityKind>('character');
  let titleInput = $state('');
  let tagInput = $state('');
  let tags = $state<string[]>([]);
  let errorMessage = $state<string | null>(null);

  function resetForm() {
    selectedKind = 'character';
    titleInput = '';
    tagInput = '';
    tags = [];
    errorMessage = null;
  }

  function addTag() {
    const trimmed = tagInput.trim().toLowerCase();
    if (trimmed && !tags.includes(trimmed)) {
      tags = [...tags, trimmed];
      tagInput = '';
    }
  }

  function removeTag(tag: string) {
    tags = tags.filter((t) => t !== tag);
  }

  async function handleCreate(e: SubmitEvent) {
    e.preventDefault();
    if (!titleInput.trim()) {
      errorMessage = t('worldbuilding.errorTitleRequired');
      return;
    }

    try {
      await worldbuilding.createEntity(selectedKind, titleInput.trim(), tags);
      resetForm();
    } catch (err) {
      errorMessage = String(err);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && worldbuilding.isCreateModalOpen) {
      worldbuilding.closeCreateModal();
      resetForm();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if worldbuilding.isCreateModalOpen}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div class="modal-card">
      <header class="modal-header">
        <h2 id="modal-title" class="modal-title">{t('worldbuilding.createModalTitle')}</h2>
        <button
          type="button"
          class="close-btn"
          onclick={() => {
            worldbuilding.closeCreateModal();
            resetForm();
          }}
          aria-label={t('common.close')}
        >
          ✕
        </button>
      </header>

      <form onsubmit={handleCreate} class="modal-form">
        <!-- Seleção visual do tipo de entidade -->
        <fieldset class="type-selection">
          <legend class="field-label">{t('worldbuilding.selectType')}</legend>
          <div class="type-grid">
            {#each ENTITY_TYPE_OPTIONS as opt}
              <button
                type="button"
                class="type-option-card"
                class:active={selectedKind === opt.kind}
                onclick={() => (selectedKind = opt.kind)}
              >
                <span class="type-icon">{opt.icon}</span>
                <span class="type-title">{t(opt.labelKey)}</span>
              </button>
            {/each}
          </div>
        </fieldset>

        <!-- Campo Título -->
        <div class="field-group">
          <label for="entity-title">{t('worldbuilding.entityTitle')}</label>
          <input
            id="entity-title"
            type="text"
            bind:value={titleInput}
            placeholder={t('worldbuilding.entityTitlePlaceholder')}
            required
            autocomplete="off"
          />
        </div>

        <!-- Tags -->
        <div class="field-group">
          <label for="entity-tags">{t('worldbuilding.tagsLabel')}</label>
          <div class="tags-input-row">
            <input
              id="entity-tags"
              type="text"
              bind:value={tagInput}
              placeholder={t('worldbuilding.tagsPlaceholder')}
              onkeydown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  addTag();
                }
              }}
            />
            <button type="button" class="add-tag-btn" onclick={addTag}>
              + {t('common.add')}
            </button>
          </div>
          {#if tags.length > 0}
            <div class="tags-chips-container">
              {#each tags as tag}
                <span class="tag-chip">
                  #{tag}
                  <button type="button" class="remove-chip-btn" onclick={() => removeTag(tag)}>×</button>
                </span>
              {/each}
            </div>
          {/if}
        </div>

        {#if errorMessage}
          <div class="error-banner">{errorMessage}</div>
        {/if}

        <footer class="modal-footer">
          <button
            type="button"
            class="cancel-btn"
            onclick={() => {
              worldbuilding.closeCreateModal();
              resetForm();
            }}
          >
            {t('common.cancel')}
          </button>
          <button type="submit" class="submit-btn" disabled={worldbuilding.isLoading}>
            {worldbuilding.isLoading ? t('common.saving') : t('worldbuilding.createButton')}
          </button>
        </footer>
      </form>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(30, 35, 29, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: var(--space-md);
  }

  .modal-card {
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    box-shadow: var(--shadow-modal);
    width: 100%;
    max-width: 520px;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    background-color: var(--color-paper-surface);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .modal-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin: 0;
  }

  .close-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 16px;
    color: var(--color-ink-muted);
    padding: 4px;
    border-radius: var(--rounded-sm);
  }

  .modal-form {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .type-selection {
    border: none;
    padding: 0;
    margin: 0;
  }

  .field-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-ink-muted);
    margin-bottom: var(--space-xs);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .type-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: var(--space-xs);
  }

  .type-option-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 4px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    cursor: pointer;
    transition: all 150ms ease;
  }

  .type-option-card:hover {
    background-color: var(--color-paper-hover);
    border-color: var(--color-olive);
  }

  .type-option-card.active {
    background-color: var(--color-olive-subtle);
    border-color: var(--color-olive);
    box-shadow: 0 0 0 1px var(--color-olive);
  }

  .type-icon {
    font-size: 20px;
  }

  .type-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-group label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-ink-muted);
  }

  .field-group input {
    padding: 8px 12px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    font-size: 14px;
    font-family: var(--font-ui);
    color: var(--color-ink-charcoal);
  }

  .field-group input:focus {
    outline: none;
    border-color: var(--color-olive);
    box-shadow: 0 0 0 2px var(--color-olive-subtle);
  }

  .tags-input-row {
    display: flex;
    gap: var(--space-xs);
  }

  .tags-input-row input {
    flex: 1;
  }

  .add-tag-btn {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    padding: 0 12px;
    font-size: 12px;
    cursor: pointer;
    color: var(--color-ink-charcoal);
  }

  .add-tag-btn:hover {
    background-color: var(--color-paper-hover);
  }

  .tags-chips-container {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
  }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background-color: var(--color-chip-bg);
    color: var(--color-bronze-text);
    padding: 2px 8px;
    border-radius: var(--rounded-full);
    font-size: 11px;
    font-weight: 500;
  }

  .remove-chip-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 12px;
    color: var(--color-bronze-text);
    padding: 0;
    line-height: 1;
  }

  .error-banner {
    background-color: #fdf2f2;
    color: var(--color-error);
    padding: 8px 12px;
    border-radius: var(--rounded-default);
    font-size: 12px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
    margin-top: var(--space-sm);
  }

  .cancel-btn {
    background: transparent;
    border: 1px solid var(--color-paper-border);
    padding: 8px 16px;
    border-radius: var(--rounded-default);
    font-size: 13px;
    cursor: pointer;
    color: var(--color-ink-muted);
  }

  .submit-btn {
    background-color: var(--color-olive);
    color: var(--color-paper-elevated);
    border: none;
    padding: 8px 18px;
    border-radius: var(--rounded-default);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .submit-btn:hover:not(:disabled) {
    background-color: var(--color-olive-hover);
  }

  .submit-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
