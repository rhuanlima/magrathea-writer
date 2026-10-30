<script lang="ts">
  import { t } from '../i18n';
  import { assessPasswordStrength } from '../crypto/password-strength';
  import { pickDirectory, safeInvoke } from '../tauri/bridge';
  import { vault } from './vault.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess: () => void;
  }

  let { isOpen, onClose, onSuccess }: Props = $props();

  let step = $state<1 | 2>(1);
  let isPasswordless = $state<boolean>(false);
  let projectTitle = $state('');
  let dirPath = $state('');
  let passphrase = $state('');
  let passphraseConfirm = $state('');
  let hasCopiedKey = $state(false);
  let confirmSaved = $state(false);
  let localError = $state<string | null>(null);
  let generatedRecoveryToken = $state('');

  // Avaliação reativa da força da senha
  let strength = $derived(assessPasswordStrength(passphrase));
  let passwordsMatch = $derived(
    isPasswordless || (passphrase.length > 0 && passphrase === passphraseConfirm)
  );
  let canProceedStep1 = $derived(
    projectTitle.trim().length > 0 &&
      dirPath.trim().length > 0 &&
      (isPasswordless || strength.isAcceptable) &&
      passwordsMatch &&
      !vault.isCreating
  );

  // Inicializa diretório padrão ao abrir
  $effect(() => {
    if (isOpen && dirPath.trim().length === 0) {
      safeInvoke<string>('get_default_vault_dir')
        .then((path) => {
          dirPath = path;
        })
        .catch(() => {
          dirPath = '/Documents';
        });
    }
  });

  async function handleBrowseDirectory() {
    try {
      const selected = await pickDirectory('Escolha a pasta onde o Mundo será criado');
      if (selected) {
        dirPath = selected;
      }
    } catch (err) {
      localError = String(err);
    }
  }

  async function handleCreateVault(e: SubmitEvent) {
    e.preventDefault();
    localError = null;

    if (!canProceedStep1) {
      if (!passwordsMatch) {
        localError = t('vault.create.passwordsDoNotMatch');
      }
      return;
    }

    try {
      const res = await vault.createVault(
        dirPath.trim(),
        projectTitle.trim(),
        isPasswordless ? '' : passphrase
      );
      if (isPasswordless) {
        onSuccess();
        onClose();
        return;
      }
      generatedRecoveryToken = res.recovery_key_token;
      step = 2;
    } catch (err) {
      localError = String(err);
    }
  }

  async function handleCopyKey() {
    try {
      await navigator.clipboard.writeText(generatedRecoveryToken);
      hasCopiedKey = true;
      setTimeout(() => {
        hasCopiedKey = false;
      }, 3000);
    } catch {
      hasCopiedKey = true;
    }
  }

  function handleFinalize() {
    if (!confirmSaved) return;
    onSuccess();
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    // Step 2 shows the one-time recovery key: never dismiss it by accident.
    if (e.key === 'Escape' && isOpen && step === 1) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div class="modal-card">
      <header class="modal-header">
        <h2 id="modal-title" class="modal-title">{t('vault.create.modalTitle')}</h2>
        {#if step === 1}
          <button
            type="button"
            class="close-button"
            onclick={onClose}
            aria-label={t('common.close')}
          >
            ✕
          </button>
        {/if}
      </header>

      {#if localError}
        <div class="alert-error" role="alert">
          {localError}
        </div>
      {/if}

      {#if step === 1}
        <!-- Passo 1: Informações e Senha Mestra -->
        <form onsubmit={handleCreateVault} class="vault-form">
          <div class="step-indicator">
            <span class="step-badge active">1</span>
            <span class="step-label">{t('vault.create.step1Title')}</span>
          </div>

          <div class="form-group">
            <label for="project-title" class="field-label">
              {t('vault.create.projectTitleLabel')}
            </label>
            <input
              id="project-title"
              type="text"
              class="text-input"
              bind:value={projectTitle}
              placeholder={t('vault.create.projectTitlePlaceholder')}
              required
              disabled={vault.isCreating}
            />
          </div>

          <div class="form-group">
            <div class="label-row">
              <label for="vault-dir" class="field-label">
                {t('vault.create.vaultDirLabel')}
              </label>
              <button
                type="button"
                class="browse-inline-btn"
                onclick={handleBrowseDirectory}
              >
                📁 Escolher Pasta...
              </button>
            </div>
            <div class="input-with-action">
              <input
                id="vault-dir"
                type="text"
                class="text-input"
                bind:value={dirPath}
                placeholder={t('vault.create.vaultDirPlaceholder')}
                required
                disabled={vault.isCreating}
              />
              <button
                type="button"
                class="btn-input-browse"
                onclick={handleBrowseDirectory}
                title="Escolher pasta no computador"
              >
                🔍
              </button>
            </div>
          </div>

          <!-- Modo de Proteção: Com ou Sem Senha -->
          <div class="form-group protection-mode-group">
            <span class="field-label">Modo de Acesso:</span>
            <div class="protection-mode-cards">
              <label class="mode-card" class:active={isPasswordless}>
                <input
                  type="radio"
                  name="protection-mode"
                  checked={isPasswordless}
                  onchange={() => (isPasswordless = true)}
                />
                <div class="mode-info">
                  <strong>🔓 Mundo Livre (Sem Senha)</strong>
                  <p>Abertura direta em 1 clique sem exigir senha nem bloqueio por inatividade.</p>
                </div>
              </label>

              <label class="mode-card" class:active={!isPasswordless}>
                <input
                  type="radio"
                  name="protection-mode"
                  checked={!isPasswordless}
                  onchange={() => (isPasswordless = false)}
                />
                <div class="mode-info">
                  <strong>🔒 Mundo Protegido (Senha Mestra)</strong>
                  <p>Cifragem rígida local com Argon2id para proteção máxima de dados confidenciais.</p>
                </div>
              </label>
            </div>
          </div>

          {#if !isPasswordless}
            <div class="form-group">
              <label for="passphrase" class="field-label">
                {t('vault.create.passphraseLabel')}
              </label>
              <input
                id="passphrase"
                type="password"
                class="text-input"
                bind:value={passphrase}
                placeholder={t('vault.create.passphrasePlaceholder')}
                required={!isPasswordless}
                disabled={vault.isCreating}
              />

              <!-- Medidor de Força de Senha Offline -->
              {#if passphrase.length > 0}
                <div class="strength-meter">
                  <div class="meter-track">
                    <div
                      class="meter-fill"
                      style="width: {(strength.score / 4) * 100}%; background-color: {strength.colorVar};"
                    ></div>
                  </div>
                  <div class="meter-info">
                    <span class="strength-tag" style="color: {strength.colorVar};">
                      {t(strength.feedbackKey)}
                    </span>
                    <span class="entropy-label">{strength.entropyBits} bits</span>
                  </div>
                </div>
              {/if}

              <p class="field-help">{t('vault.create.minRequirementsNote')}</p>
            </div>

            <div class="form-group">
              <label for="passphrase-confirm" class="field-label">
                {t('vault.create.passphraseConfirmLabel')}
              </label>
              <input
                id="passphrase-confirm"
                type="password"
                class="text-input"
                bind:value={passphraseConfirm}
                placeholder={t('vault.create.passphraseConfirmPlaceholder')}
                required={!isPasswordless}
                disabled={vault.isCreating}
              />
              {#if passphraseConfirm.length > 0 && !passwordsMatch}
                <span class="field-error">{t('vault.create.passwordsDoNotMatch')}</span>
              {/if}
            </div>
          {/if}

          <footer class="modal-footer">
            <button type="button" class="btn-secondary" onclick={onClose}>
              {t('common.cancel')}
            </button>
            <button
              type="submit"
              class="btn-primary"
              disabled={!canProceedStep1}
            >
              {#if vault.isCreating}
                {t('vault.create.creatingVault')}
              {:else}
                {t('vault.create.continueButton')}
              {/if}
            </button>
          </footer>
        </form>
      {:else}
        <!-- Passo 2: Chave de Recuperação Obrigatória -->
        <div class="recovery-step">
          <div class="step-indicator">
            <span class="step-badge completed">✓</span>
            <span class="step-label">{t('vault.create.step2Title')}</span>
          </div>

          <div class="warning-callout">
            <span class="warning-icon">⚠</span>
            <p class="warning-text">{t('vault.create.recoveryWarning')}</p>
          </div>

          <div class="token-container">
            <div class="token-display" aria-label="Chave de Recuperação">
              {generatedRecoveryToken}
            </div>
            <button
              type="button"
              class="btn-secondary copy-btn"
              onclick={handleCopyKey}
            >
              {hasCopiedKey
                ? t('vault.create.copiedTooltip')
                : t('vault.create.copyKeyButton')}
            </button>
          </div>

          <div class="confirmation-group">
            <label class="checkbox-label">
              <input
                type="checkbox"
                class="checkbox-input"
                bind:checked={confirmSaved}
              />
              <span>{t('vault.create.confirmSavedLabel')}</span>
            </label>
          </div>

          <footer class="modal-footer">
            <button
              type="button"
              class="btn-primary full-width"
              disabled={!confirmSaved}
              onclick={handleFinalize}
            >
              {t('vault.create.openVaultButton')}
            </button>
          </footer>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(30, 35, 29, 0.65);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: var(--space-md);
  }

  .modal-card {
    background-color: var(--color-paper-elevated);
    border: 1px solid var(--color-paper-border);
    border-radius: 8px;
    width: 100%;
    max-width: 580px;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.15);
    padding: var(--space-lg);
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--color-paper-border);
    padding-bottom: var(--space-sm);
  }

  .modal-title {
    font-family: var(--font-literary);
    font-size: 26px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    margin: 0;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 18px;
    color: var(--color-ink-muted);
    cursor: pointer;
    padding: var(--space-xs);
    border-radius: 4px;
  }

  .close-button:hover {
    color: var(--color-ink-charcoal);
    background-color: var(--color-paper-surface);
  }

  .step-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-xs);
  }

  .step-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background-color: var(--color-olive);
    color: #ffffff;
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
  }

  .step-badge.completed {
    background-color: var(--color-olive);
  }

  .step-label {
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .vault-form,
  .recovery-step {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .protection-mode-cards {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 4px;
  }

  .mode-card {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 6px;
    border: 1px solid var(--color-paper-border);
    background-color: var(--color-paper-base);
    cursor: pointer;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .mode-card.active {
    border-color: var(--color-olive, #5E6F45);
    background-color: var(--color-paper-surface);
  }

  .mode-card input {
    margin-top: 3px;
    cursor: pointer;
  }

  .mode-info strong {
    display: block;
    font-size: 13px;
    color: var(--color-ink-charcoal);
  }

  .mode-info p {
    margin: 2px 0 0 0;
    font-size: 11px;
    color: var(--color-ink-muted);
    line-height: 1.3;
  }

  .field-label {
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
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

  .input-with-action .text-input {
    flex: 1;
  }

  .btn-input-browse {
    padding: 8px 12px;
    background-color: var(--color-paper-surface, #F8F4EC);
    border: 1px solid var(--color-paper-border, #E5E0D8);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  .btn-input-browse:hover {
    background-color: var(--color-paper-hover, #F4EFE6);
    border-color: var(--color-olive, #5E6F45);
  }

  .text-input {
    font-family: var(--font-ui);
    font-size: 14px;
    padding: 8px 12px;
    border: 1px solid var(--color-paper-border);
    border-radius: 4px;
    background-color: var(--color-paper-base);
    color: var(--color-ink-charcoal);
    box-sizing: border-box;
    transition: border-color 0.15s ease;
  }

  .text-input:focus {
    outline: none;
    border-color: var(--color-olive);
    box-shadow: 0 0 0 2px var(--color-olive-subtle);
  }

  .field-help {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-ink-muted);
    margin: 2px 0 0 0;
  }

  .field-error {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-error);
    margin-top: 2px;
  }

  .alert-error {
    background-color: #fdf2f2;
    border: 1px solid var(--color-error);
    color: var(--color-error);
    padding: var(--space-sm) var(--space-md);
    border-radius: 4px;
    font-family: var(--font-ui);
    font-size: 13px;
  }

  .strength-meter {
    margin-top: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .meter-track {
    width: 100%;
    height: 4px;
    background-color: var(--color-paper-border);
    border-radius: 2px;
    overflow: hidden;
  }

  .meter-fill {
    height: 100%;
    transition: width 0.25s ease, background-color 0.25s ease;
  }

  .meter-info {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 500;
  }

  .entropy-label {
    color: var(--color-ink-muted);
  }

  .warning-callout {
    display: flex;
    gap: var(--space-sm);
    background-color: #fff8eb;
    border: 1px solid var(--color-bronze);
    border-radius: 6px;
    padding: var(--space-sm) var(--space-md);
  }

  .warning-icon {
    font-size: 18px;
    color: var(--color-bronze-text);
  }

  .warning-text {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-bronze-text);
    margin: 0;
    line-height: 1.4;
  }

  .token-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .token-display {
    font-family: var(--font-mono);
    font-size: 13px;
    letter-spacing: 0.5px;
    background-color: var(--color-paper-base);
    border: 1px dashed var(--color-paper-border);
    border-radius: 6px;
    padding: var(--space-md);
    color: var(--color-ink-charcoal);
    user-select: all;
    word-break: break-all;
    text-align: center;
    line-height: 1.6;
  }

  .copy-btn {
    align-self: center;
  }

  .confirmation-group {
    padding: var(--space-xs) 0;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 500;
    color: var(--color-ink-charcoal);
    cursor: pointer;
  }

  .checkbox-input {
    width: 16px;
    height: 16px;
    accent-color: var(--color-olive);
    cursor: pointer;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
    margin-top: var(--space-xs);
  }

  .btn-primary {
    background-color: var(--color-olive);
    color: #ffffff;
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: var(--color-olive-hover);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background-color: var(--color-paper-surface);
    color: var(--color-ink-charcoal);
    border: 1px solid var(--color-paper-border);
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 500;
    padding: 8px 14px;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .btn-secondary:hover {
    background-color: var(--color-paper-hover);
  }

  .full-width {
    width: 100%;
  }
</style>
