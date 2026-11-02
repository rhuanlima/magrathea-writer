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
});
