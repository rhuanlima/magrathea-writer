/**
 * Módulo de Análise Estilométrica e Estatísticas Literárias.
 *
 * Fornece métricas profundas de escrita para autores de romances e ficção:
 * - Contagem e densidade de advérbios (com foco no sufixo '-mente' e estilo "Show, don't tell")
 * - Verbos de diálogo e elocução para monitorar repetições de fórmulas de discurso
 * - Riqueza vocabular (Type-Token Ratio / TTR)
 * - Ritmo narrativo (distribuição do comprimento de frases)
 * - Proporção de diálogos vs texto narrativo
 */

export interface WordFrequency {
  word: string;
  count: number;
}

export interface ProseStyleAnalysis {
  totalWords: number;
  totalChars: number;
  totalSentences: number;
  totalParagraphs: number;

  // Ritmo e Leitura
  avgWordsPerSentence: number;
  estimatedReadingTimeMinutes: number;

  // Riqueza Lexical
  uniqueWords: number;
  vocabularyRichnessPercent: number;

  // Advérbios
  adverbsCount: number;
  adverbsPercent: number;
  menteAdverbsCount: number;
  topAdverbs: WordFrequency[];

  // Verbos de Diálogo / Elocução
  dialogueVerbsCount: number;
  topDialogueVerbs: WordFrequency[];

  // Verbos Mais Recorrentes
  topVerbs: WordFrequency[];

  // Distribuição Frasal
  shortSentencesCount: number;
  mediumSentencesCount: number;
  longSentencesCount: number;

  // Proporção Diálogo vs Narrativa
  dialogueWords: number;
  narrativeWords: number;
  dialoguePercent: number;
  narrativePercent: number;
}

/**
 * Exceções de palavras terminadas em 'mente' que NÃO são advérbios modais em português.
 */
const FALSE_MENTE_WORDS = new Set([
  'mente',
  'mentes',
  'veemente',
  'veementes',
  'clemente',
  'clementes',
  'inclemente',
  'inclementes',
  'semente',
  'sementes',
  'torrente',
  'torrentes',
  'dente',
  'dentes',
  'gente',
  'gentes',
  'urgente',
  'urgentes',
  'patente',
  'patentes',
  'latente',
  'latentes',
  'frente',
  'frentes',
  'ambiente',
  'ambientes',
  'cliente',
  'clientes',
  'presente',
  'presentes',
  'ausente',
  'ausentes',
  'frequente',
  'frequentes',
  'quente',
  'quentes',
  'valente',
  'valentes',
  'impaciente',
  'impacientes',
  'paciente',
  'pacientes',
  'consciente',
  'conscientes',
  'inconsciente',
  'inconscientes',
  'eloquente',
  'eloquentes',
  'onipresente',
  'onipotente',
  'inerente',
  'inerentes',
]);

/**
 * Lista de advérbios comuns do português (sem sufixo -mente) e inglês.
 */
const COMMON_ADVERBS = new Set([
  // Português
  'não',
  'sim',
  'muito',
  'pouco',
  'mais',
  'menos',
  'já',
  'ainda',
  'sempre',
  'nunca',
  'jamais',
  'talvez',
  'aqui',
  'ali',
  'lá',
  'cá',
  'perto',
  'longe',
  'depois',
  'antes',
  'agora',
  'logo',
  'cedo',
  'tarde',
  'bastante',
  'demais',
  'quase',
  'tão',
  'tanto',
  'bem',
  'mal',
  'depressa',
  'devagar',
  'apenas',
  'tampouco',
  'acaso',
  'adiante',
  'atrás',
  'acima',
  'abaixo',
  'além',
  'algures',
  'algures',
  'nenhures',
  'decerto',
  'quiçá',
  // Inglês
  'not',
  'never',
  'always',
  'often',
  'rarely',
  'seldom',
  'very',
  'quite',
  'too',
  'almost',
  'already',
  'still',
  'soon',
  'then',
  'here',
  'there',
  'now',
  'again',
  'maybe',
  'perhaps',
]);

/**
 * Verbos de diálogo / elocução (Dialogue tags).
 */
const DIALOGUE_VERBS = new Set([
  // Português
  'disse',
  'disseram',
  'dizia',
  'diz',
  'diziam',
  'falou',
  'falaram',
  'falava',
  'falavam',
  'fala',
  'perguntou',
  'perguntaram',
  'perguntava',
  'pergunta',
  'respondeu',
  'responderam',
  'respondia',
  'responde',
  'murmurou',
  'murmuraram',
  'murmurava',
  'sussurrou',
  'sussurraram',
  'sussurrava',
  'gritou',
  'gritaram',
  'gritava',
  'exclamou',
  'exclamaram',
  'indagou',
  'indagaram',
  'retrucou',
  'retrucaram',
  'rebateu',
  'rebateram',
  'afirmou',
  'afirmaram',
  'ordenou',
  'ordenaram',
  'declarou',
  'declararam',
  'explicou',
  'explicaram',
  'insistiu',
  'insistiram',
  'acrescentou',
  'acrescentaram',
  'comentou',
  'comentaram',
  'desabafou',
  'desabafaram',
  'pediu',
  'pediram',
  'ponderou',
  'ponderaram',
  'concluiu',
  'concluíram',
  'repetiu',
  'repetiram',
  'admitiu',
  'admitiram',
  'concordou',
  'concordaram',
  'protestou',
  'protestaram',
  'berrou',
  'berraram',
  'rosnou',
  'rosnaram',
  'gaguejou',
  'gaguejaram',
  'vociferou',
  'bradou',
  'sugeriu',
  'sugeriram',
  'interrompeu',
  'suspirou',
  'suspiraram',
  'lamentou',
  'lamentaram',
  'soluçou',
  'avisou',
  'avisaram',
  'anunciou',
  'anunciaram',
  // Inglês
  'said',
  'asked',
  'replied',
  'answered',
  'shouted',
  'whispered',
  'exclaimed',
  'muttered',
  'screamed',
  'yelled',
  'declared',
  'stated',
  'inquired',
  'commented',
  'insisted',
  'added',
  'demanded',
  'sighed',
  'groaned',
  'growled',
  'gasped',
]);

/**
 * Raízes ou formas conjugadas comuns dos principais verbos da língua portuguesa para análise geral.
 */
const GENERAL_COMMON_VERBS = new Set([
  // ser/estar
  'era',
  'eram',
  'foi',
  'foram',
  'é',
  'são',
  'seria',
  'ser',
  'sendo',
  'sido',
  'estava',
  'estavam',
  'esteve',
  'estiveram',
  'está',
  'estão',
  'estar',
  'estando',
  // ter/haver
  'tinha',
  'tinham',
  'teve',
  'tiveram',
  'tem',
  'têm',
  'ter',
  'tendo',
  'havia',
  'haviam',
  'houve',
  'há',
  'haver',
  // fazer/ir/vir
  'fazia',
  'faziam',
  'fez',
  'fizeram',
  'faz',
  'fazer',
  'fazendo',
  'ia',
  'iam',
  'foi',
  'foram',
  'vai',
  'vão',
  'ir',
  'indo',
  'vinha',
  'vinham',
  'veio',
  'vieram',
  'vem',
  'vêm',
  'vir',
  'vindo',
  // ver/olhar/sentir/ouvir
  'via',
  'viam',
  'viu',
  'viram',
  'vê',
  'ver',
  'vendo',
  'olhou',
  'olhava',
  'olhavam',
  'olhar',
  'olhando',
  'sentia',
  'sentiam',
  'sentiu',
  'sentiram',
  'sente',
  'sentir',
  'sentindo',
  'ouvia',
  'ouviam',
  'ouviu',
  'ouviram',
  'ouvir',
  // pensar/saber/querer/poder
  'pensava',
  'pensavam',
  'pensou',
  'pensar',
  'sabia',
  'sabiam',
  'soube',
  'sabe',
  'saber',
  'queria',
  'queriam',
  'quis',
  'quer',
  'querer',
  'podia',
  'podiam',
  'pôde',
  'pode',
  'poder',
  // ficar/passar/parecer/chegar/sair/dar
  'ficava',
  'ficavam',
  'ficou',
  'ficaram',
  'fica',
  'ficar',
  'passava',
  'passavam',
  'passou',
  'passar',
  'parecia',
  'pareciam',
  'pareceu',
  'parece',
  'parecer',
  'chegou',
  'chegava',
  'chegar',
  'saiu',
  'saía',
  'sair',
  'deu',
  'dava',
  'davam',
  'dá',
  'dar',
  // Inglês comuns
  'was',
  'were',
  'had',
  'went',
  'looked',
  'felt',
  'knew',
  'thought',
  'seemed',
  'came',
  'saw',
  'turned',
  'walked',
]);

/**
 * Remove blocos de metadados frontmatter (ex: --- ... ---) para não poluir as estatísticas literárias.
 */
export function stripFrontmatter(text: string): string {
  if (text.startsWith('---')) {
    const endIdx = text.indexOf('\n---', 3);
    if (endIdx !== -1) {
      return text.slice(endIdx + 4).trim();
    }
  }
  return text;
}

/**
 * Extrai palavras individuais de um texto, normalizando e filtrando pontuações.
 */
export function extractWords(text: string): string[] {
  const matches = text.match(/[\p{L}\p{N}]+(?:['’-][\p{L}\p{N}]+)*/gu);
  return matches ? matches.map((w) => w.toLowerCase()) : [];
}

/**
 * Divide o texto em frases com base na pontuação canônica (. ! ? …).
 */
export function extractSentences(text: string): string[] {
  // Ignora pontuação dentro de números ou abreviações comuns
  const rawSentences = text
    .replace(/\r\n/g, '\n')
    .split(/(?<=[.!?…])\s+(?=[A-ZÀ-Ú"—«"'\d])/u);

  return rawSentences
    .map((s) => s.trim())
    .filter((s) => s.length > 0 && extractWords(s).length > 0);
}

/**
 * Analisa a estilometria completa da prosa e devolve o conjunto detalhado de métricas.
 */
export function analyzeProseStyle(rawText: string): ProseStyleAnalysis {
  const text = stripFrontmatter(rawText);
  const words = extractWords(text);
  const totalWords = words.length;
  const totalChars = text.length;

  // Parágrafos
  const paragraphs = text
    .split(/\n\s*\n/)
    .map((p) => p.trim())
    .filter((p) => p.length > 0);
  const totalParagraphs = paragraphs.length;

  // Frases
  const sentences = extractSentences(text);
  const totalSentences = sentences.length || (totalWords > 0 ? 1 : 0);

  // Média de palavras por frase
  const avgWordsPerSentence =
    totalSentences > 0 ? Number((totalWords / totalSentences).toFixed(1)) : 0;

  // Tempo de leitura estimado (~200 palavras por minuto)
  const estimatedReadingTimeMinutes = totalWords > 0 ? Math.max(1, Math.ceil(totalWords / 200)) : 0;

  // Riqueza Lexical (TTR)
  const uniqueWordsSet = new Set(words);
  const uniqueWords = uniqueWordsSet.size;
  const vocabularyRichnessPercent =
    totalWords > 0 ? Number(((uniqueWords / totalWords) * 100).toFixed(1)) : 0;

  // Contadores e Dicionários
  const adverbsMap = new Map<string, number>();
  const dialogueVerbsMap = new Map<string, number>();
  const verbsMap = new Map<string, number>();

  let adverbsCount = 0;
  let menteAdverbsCount = 0;
  let dialogueVerbsCount = 0;

  for (const word of words) {
    // 1. Verificação de Advérbios
    const isMente =
      word.length >= 6 &&
      word.endsWith('mente') &&
      !FALSE_MENTE_WORDS.has(word);

    const isCommonAdverb = COMMON_ADVERBS.has(word);

    if (isMente || isCommonAdverb) {
      adverbsCount += 1;
      if (isMente) menteAdverbsCount += 1;
      adverbsMap.set(word, (adverbsMap.get(word) || 0) + 1);
    }

    // 2. Verificação de Verbos de Diálogo
    if (DIALOGUE_VERBS.has(word)) {
      dialogueVerbsCount += 1;
      dialogueVerbsMap.set(word, (dialogueVerbsMap.get(word) || 0) + 1);
      verbsMap.set(word, (verbsMap.get(word) || 0) + 1);
    } else if (GENERAL_COMMON_VERBS.has(word)) {
      verbsMap.set(word, (verbsMap.get(word) || 0) + 1);
    }
  }

  const adverbsPercent =
    totalWords > 0 ? Number(((adverbsCount / totalWords) * 100).toFixed(1)) : 0;

  const topAdverbs: WordFrequency[] = Array.from(adverbsMap.entries())
    .map(([word, count]) => ({ word, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10);

  const topDialogueVerbs: WordFrequency[] = Array.from(dialogueVerbsMap.entries())
    .map(([word, count]) => ({ word, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10);

  const topVerbs: WordFrequency[] = Array.from(verbsMap.entries())
    .map(([word, count]) => ({ word, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10);

  // Distribuição Frasal
  let shortSentencesCount = 0;
  let mediumSentencesCount = 0;
  let longSentencesCount = 0;

  for (const s of sentences) {
    const count = extractWords(s).length;
    if (count < 8) {
      shortSentencesCount += 1;
    } else if (count <= 20) {
      mediumSentencesCount += 1;
    } else {
      longSentencesCount += 1;
    }
  }

  // Proporção de Diálogo vs Narrativa
  // Identifica falas por travessões (—, -) no início de parágrafos/linhas ou aspas
  let dialogueWords = 0;
  const lines = text.split('\n');

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    if (trimmed.startsWith('—') || trimmed.startsWith('-') || trimmed.startsWith('–')) {
      dialogueWords += extractWords(trimmed).length;
    } else {
      // Procura trechos entre aspas na linha
      const quotesMatch = trimmed.match(/"[^"]+"|«[^»]+»|“[^”]+”/g);
      if (quotesMatch) {
        for (const q of quotesMatch) {
          dialogueWords += extractWords(q).length;
        }
      }
    }
  }

  // Garante que dialogueWords não supere totalWords
  dialogueWords = Math.min(dialogueWords, totalWords);
  const narrativeWords = Math.max(0, totalWords - dialogueWords);

  const dialoguePercent =
    totalWords > 0 ? Math.round((dialogueWords / totalWords) * 100) : 0;
  const narrativePercent = totalWords > 0 ? 100 - dialoguePercent : 0;

  return {
    totalWords,
    totalChars,
    totalSentences,
    totalParagraphs,
    avgWordsPerSentence,
    estimatedReadingTimeMinutes,
    uniqueWords,
    vocabularyRichnessPercent,
    adverbsCount,
    adverbsPercent,
    menteAdverbsCount,
    topAdverbs,
    dialogueVerbsCount,
    topDialogueVerbs,
    topVerbs,
    shortSentencesCount,
    mediumSentencesCount,
    longSentencesCount,
    dialogueWords,
    narrativeWords,
    dialoguePercent,
    narrativePercent,
  };
}
