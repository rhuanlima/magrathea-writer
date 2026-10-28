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

  let exportPathInput = $state<string>('');
  let selectedExportFormat = $state<'obsidian' | 'docx' | 'epub' | 'pdf' | 'txt'>('docx');

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
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 overflow-y-auto"
    role="dialog"
    aria-modal="true"
    aria-labelledby="compiler-modal-title"
  >
    <!-- Modal Container -->
    <div
      class="bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 rounded-xl shadow-2xl w-full max-w-5xl max-h-[90vh] flex flex-col overflow-hidden text-stone-900 dark:text-stone-100"
    >
      <!-- Header -->
      <div
        class="flex items-center justify-between px-6 py-4 border-b border-stone-200 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-900/50"
      >
        <div class="flex items-center gap-3">
          <div
            class="text-2xl p-1.5 rounded-lg bg-amber-500/10 text-amber-600 dark:text-amber-400"
          >
            📖
          </div>
          <div>
            <h2 id="compiler-modal-title" class="text-xl font-bold tracking-tight">
              {t('compiler.modalTitle')}
            </h2>
            {#if compilerStore.compiled}
              <div class="flex items-center gap-3 text-xs text-stone-500 dark:text-stone-400 mt-0.5">
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
          class="p-2 rounded-lg text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors font-bold text-lg"
          aria-label="Fechar"
        >
          ✕
        </button>
      </div>

      <!-- Abas de Navegação -->
      <div
        class="flex border-b border-stone-200 dark:border-stone-800 px-6 gap-2 bg-stone-100/50 dark:bg-stone-950/30"
      >
        <button
          onclick={() => compilerStore.setActiveTab('preview')}
          class="flex items-center gap-2 py-3 px-4 border-b-2 font-medium text-sm transition-colors {compilerStore.activeTab === 'preview' ? 'border-amber-500 text-amber-600 dark:text-amber-400' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-300'}"
        >
          📄 {t('compiler.tabPreview')}
        </button>

        <button
          onclick={() => compilerStore.setActiveTab('matter')}
          class="flex items-center gap-2 py-3 px-4 border-b-2 font-medium text-sm transition-colors {compilerStore.activeTab === 'matter' ? 'border-amber-500 text-amber-600 dark:text-amber-400' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-300'}"
        >
          ✍️ {t('compiler.tabMatter')}
        </button>

        <button
          onclick={() => compilerStore.setActiveTab('design')}
          class="flex items-center gap-2 py-3 px-4 border-b-2 font-medium text-sm transition-colors {compilerStore.activeTab === 'design' ? 'border-amber-500 text-amber-600 dark:text-amber-400' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-300'}"
        >
          ⚙️ {t('compiler.tabDesign')}
        </button>

        <button
          onclick={() => compilerStore.setActiveTab('export')}
          class="flex items-center gap-2 py-3 px-4 border-b-2 font-medium text-sm transition-colors {compilerStore.activeTab === 'export' ? 'border-amber-500 text-amber-600 dark:text-amber-400' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-300'}"
        >
          💾 {t('compiler.tabExport')}
        </button>
      </div>

      <!-- Corpo da Aba Ativa -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
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
            <div>
              <h3 class="font-semibold text-lg">Exportação Multi-Formato</h3>
              <p class="text-xs text-stone-500 dark:text-stone-400">
                Selecione o formato desejado para compilar e salvar o livro diretamente no disco.
              </p>
            </div>

            <!-- Cards de Formato -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <!-- DOCX -->
              <button
                type="button"
                onclick={() => setDefaultExportPath('docx')}
                class="p-4 rounded-xl border text-left transition-all {selectedExportFormat === 'docx' ? 'border-amber-500 bg-amber-50/20 dark:bg-amber-950/20 ring-1 ring-amber-500' : 'border-stone-200 dark:border-stone-800 hover:bg-stone-50 dark:hover:bg-stone-950'}"
              >
                <div class="flex items-center gap-3 mb-2">
                  <div class="text-xl p-2 rounded-lg bg-blue-500/10 text-blue-600 dark:text-blue-400">
                    📄
                  </div>
                  <div class="font-bold text-sm">{t('compiler.exportDocxTitle')}</div>
                </div>
                <p class="text-xs text-stone-500 dark:text-stone-400 leading-relaxed">
                  {t('compiler.exportDocxDesc')}
                </p>
              </button>

              <!-- EPUB -->
              <button
                type="button"
                onclick={() => setDefaultExportPath('epub')}
                class="p-4 rounded-xl border text-left transition-all {selectedExportFormat === 'epub' ? 'border-amber-500 bg-amber-50/20 dark:bg-amber-950/20 ring-1 ring-amber-500' : 'border-stone-200 dark:border-stone-800 hover:bg-stone-50 dark:hover:bg-stone-950'}"
              >
                <div class="flex items-center gap-3 mb-2">
                  <div class="text-xl p-2 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
                    📱
                  </div>
                  <div class="font-bold text-sm">{t('compiler.exportEpubTitle')}</div>
                </div>
                <p class="text-xs text-stone-500 dark:text-stone-400 leading-relaxed">
                  {t('compiler.exportEpubDesc')}
                </p>
              </button>

              <!-- PDF / HTML -->
              <button
                type="button"
                onclick={() => setDefaultExportPath('pdf')}
                class="p-4 rounded-xl border text-left transition-all {selectedExportFormat === 'pdf' ? 'border-amber-500 bg-amber-50/20 dark:bg-amber-950/20 ring-1 ring-amber-500' : 'border-stone-200 dark:border-stone-800 hover:bg-stone-50 dark:hover:bg-stone-950'}"
              >
                <div class="flex items-center gap-3 mb-2">
                  <div class="text-xl p-2 rounded-lg bg-red-500/10 text-red-600 dark:text-red-400">
                    🖨️
                  </div>
                  <div class="font-bold text-sm">{t('compiler.exportPdfTitle')}</div>
                </div>
                <p class="text-xs text-stone-500 dark:text-stone-400 leading-relaxed">
                  {t('compiler.exportPdfDesc')}
                </p>
              </button>

              <!-- Obsidian Markdown -->
              <button
                type="button"
                onclick={() => setDefaultExportPath('obsidian')}
                class="p-4 rounded-xl border text-left transition-all {selectedExportFormat === 'obsidian' ? 'border-amber-500 bg-amber-50/20 dark:bg-amber-950/20 ring-1 ring-amber-500' : 'border-stone-200 dark:border-stone-800 hover:bg-stone-50 dark:hover:bg-stone-950'}"
              >
                <div class="flex items-center gap-3 mb-2">
                  <div class="text-xl p-2 rounded-lg bg-purple-500/10 text-purple-600 dark:text-purple-400">
                    📁
                  </div>
                  <div class="font-bold text-sm">{t('compiler.exportObsidianTitle')}</div>
                </div>
                <p class="text-xs text-stone-500 dark:text-stone-400 leading-relaxed">
                  {t('compiler.exportObsidianDesc')}
                </p>
              </button>

              <!-- TXT + Dossiê -->
              <button
                type="button"
                onclick={() => setDefaultExportPath('txt')}
                class="p-4 rounded-xl border text-left transition-all {selectedExportFormat === 'txt' ? 'border-amber-500 bg-amber-50/20 dark:bg-amber-950/20 ring-1 ring-amber-500' : 'border-stone-200 dark:border-stone-800 hover:bg-stone-50 dark:hover:bg-stone-950'}"
              >
                <div class="flex items-center gap-3 mb-2">
                  <div class="text-xl p-2 rounded-lg bg-stone-500/10 text-stone-600 dark:text-stone-400">
                    📝
                  </div>
                  <div class="font-bold text-sm">{t('compiler.exportTxtTitle')}</div>
                </div>
                <p class="text-xs text-stone-500 dark:text-stone-400 leading-relaxed">
                  {t('compiler.exportTxtDesc')}
                </p>
              </button>
            </div>

            <!-- Destino do arquivo & Disparo -->
            <div class="p-4 bg-stone-50 dark:bg-stone-950/50 rounded-xl border border-stone-200 dark:border-stone-800 space-y-3">
              <label for="export-dest" class="block text-xs font-semibold text-stone-600 dark:text-stone-400">
                Caminho do Arquivo ou Pasta de Destino:
              </label>
              <div class="flex gap-2">
                <input
                  id="export-dest"
                  type="text"
                  bind:value={exportPathInput}
                  placeholder="/caminho/do/arquivo"
                  class="flex-1 px-3 py-2 text-sm bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
                />
                <button
                  onclick={handleExport}
                  disabled={compilerStore.isExporting || !exportPathInput.trim()}
                  class="px-5 py-2 bg-amber-600 hover:bg-amber-700 disabled:opacity-50 text-white rounded-lg font-semibold text-sm shadow-sm transition-colors flex items-center gap-2"
                >
                  💾 {compilerStore.isExporting ? t('compiler.exporting') : 'Exportar Agora'}
                </button>
              </div>

              <!-- Feedback de Exportação -->
              {#if compilerStore.exportResult}
                <div class="flex items-center gap-2 p-3 rounded-lg bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 text-xs border border-emerald-500/20">
                  <span class="text-base">✅</span>
                  <div>
                    <span class="font-bold">{t('compiler.exportSuccess')}</span>
                    {#if 'file_path' in compilerStore.exportResult}
                      <span class="block text-stone-600 dark:text-stone-300 mt-0.5">
                        Salvo em: {compilerStore.exportResult.file_path} ({compilerStore.exportResult.byte_size} bytes)
                      </span>
                    {:else if 'destination_dir' in compilerStore.exportResult}
                      <span class="block text-stone-600 dark:text-stone-300 mt-0.5">
                        Salvo em: {compilerStore.exportResult.destination_dir} ({compilerStore.exportResult.files_count} arquivos)
                      </span>
                    {/if}
                  </div>
                </div>
              {:else if compilerStore.exportError}
                <div class="flex items-center gap-2 p-3 rounded-lg bg-red-500/10 text-red-700 dark:text-red-400 text-xs border border-red-500/20">
                  <span class="text-base">⚠️</span>
                  <span>{t('compiler.exportError', { error: compilerStore.exportError })}</span>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div
        class="flex items-center justify-between px-6 py-4 border-t border-stone-200 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-900/50 text-xs text-stone-500"
      >
        <span>Magrathea Writer • Compilador Offline de Alta Fidelidade</span>
        <button
          onclick={() => compilerStore.closeModal()}
          class="px-4 py-2 rounded-lg bg-stone-200 dark:bg-stone-800 hover:bg-stone-300 dark:hover:bg-stone-700 text-stone-800 dark:text-stone-200 font-medium transition-colors"
        >
          Fechar
        </button>
      </div>
    </div>
  </div>
{/if}
