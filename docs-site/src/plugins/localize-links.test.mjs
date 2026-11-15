import { pathToFileURL } from 'node:url';

import { satteri } from '@astrojs/markdown-satteri';
import { describe, expect, it } from 'vitest';

import { localizeLinks } from './localize-links.mjs';

// Run the plugin through the same processor as the site build, so the tests
// check rendered output rather than the plugin's visitor shape.
const processor = satteri({ hastPlugins: [localizeLinks({ base: '/QSL', localePrefixes: ['ru', 'ja'] })] });
const markdown = await processor.createRenderer({ syntaxHighlight: false });
const mdx =
	processor.createMdxRenderer && (await processor.createMdxRenderer({ syntaxHighlight: false }, {}));

/** Render Markdown as if it were the content file at `relativePath`. */
async function renderMarkdown(source, relativePath) {
	const fileURL = pathToFileURL(`/repo/src/content/docs/${relativePath}`);
	return (await markdown.render(source, { fileURL })).code;
}

/** Compile MDX as if it were the content file at `relativePath`; returns the generated module. */
async function compileMdx(source, relativePath) {
	if (!mdx) throw new Error('The Sätteri processor has no MDX renderer');
	return (await mdx.process(source, `/repo/src/content/docs/${relativePath}`, {})).code;
}

describe('localizeLinks in Markdown', () => {
	it('prefixes links with the base on root-locale pages', async () => {
		expect(await renderMarkdown('[quick start](/start/quickstart/)', 'guides/memory.md')).toContain(
			'href="/QSL/start/quickstart/"',
		);
	});

	it('adds the locale of the page and keeps fragments', async () => {
		expect(await renderMarkdown('[pool](/guides/memory/#rt-slot-pool)', 'ja/guides/safety.md')).toContain(
			'href="/QSL/ja/guides/memory/#rt-slot-pool"',
		);
	});

	it('leaves external links, fragments and localized links untouched', async () => {
		const html = await renderMarkdown(
			'[docs.rs](https://docs.rs/qsl/) [layout](#layout) [done](/QSL/ru/start/)',
			'ru/index.md',
		);
		expect(html).toContain('href="https://docs.rs/qsl/"');
		expect(html).toContain('href="#layout"');
		expect(html).toContain('href="/QSL/ru/start/"');
	});

	it('does not rewrite raw HTML anchors', async () => {
		expect(await renderMarkdown('<a href="/start/">start</a>', 'index.md')).toContain('href="/start/"');
	});
});

describe('localizeLinks in MDX', () => {
	const imports = "import { LinkCard } from '@astrojs/starlight/components';\n\n";

	it('rewrites Markdown links and string href attributes of components', async () => {
		const code = await compileMdx(
			`${imports}[a](/start/)\n\n<LinkCard title="Q" href="/start/quickstart/" />\n\nSee <LinkCard href="/guides/safety/" /> too.`,
			'ru/index.mdx',
		);
		expect(code).toContain('href: "/QSL/ru/start/"');
		expect(code).toContain('href: "/QSL/ru/start/quickstart/"');
		expect(code).toContain('href: "/QSL/ru/guides/safety/"');
	});

	it('skips expression attributes', async () => {
		const code = await compileMdx(`${imports}<LinkCard title="Q" href={'/start/'} />`, 'index.mdx');
		expect(code).not.toContain('/QSL/start/');
	});
});
