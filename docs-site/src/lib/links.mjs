// @ts-check
/**
 * Link helpers shared by the Markdown plugin and components.
 *
 * Content is written with root-relative links that know nothing about the
 * deployment path or the page language, for example `/start/quickstart/`.
 * At build time these are rewritten to `/QSL/start/quickstart/` on English
 * pages and `/QSL/ja/start/quickstart/` on Japanese pages. Translators never
 * edit link targets.
 */

/**
 * Whether `href` is a path on this site (`/x`), as opposed to an external URL,
 * a protocol-relative URL (`//host/x`), a fragment or a relative path.
 *
 * @param {string} href
 * @returns {boolean}
 */
export function isRootRelative(href) {
	return href.startsWith('/') && !href.startsWith('//');
}

/**
 * Prefix a root-relative link with the base path and locale segment.
 * Links that are external or already include the base path are returned
 * unchanged, so the function is safe to apply more than once.
 *
 * @param {string} href Link as written in content.
 * @param {{ base: string; locale?: string | undefined }} options
 *   `base` is the deployment path (`/QSL` or `/QSL/`); `locale` is the URL
 *   segment of the current page, or `undefined` for a path without one.
 * @returns {string}
 */
export function localizeHref(href, { base, locale }) {
	if (!isRootRelative(href)) return href;
	const basePath = base.replace(/\/+$/, '');
	if (basePath && (href === basePath || href.startsWith(`${basePath}/`))) return href;
	if (/^\/(?:downloads|evidence|source|data|api|_astro|en|ru|zh-cn|ja|ko|es|pt-br|de|fr)\//.test(href)) return `${basePath}${href}`;
	const localePath = locale ? `/${locale}` : '';
	return `${basePath}${localePath}${href}`;
}

/**
 * Find the locale of a content file from its path.
 *
 * @param {string | undefined} filePath Absolute path of the Markdown file.
 * @param {readonly string[]} localePrefixes URL segments of the locales.
 * @returns {string | undefined} Locale segment, or `undefined` outside a locale folder.
 */
export function localeFromContentPath(filePath, localePrefixes) {
	if (!filePath) return undefined;
	const normalized = filePath.replaceAll('\\', '/');
	const marker = '/src/content/docs/';
	const index = normalized.lastIndexOf(marker);
	if (index === -1) return undefined;
	const firstSegment = normalized.slice(index + marker.length).split('/')[0];
	return localePrefixes.includes(firstSegment) ? firstSegment : undefined;
}

/**
 * Path of the current page in another locale.
 *
 * @param {string} pathname Current URL path, e.g. `/QSL/ru/guides/memory/`.
 * @param {{ base: string; from?: string | undefined; to?: string | undefined }} options
 *   `from` is the locale segment of the current page and `to` the target one;
 *   `undefined` stands for a path without a locale segment.
 * @returns {string} For example `/QSL/ja/guides/memory/`.
 */
export function switchLocalePath(pathname, { base, from, to }) {
	const basePath = base.replace(/\/+$/, '');
	let rest = basePath && pathname.startsWith(basePath) ? pathname.slice(basePath.length) : pathname;
	if (from && (rest === `/${from}` || rest.startsWith(`/${from}/`))) rest = rest.slice(from.length + 1);
	if (!rest.startsWith('/')) rest = `/${rest}`;
	return `${basePath}${to ? `/${to}` : ''}${rest}`;
}
