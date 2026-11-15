/**
 * Choosing the language a reader lands in at `/QSL/`.
 */

/** `localStorage` key holding the locale a reader picked in the language menu. */
export const LOCALE_STORAGE_KEY = 'qsl-locale';

/**
 * Pick the site locale for a reader.
 *
 * A locale the reader chose before wins. Otherwise the browser's languages
 * are tried in order of preference, first as a full tag (`pt-BR` matches the
 * `pt-br` locale), then by primary language (`pt-PT` and `pt` also match
 * `pt-br`, `zh-Hans` matches `zh-cn`). The fallback covers everything else.
 *
 * @param locales Site locales: URL segment → BCP 47 tag, e.g. `{ 'pt-br': 'pt-BR' }`.
 * @param preferred The browser's languages, most preferred first (`navigator.languages`).
 * @param stored A locale segment saved earlier, if any.
 * @param fallback Locale segment to use when nothing matches.
 */
export function pickLocale(
	locales: Readonly<Record<string, string>>,
	preferred: readonly string[],
	stored: string | null,
	fallback: string,
): string {
	if (stored && stored in locales) return stored;
	const entries = Object.entries(locales).map(([key, tag]) => [key, tag.toLowerCase()] as const);
	const primary = (tag: string) => tag.toLowerCase().split('-')[0];
	for (const tag of preferred) {
		const exact = entries.find(([, lang]) => lang === tag.toLowerCase());
		if (exact) return exact[0];
		const sameLanguage = entries.find(([, lang]) => primary(lang) === primary(tag));
		if (sameLanguage) return sameLanguage[0];
	}
	return fallback;
}
