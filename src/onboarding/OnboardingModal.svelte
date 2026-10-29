<script lang="ts">
  /**
   * Modal Interativo do Curso Básico e Introdução ao Magrathea Writer (Ajuste 3).
   *
   * Apresentado na primeira abertura do app e acessível a qualquer momento via menu "Sobre" e atalho.
   */

  import { onboardingStore, COURSE_LESSONS } from './onboarding.svelte';

  const pixKey = 'rhuan@rhuanlima.com.br';
  let copiedPix = $state(false);

  async function handleCopyPix() {
    try {
      await navigator.clipboard.writeText(pixKey);
      copiedPix = true;
      setTimeout(() => {
        copiedPix = false;
      }, 3500);
    } catch {
      // Fallback
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!onboardingStore.isOpen) return;
    if (e.key === 'Escape') {
      onboardingStore.close();
    } else if (e.key === 'ArrowRight') {
      onboardingStore.nextLesson();
    } else if (e.key === 'ArrowLeft') {
      onboardingStore.prevLesson();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if onboardingStore.isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="onboarding-backdrop" onclick={() => onboardingStore.close()}></div>

  <!-- Janela do Curso -->
  <div class="onboarding-dialog" role="dialog" aria-modal="true" aria-labelledby="lesson-title">
    <!-- Barra Superior de Progresso das Lições -->
    <div class="progress-bar-container">
      {#each COURSE_LESSONS as lesson, idx (lesson.id)}
        <button
          type="button"
          class="progress-step-btn"
          class:completed={idx < onboardingStore.currentLessonIndex}
          class:current={idx === onboardingStore.currentLessonIndex}
          onclick={() => onboardingStore.goToLesson(idx)}
          title={lesson.title}
          aria-label={lesson.title}
        >
          <div class="step-indicator-bar"></div>
        </button>
      {/each}
    </div>

    <!-- Cabeçalho da Lição -->
    <div class="onboarding-header">
      <div class="header-icon-box">{onboardingStore.currentLesson.icon}</div>
      <div class="header-text-col">
        <span class="lesson-badge">{onboardingStore.currentLesson.badge}</span>
        <h2 id="lesson-title" class="lesson-title">{onboardingStore.currentLesson.title}</h2>
        <p class="lesson-subtitle">{onboardingStore.currentLesson.subtitle}</p>
      </div>
      <button
        type="button"
        class="close-button"
        onclick={() => onboardingStore.close()}
        aria-label="Pular e Fechar"
      >
        ✕
      </button>
    </div>

    <!-- Conteúdo da Lição -->
    <div class="onboarding-body">
      {#if onboardingStore.currentLesson.id === 6}
        <!-- Destaque Especial: Encorajamento de Pagar um Café com QR Code PIX -->
        <div class="coffee-support-container">
          <div class="coffee-qr-column">
            <div class="coffee-qr-frame">
              <img
                src="/pix-qrcode.png"
                alt="QR Code PIX para apoio ao Magrathea Writer"
                class="coffee-qr-img"
              />
            </div>
            <span class="qr-caption">Aponte a câmera do seu banco</span>
          </div>

          <div class="coffee-info-column">
            <div class="coffee-copy-box">
              <label for="pix-key-input" class="pix-label">Chave PIX (E-mail)</label>
              <div class="pix-input-row">
                <input
                  id="pix-key-input"
                  type="text"
                  readonly
                  value={pixKey}
                  class="pix-key-input"
                  aria-label="Chave PIX do autor"
                />
                <button
                  type="button"
                  class="pix-copy-btn"
                  onclick={handleCopyPix}
                  title="Copiar chave PIX"
                >
                  {copiedPix ? '✓ Copiado!' : 'Copiar'}
                </button>
              </div>
              {#if copiedPix}
                <span class="copy-success-badge">Chave copiada para a área de transferência!</span>
              {/if}
            </div>

            <div class="coffee-perks">
              <span class="coffee-perk">☕ Um café quente</span>
              <span class="coffee-perk">🛡️ Zero anúncios</span>
              <span class="coffee-perk">🪐 100% livre</span>
            </div>
          </div>
        </div>
      {/if}

      <div class="cards-grid">
        {#each onboardingStore.currentLesson.bullets as bullet}
          <div class="feature-card">
            <div class="card-icon">{bullet.icon}</div>
            <div class="card-content">
              <h3 class="card-title">{bullet.title}</h3>
              <p class="card-desc">{bullet.desc}</p>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Rodapé de Navegação do Curso -->
    <div class="onboarding-footer">
      <div class="footer-left">
        {#if onboardingStore.currentLessonIndex > 0}
          <button
            type="button"
            class="nav-btn secondary"
            onclick={() => onboardingStore.prevLesson()}
          >
            ← Anterior
          </button>
        {:else}
          <button
            type="button"
            class="skip-btn"
            onclick={() => onboardingStore.close()}
          >
            Pular Introdução
          </button>
        {/if}
      </div>

      <div class="lesson-dots" role="tablist">
        {#each COURSE_LESSONS as dotLesson, idx (dotLesson.id)}
          <button
            type="button"
            class="dot-btn"
            class:active={idx === onboardingStore.currentLessonIndex}
            onclick={() => onboardingStore.goToLesson(idx)}
            title={dotLesson.title}
            aria-label="Ir para a Lição {idx + 1}"
          ></button>
        {/each}
      </div>

      <div class="footer-right">
        {#if onboardingStore.currentLessonIndex < COURSE_LESSONS.length - 1}
          <button
            type="button"
            class="nav-btn primary"
            onclick={() => onboardingStore.nextLesson()}
          >
            Próximo Módulo →
          </button>
        {:else}
          <button
            type="button"
            class="nav-btn start"
            onclick={() => onboardingStore.close()}
          >
            Começar a Escrever! 🚀
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .onboarding-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(17, 24, 39, 0.58);
    backdrop-filter: blur(5px);
    z-index: 10000;
  }

  .onboarding-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 92%;
    max-width: 680px;
    max-height: 88vh;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-xl);
    box-shadow: 0 20px 45px rgba(0, 0, 0, 0.22);
    z-index: 10001;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .progress-bar-container {
    display: flex;
    gap: 4px;
    padding: 8px 16px 0;
    background: var(--color-paper-surface);
  }

  .progress-step-btn {
    flex: 1;
    padding: 4px 0;
    background: transparent;
    border: none;
    cursor: pointer;
  }

  .step-indicator-bar {
    height: 4px;
    background: var(--color-paper-border);
    border-radius: 2px;
    transition: all 180ms ease;
  }

  .progress-step-btn.completed .step-indicator-bar {
    background: var(--color-olive);
  }

  .progress-step-btn.current .step-indicator-bar {
    background: var(--color-terracotta);
    height: 5px;
  }

  .onboarding-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.25rem 1.6rem 1rem;
    background: var(--color-paper-surface);
    border-bottom: 1px solid var(--color-paper-border);
  }

  .header-icon-box {
    font-size: 2.2rem;
    line-height: 1;
    width: 52px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    flex-shrink: 0;
  }

  .header-text-col {
    flex: 1;
    min-width: 0;
  }

  .lesson-badge {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-terracotta);
  }

  .lesson-title {
    margin: 0.15rem 0 0;
    font-family: var(--font-literary);
    font-size: 1.35rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
    line-height: 1.25;
  }

  .lesson-subtitle {
    margin: 0.2rem 0 0;
    font-family: var(--font-ui);
    font-size: 0.82rem;
    color: var(--color-ink-muted);
    line-height: 1.4;
  }

  .close-button {
    background: transparent;
    border: none;
    font-size: 1.15rem;
    color: var(--color-ink-muted);
    cursor: pointer;
    padding: 0.35rem 0.6rem;
    border-radius: var(--rounded-md);
  }

  .close-button:hover {
    color: var(--color-ink-charcoal);
    background: var(--color-paper-hover);
  }

  .onboarding-body {
    padding: 1.4rem 1.6rem;
    overflow-y: auto;
    flex: 1;
  }

  .cards-grid {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .coffee-support-container {
    display: flex;
    gap: 1.5rem;
    align-items: center;
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    padding: 1.25rem;
    margin-bottom: 1rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.04);
  }

  .coffee-qr-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .coffee-qr-frame {
    width: 130px;
    height: 130px;
    background: #ffffff;
    padding: 6px;
    border-radius: var(--rounded-md);
    border: 1px solid var(--color-paper-border);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  }

  .coffee-qr-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
  }

  .qr-caption {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--color-ink-muted);
  }

  .coffee-info-column {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .coffee-copy-box {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .pix-label {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-olive);
  }

  .pix-input-row {
    display: flex;
    gap: 0.5rem;
  }

  .pix-key-input {
    flex: 1;
    padding: 0.5rem 0.75rem;
    font-family: var(--font-mono);
    font-size: 0.82rem;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    color: var(--color-ink-charcoal);
  }

  .pix-copy-btn {
    padding: 0.5rem 1rem;
    font-family: var(--font-ui);
    font-size: 0.82rem;
    font-weight: 600;
    background: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--rounded-md);
    cursor: pointer;
    transition: all 120ms ease;
    white-space: nowrap;
  }

  .pix-copy-btn:hover {
    background: var(--color-olive-hover);
  }

  .copy-success-badge {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--color-olive);
  }

  .coffee-perks {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .coffee-perk {
    font-family: var(--font-ui);
    font-size: 0.74rem;
    padding: 0.25rem 0.6rem;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: 9999px;
    color: var(--color-ink-muted);
  }

  @media (max-width: 580px) {
    .coffee-support-container {
      flex-direction: column;
      align-items: center;
      text-align: center;
    }

    .pix-input-row {
      flex-direction: column;
    }
  }

  .feature-card {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1rem 1.2rem;
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    transition: all 120ms ease;
  }

  .feature-card:hover {
    border-color: var(--color-olive);
    background: var(--color-paper-base);
  }

  .card-icon {
    font-size: 1.5rem;
    line-height: 1;
    padding-top: 2px;
    flex-shrink: 0;
  }

  .card-content {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .card-title {
    margin: 0;
    font-family: var(--font-ui);
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .card-desc {
    margin: 0;
    font-family: var(--font-ui);
    font-size: 0.82rem;
    line-height: 1.5;
    color: var(--color-ink-muted);
  }

  .onboarding-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.6rem;
    border-top: 1px solid var(--color-paper-border);
    background: var(--color-paper-surface);
  }

  .footer-left,
  .footer-right {
    min-width: 120px;
  }

  .footer-right {
    display: flex;
    justify-content: flex-end;
  }

  .nav-btn {
    padding: 0.55rem 1.25rem;
    font-family: var(--font-ui);
    font-size: 0.85rem;
    font-weight: 600;
    border-radius: var(--rounded-md);
    cursor: pointer;
    transition: all 120ms ease;
    border: none;
  }

  .nav-btn.primary {
    background: var(--color-olive);
    color: #ffffff;
  }

  .nav-btn.primary:hover {
    background: var(--color-olive-hover);
  }

  .nav-btn.secondary {
    background: transparent;
    color: var(--color-ink-charcoal);
    border: 1px solid var(--color-paper-border);
  }

  .nav-btn.secondary:hover {
    background: var(--color-paper-hover);
  }

  .nav-btn.start {
    background: var(--color-terracotta);
    color: #ffffff;
  }

  .nav-btn.start:hover {
    background: var(--color-terracotta-hover);
  }

  .skip-btn {
    background: transparent;
    border: none;
    font-family: var(--font-ui);
    font-size: 0.82rem;
    color: var(--color-ink-faded);
    cursor: pointer;
  }

  .skip-btn:hover {
    color: var(--color-ink-charcoal);
    text-decoration: underline;
  }

  .lesson-dots {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .dot-btn {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-paper-border);
    border: none;
    cursor: pointer;
    padding: 0;
    transition: all 120ms ease;
  }

  .dot-btn.active {
    background: var(--color-terracotta);
    width: 20px;
    border-radius: 4px;
  }
</style>
