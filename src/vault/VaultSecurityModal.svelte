<script lang="ts">
  import { t } from '../i18n';
  import { vault } from './vault.svelte';
  import { assessPasswordStrength } from '../crypto/password-strength';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  let activeTab = $state<'passphrase' | 'reencrypt' | 'keyring'>('passphrase');

  // Estado para troca de senha
  let currentPass = $state('');
  let newPass = $state('');
  let confirmNewPass = $state('');
  let passphraseSuccess = $state(false);
  let passphraseError = $state<string | null>(null);

  let newPassStrength = $derived(assessPasswordStrength(newPass));

  // Estado para recifragem
  let reencryptPass = $state('');
  let newRecoveryKey = $state<string | null>(null);
  let isKeyCopied = $state(false);
  let hasConfirmedRecoveryKey = $state(false);
  let reencryptError = $state<string | null>(null);

  // Estado para chaveiro
  let keyringPass = $state('');
  let keyringMessage = $state<string | null>(null);
  let keyringError = $state<string | null>(null);

  async function handleChangePassphrase() {
    passphraseError = null;
    passphraseSuccess = false;

    if (newPass !== confirmNewPass) {
      passphraseError = t('vault.create.passwordsDoNotMatch');
      return;
    }

    if (newPass.length < 10) {
      passphraseError = t('vault.create.minRequirementsNote');
      return;
    }

    try {
      await vault.changePassphrase(currentPass, newPass);
      passphraseSuccess = true;
      currentPass = '';
      newPass = '';
      confirmNewPass = '';
    } catch (err) {
      passphraseError = String(err);
    }
  }

  async function handleReencrypt() {
    reencryptError = null;
    try {
      const generatedToken = await vault.reencryptVault(reencryptPass);
      newRecoveryKey = generatedToken;
      reencryptPass = '';
    } catch (err) {
      reencryptError = String(err);
    }
  }

  async function copyRecoveryKey() {
    if (newRecoveryKey) {
      await navigator.clipboard.writeText(newRecoveryKey);
      isKeyCopied = true;
      setTimeout(() => (isKeyCopied = false), 2500);
    }
  }

  async function handleToggleKeyring() {
    keyringError = null;
    keyringMessage = null;

    if (vault.hasKeyringEnabled) {
      try {
        await vault.removeFromKeyring();
        keyringMessage = t('vault.security.keyringDisabledSuccess');
      } catch (err) {
        keyringError = String(err);
      }
    } else {
      if (!keyringPass) {
        keyringError = t('vault.security.keyringPassphrasePrompt');
        return;
      }
      try {
        await vault.saveToKeyring(keyringPass);
        keyringPass = '';
        keyringMessage = t('vault.security.keyringEnabledSuccess');
      } catch (err) {
        keyringError = String(err);
      }
    }
  }

  function handleCloseModal() {
    passphraseSuccess = false;
    passphraseError = null;
    newRecoveryKey = null;
    reencryptError = null;
    keyringError = null;
    keyringMessage = null;
    onClose();
  }
</script>

{#if isOpen}
  <div
    class="modal-backdrop"
    onclick={handleCloseModal}
    onkeydown={(e) => e.key === 'Escape' && handleCloseModal()}
    tabindex="0"
    role="button"
  >
    <div
      class="modal-container"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      tabindex="-1"
      role="dialog"
      aria-label={t('vault.security.modalTitle')}
    >
      <header class="modal-header">
        <div class="header-title-group">
          <span class="shield-icon">🛡️</span>
          <h2>{t('vault.security.modalTitle')}</h2>
        </div>
        <button
          type="button"
          class="close-button"
          onclick={handleCloseModal}
          title={t('common.close')}
        >
          ×
        </button>
      </header>

      <nav class="tabs-nav" aria-label="Abas de segurança">
        <button
          type="button"
          class="tab-btn"
          class:active={activeTab === 'passphrase'}
          onclick={() => (activeTab = 'passphrase')}
        >
          {t('vault.security.tabChangePassphrase')}
        </button>
        <button
          type="button"
          class="tab-btn"
          class:active={activeTab === 'reencrypt'}
          onclick={() => (activeTab = 'reencrypt')}
        >
          {t('vault.security.tabReencrypt')}
        </button>
        <button
          type="button"
          class="tab-btn"
          class:active={activeTab === 'keyring'}
          onclick={() => (activeTab = 'keyring')}
        >
          {t('vault.security.tabKeyring')}
        </button>
      </nav>

      <div class="modal-body">
        {#if activeTab === 'passphrase'}
          <div class="tab-panel">
            {#if passphraseSuccess}
              <div class="success-banner">
                ✓ {t('vault.security.passphraseChangedSuccess')}
              </div>
            {/if}

            {#if passphraseError}
              <div class="error-banner">
                ⚠️ {passphraseError}
              </div>
            {/if}

            <form onsubmit={(e) => { e.preventDefault(); void handleChangePassphrase(); }}>
              <div class="form-group">
                <label for="current-pass">
                  {t('vault.security.currentPassphraseLabel')}
                </label>
                <input
                  id="current-pass"
                  type="password"
                  placeholder={t('vault.security.currentPassphrasePlaceholder')}
                  bind:value={currentPass}
                  required
                />
              </div>

              <div class="form-group">
                <label for="new-pass">
                  {t('vault.security.newPassphraseLabel')}
                </label>
                <input
                  id="new-pass"
                  type="password"
                  placeholder={t('vault.security.newPassphrasePlaceholder')}
                  bind:value={newPass}
                  required
                />
                {#if newPass.length > 0}
                  <div class="strength-meter">
                    <div class="meter-track">
                      <div
                        class="meter-fill"
                        style="width: {(newPassStrength.score / 4) * 100}%; background-color: {newPassStrength.colorVar};"
                      ></div>
                    </div>
                    <div class="meter-info">
                      <span class="strength-tag" style="color: {newPassStrength.colorVar};">
                        {t(newPassStrength.feedbackKey)}
                      </span>
                      <span class="entropy-label">{newPassStrength.entropyBits} bits</span>
                    </div>
                  </div>
                {/if}
              </div>

              <div class="form-group">
                <label for="confirm-new-pass">
                  {t('vault.security.confirmNewPassphraseLabel')}
                </label>
                <input
                  id="confirm-new-pass"
                  type="password"
                  placeholder={t('vault.security.confirmNewPassphrasePlaceholder')}
                  bind:value={confirmNewPass}
                  required
                />
              </div>

              <button
                type="submit"
                class="primary-action-btn"
                disabled={vault.isChangingPassphrase || !currentPass || newPass.length < 10}
              >
                {vault.isChangingPassphrase
                  ? t('vault.security.changingPassphrase')
                  : t('vault.security.changePassphraseButton')}
              </button>
            </form>
          </div>
        {:else if activeTab === 'reencrypt'}
          <div class="tab-panel">
            {#if newRecoveryKey}
              <div class="reencrypt-success-view">
                <div class="success-banner">
                  ✓ {t('vault.security.reencryptSuccessTitle')}
                </div>
                <p class="reencrypt-note">
                  {t('vault.security.reencryptNewKeyNote')}
                </p>

                <div class="key-display-box">
                  <code>{newRecoveryKey}</code>
                  <button
                    type="button"
                    class="copy-btn"
                    onclick={copyRecoveryKey}
                  >
                    {isKeyCopied ? t('vault.create.copiedTooltip') : t('vault.create.copyKeyButton')}
                  </button>
                </div>

                <label class="confirm-checkbox">
                  <input
                    type="checkbox"
                    bind:checked={hasConfirmedRecoveryKey}
                  />
                  <span>{t('vault.create.confirmSavedLabel')}</span>
                </label>

                <button
                  type="button"
                  class="primary-action-btn"
                  disabled={!hasConfirmedRecoveryKey}
                  onclick={handleCloseModal}
                >
                  {t('common.close')}
                </button>
              </div>
            {:else}
              <div class="warning-box">
                <h4>⚠️ {t('vault.security.reencryptTitle')}</h4>
                <p>{t('vault.security.reencryptExplanation')}</p>
                <p class="strong-warning">{t('vault.security.reencryptWarning')}</p>
              </div>

              {#if reencryptError}
                <div class="error-banner">
                  ⚠️ {reencryptError}
                </div>
              {/if}

              <form onsubmit={(e) => { e.preventDefault(); void handleReencrypt(); }}>
                <div class="form-group">
                  <label for="reenc-pass">
                    {t('vault.security.currentPassphraseLabel')}
                  </label>
                  <input
                    id="reenc-pass"
                    type="password"
                    placeholder={t('vault.security.currentPassphrasePlaceholder')}
                    bind:value={reencryptPass}
                    required
                  />
                </div>

                <button
                  type="submit"
                  class="danger-action-btn"
                  disabled={vault.isReencrypting || !reencryptPass}
                >
                  {vault.isReencrypting
                    ? t('vault.security.reencrypting')
                    : t('vault.security.reencryptButton')}
                </button>
              </form>
            {/if}
          </div>
        {:else if activeTab === 'keyring'}
          <div class="tab-panel">
            <div class="info-box">
              <h4>🔑 {t('vault.security.keyringTitle')}</h4>
              <p>{t('vault.security.keyringDescription')}</p>
              <div class="limitation-alert">
                {t('vault.security.keyringWarningLimitation')}
              </div>
            </div>

            {#if keyringMessage}
              <div class="success-banner">
                ✓ {keyringMessage}
              </div>
            {/if}

            {#if keyringError}
              <div class="error-banner">
                ⚠️ {keyringError}
              </div>
            {/if}

            <div class="keyring-toggle-section">
              <div class="toggle-status">
                <span>Status atual:</span>
                <strong>{vault.hasKeyringEnabled ? 'ATIVADO' : 'DESATIVADO'}</strong>
              </div>

              {#if !vault.hasKeyringEnabled}
                <div class="form-group">
                  <label for="keyring-pass">
                    {t('vault.security.keyringPassphrasePrompt')}
                  </label>
                  <input
                    id="keyring-pass"
                    type="password"
                    placeholder={t('vault.security.currentPassphrasePlaceholder')}
                    bind:value={keyringPass}
                  />
                </div>
              {/if}

              <button
                type="button"
                class={vault.hasKeyringEnabled ? 'danger-action-btn' : 'primary-action-btn'}
                onclick={handleToggleKeyring}
              >
                {vault.hasKeyringEnabled
                  ? 'Desativar e Apagar do Chaveiro do SO'
                  : t('vault.security.enableKeyringLabel')}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-container {
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-md);
    width: 520px;
    max-width: 95vw;
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md);
    border-bottom: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-surface);
  }

  .header-title-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .shield-icon {
    font-size: 18px;
  }

  .modal-header h2 {
    margin: 0;
    font-family: var(--font-ui);
    font-size: 15px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .close-button {
    background: transparent;
    border: none;
    font-size: 20px;
    line-height: 1;
    cursor: pointer;
    color: var(--color-ink-muted);
  }

  .close-button:hover {
    color: var(--color-ink-charcoal);
  }

  .tabs-nav {
    display: flex;
    background-color: var(--color-paper-surface);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .tab-btn {
    flex: 1;
    padding: 10px;
    border: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    color: var(--color-ink-muted);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.15s ease;
  }

  .tab-btn:hover {
    color: var(--color-ink-charcoal);
  }

  .tab-btn.active {
    color: var(--color-olive);
    border-bottom-color: var(--color-olive);
    font-weight: 600;
    background-color: var(--color-paper-base);
  }

  .modal-body {
    padding: var(--space-lg);
    max-height: 70vh;
    overflow-y: auto;
  }

  .tab-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: var(--space-md);
  }

  .form-group label {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .form-group input {
    font-family: var(--font-ui);
    font-size: 13px;
    padding: 8px 10px;
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    color: var(--color-ink-charcoal);
    outline: none;
  }

  .form-group input:focus {
    border-color: var(--color-olive);
  }

  .primary-action-btn {
    width: 100%;
    padding: 10px;
    background-color: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .primary-action-btn:hover:not(:disabled) {
    background-color: var(--color-olive-hover);
  }

  .primary-action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .danger-action-btn {
    width: 100%;
    padding: 10px;
    background-color: var(--color-error);
    color: #ffffff;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .danger-action-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .danger-action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .success-banner {
    padding: 10px;
    background-color: rgba(34, 197, 94, 0.1);
    border: 1px solid rgba(34, 197, 94, 0.3);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    color: #15803d;
  }

  .error-banner {
    padding: 10px;
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-error);
  }

  .warning-box {
    background-color: rgba(245, 158, 11, 0.1);
    border: 1px solid rgba(245, 158, 11, 0.3);
    border-radius: var(--radius-sm);
    padding: var(--space-md);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-ink-charcoal);
  }

  .warning-box h4 {
    margin: 0 0 6px 0;
    font-size: 13px;
    color: #b45309;
  }

  .strong-warning {
    font-weight: 600;
    color: #b45309;
    margin-bottom: 0;
  }

  .info-box {
    background-color: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    padding: var(--space-md);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-ink-charcoal);
  }

  .info-box h4 {
    margin: 0 0 6px 0;
    font-size: 13px;
  }

  .limitation-alert {
    margin-top: 8px;
    padding: 8px;
    background-color: var(--color-paper-elevated);
    border-left: 3px solid var(--color-olive);
    font-size: 11px;
    line-height: 1.4;
    color: var(--color-ink-muted);
  }

  .keyring-toggle-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    margin-top: var(--space-sm);
  }

  .toggle-status {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-ui);
    font-size: 13px;
  }

  .reencrypt-success-view {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .reencrypt-note {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-ink-muted);
    margin: 0;
  }

  .key-display-box {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px;
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    gap: 8px;
  }

  .key-display-box code {
    font-family: var(--font-mono);
    font-size: 11px;
    word-break: break-all;
    color: var(--color-ink-charcoal);
  }

  .strength-meter {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
  }

  .meter-track {
    height: 4px;
    background-color: var(--color-paper-border);
    border-radius: 2px;
    overflow: hidden;
  }

  .meter-fill {
    height: 100%;
    transition: width 0.2s ease, background-color 0.2s ease;
  }

  .meter-info {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    font-family: var(--font-ui);
  }

  .strength-tag {
    font-weight: 600;
  }

  .entropy-label {
    color: var(--color-ink-muted);
  }

  .copy-btn {
    font-family: var(--font-ui);
    font-size: 11px;
    padding: 4px 8px;
    background-color: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    white-space: nowrap;
  }

  .confirm-checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-ui);
    font-size: 12px;
    cursor: pointer;
  }
</style>
