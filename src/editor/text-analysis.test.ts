import { describe, it, expect } from 'vitest';
import {
  analyzeProseStyle,
  extractWords,
  extractSentences,
  stripFrontmatter,
} from './text-analysis';

describe('text-analysis: Análise Estilométrica Literária', () => {
  it('remove frontmatter corretamente mantendo apenas a prosa', () => {
    const raw = `---
title: Capítulo 1
status: draft
---
O vento uivava calmamente na colina.`;

    expect(stripFrontmatter(raw)).toBe('O vento uivava calmamente na colina.');
  });

  it('extrai palavras e ignora pontuações', () => {
    const text = '— Não vá lá, Arthur! — disse Ford, calmamente.';
    const words = extractWords(text);
    const sentences = extractSentences(text);

    expect(sentences.length).toBeGreaterThan(0);
    expect(words).toContain('não');
    expect(words).toContain('lá');
    expect(words).toContain('arthur');
    expect(words).toContain('disse');
    expect(words).toContain('ford');
    expect(words).toContain('calmamente');
  });

  it('identifica corretamente advérbios em -mente e outros advérbios comuns', () => {
    const text =
      'Ele caminhava silenciosamente pela rua. Não queria voltar para casa tão cedo.';
    const analysis = analyzeProseStyle(text);

    expect(analysis.menteAdverbsCount).toBe(1); // silenciosamente
    expect(analysis.adverbsCount).toBeGreaterThanOrEqual(3); // silenciosamente, não, tão, cedo
    expect(analysis.topAdverbs.some((a) => a.word === 'silenciosamente')).toBe(true);
  });

  it('não confunde falsos advérbios como semente ou mente com advérbios reais', () => {
    const text = 'A mente humana guarda uma semente de esperança no ambiente quente.';
    const analysis = analyzeProseStyle(text);

    expect(analysis.menteAdverbsCount).toBe(0);
  });

  it('detecta verbos de diálogo e elocução para monitorar fórmulas narrativas', () => {
    const text = `
— Onde estamos? — perguntou Arthur com medo.
— Na nave de demolição Vogon — respondeu Ford rapidamente.
— Eles vão nos destruir? — indagou o amigo.
— Provavelmente — murmurou Ford.
    `;
    const analysis = analyzeProseStyle(text);

    expect(analysis.dialogueVerbsCount).toBeGreaterThanOrEqual(4);
    const dialogueWords = analysis.topDialogueVerbs.map((v) => v.word);
    expect(dialogueWords).toContain('perguntou');
    expect(dialogueWords).toContain('respondeu');
    expect(dialogueWords).toContain('indagou');
    expect(dialogueWords).toContain('murmurou');
  });

  it('calcula a riqueza lexical (Type-Token Ratio) e distribuição de frases', () => {
    const text = 'O sol brilha. O céu está azul e a floresta canta em harmonia perfeita com a vida selvagem.';
    const analysis = analyzeProseStyle(text);

    expect(analysis.totalWords).toBeGreaterThan(0);
    expect(analysis.vocabularyRichnessPercent).toBeGreaterThan(50);
    expect(analysis.avgWordsPerSentence).toBeGreaterThan(0);
    expect(analysis.shortSentencesCount).toBeGreaterThanOrEqual(1); // 'O sol brilha.'
  });

  it('computa a proporção entre diálogo e narrativa', () => {
    const text = `
— Isto não faz sentido algum — disse o viajante.
— Nada no universo faz, meu caro amigo.
O vento continuava soprando pelas montanhas frias daquele planeta esquecido.
    `;
    const analysis = analyzeProseStyle(text);

    expect(analysis.dialoguePercent).toBeGreaterThan(0);
    expect(analysis.narrativePercent).toBeGreaterThan(0);
    expect(analysis.dialoguePercent + analysis.narrativePercent).toBe(100);
  });
});
