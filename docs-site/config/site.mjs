// @ts-check
/**
 * Site-wide settings shared by `astro.config.mjs`, Markdown plugins and
 * components. Change values here rather than in individual files.
 */

/** Stable production hostname supplied by Vercel during a build. */
const vercelHost = process.env.VERCEL_PROJECT_PRODUCTION_URL || process.env.VERCEL_URL;

/** Public origin; GitHub Pages remains the default for local and GitHub builds. */
export const SITE_URL =
	process.env.QSL_SITE_URL || (vercelHost ? `https://${vercelHost}` : 'http://127.0.0.1:4322');

/** Vercel serves the site at the domain root; GitHub Pages uses the project path. */
export const BASE_PATH = process.env.QSL_BASE_PATH ?? '';

/** Source repository of QSL. */
export const REPO_URL = 'https://github.com/zadr0/QSL';

/** QSL release the documentation describes. */
export const QSL_VERSION = '0.4.0';

/** Version of the qsl-core runtime shipped with `QSL_VERSION`. */
export const QSL_CORE_VERSION = '0.2.0';

/** Minimum supported Rust version of `QSL_VERSION`. */
export const RUST_VERSION = '1.98';

/** The `qsl` crate on crates.io. */
export const CRATES_URL = 'https://crates.io/crates/qsl';

/** Rustdoc for `QSL_VERSION`. */
export const DOCS_RS_URL = `https://docs.rs/qsl/${QSL_VERSION}/qsl/`;

/**
 * Content locales, keyed by URL segment. Every language, English included,
 * has its own folder `src/content/docs/<key>/` and URL prefix `/QSL/<key>/`.
 * `/QSL/` itself sends readers to their browser's language (src/pages/index.astro).
 *
 * To add a language: add an entry here, add translated sidebar labels in
 * `config/sidebar.mjs`, then add `src/content/docs/<key>/` and
 * `src/content/i18n/<lang>.json`. See CONTRIBUTING.md.
 *
 * @type {Record<string, { label: string; lang: string }>}
 */
export const LOCALES = {
	en: { label: 'English', lang: 'en' },
	ru: { label: 'Русский', lang: 'ru' },
	'zh-cn': { label: '简体中文', lang: 'zh-CN' },
	ja: { label: '日本語', lang: 'ja' },
	ko: { label: '한국어', lang: 'ko' },
	es: { label: 'Español', lang: 'es' },
	'pt-br': { label: 'Português (Brasil)', lang: 'pt-BR' },
	de: { label: 'Deutsch', lang: 'de' },
	fr: { label: 'Français', lang: 'fr' },
};

/** The source language. Other languages are translations of it, and fall back to it page by page. */
export const DEFAULT_LOCALE = 'en';

/** URL segments of all locales, e.g. `['en', 'ru', 'zh-cn', ...]`. */
export const LOCALE_PREFIXES = Object.keys(LOCALES);

/** URL segments of the translations: every locale except `DEFAULT_LOCALE`. */
export const TRANSLATION_LOCALES = LOCALE_PREFIXES.filter((key) => key !== DEFAULT_LOCALE);

export const DOCS_EDITION = QSL_VERSION;
