/**
 * Motor de Áudio para Modo Máquina de Escrever (Typewriter).
 *
 * Utiliza gravações reais de teclas mecânicas integradas à Web Audio API:
 * - 100% offline, latência ultrabaixa e zero dependência externa (amostras embutidas em Base64)
 * - 4 variações de teclas gravadas para digitação orgânica e hiper-realista
 * - Tecla alfanumérica: disparo aleatório entre as 4 amostras com sutil modulação de pitch (±5%)
 * - Espaço: impacto de tecla real em registro mais encorpado + ressonância oca
 * - Enter: impacto mecânico analógico + sineta clássica de fim de linha ("Ding!")
 * - Backspace: impacto de catraca de recuo
 * - Fallback transparente para síntese procedural caso decodificação de áudio não esteja pronta
 */

import { TYPEWRITER_KEY_SAMPLES_BASE64 } from './typewriter-samples';

/**
 * Converte string base64 para ArrayBuffer de modo seguro e universal.
 */
function base64ToArrayBuffer(base64: string): ArrayBuffer {
  if (typeof window === 'undefined' && typeof Buffer !== 'undefined') {
    return Buffer.from(base64, 'base64').buffer;
  }
  const binaryString = atob(base64);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}

export class TypewriterAudioEngine {
  private audioCtx: AudioContext | null = null;
  private noiseBuffer: AudioBuffer | null = null;
  private sampleBuffers: AudioBuffer[] = [];
  private isDecodingSamples = false;

  /**
   * Inicializa o contexto de áudio sob demanda no primeiro evento do usuário.
   */
  private getContext(): AudioContext | null {
    if (typeof window === 'undefined') return null;

    if (!this.audioCtx) {
      const AudioCtxClass =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;

      if (!AudioCtxClass) return null;

      try {
        this.audioCtx = new AudioCtxClass();
      } catch {
        return null;
      }
    }

    if (this.audioCtx && this.audioCtx.state === 'suspended') {
      void this.audioCtx.resume();
    }

    if (this.audioCtx && !this.isDecodingSamples && this.sampleBuffers.length === 0) {
      this.decodeSamples(this.audioCtx);
    }

    return this.audioCtx;
  }

  /**
   * Decodifica as amostras reais de teclas para a memória.
   */
  private decodeSamples(ctx: AudioContext): void {
    if (this.isDecodingSamples) return;
    this.isDecodingSamples = true;

    if (!ctx.decodeAudioData) return;

    for (const b64 of TYPEWRITER_KEY_SAMPLES_BASE64) {
      try {
        const ab = base64ToArrayBuffer(b64);
        ctx.decodeAudioData(
          ab,
          (decoded) => {
            this.sampleBuffers.push(decoded);
          },
          () => {
            // Falha silenciosa na decodificação de amostra individual
          }
        );
      } catch {
        // Fallback silencioso
      }
    }
  }

  /**
   * Toca uma das amostras reais gravadas se disponíveis.
   */
  private playRecordedSample(
    ctx: AudioContext,
    volume: number,
    pitchMin = 0.95,
    pitchMax = 1.05
  ): boolean {
    if (this.sampleBuffers.length === 0) return false;

    try {
      const idx = Math.floor(Math.random() * this.sampleBuffers.length);
      const buffer = this.sampleBuffers[idx];
      const source = ctx.createBufferSource();
      source.buffer = buffer;

      const jitter = pitchMin + Math.random() * (pitchMax - pitchMin);
      source.playbackRate.setValueAtTime(jitter, ctx.currentTime);

      const gain = ctx.createGain();
      gain.gain.setValueAtTime(volume, ctx.currentTime);

      source.connect(gain);
      gain.connect(ctx.destination);

      source.start();
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Gera um buffer de ruído branco reutilizável de 0.2 segundos (fallback).
   */
  private getNoiseBuffer(ctx: AudioContext): AudioBuffer {
    if (!this.noiseBuffer || this.noiseBuffer.sampleRate !== ctx.sampleRate) {
      const bufferSize = Math.floor(ctx.sampleRate * 0.2);
      const buffer = ctx.createBuffer(1, bufferSize, ctx.sampleRate);
      const data = buffer.getChannelData(0);
      for (let i = 0; i < bufferSize; i++) {
        data[i] = Math.random() * 2 - 1;
      }
      this.noiseBuffer = buffer;
    }
    return this.noiseBuffer;
  }

  /**
   * Toca o clique mecânico característico de uma tecla alfanumérica.
   * Utiliza preferencialmente as gravações reais fornecidas pelo autor.
   */
  playKeyClick(volume = 0.6): void {
    const ctx = this.getContext();
    if (!ctx) return;

    try {
      const effectiveVol = Math.max(0, Math.min(volume, 1));

      // Tenta tocar uma amostra real gravada
      if (this.playRecordedSample(ctx, effectiveVol, 0.94, 1.06)) {
        return;
      }

      // Fallback para sintetizador procedural
      const now = ctx.currentTime;
      const noise = ctx.createBufferSource();
      noise.buffer = this.getNoiseBuffer(ctx);

      const filter = ctx.createBiquadFilter();
      filter.type = 'bandpass';
      const jitter = 0.88 + Math.random() * 0.24;
      filter.frequency.setValueAtTime(1800 * jitter, now);
      filter.Q.setValueAtTime(3.5, now);

      const noiseGain = ctx.createGain();
      noiseGain.gain.setValueAtTime(0.4 * effectiveVol, now);
      noiseGain.gain.exponentialRampToValueAtTime(0.001, now + 0.035);

      noise.connect(filter);
      filter.connect(noiseGain);
      noiseGain.connect(ctx.destination);

      noise.start(now);
      noise.stop(now + 0.04);

      const osc = ctx.createOscillator();
      osc.type = 'triangle';
      const oscFreq = (600 + Math.random() * 150) * jitter;
      osc.frequency.setValueAtTime(oscFreq, now);
      osc.frequency.exponentialRampToValueAtTime(110, now + 0.02);

      const oscGain = ctx.createGain();
      oscGain.gain.setValueAtTime(0.5 * effectiveVol, now);
      oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.025);

      osc.connect(oscGain);
      oscGain.connect(ctx.destination);

      osc.start(now);
      osc.stop(now + 0.03);
    } catch {
      // Ignora erro se áudio falhar
    }
  }

  /**
   * Toca o som oco e mecânico da barra de espaço.
   */
  playSpacebar(volume = 0.6): void {
    const ctx = this.getContext();
    if (!ctx) return;

    try {
      const effectiveVol = Math.max(0, Math.min(volume, 1));

      // Dispara amostra real gravada em pitch mais grave
      this.playRecordedSample(ctx, effectiveVol * 0.85, 0.82, 0.88);

      // Acrescenta corpo ressonante oco
      const now = ctx.currentTime;
      const osc = ctx.createOscillator();
      osc.type = 'sine';
      osc.frequency.setValueAtTime(280, now);
      osc.frequency.exponentialRampToValueAtTime(70, now + 0.04);

      const oscGain = ctx.createGain();
      oscGain.gain.setValueAtTime(0.45 * effectiveVol, now);
      oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.045);

      osc.connect(oscGain);
      oscGain.connect(ctx.destination);

      osc.start(now);
      osc.stop(now + 0.05);
    } catch {
      // Ignora erro
    }
  }

  /**
   * Toca o clássico retorno de carro ao pressionar Enter:
   * Impacto mecânico real da tecla + "Ding!" límpido de campainha de bronze.
   */
  playEnterReturn(volume = 0.6): void {
    const ctx = this.getContext();
    if (!ctx) return;

    try {
      const now = ctx.currentTime;
      const effectiveVol = Math.max(0, Math.min(volume, 1));

      // 1. Tecla mecânica de retorno
      this.playRecordedSample(ctx, effectiveVol, 0.9, 1.0);

      // 2. Sineta "Ding!" de bronze
      const bell1 = ctx.createOscillator();
      bell1.type = 'sine';
      bell1.frequency.setValueAtTime(2380, now);

      const bellGain1 = ctx.createGain();
      bellGain1.gain.setValueAtTime(0.35 * effectiveVol, now);
      bellGain1.gain.exponentialRampToValueAtTime(0.0001, now + 0.45);

      bell1.connect(bellGain1);
      bellGain1.connect(ctx.destination);

      bell1.start(now);
      bell1.stop(now + 0.5);

      // Harmônico superior
      const bell2 = ctx.createOscillator();
      bell2.type = 'sine';
      bell2.frequency.setValueAtTime(3580, now);

      const bellGain2 = ctx.createGain();
      bellGain2.gain.setValueAtTime(0.18 * effectiveVol, now);
      bellGain2.gain.exponentialRampToValueAtTime(0.0001, now + 0.35);

      bell2.connect(bellGain2);
      bellGain2.connect(ctx.destination);

      bell2.start(now);
      bell2.stop(now + 0.4);
    } catch {
      // Ignora erro
    }
  }

  /**
   * Toca o som mecânico de recuo de catraca (Backspace / Delete).
   */
  playBackspace(volume = 0.6): void {
    const ctx = this.getContext();
    if (!ctx) return;

    try {
      const now = ctx.currentTime;
      const effectiveVol = Math.max(0, Math.min(volume, 1));

      // Dispara amostra real acelerada (1.18x) para dar efeito mecânico de clique de recuo
      this.playRecordedSample(ctx, effectiveVol * 0.9, 1.15, 1.25);

      // Leve clique de catraca complementar
      const noise = ctx.createBufferSource();
      noise.buffer = this.getNoiseBuffer(ctx);

      const filter = ctx.createBiquadFilter();
      filter.type = 'highpass';
      filter.frequency.setValueAtTime(2600, now);

      const gain = ctx.createGain();
      gain.gain.setValueAtTime(0.2 * effectiveVol, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.02);

      noise.connect(filter);
      filter.connect(gain);
      gain.connect(ctx.destination);

      noise.start(now);
      noise.stop(now + 0.025);
    } catch {
      // Ignora erro
    }
  }
}

export const typewriterAudio = new TypewriterAudioEngine();
