<script lang="ts">
  /**
   * Modal para Abertura de Cofre Existente.
   *
   * Permite ao autor selecionar ou digitar o caminho do cofre e desbloqueá-lo
   * utilizando a frase-senha mestra, chaveiro do sistema ou chave de recuperação.
   */

  import { t } from '../i18n';
  import { pickVaultPath, safeInvoke } from '../tauri/bridge';
  import { vault } from './vault.svelte';
  import { getRecentVaults, type RecentVault } from './recent-vaults';

  interface Props {
    isOpen: boolean;
    initialPath?: string;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { isOpen, initialPath, onClose, onSuccess }: Props = $props();

  let targetPath = $state('');
  let passphrase = $state('');
  let recoveryKey = $state('');
  let useRecovery = $state(false);
  let localError = $state<string | null>(null);
  let recentList = $state<RecentVault[]>([]);
  let showPassword = $state(false);
  let isTargetPasswordless = $state(false);
  let isCheckingPasswordless = $state(false);

  $effect(() => {
    if (isOpen) {
      recentList = getRecentVaults();
      if (initialPath) {
        targetPath = initialPath;
      } else if (!targetPath && recentList.length > 0) {
        targetPath = recentList[0].path;
      }
      localError = null;
      passphrase = '';
      recoveryKey = '';
    }
  });

  $effect(() => {
    const trimmed = targetPath.trim();
    if (trimmed) {
      isCheckingPasswordless = true;
      safeInvoke<boolean>('is_vault_passwordless', {
        vault_path: trimmed,
      })
        .then((pwless) => {
          isTargetPasswordless = !!pwless;
        })
        .catch(() => {
          isTargetPasswordless = false;
        })
        .finally(() => {
          isCheckingPasswordless = false;
        });
    } else {
      isTargetPasswordless = false;
      isCheckingPasswordless = false;
    }
  });

  function selectRecent(item: RecentVault) {
    targetPath = item.path;
    localError = null;
  }

  async function handleBrowseVault() {
    localError = null;
    try {
      const selected = await pickVaultPath();
      if (!selected) return;
      targetPath = selected;

      // Se o mundo for sem senha, abre imediatamente em 1 clique
      const isPasswordless = await safeInvoke<boolean>('is_vault_passwordless', {
        vault_path: selected,
      }).catch(() => false);

      if (isPasswordless) {
        await vault.openWithPassphrase('', selected);
        onSuccess?.();
        onClose();
      }
    } catch (err) {
      localError = String(err);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }

  async function handleOpenVault(e: SubmitEvent) {
    e.preventDefault();
    localError = null;

    const trimmedPath = targetPath.trim();
    if (!trimmedPath) {
      localError = t('vault.open.vaultPathPlaceholder');
      return;
    }

    try {
      if (useRecovery) {
        if (!recoveryKey.trim()) {
          localError = t('vault.unlock.invalidRecoveryKeyError');
          return;
        }
        await vault.openWithRecoveryKey(recoveryKey.trim(), trimmedPath);
      } else {
        if (isTargetPasswordless) {
          await vault.openWithPassphrase('', trimmedPath);
          onSuccess?.();
          onClose();
          return;
        }

        if (!passphrase) {
          const isPasswordless = await safeInvoke<boolean>('is_vault_passwordless', {
            vault_path: trimmedPath,
          }).catch(() => false);

          if (isPasswordless) {
            await vault.openWithPassphrase('', trimmedPath);
            onSuccess?.();
            onClose();
            return;
          }

          localError = t('vault.unlock.invalidPassphraseError');
          return;
        }
        await vault.openWithPassphrase(passphrase, trimmedPath);
      }

      onSuccess?.();
      onClose();
    } catch {
      localError = useRecovery
        ? t('vault.unlock.invalidRecoveryKeyError')
        : t('vault.open.openError');
    }
  }

  async function handleOpenWithKeyring() {
    localError = null;
    const trimmedPath = targetPath.trim();
    if (!trimmedPath) {
      localError = t('vault.open.vaultPathPlaceholder');
      return;
    }

    try {
      await vault.openWithKeyring(trimmedPath);
      onSuccess?.();
      onClose();
    } catch (err) {
      localError = String(err);
    }
  }

  function toggleMode() {
    useRecovery = !useRecovery;
    localError = null;
    passphrase = '';
    recoveryKey = '';
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div class="modal-backdrop">
    <div
      class="modal-card"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="open-vault-title"
    >
      <header class="modal-header">
        <div class="header-icon-title">
          <span class="header-icon">📂</span>
          <h2 id="open-vault-title" class="modal-title">
            {t('vault.open.modalTitle')}
          </h2>
        </div>
        <button
          type="button"
          class="btn-close"
          onclick={onClose}
          aria-label="Fechar"
        >
          ✕
        </button>
      </header>

      {#if localError}
        <div class="alert-error" role="alert">
          {localError}
        </div>
      {/if}

      <form onsubmit={handleOpenVault} class="vault-form">
        <!-- Caminho da Pasta / Arquivo .magrathea -->
        <div class="form-group">
          <div class="label-row">
            <label for="open-vault-path" class="form-label">
              {t('vault.open.vaultPathLabel')}
            </label>
            <button
              type="button"
              class="browse-inline-btn"
              onclick={handleBrowseVault}
            >
              📁 Procurar no Computador...
            </button>
          </div>
          <div class="input-with-action">
            <input
              id="open-vault-path"
              type="text"
              class="form-input code-input"
              placeholder={t('vault.open.vaultPathPlaceholder')}
              bind:value={targetPath}
              required
              autocomplete="off"
              spellcheck="false"
            />
            <button
              type="button"
              class="btn-input-browse"
              onclick={handleBrowseVault}
              title="Procurar arquivo de Mundo (.magrathea)"
            >
              🔍
            </button>
          </div>
        </div>

        <!-- Lista de Recentes Rápidos -->
        {#if recentList.length > 0}
          <div class="recents-bar">
            <span class="recents-label">{t('vault.open.recentVaultsLabel')}</span>
            <div class="recents-pills">
              {#each recentList as recent}
                <button
                  type="button"
                  class="recent-pill"
                  class:selected={targetPath === recent.path}
                  onclick={() => selectRecent(recent)}
                  title={recent.path}
                >
                  <span class="recent-icon">🔒</span>
                  <span class="recent-title">{recent.title}</span>
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Autenticação (Senha ou Chave) -->
        {#if isTargetPasswordless}
          <div class="passwordless-banner" role="status">
            <span class="passwordless-icon">🔓</span>
            <div class="passwordless-info">
              <strong>Mundo Livre (Sem Senha)</strong>
              <span>Este mundo foi criado sem proteção por senha. Clique abaixo para abri-lo diretamente.</span>
            </div>
          </div>
        {:else if !useRecovery}
          <div class="form-group">
            <div class="label-row">
              <label for="open-passphrase" class="form-label">
                {t('vault.open.passphraseLabel')}
              </label>
              <button
                type="button"
                class="toggle-show-btn"
                onclick={() => (showPassword = !showPassword)}
              >
                {showPassword ? 'Ocultar' : 'Exibir'}
              </button>
            </div>
            <input
              id="open-passphrase"
              type={showPassword ? 'text' : 'password'}
              class="form-input"
              placeholder={t('vault.open.passphrasePlaceholder')}
              bind:value={passphrase}
              required
              autocomplete="current-password"
            />
          </div>
        {:else}
          <div class="form-group">
            <label for="open-recovery-key" class="form-label">
              {t('vault.open.recoveryKeyLabel')}
            </label>
            <input
              id="open-recovery-key"
              type="text"
              class="form-input code-input"
              placeholder={t('vault.open.recoveryKeyPlaceholder')}
              bind:value={recoveryKey}
              required
              autocomplete="off"
              spellcheck="false"
            />
          </div>
        {/if}

        <div class="form-actions">
          <button
            type="submit"
            class="btn-primary"
            disabled={vault.isUnlocking || isCheckingPasswordless}
          >
            {vault.isUnlocking
              ? 'Desbloqueando...'
              : isTargetPasswordless
                ? '🔓 Entrar no Mundo (Sem Senha)'
                : t('vault.open.unlockButton')}
          </button>

          {#if !isTargetPasswordless}
            <button
              type="button"
              class="btn-text"
              onclick={toggleMode}
            >
              {useRecovery
                ? t('vault.open.usePassphrase')
                : t('vault.open.useRecoveryKey')}
            </button>
          {/if}
        </div>

        {#if !isTargetPasswordless}
          <div class="keyring-action-row">
            <button
              type="button"
              class="btn-keyring"
              onclick={handleOpenWithKeyring}
              disabled={vault.isUnlocking}
            >
              🔑 {t('vault.open.keyringButton')}
            </button>
          </div>
        {/if}
      </form>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(30, 35, 29, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(3px);
  }

  .modal-card {
    background-color: var(--color-paper-elevated, #FFFDF9);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: var(--rounded-lg, 8px);
    box-shadow: var(--shadow-lg, 0 10px 25px rgba(0, 0, 0, 0.15));
    width: 90%;
    max-width: 520px;
    padding: var(--space-xl, 24px);
    display: flex;
    flex-direction: column;
    gap: var(--space-md, 16px);
    font-family: var(--font-ui);
    color: var(--color-ink-charcoal, #1E231D);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--color-paper-border, #E5E0D8);
    padding-bottom: var(--space-sm, 12px);
  }

  .header-icon-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .header-icon {
    font-size: 20px;
  }

  .modal-title {
    margin: 0;
    font-size: 18px;
    font-family: var(--font-literary);
    font-weight: 600;
    color: var(--color-ink-charcoal, #1E231D);
  }

  .btn-close {
    background: transparent;
    border: none;
    font-size: 16px;
    color: var(--color-ink-muted, #726E65);
    cursor: pointer;
    padding: 4px;
    line-height: 1;
    border-radius: var(--radius-sm, 4px);
  }

  .btn-close:hover {
    color: var(--color-ink-charcoal, #1E231D);
    background-color: var(--color-paper-hover, #F4EFE6);
  }

  .alert-error {
    background-color: var(--color-crimson-bg, #FFF5F5);
    border: 1px solid var(--color-crimson, #9E2A2B);
    color: var(--color-crimson, #9E2A2B);
    padding: 8px 12px;
    border-radius: var(--radius-sm, 4px);
    font-size: 12px;
  }

  .vault-form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .form-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-charcoal, #1E231D);
  }

  .browse-inline-btn {
    background: transparent;
    border: none;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-olive, #5E6F45);
    cursor: pointer;
    padding: 0;
    transition: opacity 0.15s ease;
  }

  .browse-inline-btn:hover {
    text-decoration: underline;
    opacity: 0.85;
  }

  .input-with-action {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .input-with-action .form-input {
    flex: 1;
  }

  .btn-input-browse {
    padding: 8px 12px;
    background-color: var(--color-paper-surface, #F8F4EC);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: var(--rounded-default, 4px);
    cursor: pointer;
    font-size: 13px;
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  .btn-input-browse:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
    border-color: var(--color-olive, #5E6F45);
  }

  .toggle-show-btn {
    background: transparent;
    border: none;
    font-size: 11px;
    color: var(--color-olive, #5E6F45);
    cursor: pointer;
    padding: 0;
  }

  .toggle-show-btn:hover {
    text-decoration: underline;
  }

  .form-input {
    padding: 8px 10px;
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: var(--rounded-default, 4px);
    background-color: var(--color-paper-base, #FDFCFA);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-ink-charcoal, #1E231D);
    outline: none;
    transition: border-color 0.15s ease;
  }

  .form-input:focus {
    border-color: var(--color-olive, #5E6F45);
  }

  .code-input {
    font-family: var(--font-mono, monospace);
    font-size: 12px;
  }

  .recents-bar {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background-color: var(--color-paper-surface, #F8F4EC);
    padding: 8px 10px;
    border-radius: var(--radius-sm, 4px);
  }

  .passwordless-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    background-color: var(--color-paper-surface, #F8F4EC);
    border: 1px solid var(--color-olive, #5E6F45);
    border-radius: var(--rounded-default, 4px);
    padding: 12px;
    margin-top: 4px;
    margin-bottom: 4px;
  }

  .passwordless-icon {
    font-size: 24px;
    line-height: 1;
  }

  .passwordless-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12px;
    color: var(--color-ink-charcoal, #1E231D);
  }

  .passwordless-info strong {
    font-size: 13px;
    color: var(--color-olive, #5E6F45);
  }

  .recents-label {
    font-size: 11px;
    color: var(--color-ink-muted, #726E65);
    font-weight: 500;
  }

  .recents-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .recent-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background-color: var(--color-paper-elevated, #FFFDF9);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: var(--rounded-full, 9999px);
    padding: 3px 8px;
    font-size: 11px;
    color: var(--color-ink-charcoal, #1E231D);
    cursor: pointer;
    transition: all 120ms ease;
  }

  .recent-pill:hover {
    border-color: var(--color-olive, #5E6F45);
    background-color: var(--color-paper-hover, #F4EFE6);
  }

  .recent-pill.selected {
    background-color: rgba(94, 111, 69, 0.15);
    border-color: var(--color-olive, #5E6F45);
    font-weight: 600;
    color: var(--color-olive, #5E6F45);
  }

  .recent-icon {
    font-size: 10px;
  }

  .recent-title {
    max-width: 140px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .form-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 6px;
  }

  .btn-primary {
    background-color: var(--color-olive, #5E6F45);
    color: var(--color-paper-elevated, #FFFDF9);
    border: none;
    border-radius: var(--rounded-default, 4px);
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 150ms ease;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: var(--color-olive-hover, #4A5836);
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-text {
    background: transparent;
    border: none;
    font-size: 11px;
    color: var(--color-ink-muted, #726E65);
    cursor: pointer;
    text-decoration: underline;
    padding: 4px;
  }

  .btn-text:hover {
    color: var(--color-ink-charcoal, #1E231D);
  }

  .keyring-action-row {
    border-top: 1px dashed var(--color-paper-border, #E5E0D8);
    padding-top: 12px;
    display: flex;
    justify-content: center;
  }

  .btn-keyring {
    background-color: var(--color-paper-surface, #F8F4EC);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: var(--rounded-default, 4px);
    padding: 6px 12px;
    font-size: 11px;
    font-weight: 500;
    color: var(--color-ink-charcoal, #1E231D);
    cursor: pointer;
    transition: all 120ms ease;
  }

  .btn-keyring:hover:not(:disabled) {
    border-color: var(--color-olive, #5E6F45);
    background-color: var(--color-paper-hover, #F4EFE6);
  }
</style>
