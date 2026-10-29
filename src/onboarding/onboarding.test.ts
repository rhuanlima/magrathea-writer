import { describe, it, expect, beforeEach } from 'vitest';
import { onboardingStore, COURSE_LESSONS } from './onboarding.svelte';

describe('OnboardingStore: Curso Básico do Magrathea Writer', () => {
  beforeEach(() => {
    onboardingStore.close();
  });

  it('possui 6 lições essenciais bem estruturadas (incluindo encorajamento de café/apoio)', () => {
    expect(COURSE_LESSONS.length).toBe(6);
    expect(COURSE_LESSONS[0].title).toContain('Filosofia');
    expect(COURSE_LESSONS[1].title).toContain('Manuscrito');
    expect(COURSE_LESSONS[2].title).toContain('Segundo Cérebro');
    expect(COURSE_LESSONS[3].title).toContain('Ergonomia');
    expect(COURSE_LESSONS[4].title).toContain('Compilação');
    expect(COURSE_LESSONS[5].title).toContain('Café');
  });

  it('permite abrir, navegar entre lições e avançar sequencialmente', () => {
    onboardingStore.open(0);
    expect(onboardingStore.isOpen).toBe(true);
    expect(onboardingStore.currentLessonIndex).toBe(0);

    onboardingStore.nextLesson();
    expect(onboardingStore.currentLessonIndex).toBe(1);

    onboardingStore.prevLesson();
    expect(onboardingStore.currentLessonIndex).toBe(0);

    onboardingStore.goToLesson(5);
    expect(onboardingStore.currentLessonIndex).toBe(5);
    expect(onboardingStore.currentLesson.id).toBe(6);
  });

  it('encerra o curso ao avançar além da última lição', () => {
    onboardingStore.open(5);
    expect(onboardingStore.isOpen).toBe(true);

    onboardingStore.nextLesson();
    expect(onboardingStore.isOpen).toBe(false);
  });
});
