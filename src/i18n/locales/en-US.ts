import type { TranslationSchema } from '../types';

export const enUS: TranslationSchema = {
  app: {
    title: 'Magrathea Writer',
    tagline: 'Your private workshop for building worlds.',
    switchLanguage: 'Switch to Portuguese',
  },
  editor: {
    defaultTitle: 'Untitled Manuscript',
    welcomeMessage: 'Welcome to your distraction-free literary workshop.',
    wordsCount: '{count} words',
    charactersCount: '{count} characters',
    unsavedChanges: 'Unsaved changes',
    saved: 'All changes saved to vault',
  },
  common: {
    languageName: 'English',
    cancel: 'Cancel',
    confirm: 'Confirm',
    save: 'Save',
    close: 'Close',
  },
};
