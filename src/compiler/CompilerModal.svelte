<script lang="ts">
  /**
   * Modal Interativo do Compilador e Exportação Editorial (M6).
   *
   * Fornece controle de pré/pós-textuais (dedicatória, epígrafe, referências),
   * seleção de temas de diagramação, visualização do manuscrito unificado e
   * exportação multi-formato em 1 clique (Obsidian, DOCX, EPUB, PDF, TXT).
   */

  import { compilerStore } from './compiler.svelte';
  import { t } from '../i18n';
  import { pickSaveExportPath, pickDirectory } from '../tauri/bridge';

  let exportPathInput = $state<string>('');
  let selectedExportFormat = $state<'obsidian' | 'docx' | 'epub' | 'pdf' | 'txt'>('docx');
  let lastExportedPath = $state<string | null>(null);
  let showAdvancedPath = $state(false);
  let copiedFeedback = $state(false);

  function getSafeFilename(ext: string): string {
    const raw = (compilerStore.settings.title || 'manuscrito')
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9_-]/gi, '_');
    return `${raw || 'livro'}.${ext}`;
  }

  async function handleQuickExport(format: 'docx' | 'epub' | 'pdf' | 'obsidian' | 'txt') {
    selectedExportFormat = format;
    lastExportedPath = null;

    if (format === 'docx') {
      const defaultName = getSafeFilename('docx');
      const chosen = await pickSaveExportPath('Exportar Manuscrito Word (.docx)', defaultName, 'docx');
      if (chosen) {
        exportPathInput = chosen;
        const ok = await compilerStore.runExportDocx(chosen);
        if (ok) lastExportedPath = chosen;
      }
    } else if (format === 'epub') {
      const defaultName = getSafeFilename('epub');
      const chosen = await pickSaveExportPath('Exportar Livro Digital EPUB (.epub)', defaultName, 'epub');
      if (chosen) {
        exportPathInput = chosen;
        const ok = await compilerStore.runExportEpub(chosen);
        if (ok) lastExportedPath = chosen;
      }
    } else if (format === 'pdf') {
      const defaultName = getSafeFilename('html');
      const chosen = await pickSaveExportPath('Exportar para Impressão / PDF (.html)', defaultName, 'html');
      if (chosen) {
        exportPathInput = chosen;
        const ok = await compilerStore.runExportPdf(chosen);
        if (ok) lastExportedPath = chosen;
      }
    } else if (format === 'obsidian') {
      const chosen = await pickDirectory('Escolha onde criar a pasta do Cofre Obsidian');
      if (chosen) {
        exportPathInput = chosen;
        const ok = await compilerStore.runExportObsidian(chosen);
        if (ok) lastExportedPath = chosen;
      }
    } else if (format === 'txt') {
      const chosen = await pickDirectory('Escolha onde salvar o Manuscrito TXT e Dossiê');
      if (chosen) {
        exportPathInput = chosen;
        const ok = await compilerStore.runExportTxt(chosen);
        if (ok) lastExportedPath = chosen;
      }
    }
  }

  async function copyPathToClipboard() {
    if (!lastExportedPath) return;
    try {
      await navigator.clipboard.writeText(lastExportedPath);
      copiedFeedback = true;
      setTimeout(() => {
        copiedFeedback = false;
      }, 2500);
    } catch {
      // Ignora erro de clipboard se ambiente não permitir
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      compilerStore.closeModal();
    }
  }

  async function handleSaveSettings() {
    await compilerStore.saveSettingsNow();
    await compilerStore.compile();
  }

  async function handleExport() {
    const path = exportPathInput.trim();
    if (!path) return;

    if (selectedExportFormat === 'obsidian') {
      await compilerStore.runExportObsidian(path);
    } else if (selectedExportFormat === 'docx') {
      await compilerStore.runExportDocx(path);
    } else if (selectedExportFormat === 'epub') {
      await compilerStore.runExportEpub(path);
    } else if (selectedExportFormat === 'pdf') {
      await compilerStore.runExportPdf(path);
    } else if (selectedExportFormat === 'txt') {
      await compilerStore.runExportTxt(path);
    }
  }

  function setDefaultExportPath(format: 'obsidian' | 'docx' | 'epub' | 'pdf' | 'txt') {
    selectedExportFormat = format;
    const title = (compilerStore.settings.title || 'livro')
      .toLowerCase()
      .replace(/\s+/g, '_');

    if (format === 'obsidian') {
      exportPathInput = `~/Documents/${title}_obsidian`;
    } else if (format === 'docx') {
      exportPathInput = `~/Documents/${title}.docx`;
    } else if (format === 'epub') {
      exportPathInput = `~/Documents/${title}.epub`;
    } else if (format === 'pdf') {
      exportPathInput = `~/Documents/${title}.html`;
    } else if (format === 'txt') {
      exportPathInput = `~/Documents/${title}_txt`;
    }
  }

  $effect(() => {
    if (compilerStore.isOpen && !exportPathInput) {
      setDefaultExportPath('docx');
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if compilerStore.isOpen}
  <!-- Backdrop sutil para focar a atenção sem bloquear a visão do autor -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 bg-black/40 backdrop-blur-[2px] transition-opacity duration-200"
    onclick={() => compilerStore.closeModal()}
  ></div>

  <!-- Drawer Lateral à Direita (Painel Editorial) -->
  <div
    class="fixed inset-y-0 right-0 z-50 w-full sm:w-[500px] md:w-[560px] lg:w-[620px] bg-white dark:bg-stone-900 border-l border-stone-200 dark:border-stone-800 shadow-2xl flex flex-col h-full overflow-hidden text-stone-900 dark:text-stone-100"
    role="dialog"
    aria-modal="true"
    aria-labelledby="compiler-modal-title"
  >
    <!-- Header do Painel -->
    <div
      class="flex items-center justify-between px-5 py-3.5 border-b border-stone-200 dark:border-stone-800 bg-stone-50/80 dark:bg-stone-900/80"
    >
      <div class="flex items-center gap-2.5">
        <div
          class="text-xl p-1.5 rounded-lg bg-amber-500/10 text-amber-600 dark:text-amber-400"
        >
          📖
        </div>
        <div>
          <h2 id="compiler-modal-title" class="text-base font-bold tracking-tight">
            {t('compiler.modalTitle')}
          </h2>
          {#if compilerStore.compiled}
            <div class="flex items-center gap-2 text-[11px] text-stone-500 dark:text-stone-400 mt-0.5">
              <span>{t('compiler.statsWords', { count: compilerStore.compiled.total_words })}</span>
              <span>•</span>
              <span>{t('compiler.statsChars', { count: compilerStore.compiled.total_characters })}</span>
              <span>•</span>
              <span>{t('compiler.statsSections', { count: compilerStore.compiled.sections.length })}</span>
            </div>
          {/if}
        </div>
      </div>

      <button
        onclick={() => compilerStore.closeModal()}
        class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors font-bold text-base"
        aria-label="Fechar"
      >
        ✕
      </button>
    </div>

    <!-- Abas de Navegação -->
    <div
      class="flex border-b border-stone-200 dark:border-stone-800 px-4 sm:px-5 gap-1 bg-stone-100/50 dark:bg-stone-950/30 overflow-x-auto"
    >
      <button
        onclick={() => compilerStore.setActiveTab('export')}
        class="flex items-center gap-1.5 py-2.5 px-3 border-b-2 font-medium text-xs sm:text-sm transition-colors whitespace-nowrap {compilerStore.activeTab === 'export' ? 'border-amber-500 text-amber-600 dark:text-amber-400 font-semibold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-300'}"
      >
        💾 {t('compiler.tabExport')}
      </button>

      <button
        onclick={() => compilerStore.setActiveTab('preview')}
        class="flex items-center gap-1.5 py-2.5 px-3 border-b-2 font-medium text-xs sm:text-sm transition-colors whitespace-nowrap {compilerStore.activeTab === 'preview' ? 'border-amber-500 text-amber-600 dark:text-amber-400 font-semibold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-300'}"
      >
        📄 {t('compiler.tabPreview')}
      </button>

      <button
        onclick={() => compilerStore.setActiveTab('matter')}
        class="flex items-center gap-1.5 py-2.5 px-3 border-b-2 font-medium text-xs sm:text-sm transition-colors whitespace-nowrap {compilerStore.activeTab === 'matter' ? 'border-amber-500 text-amber-600 dark:text-amber-400 font-semibold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-300'}"
      >
        ✍️ {t('compiler.tabMatter')}
      </button>

      <button
        onclick={() => compilerStore.setActiveTab('design')}
        class="flex items-center gap-1.5 py-2.5 px-3 border-b-2 font-medium text-xs sm:text-sm transition-colors whitespace-nowrap {compilerStore.activeTab === 'design' ? 'border-amber-500 text-amber-600 dark:text-amber-400 font-semibold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-300'}"
      >
        ⚙️ {t('compiler.tabDesign')}
      </button>
    </div>

    <!-- Corpo da Aba Ativa -->
    <div class="flex-1 overflow-y-auto p-4 sm:p-5 space-y-5">
        <!-- Aba 1: Pré-visualização do Manuscrito Unificado -->
        {#if compilerStore.activeTab === 'preview'}
          <div class="flex items-center justify-between pb-3 border-b border-stone-200 dark:border-stone-800">
            <div>
              <h3 class="font-semibold text-lg">Visualização do Livro Compilado</h3>
              <p class="text-xs text-stone-500 dark:text-stone-400">
                O compilador unifica todas as partes, capítulos, cenas e seções pré/pós-textuais em ordem canônica.
              </p>
            </div>
            <button
              onclick={() => compilerStore.compile()}
              disabled={compilerStore.isCompiling}
              class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium rounded-lg bg-stone-100 hover:bg-stone-200 dark:bg-stone-800 dark:hover:bg-stone-700 transition-colors"
            >
              🔄 {t('compiler.recompileButton')}
            </button>
          </div>

          {#if compilerStore.compiled}
            <div
              class="p-6 bg-stone-50 dark:bg-stone-950/50 rounded-xl border border-stone-200 dark:border-stone-800 font-serif leading-relaxed text-sm whitespace-pre-wrap max-h-[500px] overflow-y-auto select-text selection:bg-amber-500/20"
            >
              {compilerStore.compiled.full_text}
            </div>
          {:else if compilerStore.isCompiling}
            <div class="py-16 text-center text-stone-500">
              <div class="text-3xl mb-2 animate-spin">⏳</div>
              <p>{t('compiler.compiling')}</p>
            </div>
          {/if}

        <!-- Aba 2: Seções Pré e Pós-Textuais (Dedicatória, Epígrafe, Referências, etc.) -->
        {:else if compilerStore.activeTab === 'matter'}
          <div class="space-y-6">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="font-semibold text-lg">Seções Editoriais</h3>
                <p class="text-xs text-stone-500 dark:text-stone-400">
                  Adicione e edite partes formais do livro. Tudo é cifrado com segurança no cofre.
                </p>
              </div>
              <button
                onclick={handleSaveSettings}
                disabled={compilerStore.isSaving}
                class="px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-lg text-xs font-semibold shadow-sm transition-colors flex items-center gap-2"
              >
                💾 {compilerStore.isSaving ? t('compiler.saving') : t('compiler.saveSettings')}
              </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <!-- Título & Subtítulo -->
              <div>
                <label for="book-title" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                  {t('compiler.titleLabel')}
                </label>
                <input
                  id="book-title"
                  type="text"
                  bind:value={compilerStore.settings.title}
                  placeholder="Título da obra"
                  class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                />
              </div>

              <div>
                <label for="book-author" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                  {t('compiler.authorLabel')}
                </label>
                <input
                  id="book-author"
                  type="text"
                  bind:value={compilerStore.settings.author}
                  placeholder="Nome do autor"
                  class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                />
              </div>
            </div>

            <!-- Dedicatória -->
            <div>
              <label for="dedication-input" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                {t('compiler.dedicationLabel')}
              </label>
              <textarea
                id="dedication-input"
                bind:value={compilerStore.settings.dedication}
                placeholder={t('compiler.dedicationPlaceholder')}
                rows="3"
                class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
              ></textarea>
            </div>

            <!-- Epígrafe & Fonte -->
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div class="md:col-span-2">
                <label for="epigraph-input" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                  {t('compiler.epigraphLabel')}
                </label>
                <textarea
                  id="epigraph-input"
                  bind:value={compilerStore.settings.epigraph}
                  placeholder={t('compiler.epigraphPlaceholder')}
                  rows="2"
                  class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                ></textarea>
              </div>
              <div>
                <label for="epigraph-src" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                  {t('compiler.epigraphSourceLabel')}
                </label>
                <input
                  id="epigraph-src"
                  type="text"
                  bind:value={compilerStore.settings.epigraph_source}
                  placeholder="Autor ou obra citada"
                  class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                />
              </div>
            </div>

            <!-- Prefácio & Posfácio -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label for="foreword-input" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                  {t('compiler.forewordLabel')}
                </label>
                <textarea
                  id="foreword-input"
                  bind:value={compilerStore.settings.foreword}
                  rows="3"
                  class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                ></textarea>
              </div>
              <div>
                <label for="afterword-input" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                  {t('compiler.afterwordLabel')}
                </label>
                <textarea
                  id="afterword-input"
                  bind:value={compilerStore.settings.afterword}
                  rows="3"
                  class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                ></textarea>
              </div>
            </div>

            <!-- Referências & Leituras -->
            <div>
              <label for="references-input" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                {t('compiler.referencesLabel')}
              </label>
              <textarea
                id="references-input"
                bind:value={compilerStore.settings.references}
                placeholder={t('compiler.referencesPlaceholder')}
                rows="3"
                class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
              ></textarea>
            </div>

            <!-- Agradecimentos & Sobre o Autor -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label for="acks-input" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                  {t('compiler.acknowledgmentsLabel')}
                </label>
                <textarea
                  id="acks-input"
                  bind:value={compilerStore.settings.acknowledgments}
                  rows="3"
                  class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                ></textarea>
              </div>
              <div>
                <label for="about-input" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                  {t('compiler.aboutAuthorLabel')}
                </label>
                <textarea
                  id="about-input"
                  bind:value={compilerStore.settings.about_author}
                  rows="3"
                  class="w-full px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                ></textarea>
              </div>
            </div>
          </div>

        <!-- Aba 3: Diagramação & Estilos -->
        {:else if compilerStore.activeTab === 'design'}
          <div class="space-y-6 max-w-2xl">
            <div>
              <h3 class="font-semibold text-lg">Opções de Diagramação</h3>
              <p class="text-xs text-stone-500 dark:text-stone-400">
                Ajuste os parâmetros de paginação e estilo tipográfico para a exportação final.
              </p>
            </div>

            <!-- Presets de Tipografia -->
            <div>
              <label for="preset-novel" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-2">
                {t('compiler.presetLabel')}
              </label>
              <div class="space-y-2">
                <label class="flex items-start gap-3 p-3 rounded-lg border border-stone-200 dark:border-stone-800 hover:bg-stone-50 dark:hover:bg-stone-950 cursor-pointer">
                  <input
                    id="preset-novel"
                    type="radio"
                    name="preset"
                    value="novel"
                    bind:group={compilerStore.settings.theme_preset}
                    class="mt-1 text-amber-600 focus:ring-amber-500"
                  />
                  <div>
                    <div class="text-sm font-medium">Romance Clássico</div>
                    <div class="text-xs text-stone-500 dark:text-stone-400">
                      {t('compiler.presetNovel')}
                    </div>
                  </div>
                </label>

                <label class="flex items-start gap-3 p-3 rounded-lg border border-stone-200 dark:border-stone-800 hover:bg-stone-50 dark:hover:bg-stone-950 cursor-pointer">
                  <input
                    type="radio"
                    name="preset"
                    value="essay"
                    bind:group={compilerStore.settings.theme_preset}
                    class="mt-1 text-amber-600 focus:ring-amber-500"
                  />
                  <div>
                    <div class="text-sm font-medium">Ensaio / Artigo</div>
                    <div class="text-xs text-stone-500 dark:text-stone-400">
                      {t('compiler.presetEssay')}
                    </div>
                  </div>
                </label>

                <label class="flex items-start gap-3 p-3 rounded-lg border border-stone-200 dark:border-stone-800 hover:bg-stone-50 dark:hover:bg-stone-950 cursor-pointer">
                  <input
                    type="radio"
                    name="preset"
                    value="poetry"
                    bind:group={compilerStore.settings.theme_preset}
                    class="mt-1 text-amber-600 focus:ring-amber-500"
                  />
                  <div>
                    <div class="text-sm font-medium">Poesia / Livre</div>
                    <div class="text-xs text-stone-500 dark:text-stone-400">
                      {t('compiler.presetPoetry')}
                    </div>
                  </div>
                </label>
              </div>
            </div>

            <!-- Separador de cenas -->
            <div>
              <label for="scene-sep" class="block text-xs font-medium text-stone-600 dark:text-stone-400 mb-1">
                {t('compiler.sceneSeparatorLabel')}
              </label>
              <input
                id="scene-sep"
                type="text"
                bind:value={compilerStore.settings.scene_separator}
                class="w-48 px-3 py-2 text-sm bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
              />
            </div>

            <!-- Checkboxes de paginação -->
            <div class="space-y-3 pt-2">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={compilerStore.settings.page_break_after_chapter}
                  class="rounded text-amber-600 focus:ring-amber-500"
                />
                <span class="text-sm font-medium">{t('compiler.pageBreakAfterChapterLabel')}</span>
              </label>

              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={compilerStore.settings.include_scene_titles}
                  class="rounded text-amber-600 focus:ring-amber-500"
                />
                <span class="text-sm font-medium">{t('compiler.includeSceneTitlesLabel')}</span>
              </label>
            </div>

            <button
              onclick={handleSaveSettings}
              disabled={compilerStore.isSaving}
              class="px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-lg text-xs font-semibold shadow-sm transition-colors flex items-center gap-2"
            >
              💾 {compilerStore.isSaving ? t('compiler.saving') : t('compiler.saveSettings')}
            </button>
          </div>

        <!-- Aba 4: Motores de Exportação Multi-Formato -->
        {:else if compilerStore.activeTab === 'export'}
          <div class="space-y-6">
            <div class="flex items-start justify-between">
              <div>
                <h3 class="font-semibold text-lg">Exportação Editorial Multi-Formato</h3>
                <p class="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
                  Escolha o formato desejado e salve diretamente no seu computador com 1 clique através do diálogo nativo.
                </p>
              </div>
              {#if compilerStore.compiled}
                <div class="text-right">
                  <span class="inline-block px-2.5 py-1 text-xs font-medium rounded-full bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
                    📖 {compilerStore.compiled.total_words} palavras • {compilerStore.compiled.sections.length} seções
                  </span>
                </div>
              {/if}
            </div>

            <!-- Feedback de Sucesso / Erro / Andamento em Destaque -->
            {#if compilerStore.isExporting}
              <div class="flex items-center gap-3 p-4 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-800 dark:text-amber-300 animate-pulse text-sm">
                <span class="text-xl">⏳</span>
                <div>
                  <div class="font-bold">Gerando e compilando publicação...</div>
                  <div class="text-xs opacity-90">Diagramando estrutura, capítulos e metadados no arquivo final.</div>
                </div>
              </div>
            {:else if compilerStore.exportResult}
              <div class="flex items-start justify-between gap-3 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-900 dark:text-emerald-300 text-sm shadow-sm">
                <div class="flex items-start gap-3">
                  <span class="text-2xl">✅</span>
                  <div>
                    <div class="font-bold text-base">{t('compiler.exportSuccess')}</div>
                    {#if 'file_path' in compilerStore.exportResult}
                      <p class="text-xs font-mono text-stone-700 dark:text-stone-300 mt-1 break-all">
                        {compilerStore.exportResult.file_path}
                      </p>
                      <span class="inline-block mt-1 text-[11px] opacity-75">
                        Tamanho final: {(compilerStore.exportResult.byte_size / 1024).toFixed(1)} KB
                      </span>
                    {:else if 'destination_dir' in compilerStore.exportResult}
                      <p class="text-xs font-mono text-stone-700 dark:text-stone-300 mt-1 break-all">
                        {compilerStore.exportResult.destination_dir}
                      </p>
                      <span class="inline-block mt-1 text-[11px] opacity-75">
                        Total gerado: {compilerStore.exportResult.files_count} arquivos organizados
                      </span>
                    {/if}
                  </div>
                </div>

                {#if lastExportedPath}
                  <button
                    type="button"
                    onclick={copyPathToClipboard}
                    class="px-3 py-1.5 rounded-lg text-xs font-medium bg-emerald-600 hover:bg-emerald-700 text-white transition-colors flex-shrink-0"
                  >
                    {copiedFeedback ? 'Copiado! ✓' : '📋 Copiar Caminho'}
                  </button>
                {/if}
              </div>
            {:else if compilerStore.exportError}
              <div class="flex items-start gap-3 p-4 rounded-xl bg-red-500/10 border border-red-500/30 text-red-900 dark:text-red-300 text-sm">
                <span class="text-xl">⚠️</span>
                <div>
                  <div class="font-bold">Falha ao exportar documento</div>
                  <div class="text-xs mt-0.5">{compilerStore.exportError}</div>
                </div>
              </div>
            {/if}

            <!-- Galeria de Cards de Formato Editorial -->
            <div class="grid grid-cols-1 gap-3">
              <!-- DOCX -->
              <div
                class="flex flex-col justify-between p-4 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900/60 shadow-sm hover:border-blue-500/50 hover:shadow-md transition-all group"
              >
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <div class="flex items-center gap-2.5">
                      <div class="w-8 h-8 rounded-lg bg-blue-500/10 text-blue-600 dark:text-blue-400 flex items-center justify-center text-lg font-bold">
                        📄
                      </div>
                      <div>
                        <h4 class="font-bold text-sm text-stone-900 dark:text-stone-100">{t('compiler.exportDocxTitle')}</h4>
                        <span class="text-[10px] font-mono uppercase tracking-wider text-blue-600 dark:text-blue-400 font-semibold">.docx • Word / LibreOffice</span>
                      </div>
                    </div>
                  </div>
                  <p class="text-xs text-stone-600 dark:text-stone-400 leading-relaxed mb-3">
                    {t('compiler.exportDocxDesc')} Inclui estilos de parágrafo, quebras automáticas de capítulo e notas.
                  </p>
                </div>

                <button
                  type="button"
                  disabled={compilerStore.isExporting}
                  onclick={() => handleQuickExport('docx')}
                  class="w-full py-2 px-3 rounded-lg bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white font-medium text-xs shadow-sm transition-colors flex items-center justify-center gap-2"
                >
                  <span>💾 Salvar como DOCX...</span>
                </button>
              </div>

              <!-- EPUB -->
              <div
                class="flex flex-col justify-between p-4 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900/60 shadow-sm hover:border-emerald-500/50 hover:shadow-md transition-all group"
              >
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <div class="flex items-center gap-2.5">
                      <div class="w-8 h-8 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 flex items-center justify-center text-lg font-bold">
                        📱
                      </div>
                      <div>
                        <h4 class="font-bold text-sm text-stone-900 dark:text-stone-100">{t('compiler.exportEpubTitle')}</h4>
                        <span class="text-[10px] font-mono uppercase tracking-wider text-emerald-600 dark:text-emerald-400 font-semibold">.epub • Leitores Digitais</span>
                      </div>
                    </div>
                  </div>
                  <p class="text-xs text-stone-600 dark:text-stone-400 leading-relaxed mb-3">
                    {t('compiler.exportEpubDesc')} Pronto para Apple Books, Kobo, Kindle e lojas de e-books com metadados.
                  </p>
                </div>

                <button
                  type="button"
                  disabled={compilerStore.isExporting}
                  onclick={() => handleQuickExport('epub')}
                  class="w-full py-2 px-3 rounded-lg bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white font-medium text-xs shadow-sm transition-colors flex items-center justify-center gap-2"
                >
                  <span>💾 Salvar como EPUB...</span>
                </button>
              </div>

              <!-- PDF / HTML -->
              <div
                class="flex flex-col justify-between p-4 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900/60 shadow-sm hover:border-rose-500/50 hover:shadow-md transition-all group"
              >
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <div class="flex items-center gap-2.5">
                      <div class="w-8 h-8 rounded-lg bg-rose-500/10 text-rose-600 dark:text-rose-400 flex items-center justify-center text-lg font-bold">
                        🖨️
                      </div>
                      <div>
                        <h4 class="font-bold text-sm text-stone-900 dark:text-stone-100">{t('compiler.exportPdfTitle')}</h4>
                        <span class="text-[10px] font-mono uppercase tracking-wider text-rose-600 dark:text-rose-400 font-semibold">.html / .pdf • Tipografia Editorial</span>
                      </div>
                    </div>
                  </div>
                  <p class="text-xs text-stone-600 dark:text-stone-400 leading-relaxed mb-3">
                    {t('compiler.exportPdfDesc')} Layout limpo e tipografia selecionada pronto para imprimir ou gerar PDF direto no navegador.
                  </p>
                </div>

                <button
                  type="button"
                  disabled={compilerStore.isExporting}
                  onclick={() => handleQuickExport('pdf')}
                  class="w-full py-2 px-3 rounded-lg bg-rose-600 hover:bg-rose-700 disabled:opacity-50 text-white font-medium text-xs shadow-sm transition-colors flex items-center justify-center gap-2"
                >
                  <span>💾 Salvar HTML / PDF...</span>
                </button>
              </div>

              <!-- Obsidian Markdown -->
              <div
                class="flex flex-col justify-between p-4 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900/60 shadow-sm hover:border-purple-500/50 hover:shadow-md transition-all group"
              >
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <div class="flex items-center gap-2.5">
                      <div class="w-8 h-8 rounded-lg bg-purple-500/10 text-purple-600 dark:text-purple-400 flex items-center justify-center text-lg font-bold">
                        📁
                      </div>
                      <div>
                        <h4 class="font-bold text-sm text-stone-900 dark:text-stone-100">{t('compiler.exportObsidianTitle')}</h4>
                        <span class="text-[10px] font-mono uppercase tracking-wider text-purple-600 dark:text-purple-400 font-semibold">.md • Hiperlinks [[ ]]</span>
                      </div>
                    </div>
                  </div>
                  <p class="text-xs text-stone-600 dark:text-stone-400 leading-relaxed mb-3">
                    {t('compiler.exportObsidianDesc')} Transforma todo o Mundo em notas com ligações bidirecionais, personagens e locais.
                  </p>
                </div>

                <button
                  type="button"
                  disabled={compilerStore.isExporting}
                  onclick={() => handleQuickExport('obsidian')}
                  class="w-full py-2 px-3 rounded-lg bg-purple-600 hover:bg-purple-700 disabled:opacity-50 text-white font-medium text-xs shadow-sm transition-colors flex items-center justify-center gap-2"
                >
                  <span>💾 Exportar para Pasta Obsidian...</span>
                </button>
              </div>

              <!-- TXT + Dossiê -->
              <div
                class="flex flex-col justify-between p-4 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900/60 shadow-sm hover:border-stone-400 hover:shadow-md transition-all group"
              >
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <div class="flex items-center gap-2.5">
                      <div class="w-8 h-8 rounded-lg bg-stone-500/10 text-stone-600 dark:text-stone-400 flex items-center justify-center text-lg font-bold">
                        📝
                      </div>
                      <div>
                        <h4 class="font-bold text-sm text-stone-900 dark:text-stone-100">{t('compiler.exportTxtTitle')}</h4>
                        <span class="text-[10px] font-mono uppercase tracking-wider text-stone-500 dark:text-stone-400 font-semibold">.txt • Texto Puro & Dossiê Completo</span>
                      </div>
                    </div>
                  </div>
                  <p class="text-xs text-stone-600 dark:text-stone-400 leading-relaxed mb-3">
                    {t('compiler.exportTxtDesc')} Salva o manuscrito consolidado em texto puro sem formatações complexas, perfeito para arquivamento ou processamentos de texto.
                  </p>
                </div>

                <button
                  type="button"
                  disabled={compilerStore.isExporting}
                  onclick={() => handleQuickExport('txt')}
                  class="w-full py-2 px-3 rounded-lg bg-stone-700 hover:bg-stone-800 disabled:opacity-50 text-white font-medium text-xs shadow-sm transition-colors flex items-center justify-center gap-2"
                >
                  <span>💾 Salvar TXT + Dossiê em Pasta...</span>
                </button>
              </div>
            </div>

            <!-- Opções Avançadas de Destino Manual (Colapsável) -->
            <div class="border-t border-stone-200 dark:border-stone-800 pt-3">
              <button
                type="button"
                onclick={() => (showAdvancedPath = !showAdvancedPath)}
                class="text-xs text-stone-500 hover:text-stone-800 dark:hover:text-stone-300 font-medium flex items-center gap-1.5 transition-colors"
              >
                <span>{showAdvancedPath ? '▼' : '▶'}</span>
                <span>Opções Avançadas: Especificar caminho manual no disco</span>
              </button>

              {#if showAdvancedPath}
                <div class="mt-2.5 p-3.5 bg-stone-50 dark:bg-stone-950/50 rounded-xl border border-stone-200 dark:border-stone-800 space-y-2.5">
                  <label for="export-dest" class="block text-xs font-semibold text-stone-600 dark:text-stone-400">
                    Caminho Completo do Arquivo ou Diretório de Destino:
                  </label>
                  <div class="flex flex-col sm:flex-row gap-2">
                    <input
                      id="export-dest"
                      type="text"
                      bind:value={exportPathInput}
                      placeholder="/Users/seu-usuario/Documents/meu_livro.docx"
                      class="flex-1 px-3 py-1.5 text-xs bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500 font-mono"
                    />
                    <button
                      type="button"
                      onclick={handleExport}
                      disabled={compilerStore.isExporting || !exportPathInput.trim()}
                      class="px-4 py-1.5 bg-amber-600 hover:bg-amber-700 disabled:opacity-50 text-white rounded-lg font-semibold text-xs shadow-sm transition-colors flex items-center justify-center gap-2"
                    >
                      Exportar
                    </button>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer do Painel -->
      <div
        class="flex items-center justify-between px-5 py-3 border-t border-stone-200 dark:border-stone-800 bg-stone-50/80 dark:bg-stone-900/80 text-xs text-stone-500"
      >
        <span>Compilador Editorial Offline</span>
        <button
          onclick={() => compilerStore.closeModal()}
          class="px-3.5 py-1.5 rounded-lg bg-stone-200 dark:bg-stone-800 hover:bg-stone-300 dark:hover:bg-stone-700 text-stone-800 dark:text-stone-200 font-medium transition-colors"
        >
          Fechar
        </button>
      </div>
    </div>
{/if}
