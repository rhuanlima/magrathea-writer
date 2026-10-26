export type ThemeMode = 'light' | 'dark' | 'system';

class ThemeManager {
  private _mode = $state<ThemeMode>('system');

  constructor() {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem('magrathea-theme') as ThemeMode | null;
      if (stored === 'light' || stored === 'dark' || stored === 'system') {
        this._mode = stored;
      }
      this.applyTheme();
    }
  }

  get mode(): ThemeMode {
    return this._mode;
  }

  setMode(mode: ThemeMode): void {
    this._mode = mode;
    if (typeof window !== 'undefined') {
      localStorage.setItem('magrathea-theme', mode);
      this.applyTheme();
    }
  }

  toggleTheme(): void {
    if (this._mode === 'light') {
      this.setMode('dark');
    } else if (this._mode === 'dark') {
      this.setMode('system');
    } else {
      this.setMode('light');
    }
  }

  private applyTheme(): void {
    if (typeof document === 'undefined') return;

    const root = document.documentElement;
    if (this._mode === 'system') {
      root.removeAttribute('data-theme');
    } else {
      root.setAttribute('data-theme', this._mode);
    }
  }
}

export const theme = new ThemeManager();
