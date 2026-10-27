/**
 * Modelo de documento Markdown + Frontmatter YAML no frontend.
 *
 * Em conformidade com o PRD (E1):
 * - Texto = Markdown + frontmatter YAML.
 * - Preservação estrita de chaves desconhecidas do frontmatter.
 * - Contagem precisa de palavras/caracteres apenas do corpo narrativo ativo.
 */

export interface ParsedDocument {
  id?: string;
  title?: string;
  slug?: string;
  type?: string;
  status?: string;
  tags: string[];
  date?: string;
  time?: string;
  /** Chaves desconhecidas preservadas integralmente na serialização. */
  extra: Record<string, unknown>;
  /** Prosa literária em Markdown puro. */
  body: string;
}

/**
 * Separa o frontmatter delimitado por `---` do corpo do texto.
 */
export function splitFrontmatter(raw: string): {
  frontmatterText: string | null;
  body: string;
} {
  const content = raw.startsWith('\uFEFF') ? raw.slice(1) : raw;

  if (!content.startsWith('---')) {
    return { frontmatterText: null, body: content };
  }

  const remainder = content.slice(3);
  let afterFirstDelimiter = '';
  if (remainder.startsWith('\r\n')) {
    afterFirstDelimiter = remainder.slice(2);
  } else if (remainder.startsWith('\n')) {
    afterFirstDelimiter = remainder.slice(1);
  } else {
    return { frontmatterText: null, body: content };
  }

  const closingIndex = afterFirstDelimiter.search(/(?:^|\r?\n)---(?:\r?\n|$)/);
  if (closingIndex === -1) {
    return { frontmatterText: null, body: content };
  }

  const newlineOffset = afterFirstDelimiter.slice(closingIndex).startsWith('\r\n')
    ? 2
    : afterFirstDelimiter.slice(closingIndex).startsWith('\n')
      ? 1
      : 0;

  const actualClosingPos = closingIndex + newlineOffset;
  const yamlPart = afterFirstDelimiter.slice(0, actualClosingPos);
  const afterClosing = afterFirstDelimiter.slice(actualClosingPos + 3);

  let bodyPart = afterClosing;
  if (bodyPart.startsWith('\r\n\r\n')) {
    bodyPart = bodyPart.slice(4);
  } else if (bodyPart.startsWith('\n\n')) {
    bodyPart = bodyPart.slice(2);
  } else if (bodyPart.startsWith('\r\n')) {
    bodyPart = bodyPart.slice(2);
  } else if (bodyPart.startsWith('\n')) {
    bodyPart = bodyPart.slice(1);
  }

  return {
    frontmatterText: yamlPart,
    body: bodyPart,
  };
}

/**
 * Parser seguro de Frontmatter YAML linha a linha, sem dependências externas perigosas.
 * Preserva chaves desconhecidas em `extra`.
 */
export function parseDocument(raw: string): ParsedDocument {
  const { frontmatterText, body } = splitFrontmatter(raw);

  const doc: ParsedDocument = {
    tags: [],
    extra: {},
    body,
  };

  if (!frontmatterText || frontmatterText.trim() === '') {
    return doc;
  }

  const lines = frontmatterText.split(/\r?\n/);
  let currentKey = '';
  let inList = false;

  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) continue;

    if (line.startsWith('- ') && inList && currentKey) {
      const itemVal = line.slice(2).trim().replace(/^['"]|['"]$/g, '');
      if (currentKey === 'tags') {
        doc.tags.push(itemVal);
      } else {
        const list = (doc.extra[currentKey] as unknown[]) || [];
        list.push(itemVal);
        doc.extra[currentKey] = list;
      }
      continue;
    }

    inList = false;
    const colonIdx = line.indexOf(':');
    if (colonIdx === -1) continue;

    const key = line.slice(0, colonIdx).trim();
    const valStr = line.slice(colonIdx + 1).trim();

    currentKey = key;

    if (valStr === '' || valStr === '~') {
      inList = true;
      if (key === 'tags') {
        doc.tags = [];
      } else {
        doc.extra[key] = [];
      }
      continue;
    }

    let parsedVal: unknown = valStr;
    if (valStr === 'true') parsedVal = true;
    else if (valStr === 'false') parsedVal = false;
    else if (/^-?\d+$/.test(valStr)) parsedVal = parseInt(valStr, 10);
    else if (/^-?\d+\.\d+$/.test(valStr)) parsedVal = parseFloat(valStr);
    else if (
      (valStr.startsWith('"') && valStr.endsWith('"')) ||
      (valStr.startsWith("'") && valStr.endsWith("'"))
    ) {
      parsedVal = valStr.slice(1, -1);
    } else if (valStr.startsWith('[') && valStr.endsWith(']')) {
      const inner = valStr.slice(1, -1).trim();
      parsedVal = inner
        ? inner.split(',').map((s) => s.trim().replace(/^['"]|['"]$/g, ''))
        : [];
    }

    if (key === 'id') doc.id = String(parsedVal);
    else if (key === 'title') doc.title = String(parsedVal);
    else if (key === 'slug') doc.slug = String(parsedVal);
    else if (key === 'type') doc.type = String(parsedVal);
    else if (key === 'status') doc.status = String(parsedVal);
    else if (key === 'date') doc.date = String(parsedVal);
    else if (key === 'time') doc.time = String(parsedVal);
    else if (key === 'tags') {
      doc.tags = Array.isArray(parsedVal) ? (parsedVal as string[]) : [String(parsedVal)];
    } else {
      doc.extra[key] = parsedVal;
    }
  }

  return doc;
}

/**
 * Serializa de volta para Markdown formatado com Frontmatter YAML se houver metadados.
 */
export function serializeDocument(doc: ParsedDocument): string {
  const fields: string[] = [];

  if (doc.id) fields.push(`id: "${doc.id}"`);
  if (doc.title) fields.push(`title: "${doc.title}"`);
  if (doc.slug) fields.push(`slug: "${doc.slug}"`);
  if (doc.type) fields.push(`type: "${doc.type}"`);
  if (doc.status) fields.push(`status: "${doc.status}"`);
  if (doc.date) fields.push(`date: "${doc.date}"`);
  if (doc.time) fields.push(`time: "${doc.time}"`);

  if (doc.tags && doc.tags.length > 0) {
    fields.push('tags:');
    for (const tag of doc.tags) {
      fields.push(`  - "${tag}"`);
    }
  }

  // Preserva todas as chaves desconhecidas do mapa extra
  for (const [key, value] of Object.entries(doc.extra)) {
    if (Array.isArray(value)) {
      fields.push(`${key}:`);
      for (const item of value) {
        fields.push(`  - ${JSON.stringify(item)}`);
      }
    } else if (typeof value === 'object' && value !== null) {
      fields.push(`${key}: ${JSON.stringify(value)}`);
    } else if (typeof value === 'string') {
      fields.push(`${key}: "${value}"`);
    } else {
      fields.push(`${key}: ${String(value)}`);
    }
  }

  if (fields.length === 0) {
    return doc.body;
  }

  const yamlBlock = fields.join('\n');
  if (doc.body.length === 0) {
    return `---\n${yamlBlock}\n---\n`;
  }
  return `---\n${yamlBlock}\n---\n\n${doc.body}`;
}

/**
 * Conta palavras ignorando frontmatter.
 */
export function countWords(body: string): number {
  const trimmed = body.trim();
  if (!trimmed) return 0;
  return trimmed.split(/\s+/).length;
}

/**
 * Conta caracteres ignorando frontmatter.
 */
export function countChars(body: string): number {
  return body.length;
}
