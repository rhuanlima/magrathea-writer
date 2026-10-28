<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Editor from './editor/Editor.svelte';
  import { i18n, t } from './i18n';
  import { theme } from './design';
  import { vault } from './vault/vault.svelte';
  import { manuscript } from './vault/manuscript.svelte';
  import CreateVaultModal from './vault/CreateVaultModal.svelte';
  import OpenVaultModal from './vault/OpenVaultModal.svelte';
  import WelcomeScreen from './vault/WelcomeScreen.svelte';
  import LockScreen from './vault/LockScreen.svelte';
  import VaultSecurityModal from './vault/VaultSecurityModal.svelte';
  import Sidebar from './manuscript/Sidebar.svelte';
  import TrashDrawer from './manuscript/TrashDrawer.svelte';
  import InspectorPanel from './inspector/InspectorPanel.svelte';
  import RenameRefactorModal from './vault/RenameRefactorModal.svelte';
  import ProductivityDashboardModal from './productivity/ProductivityDashboardModal.svelte';
  import CreateEntityModal from './worldbuilding/CreateEntityModal.svelte';
  import GlobalSearchModal from './search/GlobalSearchModal.svelte';
  import BackupSettingsModal from './backup/BackupSettingsModal.svelte';
  import CompilerModal from './compiler/CompilerModal.svelte';
  import TimelineModal from './timeline/TimelineModal.svelte';
  import { searchStore } from './search/search.svelte';
  import { backupStore } from './backup/backup.svelte';
  import { compilerStore } from './compiler/compiler.svelte';
  import { timelineStore } from './timeline/timeline.svelte';
  import { editorViewState } from './editor/editor-view-state.svelte';
  import { InactivityTracker } from './vault/inactivity';

  let isCreateModalOpen = $state(false);
  let isOpenVaultModalOpen = $state(false);
  let prefilledVaultPath = $state<string | undefined>(undefined);
  let currentContent = $state(
    `# ${t('editor.defaultTitle')}\n\n${t('editor.welcomeMessage')}`
  );
  let tracker: InactivityTracker | null = null;

  function handleOpenVaultAction(path?: string) {
    prefilledVaultPath = path;
    isOpenVaultModalOpen = true;
  }

  onMount(() => {
    void (async () => {
      await vault.refreshStatus();
      if (vault.isOpen && !vault.isLocked) {
        await manuscript.refreshTree();
        await editorViewState.refreshProductivity();
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
    await editorViewState.refreshProductivity();
  }

  async function handleVaultOpened() {
    if (vault.projectTitle) {
      currentContent = `# ${vault.projectTitle}\n\n${t('editor.welcomeMessage')}`;
    }
    await manuscript.refreshTree();
    await editorViewState.refreshProductivity();
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && editorViewState.isZenMode) {
      editorViewState.exitZen();
    }
    if ((e.metaKey || e.ctrlKey) && (e.key === 'o' || e.key === 'O')) {
      e.preventDefault();
      handleOpenVaultAction();
    }
    if ((e.metaKey || e.ctrlKey) && (e.key === 'n' || e.key === 'N') && !vault.isOpen) {
      e.preventDefault();
      isCreateModalOpen = true;
    }
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'F' || e.key === 'f')) {
      e.preventDefault();
      if (vault.isOpen && !vault.isLocked) {
        searchStore.openModal();
      }
    }
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'E' || e.key === 'e')) {
      e.preventDefault();
      if (vault.isOpen && !vault.isLocked) {
        compilerStore.openModal();
      }
    }
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'T' || e.key === 't')) {
      e.preventDefault();
      if (vault.isOpen && !vault.isLocked) {
        void timelineStore.openModal();
      }
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<main class="app-shell" class:zen-active={editorViewState.isZenMode}>
  {#if editorViewState.isZenMode}
    <div class="zen-exit-zone" role="region" aria-label={t('editor.exitZen')}>
      <button
        type="button"
        class="zen-exit-button"
        onclick={() => editorViewState.exitZen()}
        title={t('editor.exitZen')}
      >
        ✕ {t('editor.exitZen')} (Esc)
      </button>
    </div>
  {/if}

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
          onclick={() => handleOpenVaultAction()}
          title={t('vault.open.modalTitle')}
        >
          📂 {t('vault.status.openVaultButton')}
        </button>
        <button
          type="button"
          class="action-button"
          onclick={() => (isCreateModalOpen = true)}
          title={t('vault.status.newVaultButton')}
        >
          + {t('vault.status.newVaultButton')}
        </button>
      {:else}
        <button
          type="button"
          class="action-button"
          onclick={() => searchStore.openModal()}
          title={t('search.modalTitle')}
        >
          🔍 {t('search.searchTab')}
        </button>

        <button
          type="button"
          class="action-button"
          onclick={() => backupStore.openModal()}
          title={t('backup.modalTitle')}
        >
          💾 {t('backup.modalTitle')}
        </button>

        <button
          type="button"
          class="action-button"
          onclick={() => editorViewState.openProductivityDashboard()}
          title={t('productivity.dashboardTitle')}
        >
          📊 {t('productivity.goalsButton')}
        </button>

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
          class="action-button"
          onclick={() => compilerStore.openModal()}
          title={t('compiler.modalTitle')}
        >
          📖 {t('compiler.modalTitle')}
        </button>

        <button
          type="button"
          class="action-button"
          onclick={() => timelineStore.openModal()}
          title={t('timeline.modalTitle')}
        >
          ⏳ {t('timeline.modalTitle')}
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
        title={t('theme.toggle')}
      >
        {#if theme.mode === 'light'}
          ☀️ {t('theme.light')}
        {:else if theme.mode === 'dark'}
          🌙 {t('theme.dark')}
        {:else}
          💻 {t('theme.system')}
        {/if}
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
    {#if vault.isOpen}
      <Sidebar />
      <section class="editor-workspace">
        <Editor initialContent={currentContent} onChange={handleContentChange} />
      </section>
      <InspectorPanel
        {currentContent}
        onRestoreContent={(restored) => {
          currentContent = restored;
        }}
      />
    {:else}
      <WelcomeScreen
        onOpenVault={handleOpenVaultAction}
        onCreateVault={() => (isCreateModalOpen = true)}
      />
    {/if}
  </div>

  <OpenVaultModal
    isOpen={isOpenVaultModalOpen}
    initialPath={prefilledVaultPath}
    onClose={() => (isOpenVaultModalOpen = false)}
    onSuccess={handleVaultOpened}
  />

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

  <RenameRefactorModal />

  <ProductivityDashboardModal />

  <CreateEntityModal />

  <GlobalSearchModal />

  <BackupSettingsModal />

  <CompilerModal />

  <TimelineModal />

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
    transition: background-color var(--transition-focus);
  }

  /* Estilos do Modo Zen */
  .zen-exit-zone {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 20px;
    z-index: 1000;
    display: flex;
    justify-content: center;
    align-items: flex-start;
  }

  .zen-exit-button {
    opacity: 0;
    pointer-events: none;
    transform: translateY(-8px);
    transition: opacity 200ms ease, transform 200ms ease;
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    color: var(--color-ink-muted);
    font-size: 11px;
    font-weight: 500;
    font-family: var(--font-ui);
    border-radius: var(--rounded-full);
    padding: 4px 14px;
    margin-top: 6px;
    box-shadow: var(--shadow-hud);
    cursor: pointer;
  }

  .zen-exit-zone:hover .zen-exit-button {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0);
  }

  .zen-exit-button:hover {
    color: var(--color-ink-charcoal);
    border-color: var(--color-olive);
    background-color: var(--color-paper-hover);
  }

  .app-shell.zen-active .app-header {
    display: none;
  }

  .app-shell.zen-active :global(.sidebar-container),
  .app-shell.zen-active :global(.inspector-panel),
  .app-shell.zen-active :global(.editor-footer),
  .app-shell.zen-active :global(.scene-header) {
    display: none !important;
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
