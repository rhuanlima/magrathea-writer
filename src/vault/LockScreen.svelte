<script lang="ts">
  import { t } from '../i18n';
  import { vault } from './vault.svelte';

  let passphrase = $state('');
  let recoveryKey = $state('');
  let useRecovery = $state(false);
  let localError = $state<string | null>(null);

  async function handleUnlock(e: Event) {
    e.preventDefault();
    localError = null;

    try {
      if (vault.isPasswordless) {
        await vault.openWithPassphrase('');
        return;
      }

      if (useRecovery) {
        if (!recoveryKey.trim()) {
          localError = t('vault.unlock.invalidRecoveryKeyError');
          return;
        }
        await vault.openWithRecoveryKey(recoveryKey.trim());
      } else {
        if (!passphrase) {
          localError = t('vault.unlock.invalidPassphraseError');
          return;
        }
        await vault.openWithPassphrase(passphrase);
      }
      passphrase = '';
      recoveryKey = '';
    } catch {
      localError = useRecovery
        ? t('vault.unlock.invalidRecoveryKeyError')
        : t('vault.unlock.invalidPassphraseError');
    }
  }

  function toggleMode() {
    useRecovery = !useRecovery;
    localError = null;
    passphrase = '';
    recoveryKey = '';
  }

  async function handleCloseVault() {
    await vault.closeVault();
  }

  async function handleUnlockWithKeyring() {
    localError = null;
    try {
      await vault.openWithKeyring();
    } catch (err) {
      localError = String(err);
    }
  }
</script>

<div class="lock-backdrop" role="dialog" aria-modal="true" aria-labelledby="lock-title">
  <div class="lock-card">
    <div class="lock-icon-wrapper">
      <img src="/app-icon.png" alt="Magrathea" class="lock-app-icon" />
      <span class="lock-badge-icon">🔒</span>
    </div>

    <h2 id="lock-title" class="lock-title">
      {vault.projectTitle || t('vault.unlock.screenTitle')}
    </h2>

    <p class="lock-subtitle">
      {t('vault.unlock.lockedSubtitle')}
    </p>

    {#if localError || vault.errorMessage}
      <div class="error-banner" role="alert">
        <span>⚠️ {localError || vault.errorMessage}</span>
      </div>
    {/if}

    {#if vault.isPasswordless}
      <div class="passwordless-lock-box">
        <p class="passwordless-desc">
          Este mundo foi criado no modo livre (sem senha de proteção).
        </p>
        <button
          type="button"
          class="submit-button"
          onclick={() => vault.openWithPassphrase('')}
          disabled={vault.isUnlocking}
        >
          {vault.isUnlocking ? t('vault.unlock.unlocking') : '🔓 Entrar no Mundo (Sem Senha)'}
        </button>
      </div>
    {:else}
      {#if vault.hasKeyringEnabled}
        <button
          type="button"
          class="keyring-unlock-button"
          onclick={handleUnlockWithKeyring}
          disabled={vault.isUnlocking}
        >
          🔑 {vault.isUnlocking ? t('vault.unlock.unlockingWithKeyring') : t('vault.unlock.unlockWithKeyringButton')}
        </button>

        <div class="or-separator">
          <span class="or-text">ou</span>
        </div>
      {/if}

      <form class="lock-form" onsubmit={handleUnlock}>
        {#if !useRecovery}
          <div class="field-group">
            <label for="lock-passphrase" class="field-label">
              {t('vault.unlock.passphraseLabel')}
            </label>
            <input
              id="lock-passphrase"
              type="password"
              class="input-field"
              placeholder={t('vault.unlock.passphrasePlaceholder')}
              bind:value={passphrase}
              disabled={vault.isUnlocking}
              required
              autocomplete="current-password"
            />
          </div>
        {:else}
          <div class="field-group">
            <label for="lock-recovery-key" class="field-label">
              {t('vault.unlock.recoveryKeyLabel')}
            </label>
            <textarea
              id="lock-recovery-key"
              class="input-field mono"
              rows="3"
              placeholder={t('vault.unlock.recoveryKeyPlaceholder')}
              bind:value={recoveryKey}
              disabled={vault.isUnlocking}
              required
            ></textarea>
          </div>
        {/if}

        <div class="form-actions">
          <button
            type="submit"
            class="submit-button"
            disabled={vault.isUnlocking}
          >
            {vault.isUnlocking
              ? t('vault.unlock.unlocking')
              : t('vault.unlock.unlockButton')}
          </button>
        </div>
      </form>

      <div class="alt-options">
        <button
          type="button"
          class="text-link"
          onclick={toggleMode}
          disabled={vault.isUnlocking}
        >
          {useRecovery
            ? `← ${t('vault.unlock.usePassphrase')}`
            : t('vault.unlock.useRecoveryKey')}
        </button>

        <button
          type="button"
          class="text-link secondary"
          onclick={handleCloseVault}
          disabled={vault.isUnlocking}
        >
          {t('vault.unlock.closeVaultButton')}
        </button>
      </div>
    {/if}

    <div class="lock-footer">
      <small>{t('vault.unlock.autoLockNotice')}</small>
    </div>
  </div>
</div>

<style>
  .lock-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
    background-color: rgba(28, 25, 23, 0.82);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-md);
  }

  .lock-card {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-card);
    box-shadow: var(--shadow-modal);
    max-width: 440px;
    width: 100%;
    padding: var(--space-xl);
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    box-sizing: border-box;
  }

  .lock-icon-wrapper {
    position: relative;
    width: 64px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-md);
  }

  .lock-app-icon {
    width: 56px;
    height: 56px;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    object-fit: cover;
  }

  .lock-badge-icon {
    position: absolute;
    bottom: -2px;
    right: -2px;
    font-size: 16px;
    background: var(--color-paper-elevated);
    border-radius: 50%;
    padding: 2px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .lock-title {
    font-family: var(--font-serif);
    font-size: 22px;
    font-weight: 600;
    margin: 0 0 var(--space-xs) 0;
    color: var(--color-ink-charcoal);
  }

  .lock-subtitle {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-ink-muted);
    margin: 0 0 var(--space-lg) 0;
    line-height: 1.4;
  }

  .error-banner {
    width: 100%;
    box-sizing: border-box;
    background-color: #fdf2f2;
    border: 1px solid #fecaca;
    color: var(--color-error);
    border-radius: var(--rounded-default);
    padding: var(--space-sm) var(--space-md);
    font-size: 13px;
    margin-bottom: var(--space-md);
    text-align: left;
  }

  .keyring-unlock-button {
    width: 100%;
    padding: 10px;
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-olive);
    border-radius: var(--rounded-default);
    color: var(--color-olive);
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    margin-bottom: var(--space-md);
    transition: all var(--transition-focus);
  }

  .keyring-unlock-button:hover:not(:disabled) {
    background-color: rgba(94, 111, 69, 0.1);
  }

  .or-separator {
    display: flex;
    align-items: center;
    text-align: center;
    margin-bottom: var(--space-md);
  }

  .or-separator::before,
  .or-separator::after {
    content: '';
    flex: 1;
    border-bottom: 1px solid var(--color-paper-border);
  }

  .or-text {
    padding: 0 8px;
    font-size: 11px;
    color: var(--color-ink-muted);
  }

  .lock-form {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    text-align: left;
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field-label {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .input-field {
    box-sizing: border-box;
    width: 100%;
    padding: 10px 12px;
    font-family: var(--font-ui);
    font-size: 14px;
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    color: var(--color-ink-charcoal);
    outline: none;
    transition: border-color var(--transition-focus);
  }

  .input-field.mono {
    font-family: var(--font-mono);
    font-size: 12px;
    resize: none;
  }

  .input-field:focus {
    border-color: var(--color-olive);
    outline: var(--focus-ring);
    outline-offset: var(--focus-offset);
  }

  .form-actions {
    display: flex;
    flex-direction: column;
    margin-top: var(--space-xs);
  }

  .submit-button {
    background-color: var(--color-olive);
    color: #ffffff;
    border: none;
    padding: 10px 16px;
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    border-radius: var(--rounded-default);
    cursor: pointer;
    transition: background-color var(--transition-focus);
    width: 100%;
  }

  .submit-button:hover:not(:disabled) {
    background-color: var(--color-olive-hover);
  }

  .submit-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .passwordless-lock-box {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    width: 100%;
    margin-top: var(--space-md);
  }

  .passwordless-desc {
    font-size: 13px;
    color: var(--color-ink-muted);
    text-align: center;
    line-height: 1.5;
  }

  .alt-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    margin-top: var(--space-lg);
    width: 100%;
  }

  .text-link {
    background: transparent;
    border: none;
    color: var(--color-olive);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    padding: 4px;
    text-decoration: underline;
    text-underline-offset: 3px;
    transition: color var(--transition-focus);
  }

  .text-link:hover {
    color: var(--color-olive-hover);
  }

  .text-link.secondary {
    color: var(--color-ink-muted);
  }

  .text-link.secondary:hover {
    color: var(--color-ink-charcoal);
  }

  .lock-footer {
    margin-top: var(--space-lg);
    font-size: 11px;
    color: var(--color-ink-muted);
  }
</style>
