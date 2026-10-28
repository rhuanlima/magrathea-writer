import { describe, it, expect } from 'vitest';
import { createCommandPaletteStore } from './palette.svelte';

describe('M9-01: Command Palette Store', () => {
  it('deve inicializar fechada', () => {
    const store = createCommandPaletteStore();
    expect(store.isOpen).toBe(false);
  });

  it('deve alternar estado de abertura e fechamento', () => {
    const store = createCommandPaletteStore();
    store.open();
    expect(store.isOpen).toBe(true);

    store.close();
    expect(store.isOpen).toBe(false);

    store.toggle();
    expect(store.isOpen).toBe(true);

    store.toggle();
    expect(store.isOpen).toBe(false);
  });
});
