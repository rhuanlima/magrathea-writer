/**
 * Armazenamento local de histórico de cofres recentes acessados no Magrathea Writer.
 * Permite ao autor reabrir seus projetos em 1 clique sem precisar digitar o caminho novamente.
 */

export interface RecentVault {
  path: string;
  title: string;
  lastOpened: string;
}

const STORAGE_KEY = 'magrathea_recent_vaults';
const MAX_RECENTS = 6;

export function getRecentVaults(): RecentVault[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      return parsed;
    }
    return [];
  } catch {
    return [];
  }
}

export function addRecentVault(vault: { path: string; title: string }): void {
  if (!vault.path) return;
  try {
    const recents = getRecentVaults().filter((r) => r.path !== vault.path);
    const updated: RecentVault[] = [
      {
        path: vault.path,
        title: vault.title || 'Manuscrito',
        lastOpened: new Date().toISOString(),
      },
      ...recents,
    ].slice(0, MAX_RECENTS);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  } catch {
    // Falha silenciosa em ambientes sem localStorage
  }
}

export function removeRecentVault(path: string): void {
  try {
    const recents = getRecentVaults().filter((r) => r.path !== path);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(recents));
  } catch {
    // Falha silenciosa
  }
}
