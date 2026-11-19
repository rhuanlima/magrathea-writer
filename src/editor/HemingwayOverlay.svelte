<script lang="ts">
  import { onDestroy } from 'svelte';
  import { fade } from 'svelte/transition';
  import { t } from '../i18n';

  interface Props {
    active: boolean;
    cursorX: number;
    cursorY: number;
    cursorHeight?: number;
    hasCursorCoords?: boolean;
  }

  let {
    active,
    cursorX,
    cursorY,
    cursorHeight = 24,
    hasCursorCoords = true,
  }: Props = $props();

  let showExitHint = $state(false);
  let hintTimer: ReturnType<typeof setTimeout> | null = null;

  const coneScale = $derived(Math.max(0.85, Math.min(1.6, cursorHeight / 24)));

  $effect(() => {
    if (active) {
      showExitHint = true;
      if (hintTimer) {
        clearTimeout(hintTimer);
      }
      hintTimer = setTimeout(() => {
        showExitHint = false;
        hintTimer = null;
      }, 2000);
    } else {
      showExitHint = false;
      if (hintTimer) {
        clearTimeout(hintTimer);
        hintTimer = null;
      }
    }
  });

  onDestroy(() => {
    if (hintTimer) {
      clearTimeout(hintTimer);
      hintTimer = null;
    }
  });
</script>

{#if active}
  <div
    class="hemingway-overlay"
    aria-hidden="true"
    in:fade={{ duration: 320 }}
    out:fade={{ duration: 200 }}
  >
    <svg class="hemingway-mask-svg" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <filter id="hemingway-cone-blur" x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur stdDeviation="24" />
        </filter>
        <filter id="hemingway-core-blur" x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur stdDeviation="11" />
        </filter>
        <linearGradient id="hemingway-cone-fade" x1="0" y1="1" x2="0" y2="0">
          <stop offset="0%" stop-color="#000000" />
          <stop offset="52%" stop-color="#000000" />
          <stop offset="78%" stop-color="#4a4a4a" />
          <stop offset="100%" stop-color="#ffffff" />
        </linearGradient>
        <mask id="hemingway-spotlight-mask" maskUnits="userSpaceOnUse">
          <rect x="0" y="0" width="100%" height="100%" fill="#ffffff" />
          {#if hasCursorCoords}
            <g
              class="hemingway-spotlight-group"
              style="transform: translate({cursorX}px, {cursorY}px) scale({coneScale});"
            >
              <path
                d="M -46 8 L -290 -145 Q 0 -178 290 -145 L 46 8 Q 0 20 -46 8 Z"
                fill="url(#hemingway-cone-fade)"
                filter="url(#hemingway-cone-blur)"
              />
              <ellipse
                cx="0"
                cy="-16"
                rx="115"
                ry="32"
                fill="#000000"
                filter="url(#hemingway-core-blur)"
              />
            </g>
          {/if}
        </mask>
      </defs>

      <rect
        x="0"
        y="0"
        width="100%"
        height="100%"
        fill="rgba(6, 8, 12, 0.87)"
        mask="url(#hemingway-spotlight-mask)"
      />
    </svg>

    {#if hasCursorCoords}
      <div
        class="hemingway-base-pill"
        style="transform: translate3d(calc({cursorX}px - 50%), calc({cursorY}px + 5px), 0);"
      ></div>
    {/if}

    {#if showExitHint}
      <div class="hemingway-exit-hint" out:fade={{ duration: 250 }}>
        {t('editor.hemingwayExitHint')}
      </div>
    {/if}
  </div>
{/if}

<style>
  .hemingway-overlay {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    z-index: 950;
    overflow: hidden;
  }

  .hemingway-mask-svg {
    width: 100%;
    height: 100%;
    display: block;
    pointer-events: none;
  }

  .hemingway-spotlight-group {
    transition: transform 55ms cubic-bezier(0.22, 1, 0.36, 1);
    will-change: transform;
  }

  .hemingway-base-pill {
    position: fixed;
    top: 0;
    left: 0;
    width: 24px;
    height: 3px;
    border-radius: 9999px;
    background: rgba(246, 226, 179, 0.88);
    box-shadow:
      0 0 10px 1px rgba(246, 226, 179, 0.55),
      0 0 2px rgba(255, 250, 235, 0.9);
    pointer-events: none;
    transition: transform 55ms cubic-bezier(0.22, 1, 0.36, 1);
    will-change: transform;
  }

  .hemingway-exit-hint {
    position: fixed;
    top: 20px;
    left: 50%;
    transform: translateX(-50%);
    padding: 6px 14px;
    border-radius: 9999px;
    background: rgba(17, 20, 26, 0.78);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: rgba(241, 245, 249, 0.88);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.2px;
    backdrop-filter: blur(8px);
    pointer-events: none;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.28);
  }
</style>
