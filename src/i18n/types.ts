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
    focusModeToggle: string;
    zenModeToggle: string;
    exitZen: string;
  };
  vault: {
    status: {
      noVaultOpen: string;
      vaultOpen: string;
      newVaultButton: string;
      openVaultButton: string;
      lockButton: string;
    };
    open: {
      modalTitle: string;
      openVaultButton: string;
      vaultPathLabel: string;
      vaultPathPlaceholder: string;
      recentVaultsLabel: string;
      noRecentVaults: string;
      passphraseLabel: string;
      passphrasePlaceholder: string;
      unlockButton: string;
      useRecoveryKey: string;
      usePassphrase: string;
      recoveryKeyLabel: string;
      recoveryKeyPlaceholder: string;
      keyringButton: string;
      openError: string;
      welcomeTitle: string;
      welcomeSubtitle: string;
      welcomeOpenButton: string;
      welcomeCreateButton: string;
      recentSectionTitle: string;
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
    saving: string;
    close: string;
    add: string;
    delete: string;
  };
  theme: {
    toggle: string;
    light: string;
    dark: string;
    system: string;
  };
  hud: {
    title: string;
    toggle: string;
    minimize: string;
    expand: string;
    sceneWords: string;
    sessionWords: string;
    sessionTime: string;
    dailyGoal: string;
  };
  productivity: {
    dashboardTitle: string;
    goalsButton: string;
    projectTarget: string;
    todayGoal: string;
    sevenDaysAvg: string;
    estimatedFinish: string;
    completed: string;
    wordsPerDay: string;
    daysRemaining: string;
    wordsRemaining: string;
    goalReached: string;
    last7DaysChart: string;
    configureTargets: string;
    projectTargetWords: string;
    dailyTargetWords: string;
    deadlineOptional: string;
    saveTargetsButton: string;
    savedSuccess: string;
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
    tabSnapshots: string;
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
  refactor: {
    modalTitle: string;
    explanation: string;
    currentNameLabel: string;
    newNameLabel: string;
    newNamePlaceholder: string;
    previewTitle: string;
    checking: string;
    occurrencesFound: string;
    noOccurrencesFound: string;
    mentionsLabel: string;
    applyButton: string;
    applying: string;
    applySuccess: string;
    undoButton: string;
    rollingBack: string;
    rollbackSuccess: string;
    refactorAction: string;
  };
  worldbuilding: {
    tabTitle: string;
    tabAppearances: string;
    tabGallery: string;
    searchPlaceholder: string;
    newEntity: string;
    filterAll: string;
    noEntitiesFound: string;
    createFirstEntity: string;
    createModalTitle: string;
    selectType: string;
    entityTitle: string;
    entityTitlePlaceholder: string;
    tagsLabel: string;
    tagsPlaceholder: string;
    createButton: string;
    errorTitleRequired: string;
    appearancesTitle: string;
    noAppearancesYet: string;
    appearancesHint: string;
    openScene: string;
    citedAs: string;
    galleryTitle: string;
    uploadImage: string;
    noImagesYet: string;
    imagesFormatHint: string;
    imageTooLarge: string;
    confirmDeleteImage: string;
    types: {
      character: string;
      characterDesc: string;
      location: string;
      locationDesc: string;
      item: string;
      itemDesc: string;
      faction: string;
      factionDesc: string;
      lore: string;
      loreDesc: string;
    };
  };
  snapshots: {
    tabTitle: string;
    createSnapshotButton: string;
    newSnapshotPrompt: string;
    labelPlaceholder: string;
    saveSnapshot: string;
    noSnapshotsYet: string;
    diffTitle: string;
    compareWithCurrent: string;
    restoreButton: string;
    confirmRestore: string;
    manualTag: string;
    autoTag: string;
    wordsCount: string;
    closeDiff: string;
  };
  search: {
    modalTitle: string;
    searchTab: string;
    replaceTab: string;
    queryPlaceholder: string;
    replacementPlaceholder: string;
    matchCaseLabel: string;
    searchButton: string;
    previewButton: string;
    replaceAllButton: string;
    undoButton: string;
    noResults: string;
    resultsCount: string;
    occurrencesCount: string;
    linePrefix: string;
  };
  backup: {
    modalTitle: string;
    destinationLabel: string;
    destinationPlaceholder: string;
    rotationLabel: string;
    runBackupButton: string;
    sameDiskWarning: string;
    sameDiskTip: string;
    recentBackupsTitle: string;
    noBackupsYet: string;
    backupSuccess: string;
  };
  compiler: {
    modalTitle: string;
    tabPreview: string;
    tabMatter: string;
    tabDesign: string;
    tabExport: string;
    recompileButton: string;
    compiling: string;
    saving: string;
    exporting: string;
    saveSettings: string;
    statsWords: string;
    statsChars: string;
    statsSections: string;
    titleLabel: string;
    subtitleLabel: string;
    authorLabel: string;
    dedicationLabel: string;
    dedicationPlaceholder: string;
    epigraphLabel: string;
    epigraphPlaceholder: string;
    epigraphSourceLabel: string;
    forewordLabel: string;
    afterwordLabel: string;
    acknowledgmentsLabel: string;
    referencesLabel: string;
    referencesPlaceholder: string;
    aboutAuthorLabel: string;
    presetLabel: string;
    presetNovel: string;
    presetEssay: string;
    presetPoetry: string;
    sceneSeparatorLabel: string;
    includeSceneTitlesLabel: string;
    pageBreakAfterChapterLabel: string;
    exportObsidianTitle: string;
    exportObsidianDesc: string;
    exportObsidianButton: string;
    exportDocxTitle: string;
    exportDocxDesc: string;
    exportDocxButton: string;
    exportEpubTitle: string;
    exportEpubDesc: string;
    exportEpubButton: string;
    exportPdfTitle: string;
    exportPdfDesc: string;
    exportPdfButton: string;
    exportTxtTitle: string;
    exportTxtDesc: string;
    exportTxtButton: string;
    exportSuccess: string;
    exportError: string;
  };
  timeline: {
    modalTitle: string;
    bookOrderTab: string;
    chronologicalTab: string;
    totalEvents: string;
    datedEvents: string;
    flashbacksCount: string;
    flashforwardsCount: string;
    allCharacters: string;
    allLocations: string;
    searchPlaceholder: string;
    togglePacing: string;
    pacingPanelTitle: string;
    densityLabel: string;
    wordsLabel: string;
    scenesLabel: string;
    goToSceneButton: string;
    noEventsFound: string;
    flashbackBadge: string;
    flashforwardBadge: string;
    realDateBadge: string;
    relativeTimeBadge: string;
    undatedBadge: string;
  };
  graph: {
    modalTitle: string;
    tabGraph: string;
    tabCanvas: string;
    tabLocalGraph: string;
    filterAll: string;
    filterScenes: string;
    filterCharacters: string;
    filterLocations: string;
    filterItems: string;
    filterFactions: string;
    filterLore: string;
    toggleOrphans: string;
    showOrphans: string;
    hideOrphans: string;
    searchPlaceholder: string;
    statsNodes: string;
    statsEdges: string;
    statsOrphans: string;
    localGraphDepth: string;
    centerOnNode: string;
    openInEditor: string;
    inspectNode: string;
  };
  canvas: {
    addPostIt: string;
    addEntityCard: string;
    saveCanvas: string;
    saved: string;
    zoomIn: string;
    zoomOut: string;
    fitView: string;
    emptyCanvasHint: string;
    deleteNode: string;
    viewDetails: string;
    selectEntityPlaceholder: string;
  };
  commands: {
    title: string;
    placeholder: string;
    empty: string;
    navigateHint: string;
    selectHint: string;
    closeHint: string;
    categoryAction: string;
    categoryNavigation: string;
    openPalette: string;
  };
  support: {
    modalTitle: string;
    subtitle: string;
    description: string;
    callout: string;
    humor42: string;
    qrAlt: string;
    pixKeyLabel: string;
    copyPixButton: string;
    copied: string;
    cryptoLabel: string;
    offlineNotice: string;
    closeButton: string;
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
