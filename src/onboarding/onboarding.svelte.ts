/**
 * Store de Onboarding e Curso Básico do Magrathea Writer (Ajuste 3).
 *
 * Controla:
 * - Visibilidade do modal de boas-vindas / curso
 * - Lição / passo atual (1 a 5)
 * - Persistência no localStorage para abertura automática na primeira vez
 */

export interface CourseLesson {
  id: number;
  badge: string;
  title: string;
  subtitle: string;
  icon: string;
  bullets: { icon: string; title: string; desc: string }[];
}

export const COURSE_LESSONS: CourseLesson[] = [
  {
    id: 1,
    badge: 'Módulo 1 de 5',
    icon: '🪐',
    title: 'A Filosofia Magrathea: Privacidade Sagrada',
    subtitle: 'Um refúgio de escrita 100% offline, ético e imune à espionagem de terceiros.',
    bullets: [
      {
        icon: '🛡️',
        title: 'Criptografia em Nível Militar',
        desc: 'Seu cofre literário é protegido com XChaCha20-Poly1305 (256 bits) e KDF Argon2id. Tudo é cifrado antes de tocar o disco.',
      },
      {
        icon: '🚫',
        title: 'Zero Nuvem, Zero IA Espiã',
        desc: 'Nenhum servidor externo armazena seus livros. Nenhuma inteligência artificial lê ou treina com suas ideias.',
      },
      {
        icon: '🗝️',
        title: 'Chave de Recuperação de Emergência',
        desc: 'Ao criar seu mundo, anote o token de 24 palavras em papel. Ele é o único método de reaver o cofre caso esqueça a senha mestra.',
      },
    ],
  },
  {
    id: 2,
    badge: 'Módulo 2 de 5',
    icon: '🌲',
    title: 'Estrutura do Manuscrito & Lixeira Cifrada',
    subtitle: 'Hierarquia flexível para romances, contos, sagas e não-ficção.',
    bullets: [
      {
        icon: '📚',
        title: 'Partes, Capítulos e Cenas',
        desc: 'Divida sua narrativa em partes temáticas, capítulos e cenas atômicas para manter o controle total da trama.',
      },
      {
        icon: '🖐️',
        title: 'Arrastar e Soltar Hierárquico',
        desc: 'Mova capítulos de lugar ou arraste uma cena para dentro de outro capítulo facilmente pela barra lateral.',
      },
      {
        icon: '🗑️',
        title: 'Lixeira Protegida & Restauração',
        desc: 'Cenas excluídas vão para uma lixeira interna cifrada. Você pode restaurá-las com 1 clique ou esvaziar permanentemente.',
      },
    ],
  },
  {
    id: 3,
    badge: 'Módulo 3 de 5',
    icon: '🧠',
    title: 'O Segundo Cérebro: Conexões Literárias',
    subtitle: 'Worldbuilding vivo com hiperlinks bidirecionais e imagens cifradas.',
    bullets: [
      {
        icon: '🔗',
        title: 'Wikilinks Diretos',
        desc: 'Digite [[ no editor para abrir o catálogo e conectar personagens, lugares ou notas conceituais.',
      },
      {
        icon: '✨',
        title: 'Links Indiretos com Autocompletar',
        desc: 'Escreva de forma natural: digite [casa dela]( e escolha a entidade. Na publicação, o compilador mantém apenas o texto natural!',
      },
      {
        icon: '⏳',
        title: 'Tags Ocultas de Linha do Tempo',
        desc: 'Use #data_20261027 no texto da cena para posicioná-la na régua cronológica. Ela é ocultada automaticamente na publicação.',
      },
      {
        icon: '🖼️',
        title: 'Galeria de Imagens Cifradas',
        desc: 'Anexe mapas, fotos de referência e avatares às fichas. Arraste a imagem diretamente para a ficha.',
      },
    ],
  },
  {
    id: 4,
    badge: 'Módulo 4 de 5',
    icon: '✍️',
    title: 'Ergonomia Literária & Foco Total',
    subtitle: 'Tipografia calibrada e atalhos pensados para a mente do escritor.',
    bullets: [
      {
        icon: '⚙️',
        title: 'Preferências do Editor (Cmd+,)',
        desc: 'Personalize números de linha, fontes serifadas (EB Garamond), tamanho de texto e largura da mancha gráfica.',
      },
      {
        icon: '✒️',
        title: 'Barra de Pontuação Literária (✍️)',
        desc: 'Atalhos no rodapé para travessão de diálogo (—), aspas angulares («»), reticências reais (…) e quebras de cena (* * *).',
      },
      {
        icon: 'ℹ️',
        title: 'Inspetor Lateral (Cmd+I)',
        desc: 'Acompanhe menções, backlinks, sinopses e a galeria da cena ativa sem sair da página de escrita.',
      },
      {
        icon: '🧘',
        title: 'Modo Zen Imersivo (F11)',
        desc: 'Oculte toda a interface gráfica para entrar em estado de fluxo criativo profundo.',
      },
    ],
  },
  {
    id: 5,
    badge: 'Módulo 5 de 5',
    icon: '📖',
    title: 'Compilação Editorial em 1 Clique',
    subtitle: 'Do primeiro rascunho ao livro publicado em formatos profissionais.',
    bullets: [
      {
        icon: '📄',
        title: 'Microsoft Word (.docx)',
        desc: 'Formatação editorial limpa com títulos em estilos nativos, pronta para envio a editoras e agentes literários.',
      },
      {
        icon: '📱',
        title: 'Livro Digital (.epub)',
        desc: 'E-book padrão com índice navegável para Apple Books, Kobo e leitores digitais.',
      },
      {
        icon: '🖨️',
        title: 'Leitura & Impressão (.pdf / .html)',
        desc: 'Design tipográfico refinado com serifas proporcionais para revisão física em papel ou PDF.',
      },
      {
        icon: '💎',
        title: 'Cofres Abertos (.md & .txt)',
        desc: 'Exportação com total fidelidade para cofres Obsidian ou texto consolidado com dossiê de worldbuilding.',
      },
    ],
  },
];

class OnboardingStore {
  isOpen = $state(false);
  currentLessonIndex = $state(0);

  constructor() {
    this.checkFirstLaunch();
  }

  checkFirstLaunch(): void {
    if (typeof localStorage === 'undefined') return;
    const completed = localStorage.getItem('magrathea_onboarding_completed');
    if (!completed) {
      // Primeira abertura do aplicativo: abre automaticamente o curso!
      this.isOpen = true;
      this.currentLessonIndex = 0;
    }
  }

  open(lessonIndex = 0): void {
    this.currentLessonIndex = Math.min(Math.max(lessonIndex, 0), COURSE_LESSONS.length - 1);
    this.isOpen = true;
  }

  close(): void {
    this.isOpen = false;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('magrathea_onboarding_completed', 'true');
    }
  }

  nextLesson(): void {
    if (this.currentLessonIndex < COURSE_LESSONS.length - 1) {
      this.currentLessonIndex += 1;
    } else {
      this.close();
    }
  }

  prevLesson(): void {
    if (this.currentLessonIndex > 0) {
      this.currentLessonIndex -= 1;
    }
  }

  goToLesson(index: number): void {
    this.currentLessonIndex = Math.min(Math.max(index, 0), COURSE_LESSONS.length - 1);
  }

  get currentLesson(): CourseLesson {
    return COURSE_LESSONS[this.currentLessonIndex];
  }
}

export const onboardingStore = new OnboardingStore();
