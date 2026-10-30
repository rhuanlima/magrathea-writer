/**
 * Escape key guard.
 *
 * On macOS, an Escape key press that the web page does not mark as handled
 * travels up the native responder chain (`cancelOperation:`) and makes the
 * window leave fullscreen. Modals, drawers and the zen mode already react to
 * Escape on their own, but none of them calls `preventDefault()`, so the app
 * was being kicked out of fullscreen every time the user dismissed an overlay.
 *
 * This guard runs in the capture phase, before any component handler, and
 * marks every Escape keydown as handled. It never stops propagation, so the
 * existing handlers (close modal, exit zen, close autocomplete...) still run.
 */

/**
 * Installs the global Escape guard on the given target.
 *
 * @param target - Event target that receives keyboard events (defaults to `window`).
 * @returns A function that removes the guard.
 */
export function installEscapeGuard(
  target: Pick<EventTarget, 'addEventListener' | 'removeEventListener'> = window
): () => void {
  const handler = (event: Event): void => {
    if ((event as KeyboardEvent).key === 'Escape') {
      event.preventDefault();
    }
  };

  target.addEventListener('keydown', handler, { capture: true });
  return () => target.removeEventListener('keydown', handler, { capture: true });
}
