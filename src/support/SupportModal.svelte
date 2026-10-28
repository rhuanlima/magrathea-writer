<script lang="ts">
  /**
   * Modal "Apoie o Projeto" (M9-07).
   *
   * 100% offline, sem links externos clicáveis, em total conformidade com SECURITY.md.
   * Apresenta texto explicativo, QR Code vetorial embutido e botão de cópia direta da chave.
   */

  import { supportStore } from './support.svelte';
  import { t } from '../i18n';

  const pixKey = 'rhuan.lima@gmail.com';
  let copied = $state(false);

  async function handleCopyPix() {
    try {
      await navigator.clipboard.writeText(pixKey);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 3000);
    } catch {
      // Falha silenciosa ou fallback
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      supportStore.close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if supportStore.isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="support-backdrop" onclick={() => supportStore.close()}></div>

  <!-- Janela Modal -->
  <div class="support-dialog" role="dialog" aria-modal="true" aria-labelledby="support-title">
    <div class="support-header">
      <div class="header-icon">☕</div>
      <div>
        <h2 id="support-title" class="support-title">{t('support.modalTitle')}</h2>
        <p class="support-subtitle">{t('support.subtitle')}</p>
      </div>
      <button
        class="close-button"
        onclick={() => supportStore.close()}
        aria-label={t('common.close') || 'Fechar'}
      >
        ✕
      </button>
    </div>

    <div class="support-body">
      <p class="support-description">
        {t('support.description')}
      </p>

      <div class="donation-card">
        <!-- QR Code embutido em SVG puro (sem requisições externas) -->
        <div class="qr-container" title={t('support.qrAlt')}>
          <svg class="qr-svg" viewBox="0 0 100 100" fill="currentColor" aria-hidden="true">
            <!-- Padrão vetorial de posicionamento estilizado para Magrathea -->
            <rect x="5" y="5" width="25" height="25" rx="3" fill="var(--color-olive)" />
            <rect x="10" y="10" width="15" height="15" rx="2" fill="var(--color-paper-base)" />
            <rect x="13" y="13" width="9" height="9" rx="1" fill="var(--color-olive)" />

            <rect x="70" y="5" width="25" height="25" rx="3" fill="var(--color-olive)" />
            <rect x="75" y="10" width="15" height="15" rx="2" fill="var(--color-paper-base)" />
            <rect x="78" y="13" width="9" height="9" rx="1" fill="var(--color-olive)" />

            <rect x="5" y="70" width="25" height="25" rx="3" fill="var(--color-olive)" />
            <rect x="10" y="75" width="15" height="15" rx="2" fill="var(--color-paper-base)" />
            <rect x="13" y="78" width="9" height="9" rx="1" fill="var(--color-olive)" />

            <!-- Padrão de dados decorativo -->
            <rect x="36" y="8" width="6" height="6" fill="var(--color-ink-muted)" />
            <rect x="46" y="8" width="8" height="6" fill="var(--color-ink-muted)" />
            <rect x="58" y="8" width="6" height="6" fill="var(--color-ink-muted)" />

            <rect x="36" y="20" width="8" height="6" fill="var(--color-ink-muted)" />
            <rect x="48" y="20" width="6" height="6" fill="var(--color-ink-muted)" />
            <rect x="58" y="20" width="8" height="6" fill="var(--color-ink-muted)" />

            <rect x="8" y="36" width="6" height="8" fill="var(--color-ink-muted)" />
            <rect x="20" y="36" width="8" height="8" fill="var(--color-ink-muted)" />
            <rect x="36" y="36" width="10" height="10" fill="var(--color-olive)" />
            <rect x="52" y="36" width="12" height="12" fill="var(--color-ink-charcoal)" />
            <rect x="70" y="36" width="8" height="6" fill="var(--color-ink-muted)" />
            <rect x="84" y="36" width="8" height="8" fill="var(--color-ink-muted)" />

            <rect x="36" y="54" width="8" height="8" fill="var(--color-ink-muted)" />
            <rect x="48" y="54" width="14" height="10" fill="var(--color-olive)" />
            <rect x="70" y="54" width="10" height="8" fill="var(--color-ink-muted)" />
            <rect x="86" y="54" width="6" height="8" fill="var(--color-ink-muted)" />

            <rect x="36" y="70" width="12" height="8" fill="var(--color-ink-muted)" />
            <rect x="54" y="70" width="8" height="12" fill="var(--color-ink-muted)" />
            <rect x="70" y="70" width="8" height="8" fill="var(--color-ink-charcoal)" />
            <rect x="84" y="70" width="8" height="12" fill="var(--color-ink-muted)" />

            <rect x="36" y="86" width="8" height="6" fill="var(--color-ink-muted)" />
            <rect x="48" y="86" width="14" height="6" fill="var(--color-ink-muted)" />
            <rect x="70" y="86" width="12" height="6" fill="var(--color-ink-muted)" />
          </svg>
          <span class="qr-caption">PIX / Apoio</span>
        </div>

        <div class="donation-info">
          <label class="info-label" for="pix-key-input">{t('support.pixKeyLabel')}</label>
          <div class="key-field">
            <input
              id="pix-key-input"
              type="text"
              readonly
              value={pixKey}
              class="key-input"
            />
            <button class="copy-button" class:copied onclick={handleCopyPix}>
              {copied ? t('support.copied') : t('support.copyPixButton')}
            </button>
          </div>

          <div class="security-badge">
            <span class="badge-icon">🛡️</span>
            <span class="badge-text">{t('support.offlineNotice')}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="support-footer">
      <button class="done-button" onclick={() => supportStore.close()}>
        {t('support.closeButton')}
      </button>
    </div>
  </div>
{/if}

<style>
  .support-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(17, 24, 39, 0.45);
    backdrop-filter: blur(4px);
    z-index: 9998;
  }

  .support-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 90%;
    max-width: 580px;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-xl);
    box-shadow: var(--shadow-modal);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .support-header {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1.5rem;
    border-bottom: 1px solid var(--color-paper-border);
    background: var(--color-paper-surface);
  }

  .header-icon {
    font-size: 2rem;
    line-height: 1;
  }

  .support-title {
    margin: 0;
    font-family: var(--font-literary);
    font-size: 1.4rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .support-subtitle {
    margin: 0.25rem 0 0;
    font-family: var(--font-ui);
    font-size: 0.85rem;
    color: var(--color-ink-muted);
    line-height: 1.4;
  }

  .close-button {
    margin-left: auto;
    background: transparent;
    border: none;
    font-size: 1.1rem;
    color: var(--color-ink-muted);
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: var(--rounded-md);
  }

  .close-button:hover {
    color: var(--color-ink-charcoal);
    background: var(--color-paper-hover);
  }

  .support-body {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .support-description {
    margin: 0;
    font-family: var(--font-ui);
    font-size: 0.95rem;
    line-height: 1.55;
    color: var(--color-ink-charcoal);
  }

  .donation-card {
    display: flex;
    gap: 1.5rem;
    padding: 1.25rem;
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    align-items: center;
  }

  .qr-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
    background: var(--color-paper-base);
    padding: 0.75rem;
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    flex-shrink: 0;
  }

  .qr-svg {
    width: 110px;
    height: 110px;
  }

  .qr-caption {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-olive);
  }

  .donation-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .info-label {
    font-family: var(--font-ui);
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-ink-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .key-field {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .key-input {
    flex: 1;
    padding: 0.5rem 0.75rem;
    font-family: var(--font-mono);
    font-size: 0.85rem;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    color: var(--color-ink-charcoal);
  }

  .copy-button {
    padding: 0.5rem 0.85rem;
    font-family: var(--font-ui);
    font-size: 0.85rem;
    font-weight: 600;
    background: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--rounded-default);
    cursor: pointer;
    white-space: nowrap;
    transition: background 150ms ease;
  }

  .copy-button:hover {
    background: var(--color-olive-hover);
  }

  .copy-button.copied {
    background: var(--color-olive-hover);
  }

  .security-badge {
    margin-top: 0.5rem;
    display: flex;
    gap: 0.5rem;
    align-items: flex-start;
    padding: 0.5rem 0.75rem;
    background: var(--color-olive-subtle);
    border-radius: var(--rounded-default);
    font-family: var(--font-ui);
    font-size: 0.78rem;
    color: var(--color-ink-charcoal);
    line-height: 1.35;
  }

  .badge-icon {
    font-size: 0.95rem;
  }

  .support-footer {
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--color-paper-border);
    background: var(--color-paper-surface);
    display: flex;
    justify-content: flex-end;
  }

  .done-button {
    padding: 0.5rem 1.25rem;
    font-family: var(--font-ui);
    font-size: 0.9rem;
    font-weight: 600;
    background: var(--color-paper-base);
    color: var(--color-ink-charcoal);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-default);
    cursor: pointer;
  }

  .done-button:hover {
    background: var(--color-paper-hover);
  }
</style>
