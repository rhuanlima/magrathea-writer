import { describe, it, expect, beforeEach } from 'vitest';
import { focusPreferences } from './focus-preferences.svelte';

describe('FocusPreferencesStore (Foco, Pausas, Pomodoro e Bloqueio)', () => {
  beforeEach(() => {
    focusPreferences.autoLockMinutes = 30;
    focusPreferences.pomodoroEnabled = false;
    focusPreferences.pomodoroWorkMinutes = 25;
    focusPreferences.pomodoroBreakMinutes = 5;
    focusPreferences.dailyGoalWords = 1000;
    focusPreferences.dailyGoalAlertEnabled = true;
    focusPreferences.goalToastMessage = null;
  });

  it('permite ajustar tempo de bloqueio por inatividade ou desligar', () => {
    expect(focusPreferences.autoLockMinutes).toBe(30);
    focusPreferences.setAutoLockMinutes(0); // Desligado
    expect(focusPreferences.autoLockMinutes).toBe(0);
    focusPreferences.setAutoLockMinutes(15);
    expect(focusPreferences.autoLockMinutes).toBe(15);
  });

  it('gerencia ciclo e ativação de Pomodoro', () => {
    expect(focusPreferences.pomodoroEnabled).toBe(false);
    focusPreferences.togglePomodoro();
    expect(focusPreferences.pomodoroEnabled).toBe(true);
    expect(focusPreferences.pomodoroState).toBe('work');

    focusPreferences.setPomodoroWorkMinutes(45);
    expect(focusPreferences.pomodoroWorkMinutes).toBe(45);

    focusPreferences.setPomodoroBreakMinutes(10);
    expect(focusPreferences.pomodoroBreakMinutes).toBe(10);

    focusPreferences.togglePomodoro();
    expect(focusPreferences.pomodoroEnabled).toBe(false);
    expect(focusPreferences.pomodoroState).toBe('idle');
  });

  it('monitora e comemora meta diária de palavras com alerta comemorativo', () => {
    expect(focusPreferences.dailyGoalAlertEnabled).toBe(true);
    expect(focusPreferences.goalToastMessage).toBeNull();

    // Abaixo da meta
    focusPreferences.checkDailyGoalAchieved(500);
    expect(focusPreferences.goalToastMessage).toBeNull();

    // Atingiu a meta
    focusPreferences.checkDailyGoalAchieved(1005);
    expect(focusPreferences.goalToastMessage).toContain('atingiu sua meta');

    focusPreferences.dismissToast();
    expect(focusPreferences.goalToastMessage).toBeNull();
  });
});
