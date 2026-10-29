<script lang="ts">
  /**
   * Tela de boas-vindas exibida quando nenhum cofre está aberto (estado inicial).
   * Coloca 'Abrir Cofre' como opção prioritária e exibe projetos recentes do autor.
   */

  import { onMount } from 'svelte';
  import { t } from '../i18n';
  import { getRecentVaults, type RecentVault } from './recent-vaults';

  interface Props {
    onOpenVault: (prefilledPath?: string) => void;
    onCreateVault: () => void;
  }

  let { onOpenVault, onCreateVault }: Props = $props();

  let recents = $state<RecentVault[]>([]);

  onMount(() => {
    recents = getRecentVaults();
  });

  function formatRelativeDate(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(undefined, {
        day: '2-digit',
        month: 'short',
        year: 'numeric',
      });
    } catch {
      return '';
    }
  }
</script>

<div class="welcome-container" role="main" aria-label={t('vault.open.welcomeTitle')}>
  <div class="welcome-card">
    <div class="brand-hero">
      <img src="/app-icon.png" alt="Magrathea Writer" class="hero-app-icon" />
      <h1 class="hero-title">{t('vault.open.welcomeTitle')}</h1>
      <p class="hero-subtitle">{t('vault.open.welcomeSubtitle')}</p>
    </div>

    <!-- Ações Principais: Abrir em Primeiro Lugar -->
    <div class="hero-actions">
      <button
        type="button"
        class="action-card primary"
        onclick={() => onOpenVault()}
      >
        <img src="/world-file-dark.png" alt="" class="card-world-icon" />
        <div class="card-texts">
          <strong class="card-title">{t('vault.open.welcomeOpenButton')}</strong>
          <span class="card-desc">Abra um mundo existente (.magrathea) no seu computador</span>
        </div>
      </button>

      <button
        type="button"
        class="action-card secondary"
        onclick={() => onCreateVault()}
      >
        <span class="card-icon">✨</span>
        <div class="card-texts">
          <strong class="card-title">{t('vault.open.welcomeCreateButton')}</strong>
          <span class="card-desc">Construa um novo mundo literário com segurança total</span>
        </div>
      </button>
    </div>

    <!-- Seção de Cofres Recentes -->
    {#if recents.length > 0}
      <div class="recent-section">
        <h2 class="recent-title">{t('vault.open.recentSectionTitle')}</h2>
        <div class="recent-grid">
          {#each recents as item}
            <button
              type="button"
              class="recent-card"
              onclick={() => onOpenVault(item.path)}
              title={item.path}
            >
              <div class="recent-card-header">
                <img src="/world-file-dark.png" alt="" class="recent-file-icon" />
                <strong class="recent-name">{item.title}</strong>
              </div>
              <p class="recent-path">{item.path}</p>
              {#if item.lastOpened}
                <span class="recent-date">{formatRelativeDate(item.lastOpened)}</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .welcome-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-paper-base, #FDFCFA);
    padding: var(--space-xl, 24px);
    box-sizing: border-box;
    overflow-y: auto;
  }

  .welcome-card {
    max-width: 640px;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-xl, 24px);
    text-align: center;
  }

  .brand-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .hero-app-icon {
    width: 136px;
    height: 136px;
    border-radius: 28px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.16);
    margin-bottom: 12px;
    object-fit: cover;
    transition: transform 200ms ease;
  }

  .hero-app-icon:hover {
    transform: scale(1.02);
  }

  .hero-title {
    margin: 0;
    font-family: var(--font-literary, serif);
    font-size: 28px;
    font-weight: 700;
    color: var(--color-ink-charcoal, #1E231D);
    letter-spacing: -0.5px;
  }

  .hero-subtitle {
    margin: 0;
    font-family: var(--font-ui, sans-serif);
    font-size: 14px;
    color: var(--color-ink-muted, #726E65);
    max-width: 480px;
    line-height: 1.5;
  }

  .hero-actions {
    display: flex;
    gap: 16px;
    width: 100%;
  }

  .action-card {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px;
    border-radius: var(--rounded-lg, 8px);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    background-color: var(--color-paper-elevated, #FFFDF9);
    cursor: pointer;
    text-align: left;
    transition: all 180ms cubic-bezier(0.16, 1, 0.3, 1);
    box-shadow: var(--shadow-sm, 0 1px 3px rgba(0, 0, 0, 0.05));
  }

  .action-card.primary {
    border-color: var(--color-olive, #5E6F45);
    background: linear-gradient(
      135deg,
      var(--color-paper-elevated, #FFFDF9) 0%,
      rgba(94, 111, 69, 0.06) 100%
    );
  }

  .action-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md, 0 4px 12px rgba(0, 0, 0, 0.08));
  }

  .action-card.primary:hover {
    border-color: var(--color-olive-hover, #4A5836);
  }

  .card-icon {
    font-size: 26px;
  }

  .card-world-icon {
    width: 28px;
    height: 28px;
    object-fit: contain;
    flex-shrink: 0;
  }

  .card-texts {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .card-title {
    font-family: var(--font-ui, sans-serif);
    font-size: 14px;
    color: var(--color-ink-charcoal, #1E231D);
  }

  .card-desc {
    font-size: 11px;
    color: var(--color-ink-muted, #726E65);
    line-height: 1.3;
  }

  .recent-section {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
    text-align: left;
    margin-top: 8px;
    border-top: 1px solid var(--color-paper-border, #E5E0D8);
    padding-top: 20px;
  }

  .recent-title {
    margin: 0;
    font-family: var(--font-ui, sans-serif);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--color-ink-muted, #726E65);
  }

  .recent-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 10px;
  }

  .recent-card {
    background-color: var(--color-paper-surface, #F8F4EC);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: var(--rounded-default, 4px);
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    cursor: pointer;
    text-align: left;
    transition: all 120ms ease;
  }

  .recent-card:hover {
    background-color: var(--color-paper-elevated, #FFFDF9);
    border-color: var(--color-olive, #5E6F45);
  }

  .recent-card-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .recent-file-icon {
    width: 16px;
    height: 16px;
    object-fit: contain;
    flex-shrink: 0;
  }

  .recent-name {
    font-size: 13px;
    color: var(--color-ink-charcoal, #1E231D);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .recent-path {
    margin: 0;
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    color: var(--color-ink-muted, #726E65);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .recent-date {
    font-size: 10px;
    color: var(--color-ink-faded, #A8A297);
    margin-top: 2px;
  }
</style>
