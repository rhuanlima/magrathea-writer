import { describe, it, expect, vi } from 'vitest';
import { installEscapeGuard } from './escape-guard';

/** Dispatches a minimal keydown event (the unit tests run without a DOM). */
function press(target: EventTarget, key: string): Event {
  const event = new Event('keydown', { cancelable: true });
  Object.assign(event, { key });
  target.dispatchEvent(event);
  return event;
}

describe('installEscapeGuard', () => {
  it('marks Escape as handled so macOS does not leave fullscreen', () => {
    const target = new EventTarget();
    const dispose = installEscapeGuard(target);
    expect(press(target, 'Escape').defaultPrevented).toBe(true);
    dispose();
  });

  it('does not touch other keys', () => {
    const target = new EventTarget();
    const dispose = installEscapeGuard(target);
    expect(press(target, 'a').defaultPrevented).toBe(false);
    expect(press(target, 'Enter').defaultPrevented).toBe(false);
    dispose();
  });

  it('keeps component handlers working (does not stop propagation)', () => {
    const target = new EventTarget();
    const dispose = installEscapeGuard(target);
    const modalHandler = vi.fn();
    target.addEventListener('keydown', modalHandler);
    press(target, 'Escape');
    expect(modalHandler).toHaveBeenCalledTimes(1);
    dispose();
  });

  it('stops guarding after dispose', () => {
    const target = new EventTarget();
    const dispose = installEscapeGuard(target);
    dispose();
    expect(press(target, 'Escape').defaultPrevented).toBe(false);
  });
});
