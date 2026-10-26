<script lang="ts">
  import Editor from './editor/Editor.svelte';
  import { i18n, t } from './i18n';

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
        class="locale-button"
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
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: #1a1c19;
    color: #e5e5e0;
  }

  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    box-sizing: border-box;
  }

  .app-header {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    border-bottom: 1px solid #2a2e28;
    background-color: #141613;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: #5E6F45;
  }

  .logo-mark {
    font-size: 16px;
  }

  .brand-title {
    color: #f1f1eb;
  }

  .brand-tagline {
    font-size: 12px;
    font-weight: 400;
    color: #8c9085;
    margin-left: 8px;
  }

  .header-actions {
    display: flex;
    align-items: center;
  }

  .locale-button {
    background: transparent;
    border: 1px solid #3c4238;
    color: #c7cbbe;
    border-radius: 4px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .locale-button:hover {
    background: #252a22;
    border-color: #5E6F45;
    color: #f1f1eb;
  }

  .editor-workspace {
    flex: 1;
    overflow: hidden;
  }
</style>
