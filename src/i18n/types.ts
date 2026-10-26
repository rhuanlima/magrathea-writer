/**
 * Supported application locales.
 */
export type SupportedLocale = 'en-US' | 'pt-BR';

/**
 * Fallback locale when a key or language is unavailable.
 */
export const DEFAULT_LOCALE: SupportedLocale = 'en-US';

/**
 * Recursive schema type for translation messages.
 */
export interface TranslationSchema {
  app: {
    title: string;
    tagline: string;
    switchLanguage: string;
  };
  editor: {
    defaultTitle: string;
    welcomeMessage: string;
    wordsCount: string;
    charactersCount: string;
    unsavedChanges: string;
    saved: string;
  };
  common: {
    languageName: string;
    cancel: string;
    confirm: string;
    save: string;
    close: string;
  };
}

/**
 * Dot-notation paths helper for type-safe translation keys.
 */
export type NestedKeyOf<ObjectType extends object> = {
  [Key in keyof ObjectType & (string | number)]: ObjectType[Key] extends object
    ? `${Key}.${NestedKeyOf<ObjectType[Key]>}`
    : `${Key}`;
}[keyof ObjectType & (string | number)];

export type TranslationKey = NestedKeyOf<TranslationSchema>;
