/**
 * Preferências de Foco, Pausas, Bloqueio Automático, Pomodoro e Meta Diária.
 *
 * Gerencia centralizadamente:
 * - Tempo de bloqueio por inatividade (0 = Desligado, 5, 15, 30, 60 min)
 * - Timer Pomodoro integrado e alertas de ciclo
 * - Meta diária de palavras com notificação inspiradora ao atingir
 */

class FocusPreferencesStore {
  autoLockMinutes = $state<number>(30); // Padrão: 30 minutos
  pomodoroEnabled = $state<boolean>(false);
  pomodoroWorkMinutes = $state<number>(25);
  pomodoroBreakMinutes = $state<number>(5);
  dailyGoalWords = $state<number>(1000);
  dailyGoalAlertEnabled = $state<boolean>(true);

  // Estado efêmero de execução do Pomodoro
  pomodoroState = $state<'idle' | 'work' | 'break'>('idle');
  pomodoroSecondsLeft = $state<number>(25 * 60);
  pomodoroCyclesCompleted = $state<number>(0);
  private pomodoroInterval: ReturnType<typeof setInterval> | null = null;

  // Alerta comemorativo de meta atingida (1x por dia)
  private goalCelebratedDateIso = $state<string | null>(null);
  goalToastMessage = $state<string | null>(null);

  constructor() {
    this.loadFromStorage();
  }

  loadFromStorage(): void {
    if (typeof localStorage === 'undefined') return;
    try {
      const raw = localStorage.getItem('magrathea_focus_preferences');
      if (raw) {
        const parsed = JSON.parse(raw);
        if (typeof parsed.autoLockMinutes === 'number') {
          this.autoLockMinutes = parsed.autoLockMinutes;
        }
        if (typeof parsed.pomodoroEnabled === 'boolean') {
          this.pomodoroEnabled = parsed.pomodoroEnabled;
        }
        if (typeof parsed.pomodoroWorkMinutes === 'number') {
          this.pomodoroWorkMinutes = parsed.pomodoroWorkMinutes;
        }
        if (typeof parsed.pomodoroBreakMinutes === 'number') {
          this.pomodoroBreakMinutes = parsed.pomodoroBreakMinutes;
        }
        if (typeof parsed.dailyGoalWords === 'number') {
          this.dailyGoalWords = parsed.dailyGoalWords;
        }
        if (typeof parsed.dailyGoalAlertEnabled === 'boolean') {
          this.dailyGoalAlertEnabled = parsed.dailyGoalAlertEnabled;
        }
      }
    } catch {
      // Ignora falhas de parse
    }
  }

  saveToStorage(): void {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(
        'magrathea_focus_preferences',
        JSON.stringify({
          autoLockMinutes: this.autoLockMinutes,
          pomodoroEnabled: this.pomodoroEnabled,
          pomodoroWorkMinutes: this.pomodoroWorkMinutes,
          pomodoroBreakMinutes: this.pomodoroBreakMinutes,
          dailyGoalWords: this.dailyGoalWords,
          dailyGoalAlertEnabled: this.dailyGoalAlertEnabled,
        })
      );
    } catch {
      // Ignora erro
    }
  }

  setAutoLockMinutes(minutes: number): void {
    this.autoLockMinutes = minutes;
    this.saveToStorage();
  }

  togglePomodoro(): void {
    this.pomodoroEnabled = !this.pomodoroEnabled;
    if (!this.pomodoroEnabled) {
      this.stopPomodoro();
    } else {
      this.startPomodoroWork();
    }
    this.saveToStorage();
  }

  setPomodoroWorkMinutes(min: number): void {
    this.pomodoroWorkMinutes = Math.max(5, Math.min(min, 120));
    this.saveToStorage();
  }

  setPomodoroBreakMinutes(min: number): void {
    this.pomodoroBreakMinutes = Math.max(1, Math.min(min, 60));
    this.saveToStorage();
  }

  setDailyGoalWords(words: number): void {
    this.dailyGoalWords = Math.max(100, words);
    this.saveToStorage();
  }

  toggleDailyGoalAlert(): void {
    this.dailyGoalAlertEnabled = !this.dailyGoalAlertEnabled;
    this.saveToStorage();
  }

  startPomodoroWork(): void {
    if (this.pomodoroInterval) clearInterval(this.pomodoroInterval);
    this.pomodoroState = 'work';
    this.pomodoroSecondsLeft = this.pomodoroWorkMinutes * 60;

    this.pomodoroInterval = setInterval(() => {
      if (this.pomodoroSecondsLeft > 1) {
        this.pomodoroSecondsLeft -= 1;
      } else {
        this.onPomodoroWorkFinished();
      }
    }, 1000);
  }

  startPomodoroBreak(): void {
    if (this.pomodoroInterval) clearInterval(this.pomodoroInterval);
    this.pomodoroState = 'break';
    this.pomodoroSecondsLeft = this.pomodoroBreakMinutes * 60;

    this.pomodoroInterval = setInterval(() => {
      if (this.pomodoroSecondsLeft > 1) {
        this.pomodoroSecondsLeft -= 1;
      } else {
        this.onPomodoroBreakFinished();
      }
    }, 1000);
  }

  stopPomodoro(): void {
    if (this.pomodoroInterval) {
      clearInterval(this.pomodoroInterval);
      this.pomodoroInterval = null;
    }
    this.pomodoroState = 'idle';
    this.pomodoroSecondsLeft = this.pomodoroWorkMinutes * 60;
  }

  private onPomodoroWorkFinished(): void {
    this.pomodoroCyclesCompleted += 1;
    this.goalToastMessage = `🍅 Ciclo Pomodoro concluído! Hora de uma pausa revigorante de ${this.pomodoroBreakMinutes} min.`;
    setTimeout(() => {
      this.goalToastMessage = null;
    }, 6000);
    this.startPomodoroBreak();
  }

  private onPomodoroBreakFinished(): void {
    this.goalToastMessage = `✨ Pausa concluída! Pronto para o próximo ciclo de escrita?`;
    setTimeout(() => {
      this.goalToastMessage = null;
    }, 5000);
    this.startPomodoroWork();
  }

  /**
   * Verifica se a meta diária foi atingida e emite um alerta inspirador.
   */
  checkDailyGoalAchieved(totalWordsToday: number): void {
    if (!this.dailyGoalAlertEnabled) return;
    const today = new Date().toISOString().slice(0, 10);
    if (this.goalCelebratedDateIso === today) return;

    if (totalWordsToday >= this.dailyGoalWords) {
      this.goalCelebratedDateIso = today;
      this.goalToastMessage = `🎉 Parabéns! Você atingiu sua meta de ${this.dailyGoalWords.toLocaleString()} palavras de hoje!`;
      setTimeout(() => {
        this.goalToastMessage = null;
      }, 7000);
    }
  }

  dismissToast(): void {
    this.goalToastMessage = null;
  }
}

export const focusPreferences = new FocusPreferencesStore();
