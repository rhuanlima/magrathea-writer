<script lang="ts">
  import { t } from '../i18n';
  import { worldbuilding } from './worldbuilding.svelte';
  import { ENTITY_TYPE_OPTIONS } from './templates';

  function getKindIcon(kind: string): string {
    const match = ENTITY_TYPE_OPTIONS.find((o) => o.kind === kind);
    return match ? match.icon : '📄';
  }
</script>

<div class="encyclopedia-view">
  <!-- Barra de controle e busca -->
  <div class="encyclopedia-toolbar">
    <div class="search-box">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        placeholder={t('worldbuilding.searchPlaceholder')}
        value={worldbuilding.searchQuery}
        oninput={(e) => worldbuilding.setSearch((e.target as HTMLInputElement).value)}
      />
      {#if worldbuilding.searchQuery}
        <button
          type="button"
          class="clear-search-btn"
          onclick={() => worldbuilding.setSearch('')}
        >
          ✕
        </button>
      {/if}
    </div>

    <button
      type="button"
      class="new-entity-btn"
      onclick={() => worldbuilding.openCreateModal()}
      title={t('worldbuilding.createModalTitle')}
    >
      + {t('worldbuilding.newEntity')}
    </button>
  </div>

  <!-- Filtros por tipo de entidade -->
  <div class="type-filter-chips">
    <button
      type="button"
      class="filter-chip"
      class:active={worldbuilding.filterKind === 'all'}
      onclick={() => worldbuilding.setFilter('all')}
    >
      {t('worldbuilding.filterAll')}
    </button>
    {#each ENTITY_TYPE_OPTIONS as opt}
      <button
        type="button"
        class="filter-chip"
        class:active={worldbuilding.filterKind === opt.kind}
        onclick={() => worldbuilding.setFilter(opt.kind)}
      >
        <span>{opt.icon}</span>
        <span>{t(opt.labelKey)}</span>
      </button>
    {/each}
  </div>

  <!-- Lista de entidades -->
  <div class="entities-list">
    {#if worldbuilding.filteredEntities.length === 0}
      <div class="empty-state">
        <p>{t('worldbuilding.noEntitiesFound')}</p>
        <button
          type="button"
          class="create-first-btn"
          onclick={() => worldbuilding.openCreateModal()}
        >
          + {t('worldbuilding.createFirstEntity')}
        </button>
      </div>
    {:else}
      {#each worldbuilding.filteredEntities as entity}
        {@const isActive = worldbuilding.activeEntityId === entity.id}
        <div
          class="entity-row-card"
          class:active={isActive}
          onclick={() => worldbuilding.selectEntity(entity.id)}
          role="button"
          tabindex="0"
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              worldbuilding.selectEntity(entity.id);
            }
          }}
        >
          <div class="entity-icon">{getKindIcon(entity.kind)}</div>
          <div class="entity-info">
            <span class="entity-name">{entity.title}</span>
            {#if entity.tags.length > 0}
              <div class="entity-tags">
                {#each entity.tags as tag}
                  <span class="tag-badge">#{tag}</span>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .encyclopedia-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    user-select: none;
  }

  .encyclopedia-toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm);
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .search-box {
    display: flex;
    align-items: center;
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    padding: 2px 8px;
    flex: 1;
    gap: 4px;
  }

  .search-icon {
    font-size: 11px;
    color: var(--color-ink-muted);
  }

  .search-box input {
    border: none;
    background: transparent;
    font-size: 12px;
    font-family: var(--font-ui);
    color: var(--color-ink-charcoal);
    width: 100%;
    outline: none;
    padding: 2px 0;
  }

  .clear-search-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 10px;
    color: var(--color-ink-faded);
    padding: 0;
  }

  .new-entity-btn {
    background-color: var(--color-olive);
    color: var(--color-paper-elevated);
    border: none;
    border-radius: var(--rounded-default);
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: background-color 150ms ease;
  }

  .new-entity-btn:hover {
    background-color: var(--color-olive-hover);
  }

  .type-filter-chips {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px var(--space-sm);
    overflow-x: auto;
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-base);
  }

  .type-filter-chips::-webkit-scrollbar {
    display: none;
  }

  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    border: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
    color: var(--color-ink-muted);
    padding: 2px 6px;
    border-radius: var(--rounded-full);
    font-size: 10px;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
    transition: all 150ms ease;
  }

  .filter-chip:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .filter-chip.active {
    background-color: var(--color-olive-subtle);
    border-color: var(--color-olive);
    color: var(--color-olive);
    font-weight: 600;
  }

  .entities-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-xs);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .empty-state {
    padding: var(--space-lg) var(--space-sm);
    text-align: center;
    color: var(--color-ink-muted);
    font-size: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
  }

  .create-first-btn {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 500;
    color: var(--color-olive);
    cursor: pointer;
  }

  .create-first-btn:hover {
    background-color: var(--color-paper-hover);
  }

  .entity-row-card {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: var(--rounded-default);
    background-color: var(--color-paper-surface);
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .entity-row-card:hover {
    background-color: var(--color-paper-hover);
  }

  .entity-row-card.active {
    background-color: var(--color-olive-subtle);
    border-color: var(--color-olive);
  }

  .entity-icon {
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .entity-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .entity-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-ink-charcoal);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .entity-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
  }

  .tag-badge {
    font-size: 10px;
    color: var(--color-bronze-text);
  }
</style>
