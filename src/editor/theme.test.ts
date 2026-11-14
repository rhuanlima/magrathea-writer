/**
 * @vitest-environment jsdom
 */
import { describe, it, expect } from 'vitest';
import { EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { getEditorThemeExtension } from './theme';

describe('Editor Theme & Highlighting Inversion', () => {
  it('retorna extensões de tema para modo claro', () => {
    const ext = getEditorThemeExtension(false);
    expect(ext).toBeDefined();
    expect(Array.isArray(ext)).toBe(true);
  });

  it('retorna extensões de tema para modo escuro com contraste em branco', () => {
    const ext = getEditorThemeExtension(true);
    expect(ext).toBeDefined();
    expect(Array.isArray(ext)).toBe(true);
  });

  it.each([false, true])(
    'aplica a fonte escolhida ao .cm-scroller, que o CodeMirror define como monospace (escuro=%s)',
    (isDark) => {
      const view = new EditorView({
        state: EditorState.create({ doc: 'Texto', extensions: [getEditorThemeExtension(isDark)] }),
        parent: document.createElement('div'),
      });
      const css = Array.from(document.querySelectorAll('style'))
        .map((el) => el.textContent ?? '')
        .join('\n');
      view.destroy();

      const scrollerRules = [...css.matchAll(/\.cm-scroller\s*\{([^}]*)\}/g)].map((m) => m[1]);
      const themed = scrollerRules.filter((body) => body.includes('--editor-font-family'));
      expect(themed.length).toBeGreaterThan(0);
    },
  );
});
