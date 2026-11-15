import { describe, expect, it } from 'vitest';

import { pickLocale } from './locale';

const LOCALES = { en: 'en', ru: 'ru', 'zh-cn': 'zh-CN', 'pt-br': 'pt-BR', de: 'de' };

describe('pickLocale', () => {
	it('prefers a locale the reader chose before', () => {
		expect(pickLocale(LOCALES, ['de-DE'], 'ru', 'en')).toBe('ru');
	});

	it('ignores a stored value that is not a locale', () => {
		expect(pickLocale(LOCALES, ['de-DE'], 'xx', 'en')).toBe('de');
	});

	it.each([
		[['pt-BR'], 'pt-br'],
		[['pt-PT'], 'pt-br'],
		[['zh-Hans-CN'], 'zh-cn'],
		[['ZH-cn'], 'zh-cn'],
		[['de-AT', 'ru'], 'de'],
		[['it', 'ru'], 'ru'],
	])('maps browser languages %j to %s', (preferred, expected) => {
		expect(pickLocale(LOCALES, preferred, null, 'en')).toBe(expected);
	});

	it('falls back when nothing matches', () => {
		expect(pickLocale(LOCALES, ['it', 'nl'], null, 'en')).toBe('en');
		expect(pickLocale(LOCALES, [], null, 'en')).toBe('en');
	});
});
