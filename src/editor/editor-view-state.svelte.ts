/**
 * Gerenciador de estado reativo para a experiência de escrita e foco literário (M3-01 a M3-05).
 *
 * Controla:
 * - Modo Foco Dinâmico (CodeMirror)
 * - Modo Zen (UI imersiva com chrome zero)
 * - HUD Flutuante (palavras, sessão, tempo e meta)
 * - Dashboard de Produtividade do Cofre
 */

import { safeInvoke, type ProductivityOverview } from '../tauri/bridge';
import { setFocusMode } from './focus-mode';
import type { EditorView } from 'codemirror';

class EditorViewState {
  // Configurações de exibição e foco
  isFocusMode = $state<boolean>(false);
  isZenMode = $state<boolean>(false);
  isHudVisible = $state<boolean>(true);
  isHudMinimized = $state<boolean>(false);
  isProductivityModalOpen = $state<boolean>(false);

  // Métricas da sessão ativa
  sessionWords = $state<number>(0);
  sessionSeconds = $state<number>(0);
  private timerInterval: ReturnType<typeof setInterval> | null = null;
  private isTimerActive = false;

  // Dados de produtividade agregados vindos do cofre cifrado
  productivity = $state<ProductivityOverview>({
    project_target_words: 50000,
    project_deadline: null,
    daily_target_words: 1000,
    total_manuscript_words: 0,
    today_words: 0,
    today_seconds: 0,
    seven_days_avg_words: 0,
    thirty_days_avg_words: 0,
    daily_logs: {},
  });

  isLoadingProductivity = $state<boolean>(false);

  constructor() {
    if (typeof window !== 'undefined') {
      const storedFocus = localStorage.getItem('magrathea-focus-mode');
      if (storedFocus !== null) {
        this.isFocusMode = storedFocus === 'true';
      }
      const storedHud = localStorage.getItem('magrathea-hud-visible');
      if (storedHud !== null) {
        this.isHudVisible = storedHud === 'true';
      }
      const storedMinimized = localStorage.getItem('magrathea-hud-minimized');
      if (storedMinimized !== null) {
        this.isHudMinimized = storedMinimized === 'true';
      }
    }
  }

  /**
   * Obtém a data corrente em formato ISO YYYY-MM-DD.
   */
  getTodayIsoString(): string {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }

  /**
   * Alterna o Modo Foco Dinâmico e sincroniza a extensão do CodeMirror.
   */
  toggleFocus(view?: EditorView | null): void {
    this.isFocusMode = !this.isFocusMode;
    if (typeof window !== 'undefined') {
      localStorage.setItem('magrathea-focus-mode', String(this.isFocusMode));
    }
    if (view) {
      view.dispatch({
        effects: setFocusMode.of(this.isFocusMode),
      });
    }
  }

  /**
   * Sai imediatamente do Modo Foco.
   */
  exitFocus(view?: EditorView | null): void {
    if (this.isFocusMode) {
      this.isFocusMode = false;
      if (typeof window !== 'undefined') {
        localStorage.setItem('magrathea-focus-mode', 'false');
      }
      if (view) {
        view.dispatch({
          effects: setFocusMode.of(false),
        });
      }
    }
  }

  /**
   * Aplica o estado inicial de foco dinâmico no EditorView na montagem.
   */
  applyInitialFocus(view: EditorView): void {
    if (this.isFocusMode) {
      view.dispatch({
        effects: setFocusMode.of(true),
      });
    }
  }

  /**
   * Alterna o Modo Zen imersivo (chrome zero).
   */
  toggleZen(): void {
    this.isZenMode = !this.isZenMode;
  }

  /**
   * Sai imediatamente do Modo Zen.
   */
  exitZen(): void {
    this.isZenMode = false;
  }

  /**
   * Alterna a visibilidade do HUD flutuante.
   */
  toggleHud(): void {
    this.isHudVisible = !this.isHudVisible;
    if (typeof window !== 'undefined') {
      localStorage.setItem('magrathea-hud-visible', String(this.isHudVisible));
    }
  }

  /**
   * Alterna entre modo minimizado (pílula compacta) e expandido do HUD.
   */
  toggleHudMinimized(): void {
    this.isHudMinimized = !this.isHudMinimized;
    if (typeof window !== 'undefined') {
      localStorage.setItem('magrathea-hud-minimized', String(this.isHudMinimized));
    }
  }

  /**
   * Abre o modal do Dashboard de Produtividade.
   */
  openProductivityDashboard(): void {
    this.isProductivityModalOpen = true;
    void this.refreshProductivity();
  }

  /**
   * Fecha o modal do Dashboard de Produtividade.
   */
  closeProductivityDashboard(): void {
    this.isProductivityModalOpen = false;
  }

  /**
   * Inicia o cronômetro ativo da sessão literária quando o autor começa a escrever.
   */
  startSessionTimer(): void {
    if (this.isTimerActive) return;
    this.isTimerActive = true;
    this.timerInterval = setInterval(() => {
      this.sessionSeconds += 1;
    }, 1000);
  }

  /**
   * Registra incremento de palavras digitadas na sessão atual e salva periodicamente no cofre.
   */
  recordWordsDelta(delta: number): void {
    if (delta > 0) {
      this.sessionWords += delta;
      this.startSessionTimer();
    }
  }

  /**
   * Carrega os dados de metas e métricas do cofre cifrado.
   */
  async refreshProductivity(): Promise<void> {
    this.isLoadingProductivity = true;
    try {
      const todayIso = this.getTodayIsoString();
      const overview = await safeInvoke<ProductivityOverview>('get_productivity_overview', {
        today_date_iso: todayIso,
      });
      this.productivity = overview;
    } catch (err) {
      console.error('Falha ao carregar métricas de produtividade do cofre:', err);
    } finally {
      this.isLoadingProductivity = false;
    }
  }

  /**
   * Atualiza as metas configuradas no cofre e recarrega a visão do dashboard.
   */
  async updateTargets(
    projectTarget: number,
    deadline: string | null,
    dailyTarget: number
  ): Promise<void> {
    this.isLoadingProductivity = true;
    try {
      const todayIso = this.getTodayIsoString();
      const overview = await safeInvoke<ProductivityOverview>('update_productivity_targets', {
        project_target: projectTarget,
        deadline,
        daily_target: dailyTarget,
        today_date_iso: todayIso,
      });
      this.productivity = overview;
    } catch (err) {
      console.error('Falha ao atualizar metas de produtividade no cofre:', err);
      throw err;
    } finally {
      this.isLoadingProductivity = false;
    }
  }

  /**
   * Salva o progresso da sessão acumulada no cofre de forma cifrada e atômica.
   */
  async flushSessionToVault(): Promise<void> {
    if (this.sessionWords === 0 && this.sessionSeconds === 0) return;

    try {
      const todayIso = this.getTodayIsoString();
      const wordsToSave = this.sessionWords;
      const secondsToSave = this.sessionSeconds;

      // Reseta contadores locais de delta antes da gravação
      this.sessionWords = 0;
      this.sessionSeconds = 0;

      const overview = await safeInvoke<ProductivityOverview>('record_productivity_session', {
        date_iso: todayIso,
        words_delta: wordsToSave,
        session_seconds: secondsToSave,
      });
      this.productivity = overview;
    } catch (err) {
      console.error('Falha ao persistir sessão no cofre:', err);
    }
  }

  /**
   * Finaliza o cronômetro e limpa referências no descarte.
   */
  dispose(): void {
    if (this.timerInterval) {
      clearInterval(this.timerInterval);
      this.timerInterval = null;
    }
    this.isTimerActive = false;
    void this.flushSessionToVault();
  }
}

export const editorViewState = new EditorViewState();
