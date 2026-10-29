<script lang="ts">
  /**
   * Modal de Preferências do Editor Literário (Ajuste 6).
   *
   * Permite personalizar:
   * - Números de linha (Ligar / Desligar)
   * - Tamanho da fonte (14px a 28px)
   * - Família da fonte (EB Garamond, Plus Jakarta Sans, JetBrains Mono)
   * - Altura da linha (1.5, 1.8, 2.1)
   * - Largura da coluna (Estreita 640px, Padrão 720px, Ampla 860px, Fluida 100%)
   */

  import { editorPreferences } from './editor-preferences.svelte';
  import { typewriterAudio } from './typewriter-audio';

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && editorPreferences.isModalOpen) {
      editorPreferences.closeModal();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if editorPreferences.isModalOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="pref-backdrop" onclick={() => editorPreferences.closeModal()}></div>

  <!-- Modal Dialog -->
  <div class="pref-dialog" role="dialog" aria-modal="true" aria-labelledby="pref-title">
    <div class="pref-header">
      <div class="header-icon">⚙️</div>
      <div>
        <h2 id="pref-title" class="pref-title">Preferências do Editor</h2>
        <p class="pref-subtitle">Personalize tipografia, espaçamento e foco visual da sua escrita</p>
      </div>
      <button
        class="close-button"
        onclick={() => editorPreferences.closeModal()}
        aria-label="Fechar"
      >
        ✕
      </button>
    </div>

    <div class="pref-body">
      <!-- 1. Números de Linha -->
      <div class="pref-row">
        <div class="pref-label-col">
          <span class="pref-label">Números de Linha</span>
          <span class="pref-hint">Exibir contador numérico à esquerda de cada parágrafo</span>
        </div>
        <button
          type="button"
          class="toggle-btn"
          class:active={editorPreferences.showLineNumbers}
          onclick={() => editorPreferences.toggleLineNumbers()}
          role="switch"
          aria-checked={editorPreferences.showLineNumbers}
        >
          <span class="toggle-indicator"></span>
          <span>{editorPreferences.showLineNumbers ? 'Ativado' : 'Desativado'}</span>
        </button>
      </div>

      <!-- 2. Família Tipográfica -->
      <div class="pref-row">
        <div class="pref-label-col">
          <span class="pref-label">Família da Fonte</span>
          <span class="pref-hint">Tipografia principal do manuscrito</span>
        </div>
        <div class="font-choices" role="radiogroup">
          <button
            type="button"
            class="choice-btn font-garamond"
            class:active={editorPreferences.fontFamily === 'garamond'}
            onclick={() => editorPreferences.setFontFamily('garamond')}
          >
            Serif (EB Garamond)
          </button>
          <button
            type="button"
            class="choice-btn font-sans"
            class:active={editorPreferences.fontFamily === 'sans'}
            onclick={() => editorPreferences.setFontFamily('sans')}
          >
            Sans (Jakarta)
          </button>
          <button
            type="button"
            class="choice-btn font-mono"
            class:active={editorPreferences.fontFamily === 'mono'}
            onclick={() => editorPreferences.setFontFamily('mono')}
          >
            Mono (JetBrains)
          </button>
        </div>
      </div>

      <!-- 3. Tamanho da Fonte -->
      <div class="pref-row">
        <div class="pref-label-col">
          <span class="pref-label">Tamanho do Texto</span>
          <span class="pref-hint">Atualmente: {editorPreferences.fontSize}px</span>
        </div>
        <div class="size-controls">
          <button
            type="button"
            class="size-step-btn"
            onclick={() => editorPreferences.setFontSize(editorPreferences.fontSize - 1)}
            disabled={editorPreferences.fontSize <= 14}
            title="Diminuir fonte"
          >
            A-
          </button>
          <span class="size-value">{editorPreferences.fontSize}px</span>
          <button
            type="button"
            class="size-step-btn"
            onclick={() => editorPreferences.setFontSize(editorPreferences.fontSize + 1)}
            disabled={editorPreferences.fontSize >= 28}
            title="Aumentar fonte"
          >
            A+
          </button>
        </div>
      </div>

      <!-- 4. Altura da Linha (Espaçamento) -->
      <div class="pref-row">
        <div class="pref-label-col">
          <span class="pref-label">Espaçamento Entre Linhas</span>
          <span class="pref-hint">Respiro vertical para leitura</span>
        </div>
        <div class="font-choices">
          <button
            type="button"
            class="choice-btn"
            class:active={editorPreferences.lineHeight === 1.5}
            onclick={() => editorPreferences.setLineHeight(1.5)}
          >
            Compacto (1.5)
          </button>
          <button
            type="button"
            class="choice-btn"
            class:active={editorPreferences.lineHeight === 1.8}
            onclick={() => editorPreferences.setLineHeight(1.8)}
          >
            Literário (1.8)
          </button>
          <button
            type="button"
            class="choice-btn"
            class:active={editorPreferences.lineHeight === 2.1}
            onclick={() => editorPreferences.setLineHeight(2.1)}
          >
            Amplo (2.1)
          </button>
        </div>
      </div>

      <!-- 5. Largura da Coluna de Escrita -->
      <div class="pref-row">
        <div class="pref-label-col">
          <span class="pref-label">Largura da Coluna</span>
          <span class="pref-hint">Limita a largura máxima da página</span>
        </div>
        <div class="font-choices">
          <button
            type="button"
            class="choice-btn"
            class:active={editorPreferences.columnWidth === 'narrow'}
            onclick={() => editorPreferences.setColumnWidth('narrow')}
          >
            640px
          </button>
          <button
            type="button"
            class="choice-btn"
            class:active={editorPreferences.columnWidth === 'medium'}
            onclick={() => editorPreferences.setColumnWidth('medium')}
          >
            720px
          </button>
          <button
            type="button"
            class="choice-btn"
            class:active={editorPreferences.columnWidth === 'wide'}
            onclick={() => editorPreferences.setColumnWidth('wide')}
          >
            860px
          </button>
          <button
            type="button"
            class="choice-btn"
            class:active={editorPreferences.columnWidth === 'full'}
            onclick={() => editorPreferences.setColumnWidth('full')}
          >
            Fluida
          </button>
        </div>
      </div>

      <!-- 6. Modo Máquina de Escrever (Typewriter) -->
      <div class="pref-row">
        <div class="pref-label-col">
          <span class="pref-label">⌨️ Som de Máquina de Escrever</span>
          <span class="pref-hint">Cliques mecânicos analógicos, barra de espaço e sineta de retorno (Ding!)</span>
        </div>
        <div class="typewriter-controls">
          <button
            type="button"
            class="toggle-btn"
            class:active={editorPreferences.typewriterSound}
            onclick={() => {
              editorPreferences.toggleTypewriterSound();
              if (editorPreferences.typewriterSound) {
                typewriterAudio.playEnterReturn(editorPreferences.typewriterVolume);
              }
            }}
            role="switch"
            aria-checked={editorPreferences.typewriterSound}
          >
            <span class="toggle-indicator"></span>
            <span>{editorPreferences.typewriterSound ? 'Ativado' : 'Silencioso'}</span>
          </button>

          {#if editorPreferences.typewriterSound}
            <button
              type="button"
              class="test-sound-btn"
              onclick={() => typewriterAudio.playEnterReturn(editorPreferences.typewriterVolume)}
              title="Testar som da sineta de retorno"
            >
              🎵 Testar
            </button>
          {/if}
        </div>
      </div>

      <!-- 7. Rolagem Centralizada de Typewriter -->
      <div class="pref-row">
        <div class="pref-label-col">
          <span class="pref-label">Rolagem Centralizada (Typewriter Scrolling)</span>
          <span class="pref-hint">Mantém a linha ativa do cursor fixada no meio vertical da tela</span>
        </div>
        <button
          type="button"
          class="toggle-btn"
          class:active={editorPreferences.typewriterScrolling}
          onclick={() => editorPreferences.toggleTypewriterScrolling()}
          role="switch"
          aria-checked={editorPreferences.typewriterScrolling}
        >
          <span class="toggle-indicator"></span>
          <span>{editorPreferences.typewriterScrolling ? 'Ativado' : 'Desativado'}</span>
        </button>
      </div>
    </div>

    <div class="pref-footer">
      <button class="done-button" onclick={() => editorPreferences.closeModal()}>
        Salvar & Fechar
      </button>
    </div>
  </div>
{/if}

<style>
  .pref-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(17, 24, 39, 0.45);
    backdrop-filter: blur(4px);
    z-index: 9998;
  }

  .pref-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 90%;
    max-width: 540px;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-xl);
    box-shadow: var(--shadow-modal);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .pref-header {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--color-paper-border);
    background: var(--color-paper-surface);
  }

  .header-icon {
    font-size: 1.8rem;
    line-height: 1;
  }

  .pref-title {
    margin: 0;
    font-family: var(--font-literary);
    font-size: 1.35rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .pref-subtitle {
    margin: 0.2rem 0 0;
    font-family: var(--font-ui);
    font-size: 0.82rem;
    color: var(--color-ink-muted);
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

  .pref-body {
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.2rem;
  }

  .pref-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding-bottom: 0.85rem;
    border-bottom: 1px solid var(--color-paper-border);
  }

  .pref-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .pref-label-col {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .pref-label {
    font-family: var(--font-ui);
    font-size: 0.92rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .pref-hint {
    font-family: var(--font-ui);
    font-size: 0.78rem;
    color: var(--color-ink-muted);
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.75rem;
    font-family: var(--font-ui);
    font-size: 0.82rem;
    font-weight: 600;
    background: var(--color-paper-surface);
    color: var(--color-ink-muted);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    cursor: pointer;
    transition: all 150ms ease;
  }

  .toggle-btn.active {
    background: var(--color-olive-subtle);
    color: var(--color-olive);
    border-color: var(--color-olive);
  }

  .toggle-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-ink-dim);
  }

  .toggle-btn.active .toggle-indicator {
    background: var(--color-olive);
  }

  .font-choices {
    display: flex;
    gap: 0.35rem;
  }

  .choice-btn {
    padding: 0.35rem 0.65rem;
    font-family: var(--font-ui);
    font-size: 0.8rem;
    background: var(--color-paper-surface);
    color: var(--color-ink-muted);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    cursor: pointer;
    transition: all 120ms ease;
  }

  .choice-btn:hover {
    background: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .choice-btn.active {
    background: var(--color-olive);
    color: #ffffff;
    border-color: var(--color-olive);
    font-weight: 600;
  }

  .font-garamond {
    font-family: var(--font-literary);
  }

  .font-sans {
    font-family: var(--font-ui);
  }

  .font-mono {
    font-family: var(--font-mono);
  }

  .size-controls {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .size-step-btn {
    padding: 0.35rem 0.65rem;
    font-family: var(--font-ui);
    font-size: 0.85rem;
    font-weight: 600;
    background: var(--color-paper-surface);
    color: var(--color-ink-charcoal);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    cursor: pointer;
  }

  .size-step-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .size-value {
    min-width: 48px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .typewriter-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .test-sound-btn {
    padding: 0.35rem 0.65rem;
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    font-family: var(--font-ui);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    color: var(--color-ink-charcoal);
    transition: background-color 0.15s ease;
  }

  .test-sound-btn:hover {
    background: var(--color-paper-hover);
  }

  .pref-footer {
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--color-paper-border);
    background: var(--color-paper-surface);
    display: flex;
    justify-content: flex-end;
  }

  .done-button {
    padding: 0.5rem 1.25rem;
    font-family: var(--font-ui);
    font-size: 0.88rem;
    font-weight: 600;
    background: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--rounded-default);
    cursor: pointer;
  }

  .done-button:hover {
    background: var(--color-olive-hover);
  }
</style>
