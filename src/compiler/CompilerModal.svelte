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
      const defaultName = getSafeFilename('pdf');
      let chosen = await pickSaveExportPath('Salvar Livro em PDF (.pdf)', defaultName, 'pdf');
      if (chosen) {
        if (!chosen.toLowerCase().endsWith('.pdf')) {
          chosen += '.pdf';
        }
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

  async function copyPreviewToClipboard() {
    if (!compilerStore.compiled?.full_text) return;
    try {
      await navigator.clipboard.writeText(compilerStore.compiled.full_text);
      copiedFeedback = true;
      setTimeout(() => {
        copiedFeedback = false;
      }, 2000);
    } catch {
      // Fallback
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
      exportPathInput = `~/Documents/${title}.pdf`;
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
  <div class="compiler-backdrop" onclick={() => compilerStore.closeModal()}></div>

  <!-- Drawer Lateral / Modal Editorial -->
  <div
    class="compiler-drawer"
    role="dialog"
    aria-modal="true"
    aria-labelledby="compiler-modal-title"
  >
    <!-- Header do Painel -->
    <div class="compiler-header">
      <div class="header-info">
        <div class="header-icon">📖</div>
        <div>
          <h2 id="compiler-modal-title" class="header-title">
            {t('compiler.modalTitle')}
          </h2>
          {#if compilerStore.compiled}
            <div class="header-stats">
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
        type="button"
        onclick={() => compilerStore.closeModal()}
        class="close-button"
        title="Fechar (Esc)"
        aria-label="Fechar"
      >
        ✕
      </button>
    </div>

    <!-- Abas de Navegação -->
    <div class="compiler-tabs" role="tablist">
      <button
        type="button"
        onclick={() => compilerStore.setActiveTab('export')}
        class="tab-btn"
        class:active={compilerStore.activeTab === 'export'}
      >
        💾 {t('compiler.tabExport')}
      </button>

      <button
        type="button"
        onclick={() => compilerStore.setActiveTab('preview')}
        class="tab-btn"
        class:active={compilerStore.activeTab === 'preview'}
      >
        📄 {t('compiler.tabPreview')}
      </button>

      <button
        type="button"
        onclick={() => compilerStore.setActiveTab('matter')}
        class="tab-btn"
        class:active={compilerStore.activeTab === 'matter'}
      >
        ✍️ {t('compiler.tabMatter')}
      </button>

      <button
        type="button"
        onclick={() => compilerStore.setActiveTab('design')}
        class="tab-btn"
        class:active={compilerStore.activeTab === 'design'}
      >
        ⚙️ {t('compiler.tabDesign')}
      </button>
    </div>

    <!-- Corpo da Aba Ativa -->
    <div class="compiler-body">
      <!-- Aba 1: Pré-visualização do Manuscrito Unificado -->
      {#if compilerStore.activeTab === 'preview'}
        <div class="preview-header">
          <div>
            <h3 class="section-title">Pré-visualização do Manuscrito</h3>
            <p class="section-subtitle">Exibição contínua de todas as seções e capítulos compilados.</p>
          </div>
          <div class="preview-actions">
            <button
              type="button"
              onclick={() => compilerStore.compile()}
              disabled={compilerStore.isCompiling}
              class="btn-secondary"
            >
              🔄 {compilerStore.isCompiling ? t('compiler.compiling') : 'Recompilar'}
            </button>
            <button
              type="button"
              onclick={copyPreviewToClipboard}
              class="btn-primary"
            >
              {copiedFeedback ? '✓ Copiado!' : '📋 Copiar Texto'}
            </button>
          </div>
        </div>

        {#if compilerStore.compiled}
          <div class="preview-book-sheet">
            {compilerStore.compiled.full_text}
          </div>
        {:else if compilerStore.isCompiling}
          <div class="loading-state">
            <div class="loading-spinner">⏳</div>
            <p>{t('compiler.compiling')}</p>
          </div>
        {/if}

      <!-- Aba 2: Seções Pré e Pós-Textuais (Dedicatória, Epígrafe, Referências, etc.) -->
      {:else if compilerStore.activeTab === 'matter'}
        <div class="matter-container">
          <div class="matter-top-bar">
            <div>
              <h3 class="section-title">Partes do Livro (Pré e Pós-Textuais)</h3>
              <p class="section-subtitle">
                Adicione e edite referências, agradecimentos, dedicatória, epígrafe e posfácio como blocos legítimos de livro.
              </p>
            </div>
            <button
              type="button"
              onclick={handleSaveSettings}
              disabled={compilerStore.isSaving}
              class="btn-save-matter"
            >
              💾 {compilerStore.isSaving ? t('compiler.saving') : 'Salvar no Mundo'}
            </button>
          </div>

          <!-- Bloco 1: Identificação da Obra -->
          <div class="matter-card">
            <div class="matter-card-header">
              <span class="card-tag">Identificação</span>
              <h4 class="card-title">Capa & Informações da Obra</h4>
            </div>
            <div class="matter-two-cols">
              <div class="matter-field">
                <label for="book-title" class="matter-label">{t('compiler.titleLabel')}</label>
                <input
                  id="book-title"
                  type="text"
                  bind:value={compilerStore.settings.title}
                  placeholder="Título principal da obra"
                  class="matter-input"
                />
              </div>

              <div class="matter-field">
                <label for="book-author" class="matter-label">{t('compiler.authorLabel')}</label>
                <input
                  id="book-author"
                  type="text"
                  bind:value={compilerStore.settings.author}
                  placeholder="Nome do autor ou pseudônimo"
                  class="matter-input"
                />
              </div>
            </div>
          </div>

          <!-- Bloco 2: Pré-Textuais -->
          <div class="matter-card">
            <div class="matter-card-header">
              <span class="card-tag tag-pre">Pré-Textuais</span>
              <h4 class="card-title">Abertura do Livro</h4>
            </div>

            <!-- Dedicatória -->
            <div class="matter-field">
              <div class="field-info">
                <label for="dedication-input" class="matter-label">{t('compiler.dedicationLabel')}</label>
                <span class="matter-hint">A quem este livro é oferecido com carinho ou homenagem</span>
              </div>
              <textarea
                id="dedication-input"
                bind:value={compilerStore.settings.dedication}
                placeholder={t('compiler.dedicationPlaceholder')}
                rows="4"
                class="matter-textarea literary-prose"
              ></textarea>
            </div>

            <!-- Epígrafe -->
            <div class="matter-field">
              <div class="field-info">
                <label for="epigraph-input" class="matter-label">{t('compiler.epigraphLabel')}</label>
                <span class="matter-hint">Citação, verso ou pensamento temático que ecoa o tom da narrativa</span>
              </div>
              <textarea
                id="epigraph-input"
                bind:value={compilerStore.settings.epigraph}
                placeholder={t('compiler.epigraphPlaceholder')}
                rows="3"
                class="matter-textarea literary-prose"
              ></textarea>
              <div class="epigraph-source-row">
                <label for="epigraph-src" class="matter-label inline-label">{t('compiler.epigraphSourceLabel')}:</label>
                <input
                  id="epigraph-src"
                  type="text"
                  bind:value={compilerStore.settings.epigraph_source}
                  placeholder="Autor, filósofo ou obra citada"
                  class="matter-input inline-input"
                />
              </div>
            </div>

            <!-- Prefácio -->
            <div class="matter-field">
              <div class="field-info">
                <label for="foreword-input" class="matter-label">{t('compiler.forewordLabel')}</label>
                <span class="matter-hint">Apresentação preliminar que introduz o universo ou o contexto da história</span>
              </div>
              <textarea
                id="foreword-input"
                bind:value={compilerStore.settings.foreword}
                placeholder="Escreva aqui o prefácio ou nota introdutória..."
                rows="6"
                class="matter-textarea literary-prose"
              ></textarea>
            </div>
          </div>

          <!-- Bloco 3: Pós-Textuais -->
          <div class="matter-card">
            <div class="matter-card-header">
              <span class="card-tag tag-post">Pós-Textuais</span>
              <h4 class="card-title">Encerramento & Apêndices</h4>
            </div>

            <!-- Posfácio -->
            <div class="matter-field">
              <div class="field-info">
                <label for="afterword-input" class="matter-label">{t('compiler.afterwordLabel')}</label>
                <span class="matter-hint">Reflexões de bastidores, ensaio de encerramento ou considerações finais</span>
              </div>
              <textarea
                id="afterword-input"
                bind:value={compilerStore.settings.afterword}
                placeholder="Escreva aqui o posfácio..."
                rows="6"
                class="matter-textarea literary-prose"
              ></textarea>
            </div>

            <!-- Referências & Leituras -->
            <div class="matter-field">
              <div class="field-info">
                <label for="references-input" class="matter-label">{t('compiler.referencesLabel')}</label>
                <span class="matter-hint">Bibliografia, arquivos históricos consultados ou leituras sugeridas</span>
              </div>
              <textarea
                id="references-input"
                bind:value={compilerStore.settings.references}
                placeholder={t('compiler.referencesPlaceholder')}
                rows="5"
                class="matter-textarea"
              ></textarea>
            </div>

            <!-- Agradecimentos -->
            <div class="matter-field">
              <div class="field-info">
                <label for="acks-input" class="matter-label">{t('compiler.acknowledgmentsLabel')}</label>
                <span class="matter-hint">Reconhecimento a leitores beta, revisores, familiares e amigos</span>
              </div>
              <textarea
                id="acks-input"
                bind:value={compilerStore.settings.acknowledgments}
                placeholder="Agradeço imensamente a..."
                rows="5"
                class="matter-textarea literary-prose"
              ></textarea>
            </div>

            <!-- Sobre o Autor -->
            <div class="matter-field">
              <div class="field-info">
                <label for="about-input" class="matter-label">{t('compiler.aboutAuthorLabel')}</label>
                <span class="matter-hint">Minibiografia para a orelha ou página final do livro impresso</span>
              </div>
              <textarea
                id="about-input"
                bind:value={compilerStore.settings.about_author}
                placeholder="Sobre o autor(a)..."
                rows="4"
                class="matter-textarea"
              ></textarea>
            </div>
          </div>
        </div>

      <!-- Aba 3: Diagramação & Estilos -->
      {:else if compilerStore.activeTab === 'design'}
        <div class="design-container">
          <div>
            <h3 class="section-title">Opções de Diagramação</h3>
            <p class="section-subtitle">
              Ajuste os parâmetros de paginação e estilo tipográfico para a exportação final.
            </p>
          </div>

          <!-- Presets de Tipografia -->
          <div class="design-card">
            <label for="preset-novel" class="matter-label">{t('compiler.presetLabel')}</label>
            <div class="preset-choices">
              <label class="preset-option" class:selected={compilerStore.settings.theme_preset === 'novel'}>
                <input
                  id="preset-novel"
                  type="radio"
                  name="preset"
                  value="novel"
                  bind:group={compilerStore.settings.theme_preset}
                />
                <div>
                  <div class="preset-title">Romance Clássico & Ficção Narrativa</div>
                  <div class="preset-desc">{t('compiler.presetNovel')}</div>
                </div>
              </label>

              <label class="preset-option" class:selected={compilerStore.settings.theme_preset === 'poetry'}>
                <input
                  type="radio"
                  name="preset"
                  value="poetry"
                  bind:group={compilerStore.settings.theme_preset}
                />
                <div>
                  <div class="preset-title">Poesia & Versos Livres</div>
                  <div class="preset-desc">{t('compiler.presetPoetry')}</div>
                </div>
              </label>
            </div>
          </div>

          <!-- Separador e Paginação -->
          <div class="design-card">
            <div class="matter-field">
              <label for="scene-sep" class="matter-label">{t('compiler.sceneSeparatorLabel')}</label>
              <input
                id="scene-sep"
                type="text"
                bind:value={compilerStore.settings.scene_separator}
                class="matter-input sep-input"
              />
            </div>

            <div class="checkbox-group">
              <label class="checkbox-label">
                <input
                  type="checkbox"
                  bind:checked={compilerStore.settings.page_break_after_chapter}
                />
                <span>{t('compiler.pageBreakAfterChapterLabel')}</span>
              </label>

              <label class="checkbox-label">
                <input
                  type="checkbox"
                  bind:checked={compilerStore.settings.include_scene_titles}
                />
                <span>{t('compiler.includeSceneTitlesLabel')}</span>
              </label>
            </div>
          </div>

          <button
            type="button"
            onclick={handleSaveSettings}
            disabled={compilerStore.isSaving}
            class="btn-save-matter"
          >
            💾 {compilerStore.isSaving ? t('compiler.saving') : t('compiler.saveSettings')}
          </button>
        </div>

      <!-- Aba 4: Motores de Exportação Multi-Formato -->
      {:else if compilerStore.activeTab === 'export'}
        <div class="export-container">
          <div class="export-top-bar">
            <div>
              <h3 class="section-title">Exportação Editorial Multi-Formato</h3>
              <p class="section-subtitle">
                Escolha o formato desejado e salve diretamente no seu computador com 1 clique através do diálogo nativo.
              </p>
            </div>
            {#if compilerStore.compiled}
              <div class="export-badge-pill">
                📖 {compilerStore.compiled.total_words} palavras • {compilerStore.compiled.sections.length} seções
              </div>
            {/if}
          </div>

          <!-- Opções Rápidas de Compilação & Escopo Editorial -->
          <div class="export-options-box">
            <div class="box-title">
              <span>⚙️</span>
              <h4>Opções de Conteúdo da Exportação</h4>
            </div>

            <div class="box-content-grid">
              <!-- Escopo: Tudo vs Final -->
              <div class="option-column">
                <span class="option-col-label">Cenas Incluídas:</span>
                <div class="radio-pill-group">
                  <label class="radio-pill" class:active={!compilerStore.settings.only_final_scenes}>
                    <input
                      type="radio"
                      name="scopeChoice"
                      checked={!compilerStore.settings.only_final_scenes}
                      onchange={() => (compilerStore.settings.only_final_scenes = false)}
                    />
                    <span>📚 Todas as Cenas</span>
                  </label>
                  <label class="radio-pill" class:active={compilerStore.settings.only_final_scenes}>
                    <input
                      type="radio"
                      name="scopeChoice"
                      checked={compilerStore.settings.only_final_scenes}
                      onchange={() => (compilerStore.settings.only_final_scenes = true)}
                    />
                    <span>✨ Apenas Prontas (Final)</span>
                  </label>
                </div>
              </div>

              <!-- Anexar Fichas -->
              <div class="option-column">
                <span class="option-col-label">Apêndice de Construção de Mundo:</span>
                <label class="checkbox-label world-check">
                  <input
                    type="checkbox"
                    bind:checked={compilerStore.settings.include_worldbuilding_dossier}
                  />
                  <span>Anexar Fichas de Personagens e Lugares no final</span>
                </label>
              </div>
            </div>
          </div>

          <!-- Grid de Motores de Exportação em 1 Clique -->
          <div class="formats-grid">
            <!-- Word DOCX -->
            <div class="format-card">
              <div class="format-card-top">
                <div class="format-icon icon-blue">📄</div>
                <div>
                  <h4 class="format-title">Microsoft Word</h4>
                  <span class="format-ext">.docx</span>
                </div>
              </div>
              <p class="format-desc">Ideal para submissão a editoras, agentes literários e revisão compartilhada.</p>
              <button
                type="button"
                onclick={() => handleQuickExport('docx')}
                disabled={compilerStore.isExporting}
                class="btn-format btn-format-blue"
              >
                {compilerStore.isExporting && selectedExportFormat === 'docx' ? 'Exportando...' : 'Salvar Word (.docx)'}
              </button>
            </div>

            <!-- Livro Digital EPUB -->
            <div class="format-card">
              <div class="format-card-top">
                <div class="format-icon icon-emerald">📱</div>
                <div>
                  <h4 class="format-title">Livro Digital EPUB</h4>
                  <span class="format-ext">.epub</span>
                </div>
              </div>
              <p class="format-desc">Pronto para Kindle, Apple Books, Kobo e leitores de e-books com sumário.</p>
              <button
                type="button"
                onclick={() => handleQuickExport('epub')}
                disabled={compilerStore.isExporting}
                class="btn-format btn-format-emerald"
              >
                {compilerStore.isExporting && selectedExportFormat === 'epub' ? 'Exportando...' : 'Salvar EPUB (.epub)'}
              </button>
            </div>

            <!-- Livro em PDF Direto -->
            <div class="format-card">
              <div class="format-card-top">
                <div class="format-icon icon-rose">📕</div>
                <div>
                  <h4 class="format-title">Livro em PDF</h4>
                  <span class="format-ext">.pdf</span>
                </div>
              </div>
              <p class="format-desc">Documento PDF nativo diagramado em formato A5 com paginação para leitura ou gráfica.</p>
              <button
                type="button"
                onclick={() => handleQuickExport('pdf')}
                disabled={compilerStore.isExporting}
                class="btn-format btn-format-rose"
              >
                {compilerStore.isExporting && selectedExportFormat === 'pdf' ? 'Exportando...' : 'Salvar PDF (.pdf)'}
              </button>
            </div>

            <!-- Cofre Obsidian -->
            <div class="format-card">
              <div class="format-card-top">
                <div class="format-icon icon-purple">💎</div>
                <div>
                  <h4 class="format-title">Cofre Obsidian</h4>
                  <span class="format-ext">Markdown + Links</span>
                </div>
              </div>
              <p class="format-desc">Exporta todas as notas com wikilinks [[...]] preservados em pastas.</p>
              <button
                type="button"
                onclick={() => handleQuickExport('obsidian')}
                disabled={compilerStore.isExporting}
                class="btn-format btn-format-purple"
              >
                {compilerStore.isExporting && selectedExportFormat === 'obsidian' ? 'Exportando...' : 'Exportar para Obsidian'}
              </button>
            </div>

            <!-- Texto Puro TXT -->
            <div class="format-card">
              <div class="format-card-top">
                <div class="format-icon icon-stone">📝</div>
                <div>
                  <h4 class="format-title">Texto Puro + Dossiê</h4>
                  <span class="format-ext">.txt</span>
                </div>
              </div>
              <p class="format-desc">Texto cru sem formatação e arquivo de dossiê das fichas para backup.</p>
              <button
                type="button"
                onclick={() => handleQuickExport('txt')}
                disabled={compilerStore.isExporting}
                class="btn-format btn-format-stone"
              >
                {compilerStore.isExporting && selectedExportFormat === 'txt' ? 'Exportando...' : 'Salvar Texto Puro'}
              </button>
            </div>
          </div>

          <!-- Feedback de Sucesso -->
          {#if lastExportedPath}
            <div class="export-success-box">
              <div class="success-top">
                <span class="success-icon">✓</span>
                <div>
                  <strong>Exportação Concluída com Sucesso!</strong>
                  <p class="success-path">{lastExportedPath}</p>
                </div>
              </div>
              <button
                type="button"
                onclick={copyPathToClipboard}
                class="btn-copy-path"
              >
                {copiedFeedback ? '✓ Caminho Copiado' : '📋 Copiar Caminho'}
              </button>
            </div>
          {/if}

          <!-- Caminho Manual Avançado -->
          <div class="advanced-path-section">
            <button
              type="button"
              onclick={() => (showAdvancedPath = !showAdvancedPath)}
              class="btn-toggle-advanced"
            >
              <span>{showAdvancedPath ? '▼' : '▶'}</span>
              <span>Opções Avançadas: Especificar caminho manual no disco</span>
            </button>

            {#if showAdvancedPath}
              <div class="advanced-path-box">
                <label for="export-dest" class="matter-label">
                  Caminho Completo do Arquivo ou Diretório de Destino:
                </label>
                <div class="advanced-input-row">
                  <input
                    id="export-dest"
                    type="text"
                    bind:value={exportPathInput}
                    placeholder="/Users/seu-usuario/Documents/meu_livro.docx"
                    class="matter-input mono-font flex-1"
                  />
                  <button
                    type="button"
                    onclick={handleExport}
                    disabled={compilerStore.isExporting || !exportPathInput.trim()}
                    class="btn-primary"
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
    <div class="compiler-footer">
      <span>Compilador Editorial Offline • Magrathea Writer</span>
      <button
        type="button"
        onclick={() => compilerStore.closeModal()}
        class="btn-secondary"
      >
        Fechar
      </button>
    </div>
  </div>
{/if}

<style>
  .compiler-backdrop {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal, 1000);
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
  }

  .compiler-drawer {
    position: fixed;
    inset: 0;
    margin: auto;
    z-index: calc(var(--z-modal, 1000) + 1);
    width: 900px;
    max-width: 96vw;
    height: 92vh;
    background: var(--color-paper-card, #FFFFFF);
    color: var(--color-ink-charcoal, #111827);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 12px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  :global(.dark) .compiler-drawer {
    background: #1F221D;
    color: #FFFFFF;
    border-color: #383C32;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.65);
  }

  .compiler-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: var(--color-paper-subtle, #F9FAFB);
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
  }

  :global(.dark) .compiler-header {
    background: #252822;
    border-color: #383C32;
  }

  .header-info {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .header-icon {
    font-size: 1.5rem;
    line-height: 1;
  }

  .header-title {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 700;
    color: inherit;
  }

  .header-stats {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: var(--color-ink-muted, #6B7280);
    margin-top: 2px;
  }

  :global(.dark) .header-stats {
    color: #A4A89E;
  }

  .close-button {
    width: 32px;
    height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    font-size: 0.95rem;
    color: var(--color-ink-muted, #6B7280);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .close-button:hover {
    color: var(--color-ink-charcoal, #111827);
    background: var(--color-paper-hover, rgba(0, 0, 0, 0.05));
    border-color: var(--color-ink-faded, #9CA3AF);
  }

  :global(.dark) .close-button {
    border-color: #383C32;
    color: #A4A89E;
  }

  :global(.dark) .close-button:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #FFFFFF;
    border-color: #555A4E;
  }

  .compiler-tabs {
    display: flex;
    background: var(--color-paper-subtle, #F3F4F6);
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    padding: 0 16px;
    overflow-x: auto;
    gap: 4px;
  }

  :global(.dark) .compiler-tabs {
    background: #1A1C18;
    border-color: #383C32;
  }

  .tab-btn {
    padding: 10px 14px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-ink-muted, #6B7280);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  :global(.dark) .tab-btn {
    color: #A4A89E;
  }

  .tab-btn.active {
    color: var(--color-olive, #5E6F45);
    border-bottom-color: var(--color-olive, #5E6F45);
    font-weight: 700;
  }

  :global(.dark) .tab-btn.active {
    color: #A3BD84;
    border-bottom-color: #A3BD84;
  }

  .compiler-body {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
    color: inherit;
  }

  .section-subtitle {
    margin: 4px 0 0;
    font-size: 0.8rem;
    color: var(--color-ink-muted, #6B7280);
    line-height: 1.4;
  }

  :global(.dark) .section-subtitle {
    color: #A4A89E;
  }

  /* Matter Container */
  .matter-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .matter-top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .btn-save-matter {
    background: var(--color-olive, #5E6F45);
    color: #FFFFFF;
    border: none;
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  }

  .btn-save-matter:hover {
    background: #4A5836;
  }

  :global(.dark) .btn-save-matter {
    background: #5E6F45;
  }

  .matter-card {
    background: var(--color-paper-subtle, #F9FAFB);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 10px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  :global(.dark) .matter-card {
    background: #252822;
    border-color: #383C32;
  }

  .matter-card-header {
    display: flex;
    align-items: center;
    gap: 8px;
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    padding-bottom: 8px;
  }

  :global(.dark) .matter-card-header {
    border-color: #383C32;
  }

  .card-tag {
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 8px;
    border-radius: 4px;
    background: rgba(94, 111, 69, 0.12);
    color: var(--color-olive, #5E6F45);
  }

  .card-tag.tag-pre {
    background: rgba(194, 141, 88, 0.15);
    color: var(--color-bronze, #C28D58);
  }

  .card-tag.tag-post {
    background: rgba(37, 99, 235, 0.12);
    color: #2563EB;
  }

  :global(.dark) .card-tag {
    color: #A3BD84;
  }

  :global(.dark) .card-tag.tag-pre {
    color: #D4B97F;
  }

  :global(.dark) .card-tag.tag-post {
    color: #93C5FD;
  }

  .card-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .matter-two-cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  @media (max-width: 600px) {
    .matter-two-cols {
      grid-template-columns: 1fr;
    }
  }

  .matter-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .matter-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-ink-charcoal, #111827);
  }

  :global(.dark) .matter-label {
    color: #F3F4F6;
  }

  .matter-hint {
    font-size: 0.72rem;
    color: var(--color-ink-muted, #6B7280);
  }

  :global(.dark) .matter-hint {
    color: #A4A89E;
  }

  .matter-input {
    width: 100%;
    box-sizing: border-box;
    padding: 8px 12px;
    background: var(--color-paper-card, #FFFFFF);
    color: var(--color-ink-charcoal, #111827);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    font-size: 0.85rem;
    outline: none;
    transition: border-color 0.15s ease;
  }

  :global(.dark) .matter-input {
    background: #1B1D19;
    color: #FFFFFF;
    border-color: #383C32;
  }

  .matter-input:focus,
  .matter-textarea:focus {
    border-color: var(--color-olive, #5E6F45);
  }

  .matter-textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 10px 12px;
    background: var(--color-paper-card, #FFFFFF);
    color: var(--color-ink-charcoal, #111827);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    font-size: 0.9rem;
    line-height: 1.5;
    outline: none;
    resize: vertical;
    transition: border-color 0.15s ease;
  }

  :global(.dark) .matter-textarea {
    background: #1B1D19;
    color: #FFFFFF;
    border-color: #383C32;
  }

  .literary-prose {
    font-family: var(--font-literary, Georgia, serif);
  }

  .epigraph-source-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }

  .inline-label {
    white-space: nowrap;
    margin: 0;
  }

  .inline-input {
    flex: 1;
  }

  /* Preview */
  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .preview-actions {
    display: flex;
    gap: 8px;
  }

  .preview-book-sheet {
    background: var(--color-paper-base, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    padding: 24px;
    font-family: var(--font-literary, Georgia, serif);
    font-size: 0.95rem;
    line-height: 1.7;
    white-space: pre-wrap;
    max-height: 60vh;
    overflow-y: auto;
  }

  :global(.dark) .preview-book-sheet {
    background: #1B1D19;
    border-color: #383C32;
  }

  /* Design */
  .design-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 680px;
  }

  .design-card {
    background: var(--color-paper-subtle, #F9FAFB);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  :global(.dark) .design-card {
    background: #252822;
    border-color: #383C32;
  }

  .preset-choices {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .preset-option {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    cursor: pointer;
    background: var(--color-paper-card, #FFFFFF);
    transition: all 0.15s ease;
  }

  :global(.dark) .preset-option {
    background: #1B1D19;
    border-color: #383C32;
  }

  .preset-option.selected {
    border-color: var(--color-olive, #5E6F45);
    background: rgba(94, 111, 69, 0.05);
  }

  .preset-title {
    font-size: 0.85rem;
    font-weight: 600;
  }

  .preset-desc {
    font-size: 0.75rem;
    color: var(--color-ink-muted, #6B7280);
  }

  .sep-input {
    max-width: 200px;
  }

  .checkbox-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    cursor: pointer;
  }

  /* Export */
  .export-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .export-top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .export-badge-pill {
    padding: 4px 10px;
    border-radius: 20px;
    font-size: 0.75rem;
    background: rgba(94, 111, 69, 0.1);
    color: var(--color-olive, #5E6F45);
    font-weight: 600;
  }

  :global(.dark) .export-badge-pill {
    color: #A3BD84;
  }

  .export-options-box {
    background: var(--color-paper-subtle, #F9FAFB);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  :global(.dark) .export-options-box {
    background: #252822;
    border-color: #383C32;
  }

  .box-title {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .box-title h4 {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 700;
  }

  .box-content-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  @media (max-width: 600px) {
    .box-content-grid {
      grid-template-columns: 1fr;
    }
  }

  .option-column {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .option-col-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-ink-muted, #6B7280);
  }

  .radio-pill-group {
    display: flex;
    gap: 6px;
  }

  .radio-pill {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--color-paper-card, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 6px;
    font-size: 0.78rem;
    cursor: pointer;
  }

  :global(.dark) .radio-pill {
    background: #1B1D19;
    border-color: #383C32;
  }

  .radio-pill.active {
    border-color: var(--color-olive, #5E6F45);
    background: rgba(94, 111, 69, 0.08);
    color: var(--color-olive, #5E6F45);
    font-weight: 600;
  }

  .formats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 12px;
  }

  .format-card {
    background: var(--color-paper-card, #FFFFFF);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  :global(.dark) .format-card {
    background: #252822;
    border-color: #383C32;
  }

  .format-card-top {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .format-icon {
    font-size: 1.25rem;
    padding: 4px 8px;
    border-radius: 6px;
  }

  .icon-blue { background: rgba(37, 99, 235, 0.1); color: #2563EB; }
  .icon-emerald { background: rgba(16, 185, 129, 0.1); color: #10B981; }
  .icon-rose { background: rgba(225, 29, 72, 0.1); color: #E11D48; }
  .icon-purple { background: rgba(147, 51, 234, 0.1); color: #9333EA; }
  .icon-stone { background: rgba(107, 114, 128, 0.1); color: #6B7280; }

  .format-title {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 700;
  }

  .format-ext {
    font-size: 0.72rem;
    color: var(--color-ink-muted, #6B7280);
  }

  .format-desc {
    margin: 0;
    font-size: 0.78rem;
    color: var(--color-ink-muted, #6B7280);
    line-height: 1.4;
    flex: 1;
  }

  .btn-format {
    padding: 7px 12px;
    border: none;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: center;
  }

  .btn-format-blue { background: #2563EB; color: #FFFFFF; }
  .btn-format-emerald { background: #059669; color: #FFFFFF; }
  .btn-format-rose { background: #E11D48; color: #FFFFFF; }
  .btn-format-purple { background: #7C3AED; color: #FFFFFF; }
  .btn-format-stone { background: #4B5563; color: #FFFFFF; }

  .export-success-box {
    background: rgba(16, 185, 129, 0.1);
    border: 1px solid #10B981;
    border-radius: 8px;
    padding: 12px 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .success-top {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .success-icon {
    font-size: 1.25rem;
    color: #10B981;
    font-weight: 700;
  }

  .success-path {
    margin: 2px 0 0;
    font-size: 0.75rem;
    font-family: monospace;
    color: var(--color-ink-muted, #6B7280);
  }

  .btn-copy-path {
    background: transparent;
    border: 1px solid #10B981;
    color: #059669;
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
  }

  .advanced-path-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .btn-toggle-advanced {
    background: none;
    border: none;
    padding: 0;
    font-size: 0.75rem;
    color: var(--color-ink-muted, #6B7280);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .advanced-path-box {
    background: var(--color-paper-subtle, #F9FAFB);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 8px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  :global(.dark) .advanced-path-box {
    background: #252822;
    border-color: #383C32;
  }

  .advanced-input-row {
    display: flex;
    gap: 8px;
  }

  .mono-font {
    font-family: monospace;
    font-size: 0.78rem;
  }

  .flex-1 {
    flex: 1;
  }

  .btn-primary {
    background: var(--color-olive, #5E6F45);
    color: #FFFFFF;
    border: none;
    padding: 6px 14px;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-secondary {
    background: transparent;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    color: inherit;
    padding: 6px 14px;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
  }

  :global(.dark) .btn-secondary {
    border-color: #383C32;
  }

  .compiler-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 20px;
    border-top: 1px solid var(--color-paper-border, #E5E0D8);
    background: var(--color-paper-subtle, #F9FAFB);
    font-size: 0.75rem;
    color: var(--color-ink-muted, #6B7280);
  }

  :global(.dark) .compiler-footer {
    background: #252822;
    border-color: #383C32;
  }

  .loading-state {
    text-align: center;
    padding: 40px;
    color: var(--color-ink-muted, #6B7280);
  }

  .loading-spinner {
    font-size: 2rem;
    margin-bottom: 8px;
  }
</style>
