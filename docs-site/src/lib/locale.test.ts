import { describe, expect, it } from 'vitest';

import { pickLocale } from './locale';

const LOCALES = { en: 'en', ru: 'ru', 'zh-cn': 'zh-CN', 'pt-br': 'pt-BR', de: 'de' };

describe('pickLocale', () => {
	it('opens English on the first visit', () => {
		expect(pickLocale(LOCALES, null, 'en')).toBe('en');
	});

	it.each(['ru', 'de', 'pt-br'])('remembers the explicit %s choice', (stored) => {
		expect(pickLocale(LOCALES, stored, 'en')).toBe(stored);
	});

	it.each(['xx', '', 'toString'])('ignores invalid stored choice %j', (stored) => {
		expect(pickLocale(LOCALES, stored, 'en')).toBe('en');
	});
});
