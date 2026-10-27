<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Editor from './editor/Editor.svelte';
  import { i18n, t } from './i18n';
  import { theme } from './design';
  import { vault } from './vault/vault.svelte';
  import { manuscript } from './vault/manuscript.svelte';
  import CreateVaultModal from './vault/CreateVaultModal.svelte';
  import LockScreen from './vault/LockScreen.svelte';
  import VaultSecurityModal from './vault/VaultSecurityModal.svelte';
  import Sidebar from './manuscript/Sidebar.svelte';
  import TrashDrawer from './manuscript/TrashDrawer.svelte';
  import InspectorPanel from './inspector/InspectorPanel.svelte';
  import { InactivityTracker } from './vault/inactivity';

  let isCreateModalOpen = $state(false);
  let currentContent = $state(
    `# ${t('editor.defaultTitle')}\n\n${t('editor.welcomeMessage')}`
  );
  let tracker: InactivityTracker | null = null;

  onMount(() => {
    void (async () => {
      await vault.refreshStatus();
      if (vault.isOpen && !vault.isLocked) {
        await manuscript.refreshTree();
      }
    })();

    tracker = new InactivityTracker({
      onTimeout: () => {
        if (vault.isOpen && !vault.isLocked) {
          void vault.lockVault();
        }
      },
      lockOnHidden: true,
    });
    tracker.start();
  });

  onDestroy(() => {
    tracker?.stop();
  });

  function handleContentChange(updated: string) {
    currentContent = updated;
  }

  function toggleLanguage() {
    const nextLocale = i18n.locale === 'pt-BR' ? 'en-US' : 'pt-BR';
    i18n.setLocale(nextLocale);
  }

  async function handleVaultCreated() {
    if (vault.projectTitle) {
      currentContent = `# ${vault.projectTitle}\n\n${t('editor.welcomeMessage')}`;
    }
    await manuscript.refreshTree();
  }
</script>

<main class="app-shell">
  <header class="app-header">
    <div class="brand">
      <button
        type="button"
        class="sidebar-toggle-btn"
        onclick={() => manuscript.toggleSidebar()}
        title={t('sidebar.toggle')}
      >
        ☰
      </button>
      <span class="logo-mark">◈</span>
      <span class="brand-title">{t('app.title')}</span>
      {#if vault.isOpen && vault.projectTitle}
        <span class="vault-pill" title={vault.vaultPath || ''}>
          🔒 {vault.projectTitle}
        </span>
      {:else}
        <span class="brand-tagline">{t('app.tagline')}</span>
      {/if}
    </div>

    <div class="header-actions">
      {#if !vault.isOpen}
        <button
          type="button"
          class="action-button primary-action"
          onclick={() => (isCreateModalOpen = true)}
        >
          + {t('vault.status.newVaultButton')}
        </button>
      {:else}
        <button
          type="button"
          class="action-button"
          onclick={() => vault.toggleSecurityModal(true)}
          title={t('vault.security.modalTitle')}
        >
          🛡️ {t('vault.security.tabChangePassphrase')}
        </button>

        <button
          type="button"
          class="action-button"
          class:active-action={vault.isInspectorOpen}
          onclick={() => vault.toggleInspector()}
          title={t('inspector.title')}
        >
          ℹ️ {t('inspector.title')}
        </button>

        <button
          type="button"
          class="action-button lock-action"
          onclick={() => vault.lockVault()}
          title={t('vault.status.lockButton')}
        >
          🔒 {t('vault.status.lockButton')}
        </button>
      {/if}

      <button
        type="button"
        class="action-button"
        onclick={() => theme.toggleTheme()}
        title="Toggle Theme (Light / Dark / System)"
      >
        {theme.mode.toUpperCase()}
      </button>

      <button
        type="button"
        class="action-button"
        onclick={toggleLanguage}
        title={t('app.switchLanguage')}
      >
        {i18n.locale === 'pt-BR' ? 'EN' : 'PT'}
      </button>
    </div>
  </header>

  <div class="app-workspace">
    <Sidebar />
    <section class="editor-workspace">
      <Editor initialContent={currentContent} onChange={handleContentChange} />
    </section>
    <InspectorPanel />
  </div>

  <CreateVaultModal
    isOpen={isCreateModalOpen}
    onClose={() => (isCreateModalOpen = false)}
    onSuccess={handleVaultCreated}
  />

  <TrashDrawer />

  <VaultSecurityModal
    isOpen={vault.isSecurityModalOpen}
    onClose={() => vault.toggleSecurityModal(false)}
  />

  {#if vault.isLocked && vault.vaultPath}
    <LockScreen />
  {/if}
</main>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    font-family: var(--font-ui);
    background-color: var(--color-paper-base);
    color: var(--color-ink-charcoal);
  }

  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    box-sizing: border-box;
    background-color: var(--color-paper-base);
  }

  .app-header {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-md);
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: var(--color-olive);
  }

  .sidebar-toggle-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 16px;
    color: var(--color-ink-muted);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .sidebar-toggle-btn:hover {
    background-color: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .logo-mark {
    font-size: 16px;
  }

  .brand-title {
    color: var(--color-ink-charcoal);
  }

  .brand-tagline {
    font-size: 12px;
    font-weight: 400;
    color: var(--color-ink-muted);
    margin-left: var(--space-sm);
  }

  .vault-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 500;
    font-family: var(--font-ui);
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    padding: 2px 8px;
    border-radius: 12px;
    color: var(--color-ink-charcoal);
    margin-left: var(--space-sm);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .action-button {
    background: transparent;
    border: 1px solid var(--color-paper-border);
    color: var(--color-ink-muted);
    border-radius: var(--rounded-default);
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
    font-family: var(--font-mono);
    cursor: pointer;
    transition: all var(--transition-focus);
  }

  .action-button:hover {
    background: var(--color-paper-hover);
    border-color: var(--color-olive);
    color: var(--color-ink-charcoal);
  }

  .action-button.primary-action {
    background-color: var(--color-olive);
    color: #ffffff;
    border-color: var(--color-olive);
    font-family: var(--font-ui);
    font-size: 12px;
  }

  .action-button.primary-action:hover {
    background-color: var(--color-olive-hover);
    border-color: var(--color-olive-hover);
  }

  .action-button.lock-action {
    color: var(--color-error);
    border-color: var(--color-paper-border);
    font-family: var(--font-ui);
  }

  .action-button.lock-action:hover {
    background-color: #fdf2f2;
    border-color: var(--color-error);
  }

  .action-button:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-offset);
  }

  .app-workspace {
    flex: 1;
    display: flex;
    overflow: hidden;
    position: relative;
  }

  .editor-workspace {
    flex: 1;
    overflow: hidden;
    background-color: var(--color-paper-base);
  }
</style>
