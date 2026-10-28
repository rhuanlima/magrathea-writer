/**
 * Store reativa da Paleta de Comandos Global (M9-01).
 *
 * Controla o estado de visibilidade da paleta acionada por Cmd+K / Ctrl+K.
 */

export function createCommandPaletteStore() {
  let isOpen = $state<boolean>(false);

  function open() {
    isOpen = true;
  }

  function close() {
    isOpen = false;
  }

  function toggle() {
    isOpen = !isOpen;
  }

  return {
    get isOpen() {
      return isOpen;
    },
    open,
    close,
    toggle,
  };
}

export const commandPaletteStore = createCommandPaletteStore();
