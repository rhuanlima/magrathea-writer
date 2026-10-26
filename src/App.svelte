<script lang="ts">
  import Editor from './editor/Editor.svelte';
  import { i18n, t } from './i18n';
  import { theme } from './design';

  let currentContent = $state(
    `# ${t('editor.defaultTitle')}\n\n${t('editor.welcomeMessage')}`
  );

  function handleContentChange(updated: string) {
    currentContent = updated;
  }

  function toggleLanguage() {
    const nextLocale = i18n.locale === 'pt-BR' ? 'en-US' : 'pt-BR';
    i18n.setLocale(nextLocale);
  }
</script>

<main class="app-shell">
  <header class="app-header">
    <div class="brand">
      <span class="logo-mark">◈</span>
      <span class="brand-title">{t('app.title')}</span>
      <span class="brand-tagline">{t('app.tagline')}</span>
    </div>

    <div class="header-actions">
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

  <section class="editor-workspace">
    <Editor initialContent={currentContent} onChange={handleContentChange} />
  </section>
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

  .action-button:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-offset);
  }

  .editor-workspace {
    flex: 1;
    overflow: hidden;
    background-color: var(--color-paper-base);
  }
</style>
