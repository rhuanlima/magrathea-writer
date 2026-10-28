/**
 * Extensão do CodeMirror 6 para Modo Foco Dinâmico (Épico 2 / M3-01).
 *
 * Quando ativado, atenua todas as linhas do documento que não contêm o cursor ativo
 * para 35% de opacidade com transição suave, permitindo ao autor foco absoluto na
 * frase ou parágrafo em edição sem distrações visuais periféricas.
 */

import { type Extension, StateEffect, StateField } from '@codemirror/state';
import { EditorView, Decoration, type DecorationSet } from '@codemirror/view';

/**
 * Efeito de transição de estado para ativar ou desativar o modo foco dinâmico.
 */
export const setFocusMode = StateEffect.define<boolean>();

/**
 * Campo de estado que rastreia se o modo foco está habilitado no editor.
 */
export const focusModeField = StateField.define<boolean>({
  create() {
    return false;
  },
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setFocusMode)) {
        return effect.value;
      }
    }
    return value;
  },
});

const dimDecoration = Decoration.line({
  class: 'cm-focus-dim',
});

/**
 * Extensão completa do CodeMirror 6 com decorações de linha atenuadas e tema de opacidade.
 */
export const focusModeExtension: Extension = [
  focusModeField,
  EditorView.decorations.compute(['doc', 'selection', focusModeField], (state): DecorationSet => {
    const isEnabled = state.field(focusModeField, false);
    if (!isEnabled) {
      return Decoration.none;
    }

    const currentPos = state.selection.main.head;
    const currentLine = state.doc.lineAt(currentPos);
    const decorations = [];

    // Itera por todas as linhas do documento e aplica opacidade atenuada às linhas inativas
    for (let i = 1; i <= state.doc.lines; i++) {
      if (i !== currentLine.number) {
        const line = state.doc.line(i);
        decorations.push(dimDecoration.range(line.from));
      }
    }

    return Decoration.set(decorations, true);
  }),
  EditorView.baseTheme({
    '.cm-focus-dim': {
      opacity: '0.35',
      filter: 'grayscale(15%)',
      transition: 'opacity 180ms ease-in-out, filter 180ms ease-in-out',
    },
    '.cm-line:not(.cm-focus-dim)': {
      opacity: '1',
      transition: 'opacity 180ms ease-in-out',
    },
  }),
];
