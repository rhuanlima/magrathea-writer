import { describe, it, expect } from 'vitest';
import { supportStore } from './support.svelte';

describe('Support Store', () => {
  it('initializes with isOpen false', () => {
    expect(supportStore.isOpen).toBe(false);
  });

  it('can open, close and toggle modal state', () => {
    supportStore.open();
    expect(supportStore.isOpen).toBe(true);

    supportStore.close();
    expect(supportStore.isOpen).toBe(false);

    supportStore.toggle();
    expect(supportStore.isOpen).toBe(true);
    supportStore.toggle();
    expect(supportStore.isOpen).toBe(false);
  });
});
