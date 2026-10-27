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
  vault: {
    status: {
      noVaultOpen: 'No vault open',
      vaultOpen: 'Vault: {title}',
      newVaultButton: 'New Vault',
      lockButton: 'Lock Vault',
    },
    create: {
      modalTitle: 'New Literary Vault',
      step1Title: '1. Project Info & Master Passphrase',
      step2Title: '2. Emergency Recovery Key',
      projectTitleLabel: 'Work / Novel Title',
      projectTitlePlaceholder: 'E.g., The Hitchhiker’s Guide to the Galaxy',
      vaultDirLabel: 'Storage Directory (.magrathea)',
      vaultDirPlaceholder: '/path/to/your/manuscript',
      passphraseLabel: 'Master Passphrase',
      passphrasePlaceholder: 'Enter a long and secure master passphrase...',
      passphraseConfirmLabel: 'Confirm Master Passphrase',
      passphraseConfirmPlaceholder: 'Repeat master passphrase...',
      passwordsDoNotMatch: 'Passphrases do not match.',
      strengthEmpty: 'Please enter a master passphrase',
      strengthVeryWeak: 'Very weak (insecure)',
      strengthWeak: 'Weak (below recommendation)',
      strengthFair: 'Fair (minimum acceptable)',
      strengthGood: 'Good (good entropy)',
      strengthStrong: 'Strong (excellent protection)',
      minRequirementsNote:
        'Master passphrase must have at least 10 characters and fair complexity to protect your work.',
      continueButton: 'Create Vault & Generate Key',
      creatingVault: 'Encrypting and initializing vault...',
      recoveryWarning:
        'This key will only be shown on this screen. Magrathea Writer is 100% offline and has no servers to recover your work if you forget your password.',
      copyKeyButton: 'Copy Emergency Key',
      copiedTooltip: 'Copied to clipboard!',
      confirmSavedLabel:
        'I have copied and saved my recovery key in a secure location.',
      openVaultButton: 'Open Literary Studio',
    },
  },
  common: {
    languageName: 'English',
    cancel: 'Cancel',
    confirm: 'Confirm',
    save: 'Save',
    close: 'Close',
  },
};
