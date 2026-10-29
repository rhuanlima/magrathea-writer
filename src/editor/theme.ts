/**
 * Configuração de temas e Syntax Highlighting para o CodeMirror 6.
 *
 * Garante que no modo escuro a tipografia seja o inverso do modo claro,
 * trabalhando exclusivamente com variações de branco e off-white (#FFFFFF, #FDFCFA, #E5E0D8),
 * eliminando tons azuis escuros em aspas/strings ("""), citações e blocos literais.
 */

import { Compartment, type Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags as t } from '@lezer/highlight';

export const editorThemeCompartment = new Compartment();

/**
 * Destaque de sintaxe específico para o Modo Claro.
 */
const lightHighlightStyle = HighlightStyle.define([
  { tag: t.heading, color: '#1E231D', fontWeight: '600' },
  { tag: [t.string, t.special(t.string), t.docString, t.character, t.attributeValue], color: '#1E231D' },
  { tag: t.quote, color: '#696E64', fontStyle: 'italic' },
  { tag: t.emphasis, color: '#1E231D', fontStyle: 'italic' },
  { tag: t.strong, color: '#1E231D', fontWeight: 'bold' },
  { tag: t.monospace, color: '#1E231D', backgroundColor: '#EFECE1', borderRadius: '3px' },
  { tag: t.link, color: '#5E6F45', textDecoration: 'none' },
  { tag: [t.meta, t.processingInstruction, t.documentMeta], color: '#7A6230' },
  { tag: [t.comment, t.lineComment, t.blockComment], color: '#A4A89E' },
  { tag: [t.punctuation, t.separator, t.bracket], color: '#696E64' },
]);

/**
 * Destaque de sintaxe específico para o Modo Escuro.
 * Utiliza o inverso do modo claro: variações de branco/creme límpido para máxima visibilidade.
 */
const darkHighlightStyle = HighlightStyle.define([
  { tag: t.content, color: '#FFFFFF' },
  { tag: t.heading, color: '#FFFFFF', fontWeight: '600' },
  { tag: [t.string, t.special(t.string), t.docString, t.character, t.attributeValue], color: '#FFFFFF' },
  { tag: t.quote, color: '#E5E0D8', fontStyle: 'italic' },
  { tag: t.emphasis, color: '#FFFFFF', fontStyle: 'italic' },
  { tag: t.strong, color: '#FFFFFF', fontWeight: 'bold' },
  { tag: t.monospace, color: '#FFFFFF', backgroundColor: 'rgba(255, 255, 255, 0.1)', borderRadius: '3px' },
  { tag: t.link, color: '#A3BD84', textDecoration: 'none' },
  { tag: [t.meta, t.processingInstruction, t.documentMeta], color: '#D4B97F' },
  { tag: [t.comment, t.lineComment, t.blockComment], color: '#A4A89E' },
  { tag: [t.punctuation, t.separator, t.bracket], color: '#C4C8BE' },
]);

/**
 * Retorna o conjunto completo de extensões de estilo (EditorView.theme + syntaxHighlighting)
 * para o modo de cor ativo (claro ou escuro).
 */
export function getEditorThemeExtension(isDark: boolean): Extension {
  if (isDark) {
    return [
      EditorView.theme({
        '&': {
          height: '100%',
          fontSize: 'var(--editor-font-size, 18px)',
          lineHeight: 'var(--editor-line-height, 1.8)',
          fontFamily: 'var(--editor-font-family, var(--font-literary, Georgia, serif))',
          backgroundColor: '#1B1D19',
          color: '#FFFFFF',
        },
        '.cm-scroller': {
          overflowX: 'hidden !important',
          overflowY: 'auto',
          boxSizing: 'border-box',
          padding: 'var(--space-lg) var(--space-md)',
        },
        '.cm-content': {
          maxWidth: 'var(--editor-column-max-width, 720px)',
          width: '100%',
          boxSizing: 'border-box',
          margin: '0 auto',
          padding: '0 var(--space-md)',
          color: '#FFFFFF',
          caretColor: '#8AA66A',
          wordBreak: 'break-word',
          overflowWrap: 'break-word',
        },
        '.cm-line': {
          color: '#FFFFFF',
          wordBreak: 'break-word',
          overflowWrap: 'break-word',
        },
        '.cm-cursor': {
          borderLeftColor: '#8AA66A',
          borderLeftWidth: '2px',
        },
        '.cm-selectionBackground, ::selection': {
          backgroundColor: 'rgba(138, 166, 106, 0.3) !important',
        },
        '.cm-gutters': {
          backgroundColor: '#1B1D19',
          borderColor: '#383C32',
          color: '#767A6E',
        },
        '.cm-activeLine': {
          backgroundColor: 'rgba(255, 255, 255, 0.03)',
        },
      }),
      syntaxHighlighting(darkHighlightStyle),
    ];
  }

  return [
    EditorView.theme({
      '&': {
        height: '100%',
        fontSize: 'var(--editor-font-size, 18px)',
        lineHeight: 'var(--editor-line-height, 1.8)',
        fontFamily: 'var(--editor-font-family, var(--font-literary, Georgia, serif))',
        backgroundColor: 'var(--color-paper-base, #FFFFFF)',
        color: 'var(--color-ink-charcoal, #111827)',
      },
      '.cm-scroller': {
        overflowX: 'hidden !important',
        overflowY: 'auto',
        boxSizing: 'border-box',
        padding: 'var(--space-lg) var(--space-md)',
      },
      '.cm-content': {
        maxWidth: 'var(--editor-column-max-width, 720px)',
        width: '100%',
        boxSizing: 'border-box',
        margin: '0 auto',
        padding: '0 var(--space-md)',
        color: 'var(--color-ink-charcoal, #111827)',
        caretColor: 'var(--color-olive, #5E6F45)',
        wordBreak: 'break-word',
        overflowWrap: 'break-word',
      },
      '.cm-line': {
        color: 'var(--color-ink-charcoal, #111827)',
        wordBreak: 'break-word',
        overflowWrap: 'break-word',
      },
      '.cm-cursor': {
        borderLeftColor: 'var(--color-olive, #5E6F45)',
        borderLeftWidth: '2px',
      },
      '.cm-selectionBackground, ::selection': {
        backgroundColor: 'rgba(94, 111, 69, 0.2) !important',
      },
      '.cm-gutters': {
        backgroundColor: 'var(--color-paper-base, #FFFFFF)',
        borderColor: 'var(--color-paper-border, #E5E7EB)',
        color: 'var(--color-ink-faded, #94A3B8)',
      },
      '.cm-activeLine': {
        backgroundColor: 'rgba(94, 111, 69, 0.04)',
      },
    }),
    syntaxHighlighting(lightHighlightStyle),
  ];
}
