import { describe, it, expect } from 'vitest';
import { typewriterAudio, TypewriterAudioEngine } from './typewriter-audio';

describe('TypewriterAudioEngine: Sons de Máquina de Escrever', () => {
  it('instancia o motor sem erros', () => {
    expect(typewriterAudio).toBeDefined();
    expect(typewriterAudio).toBeInstanceOf(TypewriterAudioEngine);
  });

  it('executa métodos de som sem falhas mesmo em ambiente sem Web Audio (safe no-op)', () => {
    expect(() => typewriterAudio.playKeyClick(0.5)).not.toThrow();
    expect(() => typewriterAudio.playSpacebar(0.5)).not.toThrow();
    expect(() => typewriterAudio.playEnterReturn(0.5)).not.toThrow();
    expect(() => typewriterAudio.playBackspace(0.5)).not.toThrow();
  });

  it('normaliza volumes inválidos ou fora dos limites sem falhar', () => {
    expect(() => typewriterAudio.playKeyClick(-1)).not.toThrow();
    expect(() => typewriterAudio.playKeyClick(5)).not.toThrow();
    expect(() => typewriterAudio.playEnterReturn(0)).not.toThrow();
  });
});
