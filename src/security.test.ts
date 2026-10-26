import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';

// Allowed XML namespace URIs that may appear in SVG/HTML serializers
const ALLOWED_NAMESPACES = new Set([
  'http://www.w3.org/2000/svg',
  'http://www.w3.org/1999/xhtml',
  'http://www.w3.org/1999/xlink',
  'http://www.w3.org/XML/1998/namespace',
  'http://ipc.localhost',
]);

function getFilesRecursively(dir: string): string[] {
  if (!fs.existsSync(dir)) return [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  let files: string[] = [];

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files = files.concat(getFilesRecursively(fullPath));
    } else {
      files.push(fullPath);
    }
  }

  return files;
}

describe('M0-03: Security, CSP & Offline Confinement', () => {
  it('enforces strict CSP configuration in tauri.conf.json', () => {
    const tauriConfPath = path.resolve(__dirname, '../src-tauri/tauri.conf.json');
    const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'));

    const csp = tauriConf.app?.security?.csp;
    expect(csp).toBeDefined();
    expect(typeof csp).toBe('string');

    // Strict security requirements from SECURITY.md §2.1
    expect(csp).toContain("default-src 'none'");
    expect(csp).toContain("script-src 'self'");
    expect(csp).toContain("connect-src ipc: http://ipc.localhost");
    expect(csp).toContain("object-src 'none'");
    expect(csp).toContain("base-uri 'none'");
    expect(csp).toContain("form-action 'none'");

    // Must NOT contain unsafe-eval or external remote origins
    expect(csp).not.toContain('unsafe-eval');
    expect(csp).not.toMatch(/https?:\/\/(?!ipc\.localhost)[a-zA-Z0-9.-]+/);
  });

  it('enforces strict CSP meta tag in index.html', () => {
    const indexPath = path.resolve(__dirname, '../index.html');
    const indexHtml = fs.readFileSync(indexPath, 'utf-8');

    expect(indexHtml).toContain('http-equiv="Content-Security-Policy"');
    expect(indexHtml).toContain("default-src 'none'");
    expect(indexHtml).toContain("script-src 'self'");
  });

  it('restricts capabilities to minimal safe default in capabilities/default.json', () => {
    const capPath = path.resolve(
      __dirname,
      '../src-tauri/capabilities/default.json'
    );
    const cap = JSON.parse(fs.readFileSync(capPath, 'utf-8'));

    expect(cap.permissions).toBeDefined();
    expect(Array.isArray(cap.permissions)).toBe(true);

    // Only minimal core capabilities must be present
    expect(cap.permissions).toEqual(['core:default']);

    // Ensure no dangerous or remote permissions
    const serialized = JSON.stringify(cap.permissions);
    expect(serialized).not.toContain('http');
    expect(serialized).not.toContain('websocket');
    expect(serialized).not.toContain('network');
  });

  it('verifies production bundle does not contain external remote URLs', () => {
    const distDir = path.resolve(__dirname, '../dist');
    if (!fs.existsSync(distDir)) {
      // If dist does not exist yet in test runner, skip or require build
      return;
    }

    const bundleFiles = getFilesRecursively(distDir).filter((file) =>
      /\.(js|css|html)$/.test(file)
    );

    const urlRegex = /https?:\/\/[a-zA-Z0-9.-]+(?::[0-9]+)?(?:\/[^\s"')`]*)?/g;
    const foundDisallowedUrls: { file: string; url: string }[] = [];

    for (const file of bundleFiles) {
      const content = fs.readFileSync(file, 'utf-8');
      const matches = content.match(urlRegex) || [];

      for (const match of matches) {
        // Strip trailing punctuation like semicolons, commas, parenthesis
        const cleanUrl = match.replace(/[),;.]+$/, '');
        if (!ALLOWED_NAMESPACES.has(cleanUrl)) {
          foundDisallowedUrls.push({
            file: path.relative(path.resolve(__dirname, '..'), file),
            url: cleanUrl,
          });
        }
      }
    }

    expect(
      foundDisallowedUrls,
      `Disallowed external URLs found in production bundle: ${JSON.stringify(foundDisallowedUrls, null, 2)}`
    ).toEqual([]);
  });
});
