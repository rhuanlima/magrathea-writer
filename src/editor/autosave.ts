/**
 * Gerenciador de autosave debounced com salvamento atômico e cifrado.
 *
 * Em conformidade com o PRD (E1):
 * - Autosave acionado após breve período de inatividade na digitação (debounce).
 * - Garante que nenhuma gravação intermediária fique em claro.
 */

export type AutosaveStatus = 'saved' | 'unsaved' | 'saving' | 'error';

export interface AutosaveOptions {
  /** Atraso de debounce em milissegundos antes de salvar (padrão: 800ms). */
  delayMs?: number;
  /** Função assíncrona de salvamento atômico cifrado. */
  onSave: (content: string) => Promise<void>;
  /** Callback para notificação de transição de estado. */
  onStatusChange?: (status: AutosaveStatus) => void;
}

export class AutosaveManager {
  private delayMs: number;
  private onSave: (content: string) => Promise<void>;
  private onStatusChange?: (status: AutosaveStatus) => void;

  private timerId: ReturnType<typeof setTimeout> | null = null;
  private pendingContent: string | null = null;
  private currentStatus: AutosaveStatus = 'saved';
  private isDisposed = false;

  constructor(options: AutosaveOptions) {
    this.delayMs = options.delayMs ?? 800;
    this.onSave = options.onSave;
    this.onStatusChange = options.onStatusChange;
  }

  /**
   * Retorna o estado atual do autosave.
   */
  getStatus(): AutosaveStatus {
    return this.currentStatus;
  }

  /**
   * Notifica que o documento foi modificado pelo autor.
   */
  notifyChange(content: string): void {
    if (this.isDisposed) return;

    this.pendingContent = content;
    this.setStatus('unsaved');

    if (this.timerId !== null) {
      clearTimeout(this.timerId);
    }

    this.timerId = setTimeout(() => {
      void this.flush();
    }, this.delayMs);
  }

  /**
   * Força a gravação imediata de quaisquer alterações pendentes.
   */
  async flush(): Promise<void> {
    if (this.isDisposed) return;

    if (this.timerId !== null) {
      clearTimeout(this.timerId);
      this.timerId = null;
    }

    if (this.pendingContent === null) {
      return;
    }

    const contentToSave = this.pendingContent;
    this.pendingContent = null;
    this.setStatus('saving');

    try {
      await this.onSave(contentToSave);
      // Se não houver novas alterações pendentes geradas durante a gravação
      if (this.pendingContent === null) {
        this.setStatus('saved');
      }
    } catch {
      this.setStatus('error');
    }
  }

  /**
   * Encerra o gerenciador e cancela timers pendentes.
   */
  dispose(): void {
    this.isDisposed = true;
    if (this.timerId !== null) {
      clearTimeout(this.timerId);
      this.timerId = null;
    }
    this.pendingContent = null;
  }

  private setStatus(status: AutosaveStatus): void {
    if (this.currentStatus !== status) {
      this.currentStatus = status;
      this.onStatusChange?.(status);
    }
  }
}
