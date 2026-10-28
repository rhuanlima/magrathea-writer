export type ThemeMode = 'light' | 'dark' | 'system';

class ThemeManager {
  private _mode = $state<ThemeMode>('system');
  private _systemPrefersDark = $state(false);

  constructor() {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem('magrathea-theme') as ThemeMode | null;
      if (stored === 'light' || stored === 'dark' || stored === 'system') {
        this._mode = stored;
      }

      if (window.matchMedia) {
        const query = window.matchMedia('(prefers-color-scheme: dark)');
        this._systemPrefersDark = query.matches;
        query.addEventListener('change', (e) => {
          this._systemPrefersDark = e.matches;
        });
      }

      this.applyTheme();
    }
  }

  get mode(): ThemeMode {
    return this._mode;
  }

  get isDark(): boolean {
    if (this._mode === 'dark') return true;
    if (this._mode === 'light') return false;
    return this._systemPrefersDark;
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
