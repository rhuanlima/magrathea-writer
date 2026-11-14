import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { HELP_SECTIONS, formatShortcut } from './editor-help';
import { enUS } from './../i18n/locales/en-US';
import { ptBR } from './../i18n/locales/pt-BR';

describe('Ajuda rápida do editor', () => {
  it('formata atalhos com símbolos no macOS e com nomes nas demais plataformas', () => {
    expect(formatShortcut('Mod+Shift+X', true)).toEqual(['⌘', '⇧', 'X']);
    expect(formatShortcut('Mod+Alt+1', true)).toEqual(['⌘', '⌥', '1']);
    expect(formatShortcut('Mod+Shift+X', false)).toEqual(['Ctrl', 'Shift', 'X']);
    expect(formatShortcut('Mod+,', false)).toEqual(['Ctrl', ',']);
  });

  it('toda linha tem atalho ou sintaxe e tradução nos dois idiomas', () => {
    for (const locale of [enUS, ptBR]) {
      const help = locale.editorHelp as unknown as Record<string, string>;
      for (const section of HELP_SECTIONS) {
        expect(help[`section_${section.id}`], `seção ${section.id}`).toBeTruthy();
        for (const row of section.rows) {
          expect(Boolean(row.keys) !== Boolean(row.syntax), `linha ${row.id}`).toBe(true);
          expect(help[row.id], `tradução de ${row.id}`).toBeTruthy();
        }
      }
    }
  });

  it('não repete atalhos de teclado', () => {
    const keys = HELP_SECTIONS.flatMap((s) => s.rows.map((r) => r.keys)).filter(Boolean);
    expect(new Set(keys).size).toBe(keys.length);
  });

  it('atalhos do menu nativo não colidem com os da formatação (Cmd+I é itálico)', () => {
    const lib = readFileSync('src-tauri/src/lib.rs', 'utf8');
    const accelerators = [...lib.matchAll(/Some\("([^"]+)"\)/g)].map((m) => m[1]);
    expect(accelerators).not.toContain('CmdOrCtrl+I');
    expect(accelerators).not.toContain('CmdOrCtrl+B');
    expect(accelerators).not.toContain('CmdOrCtrl+U');
    // Cmd+Shift+F pertence à busca global; o Modo Zen usa F11
    expect(accelerators.filter((a) => a === 'CmdOrCtrl+Shift+F')).toHaveLength(1);
    expect(accelerators).toContain('F11');
  });

  it('o Editor não tem mais um ouvinte de Cmd+Shift+F para o Modo Zen', () => {
    const editor = readFileSync('src/editor/Editor.svelte', 'utf8');
    expect(editor).not.toMatch(/shiftKey && e\.code === 'KeyF'/);
  });
});
