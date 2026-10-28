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
    closeModal,
    setActiveTab,
    loadSettings,
    saveSettingsNow,
    compile,
    runExportObsidian,
    runExportDocx,
    runExportEpub,
    runExportPdf,
    runExportTxt,
  };
}

export const compilerStore = createCompilerStore();
