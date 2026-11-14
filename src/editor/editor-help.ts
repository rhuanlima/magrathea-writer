/**
 * Conteúdo da ajuda rápida do editor (atalhos, sintaxe e dicas).
 *
 * Os atalhos usam `Mod` para a tecla de comando (⌘ no macOS, Ctrl nos demais) e refletem os
 * registrados em `richTextKeymap` e nos ouvintes globais de `Editor.svelte` e `App.svelte`.
 */

import type { TranslationSchema } from '../i18n/types';

export type HelpTextId = Exclude<keyof TranslationSchema['editorHelp'], `section_${string}` | 'title' | 'open'>;

export interface HelpRow {
  /** Identificador usado na chave de tradução `editorHelp.<id>`. */
  id: HelpTextId;
  /** Atalho de teclado no formato `Mod+Shift+X`. */
  keys?: string;
  /** Texto Markdown que o autor digita para obter o efeito. */
  syntax?: string;
}

export interface HelpSection {
  id: 'shortcuts' | 'syntax' | 'navigation';
  rows: HelpRow[];
}

export const HELP_SECTIONS: HelpSection[] = [
  {
    id: 'shortcuts',
    rows: [
      { id: 'bold', keys: 'Mod+B' },
      { id: 'italic', keys: 'Mod+I' },
      { id: 'underline', keys: 'Mod+U' },
      { id: 'strikethrough', keys: 'Mod+Shift+X' },
      { id: 'heading1', keys: 'Mod+Alt+1' },
      { id: 'heading2', keys: 'Mod+Alt+2' },
      { id: 'heading3', keys: 'Mod+Alt+3' },
      { id: 'bulletList', keys: 'Mod+Shift+8' },
      { id: 'blockquote', keys: 'Mod+Shift+9' },
    ],
  },
  {
    id: 'syntax',
    rows: [
      { id: 'syntaxBold', syntax: '**texto**' },
      { id: 'syntaxItalic', syntax: '*texto*' },
      { id: 'syntaxStrike', syntax: '~~texto~~' },
      { id: 'syntaxUnderline', syntax: '<u>texto</u>' },
      { id: 'syntaxHeading', syntax: '# · ## · ###' },
      { id: 'syntaxList', syntax: '* item' },
      { id: 'syntaxQuote', syntax: '> citação' },
      { id: 'syntaxDialogue', syntax: '- Olá' },
      { id: 'syntaxSceneBreak', syntax: '***' },
      { id: 'syntaxWikilink', syntax: '[[Nome]]' },
    ],
  },
  {
    id: 'navigation',
    rows: [
      { id: 'commandPalette', keys: 'Mod+K' },
      { id: 'focusMode', keys: 'Mod+Alt+F' },
      { id: 'goals', keys: 'Mod+Shift+M' },
      { id: 'exportBook', keys: 'Mod+Shift+E' },
      { id: 'timeline', keys: 'Mod+Shift+T' },
      { id: 'graph', keys: 'Mod+Shift+G' },
      { id: 'preferences', keys: 'Mod+,' },
      { id: 'lockWorld', keys: 'Mod+L' },
      { id: 'shortcutHelp', keys: 'Mod+/' },
    ],
  },
];

export function isMacPlatform(): boolean {
  if (typeof navigator === 'undefined') return false;
  return /Mac|iPhone|iPad/i.test(navigator.platform || navigator.userAgent || '');
}

const MAC_SYMBOLS: Record<string, string> = { Mod: '⌘', Shift: '⇧', Alt: '⌥' };

/** Converte `Mod+Shift+X` em partes de teclas prontas para exibição na plataforma indicada. */
export function formatShortcut(keys: string, mac: boolean = isMacPlatform()): string[] {
  return keys.split('+').map((part) => {
    if (part === 'Mod') return mac ? MAC_SYMBOLS.Mod : 'Ctrl';
    return mac ? (MAC_SYMBOLS[part] ?? part) : part;
  });
}
