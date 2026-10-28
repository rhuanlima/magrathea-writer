import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';

describe('M0-04: Design Tokens & OFL Embedded Fonts', () => {
  const tokensCssPath = path.resolve(__dirname, './tokens.css');
  const fontsCssPath = path.resolve(__dirname, './fonts.css');
  const fontsDir = path.resolve(__dirname, './fonts');

  it('declares all mandatory light and dark color tokens from DESIGN.md', () => {
    const css = fs.readFileSync(tokensCssPath, 'utf-8');

    // Light mode tokens (Pure White Zen)
    expect(css).toContain('--color-paper-base: #FFFFFF');
    expect(css).toContain('--color-paper-surface: #F8F9FA');
    expect(css).toContain('--color-ink-charcoal: #111827');
    expect(css).toContain('--color-olive: #5E6F45');
    expect(css).toContain('--color-bronze-text: #7A6230');
    expect(css).toContain('--color-bronze: #C4A468');

    // Dark mode tokens
    expect(css).toContain('--color-paper-base: #1B1D19');
    expect(css).toContain('--color-paper-surface: #23261F');
    expect(css).toContain('--color-ink-charcoal: #E9E6DA');
    expect(css).toContain('--color-olive: #8AA66A');
    expect(css).toContain('--color-bronze: #D4B97F');
  });

  it('declares spacing and roundedness scale tokens', () => {
    const css = fs.readFileSync(tokensCssPath, 'utf-8');

    expect(css).toContain('--space-xs: 0.25rem');
    expect(css).toContain('--space-sm: 0.5rem');
    expect(css).toContain('--space-md: 1rem');
    expect(css).toContain('--space-lg: 1.5rem');
    expect(css).toContain('--gutter: 1rem');
    expect(css).toContain('--margin: 1rem');

    expect(css).toContain('--rounded-sm: 0.125rem');
    expect(css).toContain('--rounded-default: 0.25rem');
    expect(css).toContain('--rounded-lg: 0.5rem');
    expect(css).toContain('--rounded-full: 9999px');
  });

  it('embeds only local font files without any external font URLs', () => {
    const fontsCss = fs.readFileSync(fontsCssPath, 'utf-8');

    expect(fontsCss).toContain("font-family: 'EB Garamond'");
    expect(fontsCss).toContain("font-family: 'Plus Jakarta Sans'");
    expect(fontsCss).toContain("font-family: 'JetBrains Mono'");

    // Must strictly load from local files
    expect(fontsCss).not.toContain('http://');
    expect(fontsCss).not.toContain('https://');
    expect(fontsCss).not.toContain('fonts.googleapis.com');
  });

  it('verifies all referenced font files exist locally on disk', () => {
    const expectedFonts = [
      'eb-garamond-latin-400-normal.woff2',
      'eb-garamond-latin-400-italic.woff2',
      'eb-garamond-latin-500-normal.woff2',
      'eb-garamond-latin-600-normal.woff2',
      'plus-jakarta-sans-latin-400-normal.woff2',
      'plus-jakarta-sans-latin-500-normal.woff2',
      'plus-jakarta-sans-latin-600-normal.woff2',
      'jetbrains-mono-latin-400-normal.woff2',
    ];

    for (const font of expectedFonts) {
      const fontFilePath = path.join(fontsDir, font);
      expect(fs.existsSync(fontFilePath), `Missing font file: ${font}`).toBe(true);
      const stat = fs.statSync(fontFilePath);
      expect(stat.size).toBeGreaterThan(1000); // verify non-empty binary
    }
  });
});
