import './design';
import { mount } from 'svelte';
import App from './App.svelte';
import { installEscapeGuard } from './ui/escape-guard';

// Prevents Escape from leaving macOS fullscreen when it only dismisses an overlay.
installEscapeGuard();

const appElement = document.getElementById('app');

if (!appElement) {
  throw new Error('Application mount point (#app) not found in DOM');
}

const app = mount(App, {
  target: appElement,
});

export default app;
