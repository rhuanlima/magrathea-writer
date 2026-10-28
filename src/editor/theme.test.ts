import { describe, it, expect } from 'vitest';
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
});
