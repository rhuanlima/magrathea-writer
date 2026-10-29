<script lang="ts">
  /**
   * Modal Unificado de Preferências do Magrathea Writer (Ajustes 3 e 3.1).
   *
   * Centraliza:
   * - Tipografia e Ergonomia do Editor Literário
   * - Segurança, Criptografia e Chaves do Mundo (Vault)
   * - Foco e Produtividade
   * - Aparência e Idioma
   */

  import { preferences } from './preferences.svelte';
  import { editorPreferences } from '../editor/editor-preferences.svelte';
  import { typewriterAudio } from '../editor/typewriter-audio';
  import { vault } from '../vault/vault.svelte';
  import { theme } from '../design';
  import { i18n, t } from '../i18n';
  import { assessPasswordStrength } from '../crypto/password-strength';

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && preferences.isOpen) {
      preferences.close();
    }
  }

  // Estado para aba de segurança
  let currentPass = $state('');
  let newPass = $state('');
  let confirmNewPass = $state('');
  let passphraseSuccess = $state(false);
  let passphraseError = $state<string | null>(null);
  let newPassStrength = $derived(assessPasswordStrength(newPass));

  // Rotação de DEK
  let reencryptPass = $state('');
  let newRecoveryKey = $state<string | null>(null);
  let isKeyCopied = $state(false);
  let reencryptError = $state<string | null>(null);

  async function handleChangePassphrase() {
    passphraseError = null;
    passphraseSuccess = false;

    if (newPass !== confirmNewPass) {
      passphraseError = t('vault.create.passwordsDoNotMatch');
      return;
    }

    if (newPass.length === 0) {
      passphraseError = t('vault.create.passwordsDoNotMatch');
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
    if (!newRecoveryKey) return;
    try {
      await navigator.clipboard.writeText(newRecoveryKey);
      isKeyCopied = true;
      setTimeout(() => {
        isKeyCopied = false;
      }, 3000);
    } catch {
      // Ignora erro de clipboard
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if preferences.isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="pref-backdrop" onclick={() => preferences.close()}></div>

  <!-- Janela Principal -->
  <div class="pref-dialog" role="dialog" aria-modal="true" aria-labelledby="pref-modal-title">
    <div class="pref-header">
      <div class="header-info">
        <span class="header-icon">⚙️</span>
        <div>
          <h2 id="pref-modal-title" class="pref-title">Preferências do Magrathea Writer</h2>
          <p class="pref-subtitle">Tipografia literária, segurança do mundo e ergonomia de escrita</p>
        </div>
      </div>
      <button
        type="button"
        class="close-button"
        onclick={() => preferences.close()}
        aria-label="Fechar Preferências"
      >
        ✕
      </button>
    </div>

    <!-- Navegação por Guias (Tabs) de Sistema -->
    <div class="pref-tab-bar" aria-label="Guias de Preferências" role="tablist">
      <button
        type="button"
        role="tab"
        id="tab-editor"
        aria-selected={preferences.activeTab === 'editor'}
        class="tab-guide-btn"
        class:active={preferences.activeTab === 'editor'}
        onclick={() => preferences.setTab('editor')}
      >
        <span class="tab-guide-icon">✍️</span>
        <div class="tab-guide-texts">
          <span class="tab-guide-title">Editor Literário</span>
          <span class="tab-guide-sub">Fontes & Layout</span>
        </div>
      </button>

      <button
        type="button"
        role="tab"
        id="tab-security"
        aria-selected={preferences.activeTab === 'security'}
        class="tab-guide-btn"
        class:active={preferences.activeTab === 'security'}
        onclick={() => preferences.setTab('security')}
      >
        <span class="tab-guide-icon">🛡️</span>
        <div class="tab-guide-texts">
          <span class="tab-guide-title">Segurança</span>
          <span class="tab-guide-sub">Senha & DEK</span>
        </div>
      </button>

      <button
        type="button"
        role="tab"
        id="tab-focus"
        aria-selected={preferences.activeTab === 'focus'}
        class="tab-guide-btn"
        class:active={preferences.activeTab === 'focus'}
        onclick={() => preferences.setTab('focus')}
      >
        <span class="tab-guide-icon">🎯</span>
        <div class="tab-guide-texts">
          <span class="tab-guide-title">Foco & Pausas</span>
          <span class="tab-guide-sub">Bloqueio & Atalhos</span>
        </div>
      </button>

      <button
        type="button"
        role="tab"
        id="tab-system"
        aria-selected={preferences.activeTab === 'system'}
        class="tab-guide-btn"
        class:active={preferences.activeTab === 'system'}
        onclick={() => preferences.setTab('system')}
      >
        <span class="tab-guide-icon">🎨</span>
        <div class="tab-guide-texts">
          <span class="tab-guide-title">Sistema</span>
          <span class="tab-guide-sub">Tema & Idioma</span>
        </div>
      </button>
    </div>

    <!-- Conteúdo da Aba -->
    <div class="pref-body">
      {#if preferences.activeTab === 'editor'}
        <!-- 1. Números de Linha -->
        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Números de Linha</span>
            <span class="pref-hint">Exibir contador de linhas/parágrafos na margem esquerda</span>
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
            <span>{editorPreferences.showLineNumbers ? 'Exibindo' : 'Oculto'}</span>
          </button>
        </div>

        <!-- 2. Família Tipográfica -->
        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Família da Fonte</span>
            <span class="pref-hint">Escolha a tipografia que melhor conversa com seu estilo</span>
          </div>
          <div class="font-choices" role="radiogroup">
            <button
              type="button"
              class="choice-btn font-garamond"
              class:active={editorPreferences.fontFamily === 'garamond'}
              onclick={() => editorPreferences.setFontFamily('garamond')}
            >
              Serif (Garamond)
            </button>
            <button
              type="button"
              class="choice-btn font-sans"
              class:active={editorPreferences.fontFamily === 'sans'}
              onclick={() => editorPreferences.setFontFamily('sans')}
            >
              Sem Serifa (Jakarta)
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
            <span class="pref-hint">Dimensionamento em pontos ({editorPreferences.fontSize}px)</span>
          </div>
          <div class="size-controls">
            <button
              type="button"
              class="size-step-btn"
              onclick={() => editorPreferences.setFontSize(editorPreferences.fontSize - 1)}
              disabled={editorPreferences.fontSize <= 14}
            >
              −
            </button>
            <span class="size-value">{editorPreferences.fontSize}px</span>
            <button
              type="button"
              class="size-step-btn"
              onclick={() => editorPreferences.setFontSize(editorPreferences.fontSize + 1)}
              disabled={editorPreferences.fontSize >= 28}
            >
              +
            </button>
          </div>
        </div>

        <!-- 4. Altura da Linha -->
        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Espaçamento Entre Linhas</span>
            <span class="pref-hint">Respiro vertical para maior conforto de leitura</span>
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

        <!-- 5. Largura da Coluna -->
        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Largura da Mancha Gráfica</span>
            <span class="pref-hint">Limite de caracteres por linha para manter o ritmo ocular</span>
          </div>
          <div class="font-choices">
            <button
              type="button"
              class="choice-btn"
              class:active={editorPreferences.columnWidth === 'narrow'}
              onclick={() => editorPreferences.setColumnWidth('narrow')}
            >
              Estreita (640px)
            </button>
            <button
              type="button"
              class="choice-btn"
              class:active={editorPreferences.columnWidth === 'medium'}
              onclick={() => editorPreferences.setColumnWidth('medium')}
            >
              Equilibrada (720px)
            </button>
            <button
              type="button"
              class="choice-btn"
              class:active={editorPreferences.columnWidth === 'wide'}
              onclick={() => editorPreferences.setColumnWidth('wide')}
            >
              Ampla (860px)
            </button>
            <button
              type="button"
              class="choice-btn"
              class:active={editorPreferences.columnWidth === 'full'}
              onclick={() => editorPreferences.setColumnWidth('full')}
            >
              Fluida (100%)
            </button>
          </div>
        </div>

        <!-- 6. Modo Máquina de Escrever (Typewriter) -->
        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">⌨️ Som de Máquina de Escrever</span>
            <span class="pref-hint">Cliques mecânicos analógicos, barra de espaço e sineta de retorno (Ding!)</span>
          </div>
          <div class="typewriter-pref-actions">
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
            <span class="pref-hint">Mantém a linha ativa e o cursor sempre fixados no meio vertical da tela</span>
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

        <!-- Preview em Tempo Real -->
        <div class="live-preview-box">
          <span class="preview-caption">Pré-visualização do ritmo tipográfico:</span>
          <p
            class="preview-paragraph"
            style="
              font-family: {editorPreferences.fontFamilyCss};
              font-size: {editorPreferences.fontSize}px;
              line-height: {editorPreferences.lineHeight};
            "
          >
            — O mundo que você cria nesta tela não pertence a servidores, nuvens ou algoritmos. Cada palavra, cada silêncio e cada parágrafo existem exclusivamente na sua imaginação e na sua máquina.
          </p>
        </div>

      {:else if preferences.activeTab === 'security'}
        {#if !vault.isOpen || vault.isLocked}
          <div class="empty-tab-hint">
            <span class="empty-icon">🔒</span>
            <p>Abra ou destranque um mundo para gerenciar as configurações de criptografia.</p>
          </div>
        {:else}
          <div class="security-section">
            <div class="vault-status-badge">
              <span class="shield-icon">🛡️</span>
              <div class="status-texts">
                <strong>Cofre Protegido com Criptografia em Nível Burocracia Vogon</strong>
                <span>Algoritmo: XChaCha20-Poly1305 (256 bits) • KDF: Argon2id (64 MiB, 3 passes)</span>
              </div>
            </div>

            <div class="sub-card">
              <h4 class="sub-title">Alterar Senha Mestra do Mundo</h4>
              <p class="sub-desc">Re-deriva a Chave de Envoltório (KEK) sem alterar os dados do manuscrito.</p>

              {#if passphraseSuccess}
                <div class="banner success">✓ Senha mestra atualizada com sucesso no cofre!</div>
              {/if}
              {#if passphraseError}
                <div class="banner error">⚠️ {passphraseError}</div>
              {/if}

              <div class="form-inputs">
                <input
                  type="password"
                  placeholder="Senha mestra atual"
                  bind:value={currentPass}
                  class="field-input"
                />
                <input
                  type="password"
                  placeholder="Nova senha mestra segura"
                  bind:value={newPass}
                  class="field-input"
                />
                {#if newPass}
                  <div class="strength-bar-box">
                    <div
                      class="strength-bar"
                      style="width: {newPassStrength.score * 25}%; background-color: {newPassStrength.colorVar};"
                    ></div>
                    <span class="strength-label">{t(newPassStrength.feedbackKey)}</span>
                  </div>
                {/if}
                <input
                  type="password"
                  placeholder="Confirmar nova senha"
                  bind:value={confirmNewPass}
                  class="field-input"
                />
                <button
                  type="button"
                  class="action-btn"
                  onclick={handleChangePassphrase}
                  disabled={!currentPass || !newPass || newPass !== confirmNewPass}
                >
                  Salvar Nova Senha
                </button>
              </div>
            </div>

            <div class="sub-card">
              <h4 class="sub-title">Rotação de Chave de Dados (DEK)</h4>
              <p class="sub-desc">Gera uma nova chave interna aleatória de 256 bits e uma nova Chave de Recuperação de Emergência.</p>

              {#if reencryptError}
                <div class="banner error">⚠️ {reencryptError}</div>
              {/if}

              {#if newRecoveryKey}
                <div class="recovery-box">
                  <div class="recovery-badge">Nova Chave de Recuperação Gerada:</div>
                  <code class="recovery-token">{newRecoveryKey}</code>
                  <button type="button" class="copy-btn" onclick={copyRecoveryKey}>
                    {isKeyCopied ? '✓ Copiado!' : 'Copiar Token de Emergência'}
                  </button>
                </div>
              {:else}
                <div class="form-inline">
                  <input
                    type="password"
                    placeholder="Confirme sua senha mestra para girar a DEK"
                    bind:value={reencryptPass}
                    class="field-input"
                  />
                  <button
                    type="button"
                    class="action-btn warning"
                    onclick={handleReencrypt}
                    disabled={!reencryptPass}
                  >
                    Rotacionar DEK
                  </button>
                </div>
              {/if}
            </div>
          </div>
        {/if}

      {:else if preferences.activeTab === 'focus'}
        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Bloqueio Automático por Inatividade</span>
            <span class="pref-hint">Tranca o mundo em memória após 30 minutos de ausência real</span>
          </div>
          <span class="badge-active">Ativo (30 min)</span>
        </div>

        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Bloqueio ao Alternar Janela</span>
            <span class="pref-hint">Não bloqueia ao trocar rapidamente para o navegador ou referências</span>
          </div>
          <span class="badge-neutral">Desativado para fluidez</span>
        </div>

        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Atalhos Essenciais de Escrita</span>
            <span class="pref-hint">Combinações rápidas para não tirar a mão do teclado</span>
          </div>
          <div class="shortcuts-list">
            <span class="key-pill">Cmd+L: Trancar</span>
            <span class="key-pill">Cmd+K: Paleta</span>
            <span class="key-pill">Cmd+I: Inspetor</span>
            <span class="key-pill">Cmd+,: Preferências</span>
          </div>
        </div>

      {:else if preferences.activeTab === 'system'}
        <!-- Tema -->
        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Tema da Interface</span>
            <span class="pref-hint">Paleta clara em papel pergaminho ou modo escuro repousante</span>
          </div>
          <div class="font-choices">
            <button
              type="button"
              class="choice-btn"
              class:active={theme.mode === 'light'}
              onclick={() => theme.setMode('light')}
            >
              ☀️ Papel Claro
            </button>
            <button
              type="button"
              class="choice-btn"
              class:active={theme.mode === 'dark'}
              onclick={() => theme.setMode('dark')}
            >
              🌙 Noite Escura
            </button>
            <button
              type="button"
              class="choice-btn"
              class:active={theme.mode === 'system'}
              onclick={() => theme.setMode('system')}
            >
              💻 Sistema
            </button>
          </div>
        </div>

        <!-- Idioma -->
        <div class="pref-row">
          <div class="pref-label-col">
            <span class="pref-label">Idioma do Aplicativo</span>
            <span class="pref-hint">Interface e terminologia literária</span>
          </div>
          <div class="font-choices">
            <button
              type="button"
              class="choice-btn"
              class:active={i18n.locale === 'pt-BR'}
              onclick={() => i18n.setLocale('pt-BR')}
            >
              🇧🇷 Português (Brasil)
            </button>
            <button
              type="button"
              class="choice-btn"
              class:active={i18n.locale === 'en-US'}
              onclick={() => i18n.setLocale('en-US')}
            >
              🇺🇸 English
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- Rodapé -->
    <div class="pref-footer">
      <button type="button" class="done-button" onclick={() => preferences.close()}>
        Salvar & Fechar
      </button>
    </div>
  </div>
{/if}

<style>
  .pref-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(17, 24, 39, 0.5);
    backdrop-filter: blur(4px);
    z-index: 9998;
  }

  .pref-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 92%;
    max-width: 680px;
    max-height: 85vh;
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
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--color-paper-border);
    background: var(--color-paper-surface);
  }

  .header-info {
    display: flex;
    align-items: center;
    gap: 0.85rem;
  }

  .header-icon {
    font-size: 1.6rem;
    line-height: 1;
  }

  .pref-title {
    margin: 0;
    font-family: var(--font-literary);
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .pref-subtitle {
    margin: 0.15rem 0 0;
    font-family: var(--font-ui);
    font-size: 0.8rem;
    color: var(--color-ink-muted);
  }

  .close-button {
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

  .pref-tab-bar {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    padding: 8px 16px 0;
    background: var(--color-paper-surface);
    border-bottom: 2px solid var(--color-paper-border);
  }

  .tab-guide-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    background: transparent;
    border: 1px solid transparent;
    border-bottom: none;
    border-radius: 8px 8px 0 0;
    color: var(--color-ink-muted);
    cursor: pointer;
    transition: all 140ms ease;
    text-align: left;
    position: relative;
    bottom: -2px;
  }

  .tab-guide-btn:hover {
    background: var(--color-paper-hover);
    color: var(--color-ink-charcoal);
  }

  .tab-guide-btn.active {
    background: var(--color-paper-base);
    color: var(--color-ink-charcoal);
    border-color: var(--color-paper-border);
    border-top: 3px solid var(--color-terracotta);
    border-bottom: 2px solid var(--color-paper-base);
    box-shadow: 0 -2px 6px rgba(0, 0, 0, 0.03);
    z-index: 2;
  }

  .tab-guide-icon {
    font-size: 1.35rem;
    line-height: 1;
    flex-shrink: 0;
  }

  .tab-guide-texts {
    display: flex;
    flex-direction: column;
    gap: 1px;
    overflow: hidden;
  }

  .tab-guide-title {
    font-family: var(--font-ui);
    font-size: 0.88rem;
    font-weight: 600;
    color: inherit;
    white-space: nowrap;
  }

  .tab-guide-sub {
    font-family: var(--font-ui);
    font-size: 0.72rem;
    color: var(--color-ink-faded);
    white-space: nowrap;
  }

  .tab-guide-btn.active .tab-guide-sub {
    color: var(--color-terracotta);
    font-weight: 500;
  }

  @media (max-width: 620px) {
    .pref-tab-bar {
      grid-template-columns: repeat(2, 1fr);
      padding: 6px 8px 0;
    }
  }

  .pref-body {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    overflow-y: auto;
    flex: 1;
  }

  .pref-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding-bottom: 0.95rem;
    border-bottom: 1px solid var(--color-paper-border);
  }

  .pref-label-col {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .pref-label {
    font-family: var(--font-ui);
    font-size: 0.9rem;
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

  .typewriter-pref-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .test-sound-btn {
    padding: 0.4rem 0.65rem;
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

  .font-choices {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .choice-btn {
    padding: 0.4rem 0.75rem;
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

  .live-preview-box {
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    padding: 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .preview-caption {
    font-family: var(--font-ui);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-ink-muted);
  }

  .preview-paragraph {
    margin: 0;
    color: var(--color-ink-charcoal);
    transition: all 120ms ease;
  }

  .security-section {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .vault-status-badge {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.85rem 1rem;
    background: var(--color-olive-subtle);
    border: 1px solid var(--color-olive);
    border-radius: var(--rounded-md);
    color: var(--color-olive);
  }

  .shield-icon {
    font-size: 1.5rem;
  }

  .status-texts {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .status-texts strong {
    font-size: 0.88rem;
  }

  .status-texts span {
    font-size: 0.75rem;
    opacity: 0.9;
    font-family: var(--font-mono);
  }

  .sub-card {
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-lg);
    padding: 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .sub-title {
    margin: 0;
    font-size: 0.92rem;
    font-weight: 600;
    color: var(--color-ink-charcoal);
  }

  .sub-desc {
    margin: 0;
    font-size: 0.78rem;
    color: var(--color-ink-muted);
  }

  .form-inputs {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .form-inline {
    display: flex;
    gap: 0.5rem;
  }

  .field-input {
    width: 100%;
    padding: 0.55rem 0.75rem;
    font-family: var(--font-ui);
    font-size: 0.85rem;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
    color: var(--color-ink-charcoal);
    box-sizing: border-box;
  }

  .strength-bar-box {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    height: 8px;
    background: var(--color-paper-border);
    border-radius: 4px;
    overflow: hidden;
  }

  .strength-bar {
    height: 100%;
    transition: width 200ms ease;
  }

  .strength-label {
    font-size: 0.72rem;
    font-family: var(--font-ui);
    color: var(--color-ink-muted);
  }

  .action-btn {
    align-self: flex-start;
    padding: 0.5rem 1rem;
    font-family: var(--font-ui);
    font-size: 0.82rem;
    font-weight: 600;
    background: var(--color-olive);
    color: #ffffff;
    border: none;
    border-radius: var(--rounded-md);
    cursor: pointer;
  }

  .action-btn.warning {
    background: #d97706;
  }

  .action-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .banner {
    padding: 0.65rem 0.85rem;
    border-radius: var(--rounded-md);
    font-size: 0.82rem;
  }

  .banner.success {
    background: #dcfce7;
    color: #166534;
  }

  .banner.error {
    background: #fee2e2;
    color: #991b1b;
  }

  .recovery-box {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.85rem;
    background: var(--color-paper-base);
    border: 1px solid var(--color-paper-border);
    border-radius: var(--rounded-md);
  }

  .recovery-badge {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-terracotta);
  }

  .recovery-token {
    font-family: var(--font-mono);
    font-size: 0.85rem;
    padding: 0.5rem;
    background: var(--color-paper-surface);
    border-radius: var(--rounded-sm);
    word-break: break-all;
  }

  .copy-btn {
    align-self: flex-start;
    padding: 0.35rem 0.75rem;
    font-family: var(--font-ui);
    font-size: 0.78rem;
    background: var(--color-terracotta);
    color: #ffffff;
    border: none;
    border-radius: var(--rounded-md);
    cursor: pointer;
  }

  .empty-tab-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1.5rem;
    text-align: center;
    color: var(--color-ink-muted);
  }

  .empty-icon {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
  }

  .badge-active {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-olive);
    background: var(--color-olive-subtle);
    padding: 0.25rem 0.65rem;
    border-radius: var(--rounded-md);
  }

  .badge-neutral {
    font-size: 0.8rem;
    color: var(--color-ink-muted);
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    padding: 0.25rem 0.65rem;
    border-radius: var(--rounded-md);
  }

  .shortcuts-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .key-pill {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    background: var(--color-paper-surface);
    border: 1px solid var(--color-paper-border);
    padding: 0.25rem 0.5rem;
    border-radius: var(--rounded-md);
    color: var(--color-ink-charcoal);
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
