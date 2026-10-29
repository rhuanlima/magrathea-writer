import { describe, it, expect, beforeEach } from 'vitest';
import { editorPreferences } from './editor-preferences.svelte';

describe('Editor Preferences Store', () => {
  beforeEach(() => {
    editorPreferences.showLineNumbers = false;
    editorPreferences.fontSize = 18;
    editorPreferences.fontFamily = 'garamond';
    editorPreferences.lineHeight = 1.8;
    editorPreferences.columnWidth = 'medium';
  });

  it('toggles line numbers visibility', () => {
    expect(editorPreferences.showLineNumbers).toBe(false);
    editorPreferences.toggleLineNumbers();
    expect(editorPreferences.showLineNumbers).toBe(true);
    editorPreferences.toggleLineNumbers();
    expect(editorPreferences.showLineNumbers).toBe(false);
  });

  it('clamps font size within reasonable bounds', () => {
    editorPreferences.setFontSize(35);
    expect(editorPreferences.fontSize).toBe(28);

    editorPreferences.setFontSize(10);
    expect(editorPreferences.fontSize).toBe(14);

    editorPreferences.setFontSize(20);
    expect(editorPreferences.fontSize).toBe(20);
  });

  it('updates font family and column width correctly', () => {
    editorPreferences.setFontFamily('mono');
    expect(editorPreferences.fontFamilyCss).toBe('var(--font-mono)');

    editorPreferences.setColumnWidth('wide');
    expect(editorPreferences.columnMaxWidthPx).toBe('860px');

    editorPreferences.setColumnWidth('full');
    expect(editorPreferences.columnMaxWidthPx).toBe('100%');
  });

  it('controls modal open and close state', () => {
    expect(editorPreferences.isModalOpen).toBe(false);
    editorPreferences.openModal();
    expect(editorPreferences.isModalOpen).toBe(true);
    editorPreferences.closeModal();
    expect(editorPreferences.isModalOpen).toBe(false);
  });

  it('permite alternar modo typewriter (sons mecânicos, volume e rolagem centralizada)', () => {
    expect(editorPreferences.typewriterSound).toBe(false);
    editorPreferences.toggleTypewriterSound();
    expect(editorPreferences.typewriterSound).toBe(true);

    editorPreferences.setTypewriterVolume(0.8);
    expect(editorPreferences.typewriterVolume).toBe(0.8);

    expect(editorPreferences.typewriterScrolling).toBe(false);
    editorPreferences.toggleTypewriterScrolling();
    expect(editorPreferences.typewriterScrolling).toBe(true);
  });

  it('permite alternar modo Vim de numeração relativa decrescente', () => {
    expect(editorPreferences.relativeLineNumbers).toBe(false);
    editorPreferences.toggleRelativeLineNumbers();
    expect(editorPreferences.relativeLineNumbers).toBe(true);
    editorPreferences.toggleRelativeLineNumbers();
    expect(editorPreferences.relativeLineNumbers).toBe(false);
  });
});
