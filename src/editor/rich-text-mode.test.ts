/**
 * @vitest-environment jsdom
 */
import { describe, it, expect } from 'vitest';
import { EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import {
  richTextModeExtension,
  richTextModeField,
  setRichTextMode,
  getFrontmatterEndLine,
  buildRichTextDecorations,
  getActiveFormats,
  toggleBold,
  toggleItalic,
  toggleUnderline,
  toggleStrikethrough,
  toggleHeading,
  toggleBulletList,
  toggleBlockquote,
  clearFormatting,
} from './rich-text-mode';

function createTestView(doc: string, anchor = 0, head = anchor): EditorView {
  const parent = document.createElement('div');
  const state = EditorState.create({
    doc,
    selection: { anchor, head },
    extensions: [richTextModeExtension],
  });
  return new EditorView({ state, parent });
}

describe('Rich Text Visual Editor Mode (WYSIWYG ↔ Markdown)', () => {
  it('identifica e oculta o bloco de frontmatter YAML no Modo Visual', () => {
    const doc = `---
title: "Cena 1"
status: "draft"
---

O vento soprava forte.`;
    const state = EditorState.create({
      doc,
      extensions: [richTextModeExtension],
    });

    expect(state.field(richTextModeField)).toBe(true);
    expect(getFrontmatterEndLine(state)).toBe(5);

    const decos = buildRichTextDecorations(state);
    expect(decos.size).toBeGreaterThan(0);

    // Ao alternar para o modo Markdown (false), nenhuma decoração de ocultação é aplicada
    const mdState = state.update({
      effects: setRichTextMode.of(false),
    }).state;
    expect(mdState.field(richTextModeField)).toBe(false);
    expect(buildRichTextDecorations(mdState).size).toBe(0);
  });

  it('aplica e remove negrito (**), itálico (*), sublinhado (<u>) e tachado (~~) na seleção', () => {
    const view = createTestView('A noite estava fria.', 2, 7); // seleciona "noite"

    toggleBold(view);
    expect(view.state.doc.toString()).toBe('A **noite** estava fria.');
    // A seleção permanece sobre a palavra interna "noite" (4..9)
    expect(view.state.selection.main.from).toBe(4);
    expect(view.state.selection.main.to).toBe(9);
    expect(getActiveFormats(view.state).bold).toBe(true);

    // Chamar toggleBold novamente remove os marcadores ** ao redor da seleção
    toggleBold(view);
    expect(view.state.doc.toString()).toBe('A noite estava fria.');
    expect(getActiveFormats(view.state).bold).toBe(false);

    // Testa itálico
    toggleItalic(view);
    expect(view.state.doc.toString()).toBe('A *noite* estava fria.');
    expect(getActiveFormats(view.state).italic).toBe(true);
    toggleItalic(view);
    expect(view.state.doc.toString()).toBe('A noite estava fria.');

    // Testa sublinhado
    toggleUnderline(view);
    expect(view.state.doc.toString()).toBe('A <u>noite</u> estava fria.');
    expect(getActiveFormats(view.state).underline).toBe(true);
    toggleUnderline(view);
    expect(view.state.doc.toString()).toBe('A noite estava fria.');

    // Testa tachado
    toggleStrikethrough(view);
    expect(view.state.doc.toString()).toBe('A ~~noite~~ estava fria.');
    expect(getActiveFormats(view.state).strikethrough).toBe(true);
    toggleStrikethrough(view);
    expect(view.state.doc.toString()).toBe('A noite estava fria.');

    view.destroy();
  });

  it('alterna títulos H1, H2, H3, lista com marcadores e citação na linha ativa', () => {
    const view = createTestView('Capítulo Primeiro', 0);

    toggleHeading(view, 1);
    expect(view.state.doc.toString()).toBe('# Capítulo Primeiro');
    expect(getActiveFormats(view.state).heading).toBe(1);

    // Troca H1 -> H2
    toggleHeading(view, 2);
    expect(view.state.doc.toString()).toBe('## Capítulo Primeiro');
    expect(getActiveFormats(view.state).heading).toBe(2);

    // Clicar em H2 novamente volta para parágrafo normal
    toggleHeading(view, 2);
    expect(view.state.doc.toString()).toBe('Capítulo Primeiro');
    expect(getActiveFormats(view.state).heading).toBe(0);

    // Lista com marcadores
    toggleBulletList(view);
    expect(view.state.doc.toString()).toBe('- Capítulo Primeiro');
    expect(getActiveFormats(view.state).bulletList).toBe(true);

    // Troca lista -> citação
    toggleBlockquote(view);
    expect(view.state.doc.toString()).toBe('> Capítulo Primeiro');
    expect(getActiveFormats(view.state).blockquote).toBe(true);

    view.destroy();
  });

  it('limpa todas as marcações de formatação com clearFormatting', () => {
    const view = createTestView('## Um **grande** *mistério* <u>antigo</u> ~~perdido~~', 0);
    clearFormatting(view);
    expect(view.state.doc.toString()).toBe('Um grande mistério antigo perdido');
    view.destroy();
  });

  describe('negrito e itálico combinados (asteriscos adjacentes)', () => {
    it('Cmd+I em texto em negrito adiciona itálico sem remover o negrito', () => {
      const view = createTestView('**x**', 2, 3);
      toggleItalic(view);
      expect(view.state.doc.toString()).toBe('***x***');
      toggleItalic(view);
      expect(view.state.doc.toString()).toBe('**x**');
      view.destroy();
    });

    it('Cmd+B em texto em itálico adiciona negrito sem remover o itálico', () => {
      const view = createTestView('*x*', 1, 2);
      toggleBold(view);
      expect(view.state.doc.toString()).toBe('***x***');
      toggleBold(view);
      expect(view.state.doc.toString()).toBe('*x*');
      view.destroy();
    });

    it('remove só o negrito de um trecho em negrito+itálico', () => {
      const view = createTestView('***x***', 3, 4);
      toggleBold(view);
      expect(view.state.doc.toString()).toBe('*x*');
      view.destroy();
    });

    it('funciona com o cursor dentro da palavra e com a seleção incluindo os marcadores', () => {
      let view = createTestView('um **grande** dia', 7);
      toggleItalic(view);
      expect(view.state.doc.toString()).toBe('um ***grande*** dia');
      view.destroy();

      view = createTestView('**x**', 0, 5);
      toggleItalic(view);
      expect(view.state.doc.toString()).toBe('***x***');
      view.destroy();
    });

    it('mantém a seleção sobre o texto interno após a troca', () => {
      const view = createTestView('**noite**', 2, 7);
      toggleItalic(view);
      expect(view.state.sliceDoc(view.state.selection.main.from, view.state.selection.main.to)).toBe('noite');
      view.destroy();
    });
  });

  describe('proteção do frontmatter oculto', () => {
    const doc = '---\ntitle: a\n---\n\nTexto do corpo';
    const bodyStart = doc.indexOf('Texto');

    it('Cmd+A e apagar preserva o frontmatter e apaga só o corpo', () => {
      const view = createTestView(doc, 0, doc.length);
      view.dispatch({ changes: { from: 0, to: doc.length }, userEvent: 'delete.selection' });
      expect(view.state.doc.toString()).toBe('---\ntitle: a\n---\n\n');
      view.destroy();
    });

    it('digitar sobre uma seleção total preserva o frontmatter', () => {
      const view = createTestView(doc, 0, doc.length);
      view.dispatch({ changes: { from: 0, to: doc.length, insert: 'Novo' }, userEvent: 'input.type' });
      expect(view.state.doc.toString()).toBe('---\ntitle: a\n---\n\nNovo');
      view.destroy();
    });

    it('Delete (forward) no fim do frontmatter e recortar também são protegidos', () => {
      const view = createTestView(doc, 0, bodyStart + 5);
      view.dispatch({ changes: { from: 0, to: bodyStart + 5 }, userEvent: 'delete.cut' });
      expect(view.state.doc.toString().startsWith('---\ntitle: a\n---')).toBe(true);
      view.destroy();
    });

    it('não interfere em alterações programáticas (troca de cena) nem no modo Markdown', () => {
      const view = createTestView(doc, 0);
      view.dispatch({ changes: { from: 0, to: doc.length, insert: 'Outra cena' } });
      expect(view.state.doc.toString()).toBe('Outra cena');

      const md = createTestView(doc, 0, doc.length);
      md.dispatch({ effects: setRichTextMode.of(false) });
      md.dispatch({ changes: { from: 0, to: doc.length }, userEvent: 'delete.selection' });
      expect(md.state.doc.toString()).toBe('');
      view.destroy();
      md.destroy();
    });

    it('edições normais no corpo continuam funcionando', () => {
      const view = createTestView(doc, bodyStart);
      view.dispatch({ changes: { from: bodyStart, insert: 'X' }, userEvent: 'input.type' });
      expect(view.state.doc.toString()).toBe('---\ntitle: a\n---\n\nXTexto do corpo');
      view.destroy();
    });
  });

  describe('marcadores sob o cursor', () => {
    const hiddenRanges = (state: EditorState): Array<[number, number]> => {
      const out: Array<[number, number]> = [];
      buildRichTextDecorations(state).between(0, state.doc.length, (from, to, deco) => {
        if (to > from && !deco.spec.class && !deco.spec.widget) out.push([from, to]);
      });
      return out;
    };
    const mk = (doc: string, pos: number) =>
      EditorState.create({ doc, selection: { anchor: pos }, extensions: [richTextModeExtension] });

    it('oculta os delimitadores quando o cursor está fora do trecho formatado', () => {
      expect(hiddenRanges(mk('um **grande** dia', 16))).toEqual([[3, 5], [11, 13]]);
    });

    it('revela os delimitadores quando o cursor está dentro ou encostado no trecho', () => {
      expect(hiddenRanges(mk('um **grande** dia', 7))).toEqual([]);
      expect(hiddenRanges(mk('um **grande** dia', 3))).toEqual([]);
      expect(hiddenRanges(mk('um **grande** dia', 12))).toEqual([]);
    });

    it('revela o prefixo de título e a quebra de cena apenas na linha do cursor', () => {
      expect(hiddenRanges(mk('# Título\n\nTexto', 12))).toEqual([[0, 2]]);
      expect(hiddenRanges(mk('# Título\n\nTexto', 4))).toEqual([]);

      const scene = (pos: number) => {
        let widgets = 0;
        buildRichTextDecorations(mk('Antes\n\n***\n\nDepois', pos)).between(0, 99, (_f, _t, d) => {
          if (d.spec.widget) widgets++;
        });
        return widgets;
      };
      expect(scene(0)).toBe(1);
      expect(scene(8)).toBe(0);
    });
  });

  it('títulos de nível 4–6 também são renderizados e trocados pelos botões H1–H3', () => {
    const state = EditorState.create({ doc: 'Texto\n\n#### Subtítulo', selection: { anchor: 0 }, extensions: [richTextModeExtension] });
    let hidden = 0;
    buildRichTextDecorations(state).between(0, state.doc.length, (from, to, deco) => {
      if (to > from && !deco.spec.class && !deco.spec.widget) hidden++;
    });
    expect(hidden).toBe(1);

    const view = createTestView('#### Subtítulo', 0);
    toggleHeading(view, 2);
    expect(view.state.doc.toString()).toBe('## Subtítulo');
    view.destroy();
  });
});
