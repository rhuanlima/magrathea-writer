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
  vault: {
    status: {
      noVaultOpen: 'Nenhum cofre aberto',
      vaultOpen: 'Cofre: {title}',
      newVaultButton: 'Novo Cofre',
      lockButton: 'Bloquear Cofre',
    },
    create: {
      modalTitle: 'Novo Cofre Literário',
      step1Title: '1. Dados da Obra e Senha Mestra',
      step2Title: '2. Chave de Recuperação de Emergência',
      projectTitleLabel: 'Título da Obra / Romance',
      projectTitlePlaceholder: 'Ex: O Guia do Mochileiro das Galáxias',
      vaultDirLabel: 'Diretório de Armazenamento (.magrathea)',
      vaultDirPlaceholder: '/caminho/para/seu/manuscrito',
      passphraseLabel: 'Frase-Senha Mestra',
      passphrasePlaceholder: 'Digite uma frase-senha longa e segura...',
      passphraseConfirmLabel: 'Confirmar Frase-Senha Mestra',
      passphraseConfirmPlaceholder: 'Repita a frase-senha mestra...',
      passwordsDoNotMatch: 'As frases-senhas informadas não conferem.',
      strengthEmpty: 'Informe uma senha mestra',
      strengthVeryWeak: 'Muito fraca (insegura)',
      strengthWeak: 'Fraca (abaixo do recomendado)',
      strengthFair: 'Razoável (mínimo aceitável)',
      strengthGood: 'Boa (boa entropia)',
      strengthStrong: 'Forte (excelente proteção)',
      minRequirementsNote:
        'A senha mestra deve ter pelo menos 10 caracteres e complexidade razoável para proteger sua obra.',
      continueButton: 'Criar Cofre e Gerar Chave',
      creatingVault: 'Criptografando e inicializando cofre...',
      recoveryWarning:
        'Esta chave só será exibida nesta tela. O Magrathea Writer é 100% offline e não possui servidores para recuperar seus dados se você esquecer sua senha.',
      copyKeyButton: 'Copiar Chave de Emergência',
      copiedTooltip: 'Copiado para a área de transferência!',
      confirmSavedLabel:
        'Eu copiei e guardei minha chave de recuperação em um local seguro.',
      openVaultButton: 'Abrir Oficina Literária',
    },
  },
  common: {
    languageName: 'Português (Brasil)',
    cancel: 'Cancelar',
    confirm: 'Confirmar',
    save: 'Salvar',
    close: 'Fechar',
  },
};
