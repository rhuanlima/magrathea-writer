import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { AutosaveManager } from './autosave';

describe('AutosaveManager (Escrita Atômica Cifrada com Debounce)', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('inicia no estado saved e muda para unsaved ao receber alterações', () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const statuses: string[] = [];

    const manager = new AutosaveManager({
      delayMs: 500,
      onSave,
      onStatusChange: (s) => statuses.push(s),
    });

    expect(manager.getStatus()).toBe('saved');

    manager.notifyChange('Capítulo 1...');
    expect(manager.getStatus()).toBe('unsaved');
    expect(statuses).toEqual(['unsaved']);

    manager.dispose();
  });

  it('aplica debounce salvando apenas após o tempo de inatividade', async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const manager = new AutosaveManager({
      delayMs: 500,
      onSave,
    });

    manager.notifyChange('Texto 1');
    vi.advanceTimersByTime(200);

    manager.notifyChange('Texto 2');
    vi.advanceTimersByTime(200);

    manager.notifyChange('Texto 3 final');
    expect(onSave).not.toHaveBeenCalled();

    // Avança os 500ms finais
    await vi.advanceTimersByTimeAsync(500);

    expect(onSave).toHaveBeenCalledTimes(1);
    expect(onSave).toHaveBeenCalledWith('Texto 3 final');
    expect(manager.getStatus()).toBe('saved');

    manager.dispose();
  });

  it('força o salvamento imediato via flush', async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const manager = new AutosaveManager({
      delayMs: 2000,
      onSave,
    });

    manager.notifyChange('Conteúdo imediato');
    expect(onSave).not.toHaveBeenCalled();

    await manager.flush();
    expect(onSave).toHaveBeenCalledTimes(1);
    expect(onSave).toHaveBeenCalledWith('Conteúdo imediato');
    expect(manager.getStatus()).toBe('saved');

    manager.dispose();
  });

  it('transiciona para estado error caso a gravação atômica falhe', async () => {
    const onSave = vi.fn().mockRejectedValue(new Error('Falha no disco'));
    const manager = new AutosaveManager({
      delayMs: 500,
      onSave,
    });

    manager.notifyChange('Texto que falha');
    await vi.advanceTimersByTimeAsync(500);

    expect(manager.getStatus()).toBe('error');

    manager.dispose();
  });
});
