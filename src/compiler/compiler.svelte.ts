/**
 * Store reativa do Compilador de Manuscrito e Motores de Exportação (M6).
 *
 * Utiliza Svelte 5 Runes ($state, $derived) para gerenciamento de estado desacoplado,
 * permitindo edição de dedicatória, epígrafe, referências, seleção de temas e exportação
 * em Obsidian Markdown, DOCX, EPUB, PDF e TXT.
 */

import {
  type CompilerSettings,
  type CompiledManuscript,
  type ExportDocumentResult,
  type ExportMarkdownResult,
  loadCompilerSettings,
  saveCompilerSettings,
  compileManuscript,
  exportObsidianMarkdown,
  exportDocx,
  exportEpub,
  exportPdfHtml,
  exportTxtDossier,
  pickSaveExportPath,
  pickDirectory,
} from '../tauri/bridge';

export type CompilerTab = 'preview' | 'matter' | 'design' | 'export';

export function createCompilerStore() {
  let isOpen = $state<boolean>(false);
  let activeTab = $state<CompilerTab>('preview');
  let isCompiling = $state<boolean>(false);
  let isSaving = $state<boolean>(false);
  let isExporting = $state<boolean>(false);

  let settings = $state<CompilerSettings>({
    title: null,
    subtitle: null,
    author: null,
    dedication: null,
    epigraph: null,
    epigraph_source: null,
    foreword: null,
    afterword: null,
    acknowledgments: null,
    references: null,
    about_author: null,
    include_scene_titles: false,
    scene_separator: '* * *',
    page_break_after_chapter: true,
    theme_preset: 'novel',
    only_final_scenes: false,
    include_worldbuilding_dossier: false,
  });

  let compiled = $state<CompiledManuscript | null>(null);
  let exportResult = $state<ExportDocumentResult | ExportMarkdownResult | null>(null);
  let exportError = $state<string | null>(null);

  async function openModal(initialTab: CompilerTab = 'preview') {
    isOpen = true;
    activeTab = initialTab;
    exportError = null;
    exportResult = null;
    await loadSettings();
    await compile();
  }

  async function openMatterTab() {
    await openModal('matter');
  }

  async function openExportWithOptions() {
    await openModal('export');
  }

  function closeModal() {
    isOpen = false;
  }

  function setActiveTab(tab: CompilerTab) {
    activeTab = tab;
  }

  async function loadSettings() {
    try {
      const loaded = await loadCompilerSettings();
      if (loaded) {
        settings = { ...loaded };
      }
    } catch (err) {
      console.error('Falha ao carregar configurações do compilador:', err);
    }
  }

  async function saveSettingsNow() {
    isSaving = true;
    try {
      await saveCompilerSettings(settings);
    } catch (err) {
      console.error('Falha ao salvar configurações do compilador:', err);
    } finally {
      isSaving = false;
    }
  }

  async function compile() {
    isCompiling = true;
    exportError = null;
    try {
      const res = await compileManuscript(settings);
      compiled = res;
    } catch (err) {
      exportError = err instanceof Error ? err.message : String(err);
      console.error('Falha ao compilar manuscrito:', err);
    } finally {
      isCompiling = false;
    }
  }

  async function runExportObsidian(destinationDir: string): Promise<boolean> {
    isExporting = true;
    exportError = null;
    exportResult = null;
    try {
      const res = await exportObsidianMarkdown(destinationDir);
      exportResult = res;
      return true;
    } catch (err) {
      exportError = err instanceof Error ? err.message : String(err);
      return false;
    } finally {
      isExporting = false;
    }
  }

  async function runExportDocx(destinationFile: string): Promise<boolean> {
    isExporting = true;
    exportError = null;
    exportResult = null;
    try {
      const res = await exportDocx(destinationFile, settings);
      exportResult = res;
      return true;
    } catch (err) {
      exportError = err instanceof Error ? err.message : String(err);
      return false;
    } finally {
      isExporting = false;
    }
  }

  async function runExportEpub(destinationFile: string): Promise<boolean> {
    isExporting = true;
    exportError = null;
    exportResult = null;
    try {
      const res = await exportEpub(destinationFile, settings);
      exportResult = res;
      return true;
    } catch (err) {
      exportError = err instanceof Error ? err.message : String(err);
      return false;
    } finally {
      isExporting = false;
    }
  }

  async function runExportPdf(destinationFile: string): Promise<boolean> {
    isExporting = true;
    exportError = null;
    exportResult = null;
    try {
      const res = await exportPdfHtml(destinationFile, settings);
      exportResult = res;
      return true;
    } catch (err) {
      exportError = err instanceof Error ? err.message : String(err);
      return false;
    } finally {
      isExporting = false;
    }
  }

  async function runExportTxt(destinationDir: string): Promise<boolean> {
    isExporting = true;
    exportError = null;
    exportResult = null;
    try {
      const res = await exportTxtDossier(destinationDir, settings);
      exportResult = res;
      return true;
    } catch (err) {
      exportError = err instanceof Error ? err.message : String(err);
      return false;
    } finally {
      isExporting = false;
    }
  }

  let toastMessage = $state<string | null>(null);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  function showToast(msg: string) {
    toastMessage = msg;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastMessage = null;
    }, 4000);
  }

  async function quickExportDirect(format: 'docx' | 'epub' | 'pdf' | 'obsidian' | 'txt'): Promise<string | null> {
    await loadSettings();
    const raw = (settings.title || 'manuscrito')
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9_-]/gi, '_') || 'livro';
    let chosen: string | null = null;

    if (format === 'docx') {
      chosen = await pickSaveExportPath('Salvar Manuscrito em Word (.docx)', `${raw}.docx`, 'docx');
      if (chosen) {
        const ok = await runExportDocx(chosen);
        if (ok) {
          showToast(`✓ Manuscrito exportado para Word: ${chosen}`);
          return chosen;
        }
      }
    } else if (format === 'epub') {
      chosen = await pickSaveExportPath('Salvar Livro Digital EPUB (.epub)', `${raw}.epub`, 'epub');
      if (chosen) {
        const ok = await runExportEpub(chosen);
        if (ok) {
          showToast(`✓ Livro Digital exportado: ${chosen}`);
          return chosen;
        }
      }
    } else if (format === 'pdf') {
      chosen = await pickSaveExportPath('Salvar Livro em PDF (.pdf)', `${raw}.pdf`, 'pdf');
      if (chosen) {
        const ok = await runExportPdf(chosen);
        if (ok) {
          showToast(`✓ Livro em PDF gerado: ${chosen}`);
          return chosen;
        }
      }
    } else if (format === 'obsidian') {
      chosen = await pickDirectory('Escolha a pasta de destino para o Cofre Obsidian');
      if (chosen) {
        const ok = await runExportObsidian(chosen);
        if (ok) {
          showToast(`✓ Cofre Obsidian criado: ${chosen}`);
          return chosen;
        }
      }
    } else if (format === 'txt') {
      chosen = await pickDirectory('Escolha a pasta para salvar o Manuscrito TXT e Dossiê');
      if (chosen) {
        const ok = await runExportTxt(chosen);
        if (ok) {
          showToast(`✓ Manuscrito em texto puro exportado: ${chosen}`);
          return chosen;
        }
      }
    }
    return null;
  }

  return {
    get isOpen() {
      return isOpen;
    },
    get activeTab() {
      return activeTab;
    },
    get isCompiling() {
      return isCompiling;
    },
    get isSaving() {
      return isSaving;
    },
    get isExporting() {
      return isExporting;
    },
    get toastMessage() {
      return toastMessage;
    },
    get settings() {
      return settings;
    },
    set settings(val: CompilerSettings) {
      settings = val;
    },
    get compiled() {
      return compiled;
    },
    get exportResult() {
      return exportResult;
    },
    get exportError() {
      return exportError;
    },
    openModal,
    openMatterTab,
    openExportWithOptions,
    closeModal,
    setActiveTab,
    loadSettings,
    saveSettingsNow,
    compile,
    quickExportDirect,
    runExportObsidian,
    runExportDocx,
    runExportEpub,
    runExportPdf,
    runExportTxt,
  };
}

export const compilerStore = createCompilerStore();
