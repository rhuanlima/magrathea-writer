import { enUS } from './locales/en-US';
import { ptBR } from './locales/pt-BR';
import type { SupportedLocale, TranslationKey, TranslationSchema } from './types';
import { DEFAULT_LOCALE } from './types';

const catalogues: Record<SupportedLocale, TranslationSchema> = {
  'en-US': enUS,
  'pt-BR': ptBR,
};

/**
 * Resolves a nested dot-notation key from a translation object.
 */
function resolvePath(obj: unknown, path: string): string | undefined {
  const segments = path.split('.');
  let current: unknown = obj;

  for (const segment of segments) {
    if (current && typeof current === 'object' && segment in current) {
      current = (current as Record<string, unknown>)[segment];
    } else {
      return undefined;
    }
  }

  return typeof current === 'string' ? current : undefined;
}

/**
 * Detects the system/browser locale, falling back to DEFAULT_LOCALE.
 */
function detectSystemLocale(): SupportedLocale {
  if (typeof navigator !== 'undefined' && navigator.language) {
    const lang = navigator.language.toLowerCase();
    if (lang.startsWith('pt')) {
      return 'pt-BR';
    }
  }
  return DEFAULT_LOCALE;
}

/**
 * Reactive i18n store using Svelte 5 runes.
 */
class I18nManager {
  private _locale = $state<SupportedLocale>(detectSystemLocale());

  get locale(): SupportedLocale {
    return this._locale;
  }

  setLocale(locale: SupportedLocale): void {
    if (locale in catalogues) {
      this._locale = locale;
    }
  }

  t(key: TranslationKey, params?: Record<string, string | number>): string {
    const currentCatalogue = catalogues[this._locale];
    let template = resolvePath(currentCatalogue, key);

    // Fallback to default locale if missing in current catalogue
    if (template === undefined && this._locale !== DEFAULT_LOCALE) {
      template = resolvePath(catalogues[DEFAULT_LOCALE], key);
    }

    if (template === undefined) {
      return key;
    }

    if (!params) {
      return template;
    }

    return template.replace(/\{(\w+)\}/g, (_, placeholder) => {
      const value = params[placeholder];
      return value !== undefined ? String(value) : `{${placeholder}}`;
    });
  }
}

export const i18n = new I18nManager();
export const t = (key: TranslationKey, params?: Record<string, string | number>) =>
  i18n.t(key, params);

export type { SupportedLocale, TranslationKey, TranslationSchema } from './types';
export { DEFAULT_LOCALE } from './types';
