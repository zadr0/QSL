import { describe, expect, it } from 'vitest';

import { isRootRelative, localeFromContentPath, localizeHref, switchLocalePath } from './links.mjs';

describe('isRootRelative', () => {
	it.each([
		['/start/', true],
		['//cdn.example/x', false],
		['https://docs.rs/', false],
		['#anchor', false],
		['../relative/', false],
	])('%s -> %s', (href, expected) => {
		expect(isRootRelative(href)).toBe(expected);
	});
});

describe('localizeHref', () => {
	it('adds base and locale', () => {
		expect(localizeHref('/start/quickstart/', { base: '/QSL', locale: 'ja' })).toBe(
			'/QSL/ja/start/quickstart/',
		);
	});

	it('adds only the base for the root locale', () => {
		expect(localizeHref('/start/quickstart/', { base: '/QSL/', locale: undefined })).toBe(
			'/QSL/start/quickstart/',
		);
	});

	it('keeps fragments', () => {
		expect(localizeHref('/guides/memory/#rt-slot-pool', { base: '/QSL', locale: 'de' })).toBe(
			'/QSL/de/guides/memory/#rt-slot-pool',
		);
	});

	it('is idempotent and leaves external links alone', () => {
		expect(localizeHref('/QSL/ru/start/', { base: '/QSL', locale: 'ru' })).toBe('/QSL/ru/start/');
		expect(localizeHref('https://docs.rs/qsl/', { base: '/QSL', locale: 'ru' })).toBe('https://docs.rs/qsl/');
	});

	it('works without a base path', () => {
		expect(localizeHref('/start/', { base: '/', locale: 'fr' })).toBe('/fr/start/');
	});
});

describe('localeFromContentPath', () => {
	const prefixes = ['ru', 'zh-cn', 'pt-br'];

	it('finds the locale segment on POSIX and Windows paths', () => {
		expect(localeFromContentPath('/repo/src/content/docs/zh-cn/start/quickstart.mdx', prefixes)).toBe(
			'zh-cn',
		);
		expect(localeFromContentPath('C:\\repo\\src\\content\\docs\\pt-br\\index.mdx', prefixes)).toBe('pt-br');
	});

	it('returns undefined for root-locale pages and unknown paths', () => {
		expect(localeFromContentPath('/repo/src/content/docs/start/quickstart.mdx', prefixes)).toBeUndefined();
		expect(localeFromContentPath('/elsewhere/file.md', prefixes)).toBeUndefined();
		expect(localeFromContentPath(undefined, prefixes)).toBeUndefined();
	});
});

describe('switchLocalePath', () => {
	it.each([
		['/QSL/guides/memory/', undefined, 'ja', '/QSL/ja/guides/memory/'],
		['/QSL/ru/guides/memory/', 'ru', undefined, '/QSL/guides/memory/'],
		['/QSL/ru/guides/memory/', 'ru', 'pt-br', '/QSL/pt-br/guides/memory/'],
		['/QSL/', undefined, 'de', '/QSL/de/'],
		['/QSL/fr/', 'fr', undefined, '/QSL/'],
		['/QSL/fr', 'fr', 'ko', '/QSL/ko/'],
	])('%s from %s to %s', (pathname, from, to, expected) => {
		expect(switchLocalePath(pathname, { base: '/QSL/', from, to })).toBe(expected);
	});

	it('does not strip a segment that only starts like the locale', () => {
		expect(switchLocalePath('/QSL/ruby/', { base: '/QSL', from: 'ru', to: 'de' })).toBe('/QSL/de/ruby/');
	});

	it('works without a base path', () => {
		expect(switchLocalePath('/ja/start/', { base: '/', from: 'ja', to: undefined })).toBe('/start/');
	});
});
