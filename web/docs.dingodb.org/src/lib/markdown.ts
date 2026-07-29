import { marked, type Tokens } from 'marked';
import { slugify } from './frontmatter';

marked.setOptions({
  gfm: true,
  breaks: false,
});

const renderer = new marked.Renderer();

renderer.heading = function ({ tokens, depth }: Tokens.Heading) {
  const text = this.parser.parseInline(tokens);
  const plain = text.replace(/<[^>]+>/g, '');
  const id = slugify(plain);
  return `<h${depth} id="${id}"><a class="heading-anchor" href="#${id}">${text}</a></h${depth}>\n`;
};

renderer.code = function ({ text, lang }: Tokens.Code) {
  const language = lang || 'text';
  const id = `code-${Math.random().toString(36).slice(2, 9)}`;
  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
  return `<div class="code-block"><div class="code-block-head"><span>${language}</span><button type="button" class="copy-btn" data-copy-target="${id}">Copy</button></div><pre id="${id}" tabindex="0"><code class="language-${language}">${escaped}</code></pre></div>\n`;
};

renderer.table = function (token: Tokens.Table) {
  const header = token.header
    .map((cell) => `<th>${this.parser.parseInline(cell.tokens)}</th>`)
    .join('');
  const body = token.rows
    .map(
      (row) =>
        `<tr>${row
          .map((cell) => `<td>${this.parser.parseInline(cell.tokens)}</td>`)
          .join('')}</tr>`,
    )
    .join('');
  return `<div class="table-wrap" tabindex="0"><table><thead><tr>${header}</tr></thead><tbody>${body}</tbody></table></div>\n`;
};

export function renderMarkdown(src: string): string {
  return marked.parse(src, { renderer }) as string;
}
