import type { TranslationSchema } from '../types';

export const ptBR: TranslationSchema = {
  app: {
    title: 'Magrathea Writer',
    tagline: 'Sua oficina particular de criação de mundos.',
    switchLanguage: 'Mudar para Inglês',
  },
  editor: {
    defaultTitle: 'Manuscrito sem título',
    welcomeMessage: 'Boas-vindas à sua oficina literária livre de distrações.',
    wordsCount: '{count} palavras',
    charactersCount: '{count} caracteres',
    unsavedChanges: 'Alterações não salvas',
    saved: 'Todas as alterações salvas no cofre',
  },
  common: {
    languageName: 'Português (Brasil)',
    cancel: 'Cancelar',
    confirm: 'Confirmar',
    save: 'Salvar',
    close: 'Fechar',
  },
};
