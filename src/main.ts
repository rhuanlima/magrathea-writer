import { mount } from 'svelte';
import App from './App.svelte';

const appElement = document.getElementById('app');

if (!appElement) {
  throw new Error('Application mount point (#app) not found in DOM');
}

const app = mount(App, {
  target: appElement,
});

export default app;
