/**
 * Módulo de avaliação local e offline de força de senha para o Magrathea Writer.
 *
 * Implementa cálculo determinístico de entropia e pontuação de 0 (muito fraca) a 4 (forte),
 * sem nenhuma chamada de rede ou dependência externa.
 */

import type { TranslationKey } from '../i18n/types';

export type PasswordStrengthScore = 0 | 1 | 2 | 3 | 4;

export interface PasswordStrengthResult {
  score: PasswordStrengthScore;
  entropyBits: number;
  feedbackKey: TranslationKey;
  isAcceptable: boolean;
  colorVar: string;
}

const COMMON_PREDICTABLE_PATTERNS = [
  '123456',
  'password',
  'senha',
  'magrathea',
  'admin',
  'qwerty',
  'asdfgh',
  '12345678',
  '123456789',
  'livro',
  'romance',
  'autor',
];

/**
 * Avalia a força de uma frase-senha mestra localmente.
 */
export function assessPasswordStrength(password: string): PasswordStrengthResult {
  if (!password || password.length === 0) {
    return {
      score: 0,
      entropyBits: 0,
      feedbackKey: 'vault.create.strengthEmpty',
      isAcceptable: false,
      colorVar: 'var(--color-error)',
    };
  }

  const length = password.length;
  const lower = password.toLowerCase();

  // Verifica padrões fracos e comuns
  const hasCommonPattern = COMMON_PREDICTABLE_PATTERNS.some((pattern) =>
    lower.includes(pattern)
  );

  // Análise de pool de caracteres
  let poolSize = 0;
  const hasLower = /[a-z]/.test(password);
  const hasUpper = /[A-Z]/.test(password);
  const hasDigit = /[0-9]/.test(password);
  const hasSpecial = /[^a-zA-Z0-9\s]/.test(password);
  const hasWhitespace = /\s/.test(password);

  if (hasLower) poolSize += 26;
  if (hasUpper) poolSize += 26;
  if (hasDigit) poolSize += 10;
  if (hasSpecial) poolSize += 33;
  if (hasWhitespace) poolSize += 2;

  // Cálculo de entropia de Shannon simplificada
  let entropy = length * Math.log2(Math.max(poolSize, 2));

  // Penalidades para repetições sequenciais e padrões comuns
  if (/(.)\1{2,}/.test(password)) {
    entropy = Math.max(0, entropy - 10);
  }
  if (hasCommonPattern) {
    entropy = Math.max(0, entropy - 20);
  }

  // Bônus para passphrases longas separadas por espaços (estilo Diceware)
  const words = password.trim().split(/\s+/);
  if (words.length >= 4 && length >= 16) {
    entropy += 15;
  }

  let score: PasswordStrengthScore = 0;
  let feedbackKey: TranslationKey = 'vault.create.strengthVeryWeak';
  let colorVar = 'var(--color-error)';

  if (length < 10 || hasCommonPattern || entropy < 25) {
    score = 0;
    feedbackKey = 'vault.create.strengthVeryWeak';
    colorVar = 'var(--color-error)';
  } else if (length < 12 || entropy < 38) {
    score = 1;
    feedbackKey = 'vault.create.strengthWeak';
    colorVar = 'var(--color-error)';
  } else if (entropy < 52) {
    score = 2;
    feedbackKey = 'vault.create.strengthFair';
    colorVar = 'var(--color-bronze)';
  } else if (entropy < 68) {
    score = 3;
    feedbackKey = 'vault.create.strengthGood';
    colorVar = 'var(--color-olive)';
  } else {
    score = 4;
    feedbackKey = 'vault.create.strengthStrong';
    colorVar = 'var(--color-olive)';
  }

  // Uma senha é aceitável para criação se tiver no mínimo 10 caracteres e score >= 2
  const isAcceptable = length >= 10 && score >= 2;

  return {
    score,
    entropyBits: Math.round(entropy),
    feedbackKey,
    isAcceptable,
    colorVar,
  };
}
