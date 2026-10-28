/**
 * Store global unificada de Preferências do Magrathea Writer (Ajustes 3 e 3.1).
 *
 * Gerencia o modal central de preferências acessível via:
 * - Botão na barra de status inferior (`⚙️ Preferências`)
 * - Atalho de teclado do sistema (`Cmd+,` / `Ctrl+,`)
 * - Menu nativo do sistema operacional (`Magrathea Writer > Preferências...`)
 * - Paleta de Comandos (`Cmd+K`)
 */

import { editorPreferences } from '../editor/editor-preferences.svelte';

export type PreferencesTab = 'editor' | 'security' | 'focus' | 'system';

class PreferencesStore {
  isOpen = $state(false);
  activeTab = $state<PreferencesTab>('editor');

  open(tab: PreferencesTab = 'editor'): void {
    this.activeTab = tab;
    this.isOpen = true;
    // Sincroniza também com a store antiga caso seja consultada
    editorPreferences.openModal();
  }

  close(): void {
    this.isOpen = false;
    editorPreferences.closeModal();
  }

  setTab(tab: PreferencesTab): void {
    this.activeTab = tab;
  }
}

export const preferences = new PreferencesStore();
