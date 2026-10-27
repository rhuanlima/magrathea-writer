/**
 * Monitor de inatividade e eventos do sistema para bloqueio automático de segurança.
 *
 * Em conformidade com o PRD (E13):
 * - Dispara bloqueio por inatividade após período configurável (padrão 15 min).
 * - Monitora visibilidade da janela e eventos do autor para proteger o manuscrito.
 */

export interface InactivityOptions {
  /** Tempo limite de inatividade em milissegundos (padrão: 15 minutos). */
  timeoutMs?: number;
  /** Callback acionado quando a inatividade expira. */
  onTimeout: () => void | Promise<void>;
  /** Se deve bloquear imediatamente ao ocultar a aplicação (visibilitychange). */
  lockOnHidden?: boolean;
}

export const DEFAULT_INACTIVITY_TIMEOUT_MS = 15 * 60 * 1000; // 15 minutos

export class InactivityTracker {
  private timeoutMs: number;
  private onTimeout: () => void | Promise<void>;
  private lockOnHidden: boolean;
  private timerId: ReturnType<typeof setTimeout> | null = null;
  private isRunning = false;

  constructor(options: InactivityOptions) {
    this.timeoutMs = options.timeoutMs ?? DEFAULT_INACTIVITY_TIMEOUT_MS;
    this.onTimeout = options.onTimeout;
    this.lockOnHidden = options.lockOnHidden ?? false;

    this.handleActivity = this.handleActivity.bind(this);
    this.handleVisibility = this.handleVisibility.bind(this);
  }

  /**
   * Inicia o monitoramento de eventos de atividade.
   */
  start(): void {
    if (this.isRunning) return;
    this.isRunning = true;

    if (typeof window !== 'undefined') {
      window.addEventListener('pointerdown', this.handleActivity, { passive: true });
      window.addEventListener('keydown', this.handleActivity, { passive: true });
      window.addEventListener('scroll', this.handleActivity, { passive: true });
    }

    if (typeof document !== 'undefined') {
      document.addEventListener('visibilitychange', this.handleVisibility);
    }

    this.reset();
  }

  /**
   * Para o monitoramento e limpa timers ativos.
   */
  stop(): void {
    if (!this.isRunning) return;
    this.isRunning = false;

    if (this.timerId !== null) {
      clearTimeout(this.timerId);
      this.timerId = null;
    }

    if (typeof window !== 'undefined') {
      window.removeEventListener('pointerdown', this.handleActivity);
      window.removeEventListener('keydown', this.handleActivity);
      window.removeEventListener('scroll', this.handleActivity);
    }

    if (typeof document !== 'undefined') {
      document.removeEventListener('visibilitychange', this.handleVisibility);
    }
  }

  /**
   * Reinicia a contagem de tempo de inatividade.
   */
  reset(): void {
    if (!this.isRunning) return;

    if (this.timerId !== null) {
      clearTimeout(this.timerId);
    }

    this.timerId = setTimeout(() => {
      this.triggerTimeout();
    }, this.timeoutMs);
  }

  /**
   * Força o disparo imediato do callback de inatividade (ex: testes ou logout forçado).
   */
  triggerTimeout(): void {
    void this.onTimeout();
  }

  /**
   * Retorna se o monitor está em execução ativa.
   */
  isActive(): boolean {
    return this.isRunning;
  }

  private handleActivity(): void {
    this.reset();
  }

  private handleVisibility(): void {
    if (typeof document !== 'undefined' && document.hidden && this.lockOnHidden) {
      this.triggerTimeout();
    }
  }
}
