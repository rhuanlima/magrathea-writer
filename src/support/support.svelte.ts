/**
 * Store reativa do modal de Apoio ao Projeto (M9-07).
 */

class SupportStore {
  isOpen = $state(false);

  open(): void {
    this.isOpen = true;
  }

  close(): void {
    this.isOpen = false;
  }

  toggle(): void {
    this.isOpen = !this.isOpen;
  }
}

export const supportStore = new SupportStore();
