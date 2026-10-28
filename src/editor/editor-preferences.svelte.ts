/**
 * Preferências do Editor Literário (Ajuste 6).
 *
 * Controla:
 * - Visibilidade dos números de linha (line numbers)
 * - Tamanho da fonte (fontSize em px)
 * - Família tipográfica (EB Garamond, Plus Jakarta Sans, JetBrains Mono)
 * - Altura da linha (lineHeight)
 * - Largura da coluna de escrita (coluna 640px, 720px, 860px ou full)
 */

export type FontFamilyChoice = 'garamond' | 'sans' | 'mono';
export type ColumnWidthChoice = 'narrow' | 'medium' | 'wide' | 'full';

class EditorPreferencesStore {
  showLineNumbers = $state<boolean>(false);
  fontSize = $state<number>(18);
  fontFamily = $state<FontFamilyChoice>('garamond');
  lineHeight = $state<number>(1.8);
  columnWidth = $state<ColumnWidthChoice>('medium');
  isModalOpen = $state<boolean>(false);

  constructor() {
    this.loadFromStorage();
  }

  loadFromStorage(): void {
    if (typeof localStorage === 'undefined') return;
    try {
      const raw = localStorage.getItem('magrathea_editor_preferences');
      if (raw) {
        const parsed = JSON.parse(raw);
        if (typeof parsed.showLineNumbers === 'boolean') this.showLineNumbers = parsed.showLineNumbers;
        if (typeof parsed.fontSize === 'number') this.fontSize = parsed.fontSize;
        if (['garamond', 'sans', 'mono'].includes(parsed.fontFamily)) this.fontFamily = parsed.fontFamily;
        if (typeof parsed.lineHeight === 'number') this.lineHeight = parsed.lineHeight;
        if (['narrow', 'medium', 'wide', 'full'].includes(parsed.columnWidth)) this.columnWidth = parsed.columnWidth;
      }
    } catch {
      // Ignora falhas de parse
    }
  }

  saveToStorage(): void {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(
        'magrathea_editor_preferences',
        JSON.stringify({
          showLineNumbers: this.showLineNumbers,
          fontSize: this.fontSize,
          fontFamily: this.fontFamily,
          lineHeight: this.lineHeight,
          columnWidth: this.columnWidth,
        })
      );
    } catch {
      // Ignora falhas
    }
  }

  toggleLineNumbers(): void {
    this.showLineNumbers = !this.showLineNumbers;
    this.saveToStorage();
  }

  setFontSize(size: number): void {
    this.fontSize = Math.min(Math.max(size, 14), 28);
    this.saveToStorage();
  }

  setFontFamily(family: FontFamilyChoice): void {
    this.fontFamily = family;
    this.saveToStorage();
  }

  setLineHeight(height: number): void {
    this.lineHeight = height;
    this.saveToStorage();
  }

  setColumnWidth(width: ColumnWidthChoice): void {
    this.columnWidth = width;
    this.saveToStorage();
  }

  openModal(): void {
    this.isModalOpen = true;
  }

  closeModal(): void {
    this.isModalOpen = false;
  }

  get columnMaxWidthPx(): string {
    switch (this.columnWidth) {
      case 'narrow':
        return '640px';
      case 'medium':
        return '720px';
      case 'wide':
        return '860px';
      case 'full':
        return '100%';
    }
  }

  get fontFamilyCss(): string {
    switch (this.fontFamily) {
      case 'garamond':
        return 'var(--font-literary)';
      case 'sans':
        return 'var(--font-ui)';
      case 'mono':
        return 'var(--font-mono)';
    }
  }
}

export const editorPreferences = new EditorPreferencesStore();
