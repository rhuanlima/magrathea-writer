<script lang="ts">
  import Editor from './editor/Editor.svelte';

  let currentContent = $state('# Magrathea Writer\n\nWelcome to your distraction-free literary workshop.');

  function handleContentChange(updated: string) {
    currentContent = updated;
  }
</script>

<main class="app-shell">
  <header class="app-header">
    <div class="brand">
      <span class="logo-mark">◈</span>
      <span class="brand-title">Magrathea Writer</span>
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

  .editor-workspace {
    flex: 1;
    overflow: hidden;
  }
</style>
