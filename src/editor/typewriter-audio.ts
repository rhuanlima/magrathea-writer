/**
 * Motor de Áudio Sintético para Modo Máquina de Escrever (Typewriter).
 *
 * Utiliza Web Audio API pura de baixa latência e zero dependência de arquivos externos:
 * - 100% offline, leve e seguro para memória
 * - Tecla comum: clique percussivo com ruído metálico e variação aleatória de tom
 * - Espaço: impacto oco e mecânico mais grave
 * - Enter: a famosa sineta metálica ("Ding!") + retorno de carro analógico
 * - Backspace: duplo clique mecânico de catraca de recuo
 */

export class TypewriterAudioEngine {
  private audioCtx: AudioContext | null = null;
  private noiseBuffer: AudioBuffer | null = null;

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

    return this.audioCtx;
  }

  /**
   * Gera um buffer de ruído branco reutilizável de 0.2 segundos.
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
   */
  playKeyClick(volume = 0.6): void {
    const ctx = this.getContext();
    if (!ctx) return;

    try {
      const now = ctx.currentTime;
      const effectiveVol = Math.max(0, Math.min(volume, 1));

      // 1. Componente de Ruído Metálico (impacto da barra de tipos)
      const noise = ctx.createBufferSource();
      noise.buffer = this.getNoiseBuffer(ctx);

      const filter = ctx.createBiquadFilter();
      filter.type = 'bandpass';
      // Variação orgânica de ±12% no tom central para cada tecla
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

      // 2. Componente Tonal de Golpe (thump do tipo contra o cilindro de borracha)
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
      const now = ctx.currentTime;
      const effectiveVol = Math.max(0, Math.min(volume, 1));

      // Ruído mais grave e aberto
      const noise = ctx.createBufferSource();
      noise.buffer = this.getNoiseBuffer(ctx);

      const filter = ctx.createBiquadFilter();
      filter.type = 'bandpass';
      filter.frequency.setValueAtTime(750, now);
      filter.Q.setValueAtTime(1.8, now);

      const noiseGain = ctx.createGain();
      noiseGain.gain.setValueAtTime(0.45 * effectiveVol, now);
      noiseGain.gain.exponentialRampToValueAtTime(0.001, now + 0.045);

      noise.connect(filter);
      filter.connect(noiseGain);
      noiseGain.connect(ctx.destination);

      noise.start(now);
      noise.stop(now + 0.05);

      // Thump oco
      const osc = ctx.createOscillator();
      osc.type = 'sine';
      osc.frequency.setValueAtTime(320, now);
      osc.frequency.exponentialRampToValueAtTime(80, now + 0.035);

      const oscGain = ctx.createGain();
      oscGain.gain.setValueAtTime(0.6 * effectiveVol, now);
      oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);

      osc.connect(oscGain);
      oscGain.connect(ctx.destination);

      osc.start(now);
      osc.stop(now + 0.045);
    } catch {
      // Ignora erro
    }
  }

  /**
   * Toca o clássico retorno de carro ao pressionar Enter:
   * "Ding!" límpido de campainha de bronze + impacto mecânico.
   */
  playEnterReturn(volume = 0.6): void {
    const ctx = this.getContext();
    if (!ctx) return;

    try {
      const now = ctx.currentTime;
      const effectiveVol = Math.max(0, Math.min(volume, 1));

      // 1. Sineta "Ding!" de bronze
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

      // Harmônico brilhante do sino
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

      // 2. Impacto mecânico de retorno do carro (lever shift)
      const osc = ctx.createOscillator();
      osc.type = 'triangle';
      osc.frequency.setValueAtTime(450, now);
      osc.frequency.exponentialRampToValueAtTime(100, now + 0.05);

      const oscGain = ctx.createGain();
      oscGain.gain.setValueAtTime(0.5 * effectiveVol, now);
      oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.06);

      osc.connect(oscGain);
      oscGain.connect(ctx.destination);

      osc.start(now);
      osc.stop(now + 0.07);
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

      // Duplo clique mecânico em sequência rápida
      for (let i = 0; i < 2; i++) {
        const offset = i * 0.018;

        const noise = ctx.createBufferSource();
        noise.buffer = this.getNoiseBuffer(ctx);

        const filter = ctx.createBiquadFilter();
        filter.type = 'highpass';
        filter.frequency.setValueAtTime(2200 + i * 400, now + offset);

        const gain = ctx.createGain();
        gain.gain.setValueAtTime(0.3 * effectiveVol, now + offset);
        gain.gain.exponentialRampToValueAtTime(0.001, now + offset + 0.02);

        noise.connect(filter);
        filter.connect(gain);
        gain.connect(ctx.destination);

        noise.start(now + offset);
        noise.stop(now + offset + 0.025);
      }
    } catch {
      // Ignora erro
    }
  }
}

export const typewriterAudio = new TypewriterAudioEngine();
