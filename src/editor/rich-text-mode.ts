/**
 * Extensão de Editor Visual (Rich Text / WYSIWYG) para o CodeMirror 6.
 *
 * Permite ao autor alternar entre:
 * - **Modo Editor Comum (Visual / Rich Text):** oculta o frontmatter YAML e todas as
 *   marcações de sintaxe Markdown (`**`, `*`, `~~`, `<u>`, `#`, `>`, `-`), exibindo o
 *   texto diretamente formatado como em processadores de texto tradicionais (Word/Docs),
 *   mantendo 100% de compatibilidade com o modelo Markdown do cofre cifrado.
 * - **Modo Markdown (Código):** exibe todos os delimitadores Markdown e o bloco de
 *   frontmatter YAML bruto.
 */

import {
  type Extension,
  EditorState,
  type Range,
  StateEffect,
  StateField,
  Transaction,
} from '@codemirror/state';
import {
  EditorView,
  Decoration,
  type DecorationSet,
  WidgetType,
  keymap,
} from '@codemirror/view';

/**
 * Efeito para ativar (`true`) ou desativar (`false`) o Modo Editor Visual (Rich Text).
 */
export const setRichTextMode = StateEffect.define<boolean>();

/**
 * Campo de estado que armazena se o Modo Visual (Rich Text) está ativo.
 */
export const richTextModeField = StateField.define<boolean>({
  create() {
    return true;
  },
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setRichTextMode)) {
        return effect.value;
      }
    }
    return value;
  },
});

/**
 * Estado das formatações ativas na posição atual do cursor/seleção.
 */
export interface ActiveFormats {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  heading: 0 | 1 | 2 | 3;
  blockquote: boolean;
  bulletList: boolean;
}

export const DEFAULT_ACTIVE_FORMATS: ActiveFormats = {
  bold: false,
  italic: false,
  underline: false,
  strikethrough: false,
  heading: 0,
  blockquote: false,
  bulletList: false,
};

class BulletWidget extends WidgetType {
  eq(): boolean {
    return true;
  }

  toDOM(): HTMLElement {
    const span = document.createElement('span');
    span.className = 'cm-rt-bullet-widget';
    span.textContent = '• ';
    span.setAttribute('aria-hidden', 'true');
    return span;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

class SceneBreakWidget extends WidgetType {
  eq(): boolean {
    return true;
  }

  toDOM(): HTMLElement {
    const span = document.createElement('span');
    span.className = 'cm-rt-scene-break-widget';
    span.textContent = '✦   ✦   ✦';
    span.setAttribute('aria-label', 'Quebra de cena');
    return span;
  }

  ignoreEvent(): boolean {
    return false;
  }
}

const frontmatterHiddenLineDeco = Decoration.line({ class: 'cm-rt-frontmatter-hidden' });
const h1LineDeco = Decoration.line({ class: 'cm-rt-h1' });
const h2LineDeco = Decoration.line({ class: 'cm-rt-h2' });
const h3LineDeco = Decoration.line({ class: 'cm-rt-h3' });
const blockquoteLineDeco = Decoration.line({ class: 'cm-rt-blockquote' });
const listItemLineDeco = Decoration.line({ class: 'cm-rt-list-item' });
const sceneBreakLineDeco = Decoration.line({ class: 'cm-rt-scene-break-line' });

const hiddenDelimiterDeco = Decoration.replace({});
const bulletReplaceDeco = Decoration.replace({ widget: new BulletWidget() });
const sceneBreakReplaceDeco = Decoration.replace({ widget: new SceneBreakWidget() });

const boldMarkDeco = Decoration.mark({ class: 'cm-rt-bold' });
const italicMarkDeco = Decoration.mark({ class: 'cm-rt-italic' });
const boldItalicMarkDeco = Decoration.mark({ class: 'cm-rt-bold cm-rt-italic' });
const underlineMarkDeco = Decoration.mark({ class: 'cm-rt-underline' });
const strikethroughMarkDeco = Decoration.mark({ class: 'cm-rt-strikethrough' });
const codeMarkDeco = Decoration.mark({ class: 'cm-rt-code' });

/**
 * Identifica a linha final (1-indexed) do bloco de frontmatter YAML no início do documento.
 * Retorna `0` caso o documento não possua frontmatter válido.
 */
export function getFrontmatterEndLine(state: EditorState): number {
  if (state.doc.lines < 2) return 0;
  const firstLine = state.doc.line(1).text.replace(/^\uFEFF/, '').trim();
  if (firstLine !== '---') return 0;

  for (let i = 2; i <= state.doc.lines; i++) {
    if (state.doc.line(i).text.trim() === '---') {
      // Se houver uma linha em branco logo após o fechamento e mais linhas de corpo,
      // ocultamos também a linha em branco separadora para o texto iniciar no topo.
      if (i + 1 < state.doc.lines && state.doc.line(i + 1).text.trim() === '') {
        return i + 1;
      }
      // Só oculta se existir pelo menos uma linha de corpo após o frontmatter
      if (i < state.doc.lines) {
        return i;
      }
      return 0;
    }
  }
  return 0;
}

interface InlineTokenMatch {
  from: number;
  to: number;
  openLen: number;
  closeLen: number;
  markDeco: Decoration;
}

/**
 * Extrai tokens inline de formatação de uma linha sem sobreposição e ignorando [[wikilinks]].
 */
function findInlineMatches(lineText: string, startOffset: number): InlineTokenMatch[] {
  const occupied = new Uint8Array(lineText.length);

  // Protege [[wikilinks]] para que a decoração de chip bronze funcione livremente
  const wikilinkRegex = /\[\[[^\]\n]+\]\]/g;
  let wlMatch: RegExpExecArray | null;
  while ((wlMatch = wikilinkRegex.exec(lineText)) !== null) {
    for (let i = wlMatch.index; i < wlMatch.index + wlMatch[0].length; i++) {
      occupied[i] = 1;
    }
  }

  const patterns: Array<{
    regex: RegExp;
    openLen: number;
    closeLen: number;
    markDeco: Decoration;
  }> = [
    { regex: /\*\*\*([^*\n]+)\*\*\*/g, openLen: 3, closeLen: 3, markDeco: boldItalicMarkDeco },
    { regex: /\*\*([^*\n]+)\*\*/g, openLen: 2, closeLen: 2, markDeco: boldMarkDeco },
    { regex: /__([^_\n]+)__/g, openLen: 2, closeLen: 2, markDeco: boldMarkDeco },
    { regex: /~~([^~\n]+)~~/g, openLen: 2, closeLen: 2, markDeco: strikethroughMarkDeco },
    { regex: /<u>([^<\n]+)<\/u>/g, openLen: 3, closeLen: 4, markDeco: underlineMarkDeco },
    { regex: /`([^`\n]+)`/g, openLen: 1, closeLen: 1, markDeco: codeMarkDeco },
    { regex: /(?<!\*)\*([^*\n]+)\*(?!\*)/g, openLen: 1, closeLen: 1, markDeco: italicMarkDeco },
    { regex: /(?<![_a-zA-Z0-9])_([^_\n]+)_(?![_a-zA-Z0-9])/g, openLen: 1, closeLen: 1, markDeco: italicMarkDeco },
  ];

  const results: InlineTokenMatch[] = [];

  for (const pat of patterns) {
    let m: RegExpExecArray | null;
    while ((m = pat.regex.exec(lineText)) !== null) {
      const matchStart = m.index;
      const matchEnd = matchStart + m[0].length;
      if (matchStart < startOffset) continue;

      let overlaps = false;
      for (let i = matchStart; i < matchEnd; i++) {
        if (occupied[i]) {
          overlaps = true;
          break;
        }
      }
      if (overlaps) continue;

      for (let i = matchStart; i < matchEnd; i++) {
        occupied[i] = 1;
      }

      results.push({
        from: matchStart,
        to: matchEnd,
        openLen: pat.openLen,
        closeLen: pat.closeLen,
        markDeco: pat.markDeco,
      });
    }
  }

  return results.sort((a, b) => a.from - b.from);
}

/**
 * Constrói o conjunto de decorações visuais para o Modo Editor Comum (Rich Text).
 */
export function buildRichTextDecorations(state: EditorState): DecorationSet {
  const isEnabled = state.field(richTextModeField, false);
  if (!isEnabled) {
    return Decoration.none;
  }

  const decorations: Range<Decoration>[] = [];
  const fmEndLine = getFrontmatterEndLine(state);

  for (let i = 1; i <= state.doc.lines; i++) {
    const line = state.doc.line(i);

    if (i <= fmEndLine) {
      decorations.push(frontmatterHiddenLineDeco.range(line.from));
      continue;
    }

    const text = line.text;
    if (!text) continue;

    // 1. Quebra de cena (*** ou --- isolados na linha, fora do frontmatter)
    if (/^\s*(\*{3,}|-{3,})\s*$/.test(text)) {
      decorations.push(sceneBreakLineDeco.range(line.from));
      decorations.push(sceneBreakReplaceDeco.range(line.from, line.to));
      continue;
    }

    let contentStartOffset = 0;

    // 2. Títulos (#, ##, ###)
    const headingMatch = /^(#{1,3})\s+(.+)$/.exec(text);
    if (headingMatch) {
      const hashes = headingMatch[1];
      const prefixLen = hashes.length + 1;
      const level = hashes.length;
      const lineDeco = level === 1 ? h1LineDeco : level === 2 ? h2LineDeco : h3LineDeco;
      decorations.push(lineDeco.range(line.from));
      decorations.push(hiddenDelimiterDeco.range(line.from, line.from + prefixLen));
      contentStartOffset = prefixLen;
    } else {
      // 3. Citação (> )
      const quoteMatch = /^(>\s+)(.*)$/.exec(text);
      if (quoteMatch) {
        const prefixLen = quoteMatch[1].length;
        decorations.push(blockquoteLineDeco.range(line.from));
        decorations.push(hiddenDelimiterDeco.range(line.from, line.from + prefixLen));
        contentStartOffset = prefixLen;
      } else {
        // 4. Lista com marcadores (- ou * seguidos de espaço)
        const bulletMatch = /^([-*]\s+)(.*)$/.exec(text);
        if (bulletMatch) {
          const prefixLen = bulletMatch[1].length;
          decorations.push(listItemLineDeco.range(line.from));
          decorations.push(bulletReplaceDeco.range(line.from, line.from + prefixLen));
          contentStartOffset = prefixLen;
        } else if (/^\d+\.\s+/.test(text)) {
          decorations.push(listItemLineDeco.range(line.from));
        }
      }
    }

    // 5. Formatações inline dentro da linha
    const inlineMatches = findInlineMatches(text, contentStartOffset);
    for (const token of inlineMatches) {
      const absFrom = line.from + token.from;
      const absTo = line.from + token.to;
      const innerFrom = absFrom + token.openLen;
      const innerTo = absTo - token.closeLen;

      if (innerFrom < innerTo) {
        decorations.push(hiddenDelimiterDeco.range(absFrom, innerFrom));
        decorations.push(token.markDeco.range(innerFrom, innerTo));
        decorations.push(hiddenDelimiterDeco.range(innerTo, absTo));
      }
    }
  }

  return Decoration.set(decorations, true);
}

/**
 * Protege o bloco de frontmatter oculto no Modo Visual contra deleção acidental por Backspace
 * e reposiciona o cursor para o início do corpo do texto caso caia dentro do frontmatter oculto.
 */
const frontmatterVisualGuard = EditorState.transactionFilter.of((tr: Transaction) => {
  const isVisual = tr.startState.field(richTextModeField, false);
  if (!isVisual) return tr;

  const fmEndLine = getFrontmatterEndLine(tr.startState);
  if (fmEndLine === 0 || fmEndLine >= tr.startState.doc.lines) return tr;

  const bodyStartPos = tr.startState.doc.line(fmEndLine + 1).from;

  // Se o usuário pressionar Backspace no início do primeiro parágrafo do corpo, impede apagar o frontmatter
  if (tr.isUserEvent('delete.backward')) {
    let touchesFrontmatter = false;
    tr.changes.iterChangedRanges((fromA) => {
      if (fromA < bodyStartPos) {
        touchesFrontmatter = true;
      }
    });
    if (touchesFrontmatter) {
      return [];
    }
  }

  return tr;
});

/**
 * Detecta quais estilos estão ativos na posição atual do cursor/seleção para destacar a barra de ferramentas.
 */
export function getActiveFormats(state: EditorState): ActiveFormats {
  const { from, to } = state.selection.main;
  const line = state.doc.lineAt(from);
  const text = line.text;

  const headingMatch = /^(#{1,3})\s+/.exec(text);
  const heading = (headingMatch ? headingMatch[1].length : 0) as 0 | 1 | 2 | 3;
  const blockquote = /^>\s+/.test(text);
  const bulletList = /^[-*]\s+/.test(text) && !/^\s*(\*{3,}|-{3,})\s*$/.test(text);

  const relFrom = Math.max(0, from - line.from);
  const relTo = Math.min(text.length, to - line.from);

  const isInside = (regex: RegExp): boolean => {
    let m: RegExpExecArray | null;
    while ((m = regex.exec(text)) !== null) {
      const mStart = m.index;
      const mEnd = mStart + m[0].length;
      if (relFrom >= mStart && relTo <= mEnd) {
        return true;
      }
    }
    return false;
  };

  const boldItalic = isInside(/\*\*\*([^*\n]+)\*\*\*/g);
  const bold =
    boldItalic || isInside(/\*\*([^*\n]+)\*\*/g) || isInside(/__([^_\n]+)__/g);
  const italic =
    boldItalic ||
    isInside(/(?<!\*)\*([^*\n]+)\*(?!\*)/g) ||
    isInside(/(?<![_a-zA-Z0-9])_([^_\n]+)_(?![_a-zA-Z0-9])/g);
  const underline = isInside(/<u>([^<\n]+)<\/u>/g);
  const strikethrough = isInside(/~~([^~\n]+)~~/g);

  return {
    bold,
    italic,
    underline,
    strikethrough,
    heading,
    blockquote,
    bulletList,
  };
}

/**
 * Alterna uma formatação inline simétrica (ex.: `**` para negrito, `*` para itálico, `~~` para tachado).
 * Funciona tanto com texto selecionado quanto quando o cursor está dentro de uma palavra já formatada.
 */
export function toggleSymmetricFormat(view: EditorView, marker: string): boolean {
  const { state, dispatch } = view;
  const { from, to } = state.selection.main;
  const len = marker.length;
  const selected = state.sliceDoc(from, to);

  // Caso 1: A própria seleção começa e termina com o marcador
  if (
    selected.length >= len * 2 &&
    selected.startsWith(marker) &&
    selected.endsWith(marker)
  ) {
    const unwrapped = selected.slice(len, selected.length - len);
    dispatch({
      changes: { from, to, insert: unwrapped },
      selection: { anchor: from, head: from + unwrapped.length },
    });
    view.focus();
    return true;
  }

  // Caso 2: Os caracteres imediatamente ao redor da seleção são o marcador (comum no Modo Visual)
  const before = state.sliceDoc(Math.max(0, from - len), from);
  const after = state.sliceDoc(to, Math.min(state.doc.length, to + len));
  if (before === marker && after === marker) {
    dispatch({
      changes: [
        { from: from - len, to: from, insert: '' },
        { from: to, to: to + len, insert: '' },
      ],
      selection: { anchor: from - len, head: to - len },
    });
    view.focus();
    return true;
  }

  // Caso 3: Cursor sem seleção dentro de um trecho já formatado na linha atual -> remove a formatação
  const line = state.doc.lineAt(from);
  const relPos = from - line.from;
  const escaped = marker.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const spanRegex =
    marker === '*'
      ? /(?<!\*)\*([^*\n]+)\*(?!\*)/g
      : new RegExp(`${escaped}([^${escaped[0]}\\n]+)${escaped}`, 'g');

  let m: RegExpExecArray | null;
  while ((m = spanRegex.exec(line.text)) !== null) {
    const mStart = m.index;
    const mEnd = mStart + m[0].length;
    if (relPos >= mStart && relPos <= mEnd && from === to) {
      const absStart = line.from + mStart;
      const absEnd = line.from + mEnd;
      const innerText = m[1];
      const newCursor = Math.max(absStart, Math.min(absStart + innerText.length, from - len));
      dispatch({
        changes: { from: absStart, to: absEnd, insert: innerText },
        selection: { anchor: newCursor },
      });
      view.focus();
      return true;
    }
  }

  // Caso 4: Aplicar formatação à seleção existente
  if (from !== to) {
    const wrapped = `${marker}${selected}${marker}`;
    dispatch({
      changes: { from, to, insert: wrapped },
      selection: { anchor: from + len, head: to + len },
    });
    view.focus();
    return true;
  }

  // Caso 5: Sem seleção — se o cursor estiver sobre uma palavra, formata a palavra; senão insere os marcadores
  const wordAt = state.wordAt(from);
  if (wordAt) {
    const wordText = state.sliceDoc(wordAt.from, wordAt.to);
    const wrapped = `${marker}${wordText}${marker}`;
    dispatch({
      changes: { from: wordAt.from, to: wordAt.to, insert: wrapped },
      selection: { anchor: from + len },
    });
  } else {
    dispatch({
      changes: { from, insert: `${marker}${marker}` },
      selection: { anchor: from + len },
    });
  }

  view.focus();
  return true;
}

/**
 * Alterna Negrito (`**texto**`).
 */
export function toggleBold(view: EditorView): boolean {
  return toggleSymmetricFormat(view, '**');
}

/**
 * Alterna Itálico (`*texto*`).
 */
export function toggleItalic(view: EditorView): boolean {
  return toggleSymmetricFormat(view, '*');
}

/**
 * Alterna Tachado (`~~texto~~`).
 */
export function toggleStrikethrough(view: EditorView): boolean {
  return toggleSymmetricFormat(view, '~~');
}

/**
 * Alterna Sublinhado (`<u>texto</u>`).
 */
export function toggleUnderline(view: EditorView): boolean {
  const { state, dispatch } = view;
  const { from, to } = state.selection.main;
  const openTag = '<u>';
  const closeTag = '</u>';
  const selected = state.sliceDoc(from, to);

  if (selected.startsWith(openTag) && selected.endsWith(closeTag)) {
    const unwrapped = selected.slice(openTag.length, selected.length - closeTag.length);
    dispatch({
      changes: { from, to, insert: unwrapped },
      selection: { anchor: from, head: from + unwrapped.length },
    });
    view.focus();
    return true;
  }

  const before = state.sliceDoc(Math.max(0, from - openTag.length), from);
  const after = state.sliceDoc(to, Math.min(state.doc.length, to + closeTag.length));
  if (before === openTag && after === closeTag) {
    dispatch({
      changes: [
        { from: from - openTag.length, to: from, insert: '' },
        { from: to, to: to + closeTag.length, insert: '' },
      ],
      selection: { anchor: from - openTag.length, head: to - openTag.length },
    });
    view.focus();
    return true;
  }

  const line = state.doc.lineAt(from);
  const relPos = from - line.from;
  const regex = /<u>([^<\n]+)<\/u>/g;
  let m: RegExpExecArray | null;
  while ((m = regex.exec(line.text)) !== null) {
    const mStart = m.index;
    const mEnd = mStart + m[0].length;
    if (relPos >= mStart && relPos <= mEnd && from === to) {
      const absStart = line.from + mStart;
      const absEnd = line.from + mEnd;
      const innerText = m[1];
      dispatch({
        changes: { from: absStart, to: absEnd, insert: innerText },
        selection: { anchor: Math.max(absStart, from - openTag.length) },
      });
      view.focus();
      return true;
    }
  }

  if (from !== to) {
    dispatch({
      changes: { from, to, insert: `${openTag}${selected}${closeTag}` },
      selection: { anchor: from + openTag.length, head: to + openTag.length },
    });
  } else {
    const wordAt = state.wordAt(from);
    if (wordAt) {
      const wordText = state.sliceDoc(wordAt.from, wordAt.to);
      dispatch({
        changes: { from: wordAt.from, to: wordAt.to, insert: `${openTag}${wordText}${closeTag}` },
        selection: { anchor: from + openTag.length },
      });
    } else {
      dispatch({
        changes: { from, insert: `${openTag}${closeTag}` },
        selection: { anchor: from + openTag.length },
      });
    }
  }

  view.focus();
  return true;
}

/**
 * Alterna Título de nível 1, 2 ou 3 na linha atual (`# `, `## `, `### `).
 */
export function toggleHeading(view: EditorView, level: 1 | 2 | 3): boolean {
  const { state, dispatch } = view;
  const line = state.doc.lineAt(state.selection.main.from);
  const targetPrefix = `${'#'.repeat(level)} `;
  const existingHeading = /^(#{1,3})\s+/.exec(line.text);

  if (existingHeading) {
    const existingLen = existingHeading[0].length;
    if (existingHeading[1].length === level) {
      // Já é deste nível -> volta a ser parágrafo comum
      dispatch({
        changes: { from: line.from, to: line.from + existingLen, insert: '' },
      });
    } else {
      // Troca o nível do título
      dispatch({
        changes: { from: line.from, to: line.from + existingLen, insert: targetPrefix },
      });
    }
  } else {
    // Remove eventual prefixo de lista/citação antes de transformar em título
    const blockPrefix = /^(?:>\s+|[-*]\s+)/.exec(line.text);
    const replaceLen = blockPrefix ? blockPrefix[0].length : 0;
    dispatch({
      changes: { from: line.from, to: line.from + replaceLen, insert: targetPrefix },
    });
  }

  view.focus();
  return true;
}

/**
 * Alterna Lista com Marcadores (`- `) na linha atual.
 */
export function toggleBulletList(view: EditorView): boolean {
  const { state, dispatch } = view;
  const line = state.doc.lineAt(state.selection.main.from);
  const existingBullet = /^([-*]\s+)/.exec(line.text);

  if (existingBullet && !/^\s*(\*{3,}|-{3,})\s*$/.test(line.text)) {
    dispatch({
      changes: { from: line.from, to: line.from + existingBullet[1].length, insert: '' },
    });
  } else {
    const otherPrefix = /^(?:#{1,3}\s+|>\s+)/.exec(line.text);
    const replaceLen = otherPrefix ? otherPrefix[0].length : 0;
    dispatch({
      changes: { from: line.from, to: line.from + replaceLen, insert: '- ' },
    });
  }

  view.focus();
  return true;
}

/**
 * Alterna Citação (`> `) na linha atual.
 */
export function toggleBlockquote(view: EditorView): boolean {
  const { state, dispatch } = view;
  const line = state.doc.lineAt(state.selection.main.from);
  const existingQuote = /^(>\s+)/.exec(line.text);

  if (existingQuote) {
    dispatch({
      changes: { from: line.from, to: line.from + existingQuote[1].length, insert: '' },
    });
  } else {
    const otherPrefix = /^(?:#{1,3}\s+|[-*]\s+)/.exec(line.text);
    const replaceLen = otherPrefix ? otherPrefix[0].length : 0;
    dispatch({
      changes: { from: line.from, to: line.from + replaceLen, insert: '> ' },
    });
  }

  view.focus();
  return true;
}

/**
 * Remove todas as marcações de formatação da seleção atual ou da linha atual.
 */
export function clearFormatting(view: EditorView): boolean {
  const { state, dispatch } = view;
  const { from, to } = state.selection.main;
  const targetRange =
    from !== to ? { from, to } : { from: state.doc.lineAt(from).from, to: state.doc.lineAt(from).to };
  const raw = state.sliceDoc(targetRange.from, targetRange.to);

  const cleaned = raw
    .replace(/^(?:#{1,6}\s+|>\s+|[-*]\s+)/gm, '')
    .replace(/\*\*\*([^*\n]+)\*\*\*/g, '$1')
    .replace(/\*\*([^*\n]+)\*\*/g, '$1')
    .replace(/__([^_\n]+)__/g, '$1')
    .replace(/(?<!\*)\*([^*\n]+)\*(?!\*)/g, '$1')
    .replace(/(?<![_a-zA-Z0-9])_([^_\n]+)_(?![_a-zA-Z0-9])/g, '$1')
    .replace(/~~([^~\n]+)~~/g, '$1')
    .replace(/<u>([^<\n]+)<\/u>/g, '$1')
    .replace(/`([^`\n]+)`/g, '$1');

  if (cleaned !== raw) {
    dispatch({
      changes: { from: targetRange.from, to: targetRange.to, insert: cleaned },
    });
  }

  view.focus();
  return true;
}

/**
 * Atalhos de teclado universais de processador de texto (funcionam no Modo Visual e no Modo Markdown).
 */
export const richTextKeymap: Extension = keymap.of([
  { key: 'Mod-b', run: toggleBold },
  { key: 'Mod-i', run: toggleItalic },
  { key: 'Mod-u', run: toggleUnderline },
  { key: 'Mod-Shift-x', run: toggleStrikethrough },
  { key: 'Mod-Alt-1', run: (v) => toggleHeading(v, 1) },
  { key: 'Mod-Alt-2', run: (v) => toggleHeading(v, 2) },
  { key: 'Mod-Alt-3', run: (v) => toggleHeading(v, 3) },
  { key: 'Mod-Shift-8', run: toggleBulletList },
  { key: 'Mod-Shift-9', run: toggleBlockquote },
]);

/**
 * Extensão completa de Editor Visual (Rich Text / WYSIWYG) para CodeMirror 6.
 */
export const richTextModeExtension: Extension = [
  richTextModeField,
  frontmatterVisualGuard,
  EditorView.decorations.compute(['doc', richTextModeField], (state): DecorationSet => {
    return buildRichTextDecorations(state);
  }),
  EditorView.baseTheme({
    '.cm-rt-frontmatter-hidden': {
      display: 'none !important',
    },
    '.cm-rt-h1': {
      fontSize: '1.65em',
      fontWeight: '600',
      lineHeight: '1.35',
      paddingTop: '0.35em',
      paddingBottom: '0.15em',
    },
    '.cm-rt-h2': {
      fontSize: '1.35em',
      fontWeight: '600',
      lineHeight: '1.4',
      paddingTop: '0.3em',
      paddingBottom: '0.1em',
    },
    '.cm-rt-h3': {
      fontSize: '1.15em',
      fontWeight: '600',
      lineHeight: '1.45',
      paddingTop: '0.2em',
      paddingBottom: '0.05em',
    },
    '.cm-rt-blockquote': {
      borderLeft: '3px solid var(--color-olive, #5E6F45)',
      paddingLeft: '14px !important',
      fontStyle: 'italic',
      opacity: '0.92',
    },
    '.cm-rt-list-item': {
      paddingLeft: '18px !important',
    },
    '.cm-rt-bullet-widget': {
      color: 'var(--color-olive, #5E6F45)',
      fontWeight: '700',
      userSelect: 'none',
    },
    '.cm-rt-scene-break-line': {
      textAlign: 'center',
      paddingTop: '0.6em',
      paddingBottom: '0.6em',
    },
    '.cm-rt-scene-break-widget': {
      display: 'inline-block',
      color: 'var(--color-bronze, #C4A468)',
      letterSpacing: '0.35em',
      fontSize: '0.95em',
      userSelect: 'none',
    },
    '.cm-rt-bold': {
      fontWeight: '700',
    },
    '.cm-rt-italic': {
      fontStyle: 'italic',
    },
    '.cm-rt-underline': {
      textDecoration: 'underline',
      textUnderlineOffset: '3px',
    },
    '.cm-rt-strikethrough': {
      textDecoration: 'line-through',
      opacity: '0.78',
    },
    '.cm-rt-code': {
      fontFamily: 'var(--font-mono, monospace)',
      fontSize: '0.9em',
      padding: '1px 5px',
      borderRadius: '3px',
      backgroundColor: 'rgba(122, 98, 48, 0.12)',
    },
  }),
];
