import { parseMarkdownFile, type DocPage } from './frontmatter';

const modules = import.meta.glob('../content/**/*.md', {
  eager: true,
  query: '?raw',
  import: 'default',
}) as Record<string, string>;

function pathToSlug(path: string): string {
  return path
    .replace(/^\.\.\/content\//, '')
    .replace(/\.md$/, '')
    .replace(/\/index$/, '');
}

let cache: DocPage[] | null = null;

export function getAllDocs(): DocPage[] {
  if (cache) return cache;
  const pages = Object.entries(modules).map(([path, raw]) => {
    const slug = pathToSlug(path);
    return parseMarkdownFile(raw, slug);
  });
  pages.sort((a, b) => {
    const sa = a.data.section ?? '';
    const sb = b.data.section ?? '';
    if (sa !== sb) return sa.localeCompare(sb);
    return (a.data.order ?? 0) - (b.data.order ?? 0) || a.slug.localeCompare(b.slug);
  });
  cache = pages;
  return pages;
}

export function getDoc(slug: string): DocPage | undefined {
  return getAllDocs().find((p) => p.slug === slug);
}

export function getSectionPages(section: string): DocPage[] {
  return getAllDocs().filter((p) => p.data.section === section || p.slug.startsWith(section + '/'));
}

export function getPrevNext(slug: string): { prev?: DocPage; next?: DocPage } {
  const all = getAllDocs();
  const idx = all.findIndex((p) => p.slug === slug);
  if (idx === -1) return {};
  return {
    prev: idx > 0 ? all[idx - 1] : undefined,
    next: idx < all.length - 1 ? all[idx + 1] : undefined,
  };
}

export function searchIndex() {
  return getAllDocs().map((p) => ({
    href: p.href,
    title: p.data.title,
    description: p.data.description,
    status: p.data.status,
    class: p.data.class,
    surface: p.data.applies_to.surface,
    product: p.data.applies_to.product,
    keywords: p.data.keywords,
    section: p.data.section,
    body: p.body.slice(0, 2000),
  }));
}
