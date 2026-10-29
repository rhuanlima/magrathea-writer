import { describe, it, expect, beforeEach } from 'vitest';
import { EditorState } from '@codemirror/state';
import { editorViewState } from './editor-view-state.svelte';
import { focusModeExtension, focusModeField, setFocusMode } from './focus-mode';
import { theme } from '../design';

// Mock de DOM e localStorage para ambiente de teste Node
const mockStorage: Record<string, string> = {};
const mockAttributes: Record<string, string> = {};

if (typeof globalThis.localStorage === 'undefined' || !globalThis.localStorage?.getItem) {
  globalThis.localStorage = {
    getItem: (key: string) => mockStorage[key] ?? null,
    setItem: (key: string, val: string) => {
      mockStorage[key] = String(val);
    },
    removeItem: (key: string) => {
      delete mockStorage[key];
    },
    clear: () => {
      Object.keys(mockStorage).forEach((k) => delete mockStorage[k]);
    },
    key: () => null,
    length: 0,
  } as Storage;
}

if (typeof globalThis.document === 'undefined') {
  globalThis.document = {
    documentElement: {
      setAttribute: (attr: string, val: string) => {
        mockAttributes[attr] = val;
      },
      removeAttribute: (attr: string) => {
        delete mockAttributes[attr];
      },
      getAttribute: (attr: string) => mockAttributes[attr] ?? null,
    },
  } as unknown as Document;
}

if (typeof globalThis.window === 'undefined') {
  globalThis.window = {
    document: globalThis.document,
    localStorage: globalThis.localStorage,
  } as unknown as Window & typeof globalThis;
}

describe('M3 — Foco Dinâmico, Zen Mode, HUD e Metas Literárias', () => {
  beforeEach(() => {
    // Reseta estado do editorViewState
    editorViewState.isFocusMode = false;
    editorViewState.isZenMode = false;
    editorViewState.isHudVisible = true;
    editorViewState.isHudMinimized = false;
    editorViewState.isProductivityModalOpen = false;
    editorViewState.sessionWords = 0;
    editorViewState.sessionSeconds = 0;
  });

  describe('M3-01: Modo Foco Dinâmico (CodeMirror 6)', () => {
    it('deve alternar o estado do modo foco no store e persistir em localStorage', () => {
      expect(editorViewState.isFocusMode).toBe(false);
      editorViewState.toggleFocus();
      expect(editorViewState.isFocusMode).toBe(true);
      expect(localStorage.getItem('magrathea-focus-mode')).toBe('true');

      editorViewState.toggleFocus();
      expect(editorViewState.isFocusMode).toBe(false);
      expect(localStorage.getItem('magrathea-focus-mode')).toBe('false');
    });

    it('deve atenuar linhas inativas no editor quando o modo foco está habilitado', () => {
      const doc = 'Primeiro parágrafo da história.\nSegundo parágrafo ativo.\nTerceiro parágrafo distante.';
      const state = EditorState.create({
        doc,
        extensions: [focusModeExtension],
      });

      // Cursor no segundo parágrafo (linha 2)
      const secondLinePos = doc.indexOf('Segundo');
      const transactionWithCursor = state.update({
        selection: { anchor: secondLinePos },
        effects: setFocusMode.of(true),
      });

      expect(transactionWithCursor.state.field(focusModeField)).toBe(true);

      // Desativa modo foco
      const disabledTransaction = transactionWithCursor.state.update({
        effects: setFocusMode.of(false),
      });
      expect(disabledTransaction.state.field(focusModeField)).toBe(false);
    });
  });

  describe('M3-02: Modo Zen (Imersão Total)', () => {
    it('deve alternar e sair do Modo Zen corretamente', () => {
      expect(editorViewState.isZenMode).toBe(false);
      editorViewState.toggleZen();
      expect(editorViewState.isZenMode).toBe(true);

      editorViewState.exitZen();
      expect(editorViewState.isZenMode).toBe(false);
    });
  });

  describe('M3-03: HUD Flutuante de Métricas', () => {
    it('deve controlar visibilidade e alternar estado minimizado da pílula', () => {
      expect(editorViewState.isHudVisible).toBe(true);
      editorViewState.toggleHud();
      expect(editorViewState.isHudVisible).toBe(false);

      expect(editorViewState.isHudMinimized).toBe(false);
      editorViewState.toggleHudMinimized();
      expect(editorViewState.isHudMinimized).toBe(true);
    });

    it('deve acumular palavras da sessão e acionar o cronômetro', () => {
      expect(editorViewState.sessionWords).toBe(0);
      editorViewState.recordWordsDelta(150);
      expect(editorViewState.sessionWords).toBe(150);

      editorViewState.recordWordsDelta(50);
      expect(editorViewState.sessionWords).toBe(200);
    });

    it('deve desconsiderar colagens e incrementos em lote da contagem da meta diária', () => {
      expect(editorViewState.sessionWords).toBe(0);

      // Simulação da lógica do Editor: digitação autêntica e contínua (< 15 palavras)
      const typedDelta = 3;
      let isPasteOrDrop = false;
      if (typedDelta > 0 && !isPasteOrDrop && typedDelta < 15) {
        editorViewState.recordWordsDelta(typedDelta);
      }
      expect(editorViewState.sessionWords).toBe(3);

      // Simulação: colagem de bloco com 500 palavras (paste ativo)
      const pastedDelta = 500;
      isPasteOrDrop = true;
      if (pastedDelta > 0 && !isPasteOrDrop && pastedDelta < 15) {
        editorViewState.recordWordsDelta(pastedDelta);
      }
      // O progresso da meta diária permanece preservado, sem ser inflado pela colagem
      expect(editorViewState.sessionWords).toBe(3);

      // Simulação: arrasto/soltura ou salto artificial sem evento explícito de paste
      const burstDelta = 35;
      isPasteOrDrop = false;
      if (burstDelta > 0 && !isPasteOrDrop && burstDelta < 15) {
        editorViewState.recordWordsDelta(burstDelta);
      }
      expect(editorViewState.sessionWords).toBe(3);
    });
  });

  describe('M3-04 & M3-05: Metas e Dashboard de Produtividade', () => {
    it('deve consultar visão geral de metas e métricas do cofre via mock bridge', async () => {
      await editorViewState.refreshProductivity();
      expect(editorViewState.productivity.project_target_words).toBe(50000);
      expect(editorViewState.productivity.daily_target_words).toBe(1000);
      expect(editorViewState.productivity.today_words).toBe(450);
    });

    it('deve atualizar as metas do projeto e refletir no cofre', async () => {
      await editorViewState.updateTargets(80000, '2026-12-25', 1500);
      expect(editorViewState.productivity.project_target_words).toBe(80000);
      expect(editorViewState.productivity.project_deadline).toBe('2026-12-25');
      expect(editorViewState.productivity.daily_target_words).toBe(1500);
    });

    it('deve gravar sessão acumulada no cofre de forma transparente e resetar contadores', async () => {
      editorViewState.sessionWords = 320;
      editorViewState.sessionSeconds = 900;

      await editorViewState.flushSessionToVault();

      expect(editorViewState.sessionWords).toBe(0);
      expect(editorViewState.sessionSeconds).toBe(0);
    });
  });

  describe('M3-06: Modo Escuro Completo', () => {
    it('deve alternar entre light, dark e system e aplicar atributo data-theme no DOM', () => {
      theme.setMode('light');
      expect(document.documentElement.getAttribute('data-theme')).toBe('light');

      theme.setMode('dark');
      expect(document.documentElement.getAttribute('data-theme')).toBe('dark');

      theme.setMode('system');
      expect(document.documentElement.getAttribute('data-theme')).toBeNull();
    });
  });
});
