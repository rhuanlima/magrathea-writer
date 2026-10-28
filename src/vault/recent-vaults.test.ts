import { describe, it, expect, beforeEach } from 'vitest';
import { getRecentVaults, addRecentVault, removeRecentVault } from './recent-vaults';

describe('Recent Vaults Manager', () => {
  let store: Record<string, string> = {};

  beforeEach(() => {
    store = {};
    const mockStorage = {
      getItem: (key: string) => store[key] || null,
      setItem: (key: string, value: string) => {
        store[key] = value;
      },
      removeItem: (key: string) => {
        delete store[key];
      },
      clear: () => {
        store = {};
      },
    };
    Object.defineProperty(globalThis, 'localStorage', {
      value: mockStorage,
      writable: true,
      configurable: true,
    });
  });

  it('retorna lista vazia inicialmente', () => {
    expect(getRecentVaults()).toEqual([]);
  });

  it('adiciona um cofre à lista de recentes', () => {
    addRecentVault({ path: '/romances/duna', title: 'Duna' });
    const recents = getRecentVaults();
    expect(recents.length).toBe(1);
    expect(recents[0].title).toBe('Duna');
    expect(recents[0].path).toBe('/romances/duna');
  });

  it('evita duplicatas e mantém o mais recente no topo', () => {
    addRecentVault({ path: '/romances/duna', title: 'Duna' });
    addRecentVault({ path: '/romances/fundacao', title: 'Fundação' });
    addRecentVault({ path: '/romances/duna', title: 'Duna (Reaberto)' });

    const recents = getRecentVaults();
    expect(recents.length).toBe(2);
    expect(recents[0].path).toBe('/romances/duna');
    expect(recents[0].title).toBe('Duna (Reaberto)');
    expect(recents[1].path).toBe('/romances/fundacao');
  });

  it('remove um cofre da lista', () => {
    addRecentVault({ path: '/romances/duna', title: 'Duna' });
    removeRecentVault('/romances/duna');
    expect(getRecentVaults()).toEqual([]);
  });
});
