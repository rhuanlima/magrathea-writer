import { describe, it, expect, beforeEach } from 'vitest';
import { createCompilerStore } from './compiler.svelte';

describe('Compiler Store and Multi-format Export (M6)', () => {
  let store: ReturnType<typeof createCompilerStore>;

  beforeEach(() => {
    store = createCompilerStore();
  });

  it('deve inicializar com modal fechado e aba de preview', () => {
    expect(store.isOpen).toBe(false);
    expect(store.activeTab).toBe('preview');
    expect(store.isCompiling).toBe(false);
  });

  it('deve abrir o modal, carregar configurações e compilar', async () => {
    await store.openModal('matter');
    expect(store.isOpen).toBe(true);
    expect(store.activeTab).toBe('matter');

    expect(store.settings.dedication).toBeDefined();
    expect(store.compiled).not.toBeNull();
    expect(store.compiled?.sections.length).toBeGreaterThan(0);
  });

  it('deve permitir atualizar e salvar seções pré/pós-textuais como dedicatória e referências', async () => {
    await store.openModal('matter');

    store.settings.dedication = 'Dedicado aos exploradores do desconhecido.';
    store.settings.references = '1. Cartografia Estelar de Magrathea.\n2. Guia do Mochileiro.';
    store.settings.epigraph = 'O universo é grande, muito grande.';
    store.settings.epigraph_source = 'Pensador Cósmico';

    await store.saveSettingsNow();
    await store.compile();

    expect(store.compiled?.full_text).toContain('Dedicatória');
    expect(store.compiled?.full_text).toContain('Dedicado aos exploradores do desconhecido.');
    expect(store.compiled?.full_text).toContain('Referências e Leituras');
  });

  it('deve alternar abas de navegação corretamente', () => {
    store.setActiveTab('design');
    expect(store.activeTab).toBe('design');

    store.setActiveTab('export');
    expect(store.activeTab).toBe('export');

    store.closeModal();
    expect(store.isOpen).toBe(false);
  });

  it('deve executar exportação em Markdown Aberto (Obsidian)', async () => {
    await store.openModal('export');
    const success = await store.runExportObsidian('/tmp/export_obsidian');
    expect(success).toBe(true);
    expect(store.exportResult).toBeDefined();
    if (store.exportResult && 'destination_dir' in store.exportResult) {
      expect(store.exportResult.destination_dir).toBeDefined();
      expect(store.exportResult.files_count).toBeGreaterThan(0);
    }
  });

  it('deve executar exportação em DOCX', async () => {
    await store.openModal('export');
    const success = await store.runExportDocx('/tmp/manuscrito.docx');
    expect(success).toBe(true);
    expect(store.exportResult).toBeDefined();
    if (store.exportResult && 'format' in store.exportResult) {
      expect(store.exportResult.format).toBe('docx');
      expect(store.exportResult.byte_size).toBeGreaterThan(0);
    }
  });

  it('deve executar exportação em EPUB', async () => {
    await store.openModal('export');
    const success = await store.runExportEpub('/tmp/manuscrito.epub');
    expect(success).toBe(true);
    expect(store.exportResult).toBeDefined();
    if (store.exportResult && 'format' in store.exportResult) {
      expect(store.exportResult.format).toBe('epub');
    }
  });

  it('deve executar exportação em PDF/HTML', async () => {
    await store.openModal('export');
    const success = await store.runExportPdf('/tmp/manuscrito.html');
    expect(success).toBe(true);
    expect(store.exportResult).toBeDefined();
    if (store.exportResult && 'format' in store.exportResult) {
      expect(store.exportResult.format).toBe('html_pdf');
    }
  });

  it('deve executar exportação em TXT + Dossiê', async () => {
    await store.openModal('export');
    const success = await store.runExportTxt('/tmp/export_txt');
    expect(success).toBe(true);
    expect(store.exportResult).toBeDefined();
    if (store.exportResult && 'format' in store.exportResult) {
      expect(store.exportResult.format).toBe('txt');
    }
  });

  it('deve suportar opções de escopo (tudo vs final) e inclusão de fichas', async () => {
    await store.openExportWithOptions();
    expect(store.isOpen).toBe(true);
    expect(store.activeTab).toBe('export');

    store.settings.only_final_scenes = true;
    store.settings.include_worldbuilding_dossier = true;
    await store.saveSettingsNow();
    await store.compile();

    expect(store.settings.only_final_scenes).toBe(true);
    expect(store.settings.include_worldbuilding_dossier).toBe(true);

    await store.openMatterTab();
    expect(store.activeTab).toBe('matter');
  });
});
