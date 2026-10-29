import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { InactivityTracker } from './inactivity';

describe('InactivityTracker (Monitor de Inatividade)', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('inicia e dispara callback após expiração do timeout', () => {
    const onTimeout = vi.fn();
    const tracker = new InactivityTracker({
      timeoutMs: 1000,
      onTimeout,
    });

    tracker.start();
    expect(tracker.isActive()).toBe(true);
    expect(onTimeout).not.toHaveBeenCalled();

    vi.advanceTimersByTime(999);
    expect(onTimeout).not.toHaveBeenCalled();

    vi.advanceTimersByTime(2);
    expect(onTimeout).toHaveBeenCalledTimes(1);

    tracker.stop();
    expect(tracker.isActive()).toBe(false);
  });

  it('reinicia a contagem quando o método reset é chamado', () => {
    const onTimeout = vi.fn();
    const tracker = new InactivityTracker({
      timeoutMs: 1000,
      onTimeout,
    });

    tracker.start();
    vi.advanceTimersByTime(600);
    tracker.reset();

    vi.advanceTimersByTime(600);
    expect(onTimeout).not.toHaveBeenCalled();

    vi.advanceTimersByTime(401);
    expect(onTimeout).toHaveBeenCalledTimes(1);

    tracker.stop();
  });

  it('permite forçar o timeout imediatamente via triggerTimeout', () => {
    const onTimeout = vi.fn();
    const tracker = new InactivityTracker({
      timeoutMs: 60000,
      onTimeout,
    });

    tracker.start();
    tracker.triggerTimeout();
    expect(onTimeout).toHaveBeenCalledTimes(1);

    tracker.stop();
  });

  it('permite reconfigurar dinamicamente o tempo de timeout via setTimeoutMs', () => {
    const onTimeout = vi.fn();
    const tracker = new InactivityTracker({
      timeoutMs: 1000,
      onTimeout,
    });

    tracker.start();
    tracker.setTimeoutMs(500);

    vi.advanceTimersByTime(499);
    expect(onTimeout).not.toHaveBeenCalled();

    vi.advanceTimersByTime(2);
    expect(onTimeout).toHaveBeenCalledTimes(1);

    tracker.stop();
  });
});
