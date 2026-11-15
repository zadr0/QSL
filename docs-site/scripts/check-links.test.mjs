import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';

import { afterAll, describe, expect, it } from 'vitest';

import { checkLinks, extractIds, extractLinks, toSitePath } from './check-links.mjs';

const options = { base: '/QSL', site: 'https://zadr0.github.io' };

describe('extractLinks', () => {
	it('reads href and src with either quote style and decodes entities', () => {
		const html = `<a class="x" href="/QSL/a/?x=1&amp;y=2">a</a><IMG src='/QSL/logo.svg'>`;
		expect(extractLinks(html)).toEqual([
			{ tag: 'a', url: '/QSL/a/?x=1&y=2' },
			{ tag: 'img', url: '/QSL/logo.svg' },
		]);
	});

	it('ignores attributes that only end in href', () => {
		expect(extractLinks('<link data-href="/QSL/x/">')).toEqual([]);
	});
});

describe('extractIds', () => {
	it('collects ids', () => {
		expect([...extractIds('<h2 id="one"></h2><div id=\'two\'></div>')]).toEqual(['one', 'two']);
	});
});

describe('toSitePath', () => {
	it.each([
		['/QSL/', { path: '/', fragment: '' }],
		['/QSL', { path: '/', fragment: '' }],
		['/QSL/guides/memory/#rt-slot-pool', { path: '/guides/memory/', fragment: 'rt-slot-pool' }],
		['https://zadr0.github.io/QSL/ja/', { path: '/ja/', fragment: '' }],
		['/QSL/search/?q=x', { path: '/search/', fragment: '' }],
	])('maps %s', (link, expected) => {
		expect(toSitePath(link, options)).toEqual(expected);
	});

	it.each(['https://docs.rs/qsl/', '/QSLX/', '/other/', '//cdn.example/QSL/'])('ignores %s', (link) => {
		expect(toSitePath(link, options)).toBeNull();
	});
});

describe('checkLinks', () => {
	const outDir = mkdtempSync(join(tmpdir(), 'qsl-links-'));
	afterAll(() => rmSync(outDir, { recursive: true, force: true }));

	/** @param {Record<string, string>} files Paths relative to the output directory. */
	const writeSite = (files) => {
		for (const [path, html] of Object.entries(files)) {
			mkdirSync(dirname(join(outDir, path)), { recursive: true });
			writeFileSync(join(outDir, path), html);
		}
	};

	writeSite({
		'index.html': [
			'<a href="/QSL/guides/memory/#pool">ok</a>',
			'<a href="#top">ok</a><h1 id="top">QSL</h1>',
			'<img src="/QSL/logo.svg">',
			'<a href="https://zadr0.github.io/QSL/guides/memory/">ok</a>',
			'<a href="https://docs.rs/qsl/">external</a>',
			'<a href="/QSL/guides/missing/">broken</a>',
			'<a href="/QSL/guides/memory/#nowhere">broken</a>',
			'<a href="#gone">broken</a>',
			'<a href="/guides/memory/">not localized</a>',
		].join('\n'),
		'guides/memory/index.html': '<h2 id="pool">Pool</h2>',
		'logo.svg': '<svg></svg>',
		'404.html': '<link rel="canonical" href="https://zadr0.github.io/QSL/404/"><a href="/QSL/">home</a>',
	});

	it('reports each broken link with a reason and counts the pages', () => {
		const { broken, pageCount } = checkLinks(outDir, options);
		expect(pageCount).toBe(3);
		expect(broken).toEqual([
			{ page: 'index.html', link: '/QSL/guides/missing/', reason: 'no such file' },
			{ page: 'index.html', link: '/QSL/guides/memory/#nowhere', reason: 'missing anchor' },
			{ page: 'index.html', link: '#gone', reason: 'missing anchor on the same page' },
			{ page: 'index.html', link: '/guides/memory/', reason: 'outside the base path' },
		]);
	});
});
