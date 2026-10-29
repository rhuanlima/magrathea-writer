<script lang="ts">
  /**
   * Modal "Apoie o Projeto" (M9-07 & Ajuste 4 de UX).
   *
   * 100% offline, em total conformidade com SECURITY.md.
   * Apresenta texto divertido com referências ao Guia do Mochileiro das Galáxias,
   * layout perfeitamente alinhado e responsivo, e cópia direta da chave PIX.
   */

  import { supportStore } from './support.svelte';
  import { t } from '../i18n';

  const pixKey = 'rhuan@rhuanlima.com.br';
  let copied = $state(false);

  async function handleCopyPix() {
    try {
      await navigator.clipboard.writeText(pixKey);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 3500);
    } catch {
      // Falha silenciosa ou fallback
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && supportStore.isOpen) {
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
    <!-- Cabeçalho -->
    <div class="support-header">
      <div class="header-icon-box">
        <img src="/app-icon.png" alt="Magrathea Writer" class="support-header-logo" />
      </div>
      <div class="header-text-col">
        <h2 id="support-title" class="support-title">{t('support.modalTitle')}</h2>
        <p class="support-subtitle">{t('support.subtitle')}</p>
      </div>
      <button
        type="button"
        class="close-button"
        onclick={() => supportStore.close()}
        aria-label="Fechar"
      >
        ✕
      </button>
    </div>

    <!-- Corpo com Rolagem Limpa -->
    <div class="support-body">
      <!-- Mensagem Explicativa e Divertida -->
      <div class="literary-callout">
        <p class="callout-lead">
          {t('support.description')}
        </p>
        <p class="callout-body">
          {t('support.callout')}
        </p>
        <div class="hitchhiker-quote">
          <span class="quote-star">✨</span>
          <span>{t('support.humor42')}</span>
        </div>
      </div>

      <!-- Sugestões Divertidas de Apoio -->
      <div class="tier-pills">
        <span class="tier-pill">☕ Um Café Interestelar</span>
        <span class="tier-pill">🪐 Um Planeta Personalizado</span>
        <span class="tier-pill highlight">🌌 A Resposta (42)</span>
        <span class="tier-pill">❤️ Qualquer valor de coração</span>
      </div>

      <!-- Cartão Principal de Doação PIX -->
      <div class="donation-card">
        <!-- QR Code Oficial Real -->
        <div class="qr-container" title={t('support.qrAlt')}>
          <div class="qr-frame">
            <img src="/pix-qrcode.png" alt={t('support.qrAlt')} class="qr-real-img" />
          </div>
          <span class="qr-caption font-mono">PIX / BRASIL</span>
        </div>

        <!-- Informações e Botão de Ação -->
        <div class="donation-info">
          <span class="info-label">{t('support.pixKeyLabel')}</span>

          <div class="pix-key-display">
            <code class="pix-key-text">{pixKey}</code>
            <span class="pix-type-badge">E-mail</span>
          </div>

          <button
            type="button"
            class="copy-action-btn"
            class:copied
            onclick={handleCopyPix}
          >
            {#if copied}
              <span>{t('support.copied')}</span>
            {:else}
              <span class="btn-copy-icon">📋</span>
              <span>{t('support.copyPixButton')}</span>
            {/if}
          </button>
        </div>
      </div>

      <!-- Selo de Privacidade Estrita -->
      <div class="offline-security-banner">
        <span class="badge-icon">🛡️</span>
        <span class="badge-text">{t('support.offlineNotice')}</span>
      </div>
    </div>

    <!-- Rodapé -->
    <div class="support-footer">
      <button type="button" class="done-button" onclick={() => supportStore.close()}>
        {t('support.closeButton')}
      </button>
    </div>
  </div>
{/if}

<style>
  .support-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(17, 24, 39, 0.52);
    backdrop-filter: blur(4px);
    z-index: 9998;
  }

  .support-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 92%;
    max-width: 620px;
    max-height: 88vh;
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
    align-items: center;
    gap: 1rem;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--color-paper-border);
    background: var(--color-paper-surface);
  }

  .header-icon-box {
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    background: transparent;
    border-radius: var(--rounded-lg);
  }

  .support-header-logo {
    width: 40px;
    height: 40px;
    border-radius: 9px;
    object-fit: cover;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  }

  .header-text-col {
    flex: 1;
  }

  .support-title {
    margin: 0;
    font-family: var(--font-literary);
    font-size: 1.35rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .support-subtitle {
    margin: 0.15rem 0 0;
    font-family: var(--font-ui);
    font-size: 0.82rem;
    color: var(--color-ink-muted);
    line-height: 1.35;
  }

  .close-button {
    background: transparent;
    border: none;
    font-size: 1.15rem;
    color: var(--color-ink-muted);
    cursor: pointer;
    padding: 0.3rem 0.55rem;
    border-radius: var(--rounded-md);
  }

  .close-button:hover {
    color: var(--color-ink-charcoal);
    background: var(--color-paper-hover);
  }

  .support-body {
    padding: 1.4rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    overflow-y: auto;
    flex: 1;
  }

  .literary-callout {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .callout-lead {
    margin: 0;
    font-family: var(--font-ui);
    font-size: 0.92rem;
    line-height: 1.55;
    color: var(--color-ink-charcoal);
  }

  .callout-body {
    margin: 0;
    font-family: var(--font-ui);
    font-size: 0.88rem;
    line-height: 1.5;
    color: var(--color-ink-muted);
  }

  .hitchhiker-quote {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
    padding: 0.75rem 1rem;
    background: var(--color-olive-subtle);
    border-left: 3px solid var(--color-olive);
    border-radius: var(--rounded-md);
    font-family: var(--font-literary);
    font-size: 0.95rem;
    color: var(--color-ink-charcoal);
    line-height: 1.4;
  }

  .quote-star {
    font-size: 1.1rem;
    line-height: 1;
  }

  .tier-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .tier-pill {
    padding: 0.3rem 0.75rem;
    font-family: var(--font-ui);
    font-size: 0.76rem;
    font-weight: 500;
    background: var(--color-paper-surface);
    color: var(--color-ink-muted);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-full, 9999px);
    user-select: none;
  }

  .tier-pill.highlight {
    background: var(--color-terracotta-subtle);
    color: var(--color-terracotta);
    border-color: var(--color-terracotta);
    font-weight: 600;
  }

  .donation-card {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 1.5rem;
    padding: 1.25rem 1.4rem;
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
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

  .qr-frame {
    width: 108px;
    height: 108px;
    background: #ffffff;
    padding: 4px;
    border-radius: var(--rounded-sm);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .qr-real-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
  }

  .qr-caption {
    font-size: 0.65rem;
    color: var(--color-ink-muted);
    letter-spacing: 0.08em;
  }

  .donation-info {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    flex: 1;
    min-width: 0;
  }

  .info-label {
    font-family: var(--font-ui);
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--color-ink-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .pix-key-display {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.6rem 0.85rem;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
  }

  .pix-key-text {
    font-family: var(--font-mono);
    font-size: 0.92rem;
    color: var(--color-ink-charcoal);
    font-weight: 600;
    word-break: break-all;
  }

  .pix-type-badge {
    font-family: var(--font-ui);
    font-size: 0.7rem;
    padding: 0.15rem 0.45rem;
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: 4px;
    color: var(--color-ink-muted);
    white-space: nowrap;
  }

  .copy-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.65rem 1.25rem;
    font-family: var(--font-ui);
    font-size: 0.88rem;
    font-weight: 600;
    background: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--rounded-md);
    cursor: pointer;
    transition: all 150ms ease;
  }

  .copy-action-btn:hover {
    background: var(--color-olive-hover);
    transform: translateY(-1px);
  }

  .copy-action-btn.copied {
    background: #166534;
  }

  .btn-copy-icon {
    font-size: 1rem;
    line-height: 1;
  }

  .offline-security-banner {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.75rem 1rem;
    background: var(--color-paper-surface);
    border: 1px dashed var(--color-paper-border);
    border-radius: var(--rounded-md);
    font-family: var(--font-ui);
    font-size: 0.78rem;
    color: var(--color-ink-muted);
    line-height: 1.4;
  }

  .badge-icon {
    font-size: 1.15rem;
    flex-shrink: 0;
  }

  .support-footer {
    display: flex;
    justify-content: flex-end;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--color-paper-border);
    background: var(--color-paper-surface);
  }

  .done-button {
    padding: 0.55rem 1.4rem;
    font-family: var(--font-ui);
    font-size: 0.88rem;
    font-weight: 600;
    background: var(--color-terracotta);
    color: #ffffff;
    border: none;
    border-radius: var(--rounded-default);
    cursor: pointer;
    transition: background-color 150ms ease;
  }

  .done-button:hover {
    background: var(--color-terracotta-hover);
  }

  @media (max-width: 540px) {
    .donation-card {
      flex-direction: column;
      align-items: stretch;
    }
    .qr-container {
      align-self: center;
    }
  }
</style>
