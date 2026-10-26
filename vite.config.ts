import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

/**
 * Strips external documentation URLs embedded by framework error messages
 * ensuring absolute offline purity in production bundles.
 */
function sanitizeBundleUrls() {
  return {
    name: 'sanitize-bundle-urls',
    transform(code: string) {
      if (code.includes('https://svelte.dev/e/')) {
        return {
          code: code.replaceAll('https://svelte.dev/e/', 'svelte-err://'),
          map: null,
        };
      }
      return null;
    },
  };
}

export default defineConfig({
  plugins: [svelte(), sanitizeBundleUrls()],
  clearScreen: false,
  css: {
    postcss: {},
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  test: {
    environment: 'node',
    globals: true,
  },
});
