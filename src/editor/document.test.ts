import { describe, it, expect } from 'vitest';
import {
  parseDocument,
  serializeDocument,
  splitFrontmatter,
  countWords,
  countChars,
} from './document';

describe('document (Modelo Markdown + Frontmatter YAML M1-05)', () => {
  it('separa texto sem frontmatter mantendo todo o corpo', () => {
    const raw = '# Capítulo 1\n\nEra uma noite chuvosa.';
    const split = splitFrontmatter(raw);
    expect(split.frontmatterText).toBeNull();
    expect(split.body).toBe(raw);

    const doc = parseDocument(raw);
    expect(doc.body).toBe(raw);
    expect(doc.title).toBeUndefined();
    expect(countWords(doc.body)).toBe(7);
  });

  it('faz parse de frontmatter padrão e preserva chaves desconhecidas', () => {
    const raw = `---
id: "scene-42"
title: "O Ponto de Virada"
type: "scene"
status: "draft"
tags:
  - suspense
  - climax
custom_flag: true
author_notes: "Revisar diálogo no final"
word_target: 1500
---

Arthur caminhou até a janela e observou o horizonte.`;

    const doc = parseDocument(raw);
    expect(doc.id).toBe('scene-42');
    expect(doc.title).toBe('O Ponto de Virada');
    expect(doc.type).toBe('scene');
    expect(doc.status).toBe('draft');
    expect(doc.tags).toEqual(['suspense', 'climax']);

    // Preservação de chaves desconhecidas
    expect(doc.extra.custom_flag).toBe(true);
    expect(doc.extra.author_notes).toBe('Revisar diálogo no final');
    expect(doc.extra.word_target).toBe(1500);

    // Contagem de palavras ignora frontmatter
    expect(countWords(doc.body)).toBe(9);
    expect(countChars(doc.body)).toBe(52);
  });

  it('realiza roundtrip serialization mantendo campos desconhecidos intactos', () => {
    const doc = {
      title: 'Cena de Teste',
      type: 'scene',
      status: 'final',
      tags: ['acao'],
      extra: {
        plugin_id: 'plugin-v1',
        score: 99,
        active: false,
      },
      body: 'Texto da cena com narrativa.',
    };

    const serialized = serializeDocument(doc);
    expect(serialized).toContain('title: "Cena de Teste"');
    expect(serialized).toContain('plugin_id: "plugin-v1"');
    expect(serialized).toContain('score: 99');
    expect(serialized).toContain('active: false');

    const reparsed = parseDocument(serialized);
    expect(reparsed.title).toBe(doc.title);
    expect(reparsed.status).toBe(doc.status);
    expect(reparsed.tags).toEqual(doc.tags);
    expect(reparsed.extra.plugin_id).toBe('plugin-v1');
    expect(reparsed.extra.score).toBe(99);
    expect(reparsed.extra.active).toBe(false);
    expect(reparsed.body).toBe(doc.body);
  });

  it('não confunde divisores horizontais --- no corpo com o delimitador de frontmatter', () => {
    const raw = `---
title: "Com Divisores"
---

Seção 1

---

Seção 2`;

    const doc = parseDocument(raw);
    expect(doc.title).toBe('Com Divisores');
    expect(doc.body).toContain('---');
    expect(doc.body).toContain('Seção 2');
    expect(countWords(doc.body)).toBe(5); // "Seção", "1", "---", "Seção", "2"
  });
});
