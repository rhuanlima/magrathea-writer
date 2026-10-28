import { describe, it, expect } from 'vitest';
import { assessPasswordStrength } from './password-strength';

describe('password-strength (Medidor local de entropia)', () => {
  it('identifica senhas vazias como inaceitáveis e curtas com score baixo sem bloquear uso', () => {
    const empty = assessPasswordStrength('');
    expect(empty.score).toBe(0);
    expect(empty.isAcceptable).toBe(false);

    const short = assessPasswordStrength('12345');
    expect(short.score).toBe(0);
    expect(short.isAcceptable).toBe(true);
  });

  it('penaliza padrões comuns e óbvios sem bloquear uso soberano do autor', () => {
    const predictable = assessPasswordStrength('senha12345678');
    expect(predictable.score).toBe(0);
    expect(predictable.isAcceptable).toBe(true);

    const magrathea = assessPasswordStrength('magrathea-password');
    expect(magrathea.score).toBeLessThanOrEqual(1);
    expect(magrathea.isAcceptable).toBe(true);
  });

  it('aceita senhas com boa entropia e variedade de caracteres', () => {
    const fair = assessPasswordStrength('ChaveForteValida!2026');
    expect(fair.score).toBeGreaterThanOrEqual(2);
    expect(fair.isAcceptable).toBe(true);
  });

  it('classifica frases longas estilo diceware como muito fortes', () => {
    const diceware = assessPasswordStrength('correto cavalo bateria grampo universo');
    expect(diceware.score).toBe(4);
    expect(diceware.isAcceptable).toBe(true);
    expect(diceware.colorVar).toBe('var(--color-olive)');
  });
});
