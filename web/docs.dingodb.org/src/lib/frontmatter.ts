export interface DocFrontmatter {
  title: string;
  description: string;
  class: string;
  status: string;
  applies_to: {
    product: string;
    surface: string;
  };
  source: {
    path: string;
    revision?: string;
  };
  last_verified: string;
  owners: string[];
  keywords: string[];
  claim_ids: string[];
  spec_state?: string;
  order?: number;
  section?: string;
}

export interface DocPage {
  slug: string;
  href: string;
  data: DocFrontmatter;
  body: string;
  headings: { depth: number; slug: string; text: string }[];
}

/** Minimal YAML-ish frontmatter parser for docs pages. */
export function parseMarkdownFile(raw: string, slug: string): DocPage {
  let data: Record<string, unknown> = {};
  let body = raw;

  if (raw.startsWith('---')) {
    const end = raw.indexOf('\n---', 3);
    if (end !== -1) {
      const yaml = raw.slice(4, end).trim();
      data = parseSimpleYaml(yaml);
      body = raw.slice(end + 4).replace(/^\n/, '');
    }
  }

  const fm = normalizeFrontmatter(data, slug);
  const headings = extractHeadings(body);

  return {
    slug,
    href: `/${slug}/`,
    data: fm,
    body,
    headings,
  };
}

function normalizeFrontmatter(
  data: Record<string, unknown>,
  slug: string,
): DocFrontmatter {
  const applies =
    typeof data.applies_to === 'object' && data.applies_to
      ? (data.applies_to as Record<string, unknown>)
      : {};
  const source =
    typeof data.source === 'object' && data.source
      ? (data.source as Record<string, unknown>)
      : {};

  return {
    title: String(data.title ?? slug),
    description: String(data.description ?? ''),
    class: String(data.class ?? 'how-to'),
    status: String(data.status ?? 'experimental'),
    applies_to: {
      product: String(applies.product ?? '0.2'),
      surface: String(applies.surface ?? 'embedded-single-node'),
    },
    source: {
      path: String(source.path ?? ''),
      revision: source.revision ? String(source.revision) : 'generated',
    },
    last_verified: String(data.last_verified ?? '2026-07-30'),
    owners: asStringArray(data.owners),
    keywords: asStringArray(data.keywords),
    claim_ids: asStringArray(data.claim_ids),
    spec_state: data.spec_state ? String(data.spec_state) : undefined,
    order: typeof data.order === 'number' ? data.order : Number(data.order) || 0,
    section: data.section ? String(data.section) : slug.split('/')[0],
  };
}

function asStringArray(v: unknown): string[] {
  if (Array.isArray(v)) return v.map(String);
  return [];
}

function parseSimpleYaml(yaml: string): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  let currentKey: string | null = null;
  let currentObj: Record<string, unknown> | null = null;
  let currentArr: string[] | null = null;
  let arrKey: string | null = null;

  const lines = yaml.split('\n');
  for (const line of lines) {
    if (!line.trim() || line.trim().startsWith('#')) continue;

    const arrItem = line.match(/^\s+-\s+(.*)$/);
    if (arrItem && arrKey) {
      if (!currentArr) currentArr = [];
      currentArr.push(unquote(arrItem[1].trim()));
      result[arrKey] = currentArr;
      continue;
    }

    const nested = line.match(/^\s{2}([\w_]+):\s*(.*)$/);
    if (nested && currentKey) {
      if (!currentObj) {
        currentObj = {};
        result[currentKey] = currentObj;
      }
      const val = nested[2].trim();
      currentObj[nested[1]] = val ? unquote(val) : '';
      currentArr = null;
      arrKey = null;
      continue;
    }

    const top = line.match(/^([\w_]+):\s*(.*)$/);
    if (top) {
      if (currentArr && arrKey) {
        result[arrKey] = currentArr;
      }
      currentKey = top[1];
      const val = top[2].trim();
      currentObj = null;
      currentArr = null;
      arrKey = null;
      if (val === '' || val === '|' || val === '>') {
        // object or array follows
        if (val === '') {
          // could be object or array
        }
        continue;
      }
      if (val === '[]') {
        result[currentKey] = [];
        continue;
      }
      result[currentKey] = unquote(val);
      currentKey = null;
      continue;
    }

    // bare key ending with : for list
    const bare = line.match(/^([\w_]+):\s*$/);
    if (bare) {
      currentKey = bare[1];
      arrKey = bare[1];
      currentArr = [];
      currentObj = null;
      result[currentKey] = currentArr;
    }
  }

  return result;
}

function unquote(s: string): string {
  if (
    (s.startsWith('"') && s.endsWith('"')) ||
    (s.startsWith("'") && s.endsWith("'"))
  ) {
    return s.slice(1, -1);
  }
  return s;
}

export function extractHeadings(body: string): DocPage['headings'] {
  const headings: DocPage['headings'] = [];
  const re = /^(#{2,3})\s+(.+)$/gm;
  let m: RegExpExecArray | null;
  while ((m = re.exec(body))) {
    const text = m[2].replace(/`/g, '').trim();
    headings.push({
      depth: m[1].length,
      slug: slugify(text),
      text,
    });
  }
  return headings;
}

export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, '')
    .trim()
    .replace(/\s+/g, '-');
}
