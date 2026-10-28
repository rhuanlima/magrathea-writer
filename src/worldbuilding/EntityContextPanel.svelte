<script lang="ts">
  /**
   * Painel de Contexto de Personagens, Locais e Objetos para consulta durante a escrita (Épico 5).
   *
   * Permite que o escritor consulte rapidamente fichas, papéis narrativos,
   * motivações e notas de entidades enquanto escreve sua cena, além de permitir
   * saltar diretamente para editar a ficha completa da entidade.
   */

  import { manuscript } from '../vault/manuscript.svelte';
  import { ENTITY_TYPE_OPTIONS } from './templates';

  interface Props {
    currentContent?: string;
  }

  let { currentContent = '' }: Props = $props();

  let allEntities = $derived(
    manuscript.items.filter(
      (i) => !['part', 'chapter', 'scene'].includes(i.kind)
    )
  );

  let mentionedEntities = $derived(
    allEntities.filter((e) => {
      const titleLower = e.title.toLowerCase();
      const contentLower = currentContent.toLowerCase();
      return (
        contentLower.includes(`[[${titleLower}`) ||
        contentLower.includes(titleLower)
      );
    })
  );

  let otherEntities = $derived(
    allEntities.filter((e) => !mentionedEntities.some((m) => m.id === e.id))
  );

  function getKindIcon(kind: string): string {
    const match = ENTITY_TYPE_OPTIONS.find((o) => o.kind === kind);
    return match ? match.icon : '📄';
  }

  function openEntityForEditing(id: string) {
    manuscript.selectScene(id);
  }
</script>

<div class="entity-context-panel">
  <!-- Entidades Presentes nesta cena -->
  <div class="context-section">
    <h4 class="section-title">
      <span>✨ Presentes nesta Cena</span>
      {#if mentionedEntities.length > 0}
        <span class="count-pill">{mentionedEntities.length}</span>
      {/if}
    </h4>

    {#if mentionedEntities.length === 0}
      <div class="empty-context-hint">
        Nenhum personagem ou local mencionado diretamente no texto ainda. Use <code>[[Nome]]</code> para citar e conectar!
      </div>
    {:else}
      <div class="entities-grid">
        {#each mentionedEntities as ent (ent.id)}
          <div class="entity-context-card mentioned">
            <div class="card-header">
              <span class="entity-icon">{getKindIcon(ent.kind)}</span>
              <div class="entity-name-col">
                <span class="entity-title">{ent.title}</span>
                <span class="entity-kind-badge">{ent.kind}</span>
              </div>
              <button
                type="button"
                class="edit-sheet-btn"
                onclick={() => openEntityForEditing(ent.id)}
                title="Abrir ficha completa no editor"
              >
                ✏️ Editar Ficha
              </button>
            </div>

            {#if ent.tags.length > 0}
              <div class="card-tags">
                {#each ent.tags as tag}
                  <span class="tag-pill">#{tag}</span>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Todas as Entidades do Projeto -->
  <div class="context-section">
    <h4 class="section-title">
      <span>📚 Enciclopédia do Mundo</span>
      {#if otherEntities.length > 0}
        <span class="count-pill">{otherEntities.length}</span>
      {/if}
    </h4>

    {#if otherEntities.length === 0 && mentionedEntities.length === 0}
      <div class="empty-context-hint">
        Nenhum personagem, local ou lore criado no cofre. Crie fichas na aba Enciclopédia para consultar o contexto aqui!
      </div>
    {:else if otherEntities.length === 0}
      <div class="empty-context-hint">
        Todas as entidades do seu mundo já estão presentes nesta cena!
      </div>
    {:else}
      <div class="entities-grid">
        {#each otherEntities as ent (ent.id)}
          <div class="entity-context-card">
            <div class="card-header">
              <span class="entity-icon">{getKindIcon(ent.kind)}</span>
              <div class="entity-name-col">
                <span class="entity-title">{ent.title}</span>
                <span class="entity-kind-badge">{ent.kind}</span>
              </div>
              <button
                type="button"
                class="edit-sheet-btn"
                onclick={() => openEntityForEditing(ent.id)}
                title="Abrir ficha completa no editor"
              >
                ✏️ Editar
              </button>
            </div>

            {#if ent.tags.length > 0}
              <div class="card-tags">
                {#each ent.tags as tag}
                  <span class="tag-pill">#{tag}</span>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .entity-context-panel {
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
    font-family: var(--font-ui);
    box-sizing: border-box;
  }

  .context-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .section-title {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--color-ink-muted);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .count-pill {
    padding: 1px 6px;
    border-radius: var(--radius-full);
    font-size: 10px;
    font-weight: 600;
    background-color: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .empty-context-hint {
    padding: var(--space-md);
    border-radius: var(--radius-sm);
    background-color: var(--color-paper-surface);
    border: 1px dashed var(--color-paper-border);
    font-size: 12px;
    line-height: 1.4;
    color: var(--color-ink-muted);
    text-align: center;
  }

  .empty-context-hint code {
    font-family: var(--font-mono);
    color: var(--color-bronze);
    background-color: var(--color-paper-hover);
    padding: 1px 4px;
    border-radius: 3px;
  }

  .entities-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .entity-context-card {
    padding: 10px;
    border-radius: var(--radius-sm);
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    display: flex;
    flex-direction: column;
    gap: 6px;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .entity-context-card.mentioned {
    border-color: var(--color-bronze);
    background-color: var(--color-paper-surface);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .entity-icon {
    font-size: 16px;
    line-height: 1;
  }

  .entity-name-col {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .entity-title {
    font-weight: 600;
    font-size: 13px;
    color: var(--color-ink-charcoal);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .entity-kind-badge {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-ink-muted);
  }

  .edit-sheet-btn {
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
    color: var(--color-ink-charcoal);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease;
    white-space: nowrap;
  }

  .edit-sheet-btn:hover {
    background-color: var(--color-paper-hover);
    border-color: var(--color-bronze);
  }

  .card-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .tag-pill {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: var(--radius-full);
    background-color: var(--color-paper-hover);
    color: var(--color-ink-muted);
  }
</style>
