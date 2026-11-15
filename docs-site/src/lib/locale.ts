/** Language selection for the site entry point. */

/** Locale explicitly chosen in the language menu. */
export const LOCALE_STORAGE_KEY = 'qsl-locale';

/**
 * Open English on the first visit. Remember an explicit language choice;
 * browser and operating-system languages do not change the default.
 */
export function pickLocale(
	locales: Readonly<Record<string, string>>,
	stored: string | null,
	fallback: string,
): string {
	return stored && Object.hasOwn(locales, stored) ? stored : fallback;
}
