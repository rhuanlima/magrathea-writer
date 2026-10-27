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
    saving: string;
  };
  vault: {
    status: {
      noVaultOpen: string;
      vaultOpen: string;
      newVaultButton: string;
      lockButton: string;
    };
    create: {
      modalTitle: string;
      step1Title: string;
      step2Title: string;
      projectTitleLabel: string;
      projectTitlePlaceholder: string;
      vaultDirLabel: string;
      vaultDirPlaceholder: string;
      passphraseLabel: string;
      passphrasePlaceholder: string;
      passphraseConfirmLabel: string;
      passphraseConfirmPlaceholder: string;
      passwordsDoNotMatch: string;
      strengthEmpty: string;
      strengthVeryWeak: string;
      strengthWeak: string;
      strengthFair: string;
      strengthGood: string;
      strengthStrong: string;
      minRequirementsNote: string;
      continueButton: string;
      creatingVault: string;
      recoveryWarning: string;
      copyKeyButton: string;
      copiedTooltip: string;
      confirmSavedLabel: string;
      openVaultButton: string;
    };
    unlock: {
      screenTitle: string;
      lockedSubtitle: string;
      passphraseLabel: string;
      passphrasePlaceholder: string;
      unlockButton: string;
      unlocking: string;
      useRecoveryKey: string;
      usePassphrase: string;
      recoveryKeyLabel: string;
      recoveryKeyPlaceholder: string;
      invalidPassphraseError: string;
      invalidRecoveryKeyError: string;
      closeVaultButton: string;
      autoLockNotice: string;
      unlockWithKeyringButton: string;
      unlockingWithKeyring: string;
    };
    security: {
      modalTitle: string;
      tabChangePassphrase: string;
      tabReencrypt: string;
      tabKeyring: string;
      currentPassphraseLabel: string;
      currentPassphrasePlaceholder: string;
      newPassphraseLabel: string;
      newPassphrasePlaceholder: string;
      confirmNewPassphraseLabel: string;
      confirmNewPassphrasePlaceholder: string;
      changePassphraseButton: string;
      changingPassphrase: string;
      passphraseChangedSuccess: string;
      reencryptTitle: string;
      reencryptExplanation: string;
      reencryptWarning: string;
      reencryptButton: string;
      reencrypting: string;
      reencryptSuccessTitle: string;
      reencryptNewKeyNote: string;
      keyringTitle: string;
      keyringDescription: string;
      keyringWarningLimitation: string;
      enableKeyringLabel: string;
      keyringPassphrasePrompt: string;
      keyringEnabledSuccess: string;
      keyringDisabledSuccess: string;
    };
  };
  common: {
    languageName: string;
    cancel: string;
    confirm: string;
    save: string;
    close: string;
  };
  sidebar: {
    toggle: string;
  };
  manuscript: {
    treeTitle: string;
    newPart: string;
    newChapter: string;
    newScene: string;
    emptyTree: string;
    trash: string;
    emptyTrash: string;
    restore: string;
    noTrashItems: string;
    confirmEmptyTrash: string;
    rename: string;
    delete: string;
    tags: string;
    tagPlaceholder: string;
    moveUp: string;
    moveDown: string;
    status: {
      draft: string;
      revised: string;
      final: string;
    };
  };
  literary: {
    emDash: string;
    quotes: string;
    guillemets: string;
    ellipsis: string;
    sceneBreak: string;
  };
  inspector: {
    title: string;
    close: string;
    defaultKind: string;
    noActiveItem: string;
    selectItemToInspect: string;
    tabMetadata: string;
    tabBacklinks: string;
    singleMention: string;
    multipleMentions: string;
    noBacklinksFound: string;
    backlinksHint: string;
  };
  metadata: {
    status: string;
    tags: string;
    noTags: string;
    slug: string;
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
