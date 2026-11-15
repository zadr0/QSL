#!/usr/bin/env node
// @ts-check
/**
 * Compares every translation (`src/content/docs/<locale>/`) with the English
 * source (`src/content/docs/en/`) and reports drift.
 *
 * For each translation locale in `config/site.mjs` it checks that:
 * - every English page has a translation (missing pages fall back to English
 *   on the site, so this is a warning unless `--strict` is passed);
 * - each translated page has a `title` and a `description`;
 * - every relative import of a translated page points at an existing file;
 * - the page keeps the structure of the English page: the same sequence of
 *   heading levels, code block languages, MDX components and anchors
 *   (`{#id}` after a heading, `id="..."` on a component), and the same set
 *   of link targets;
 * - `src/content/i18n/<lang>.json` has no keys that English lacks, and (with
 *   `--strict`) every English key.
 *
 * Structure is compared rather than text, so the check catches a dropped
 * section, a translated link or a lost code block without judging wording.
 *
 * Usage: node scripts/check-translations.mjs [--strict]
 * Exit code 1 on structural errors, or on missing translations with --strict.
 */
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

import { DEFAULT_LOCALE, LOCALES, TRANSLATION_LOCALES } from '../config/site.mjs';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const DOCS_DIR = join(ROOT, 'src', 'content', 'docs');
const I18N_DIR = join(ROOT, 'src', 'content', 'i18n');

/**
 * @typedef {object} PageShape
 * @property {Record<string, unknown>} frontmatter Top-level scalar keys only.
 * @property {string[]} headings Heading levels in order, e.g. `['h2', 'h3']`.
 * @property {string[]} codeBlocks Fence languages in order.
 * @property {string[]} components MDX component names in order.
 * @property {string[]} anchors Explicit ids in order: `{#id}` after headings and `id` props.
 * @property {string[]} links Sorted link targets (Markdown and `href` props).
 */

/**
 * Split a page into frontmatter and body.
 *
 * @param {string} source
 * @returns {{ frontmatter: Record<string, string>; body: string }}
 */
export function splitFrontmatter(source) {
	const match = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?/.exec(source);
	if (!match) return { frontmatter: {}, body: source };
	/** @type {Record<string, string>} */
	const frontmatter = {};
	for (const line of match[1].split(/\r?\n/)) {
		const field = /^([A-Za-z][\w-]*):\s*(.*)$/.exec(line);
		if (field) frontmatter[field[1]] = field[2].trim();
	}
	return { frontmatter, body: source.slice(match[0].length) };
}

/**
 * Describe the structure of a Markdown or MDX page.
 *
 * @param {string} source
 * @returns {PageShape}
 */
export function pageShape(source) {
	const { frontmatter, body } = splitFrontmatter(source);
	/** @type {string[]} */
	const headings = [];
	/** @type {string[]} */
	const codeBlocks = [];
	/** @type {string[]} */
	const components = [];
	/** @type {string[]} */
	const anchors = [];
	/** @type {string[]} */
	const links = [];

	let fence = '';
	for (const line of body.split(/\r?\n/)) {
		const fenceMatch = /^\s*(`{3,}|~{3,})(\S*)/.exec(line);
		if (fence) {
			if (fenceMatch && fenceMatch[1].startsWith(fence) && !fenceMatch[2]) fence = '';
			continue;
		}
		if (fenceMatch) {
			fence = fenceMatch[1];
			codeBlocks.push(fenceMatch[2] || 'text');
			continue;
		}
		const heading = /^(#{1,6})\s.*?(?:\{#([\w-]+)\})?\s*$/.exec(line);
		if (heading) {
			headings.push(`h${heading[1].length}`);
			if (heading[2]) anchors.push(heading[2]);
		}
		// Inline code is text: `CachedFrame<N>` is not a component.
		const markup = line.replace(/`[^`\n]*`/g, '``');
		for (const tag of markup.matchAll(/<([A-Z][A-Za-z]*)\b/g)) components.push(tag[1]);
		for (const link of markup.matchAll(/\]\(([^)\s]+)(?:\s[^)]*)?\)/g)) links.push(link[1]);
		for (const link of markup.matchAll(/\shref=["']([^"']+)["']/g)) links.push(link[1]);
		for (const id of markup.matchAll(/<[A-Z][^>]*\sid=["']([^"']+)["']/g)) anchors.push(id[1]);
	}
	return { frontmatter, headings, codeBlocks, components, anchors, links: links.sort() };
}

/**
 * List differences between an English page and its translation.
 *
 * @param {PageShape} source
 * @param {PageShape} translation
 * @returns {string[]}
 */
export function compareShapes(source, translation) {
	const problems = [];
	for (const key of ['title', 'description']) {
		if (!translation.frontmatter[key]) problems.push(`missing frontmatter "${key}"`);
	}
	/** @type {Array<[keyof PageShape, string]>} */
	const sequences = [
		['headings', 'headings'],
		['codeBlocks', 'code blocks'],
		['components', 'components'],
		['anchors', 'anchors'],
		['links', 'links'],
	];
	for (const [key, label] of sequences) {
		const a = /** @type {string[]} */ (source[key]);
		const b = /** @type {string[]} */ (translation[key]);
		if (a.join('\n') !== b.join('\n')) {
			problems.push(`${label} differ: expected [${a.join(', ')}], found [${b.join(', ')}]`);
		}
	}
	return problems;
}

/**
 * List the relative imports of a page that do not point at an existing file.
 *
 * @param {string} source
 * @param {string} file Absolute path of the page.
 * @returns {string[]} The unresolved import paths, as written.
 */
export function unresolvedImports(source, file) {
	const { body } = splitFrontmatter(source);
	return [...body.matchAll(/^import\s[^;]*?\sfrom\s+["'](\.{1,2}\/[^"']+)["']/gm)]
		.map((match) => match[1])
		.filter((path) => !existsSync(join(dirname(file), path)));
}

/**
 * @param {string} dir
 * @returns {string[]} Paths relative to `dir`, with forward slashes.
 */
function listPages(dir) {
	if (!existsSync(dir)) return [];
	return readdirSync(dir, { recursive: true, encoding: 'utf8' })
		.filter((file) => /\.mdx?$/.test(file))
		.map((file) => file.split(sep).join('/'));
}

/**
 * @param {{ strict: boolean }} options
 * @returns {{ errors: string[]; warnings: string[] }}
 */
export function checkTranslations({ strict }) {
	/** @type {string[]} */
	const errors = [];
	/** @type {string[]} */
	const warnings = [];
	const englishDir = join(DOCS_DIR, DEFAULT_LOCALE);
	const englishPages = listPages(englishDir);
	const englishStrings = Object.keys(JSON.parse(readFileSync(join(I18N_DIR, 'en.json'), 'utf8')));

	for (const locale of TRANSLATION_LOCALES) {
		const localeDir = join(DOCS_DIR, locale);
		const translated = new Set(listPages(localeDir));
		for (const page of englishPages) {
			const name = `${locale}/${page}`;
			if (!translated.has(page)) {
				(strict ? errors : warnings).push(`${name}: not translated`);
				continue;
			}
			const file = join(localeDir, page);
			const text = readFileSync(file, 'utf8');
			const source = pageShape(readFileSync(join(englishDir, page), 'utf8'));
			for (const problem of compareShapes(source, pageShape(text))) errors.push(`${name}: ${problem}`);
			for (const path of unresolvedImports(text, file)) {
				errors.push(`${name}: import "${path}" does not resolve`);
			}
			translated.delete(page);
		}
		for (const extra of translated) errors.push(`${locale}/${extra}: no English page with this name`);

		const lang = LOCALES[locale].lang;
		const stringsFile = join(I18N_DIR, `${lang}.json`);
		if (!existsSync(stringsFile)) {
			(strict ? errors : warnings).push(`${relative(ROOT, stringsFile)}: missing`);
			continue;
		}
		const keys = new Set(Object.keys(JSON.parse(readFileSync(stringsFile, 'utf8'))));
		for (const key of keys) {
			if (!englishStrings.includes(key)) errors.push(`${lang}.json: unknown key "${key}"`);
		}
		for (const key of englishStrings) {
			if (!keys.has(key)) (strict ? errors : warnings).push(`${lang}.json: missing key "${key}"`);
		}
	}
	return { errors, warnings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
	const strict = process.argv.includes('--strict');
	const { errors, warnings } = checkTranslations({ strict });
	for (const warning of warnings) console.warn(`warning: ${warning}`);
	for (const error of errors) console.error(`error: ${error}`);
	if (errors.length > 0) {
		console.error(`\n${errors.length} translation ${errors.length === 1 ? 'problem' : 'problems'}.`);
		process.exit(1);
	}
	console.log(
		`Translations match the English structure (${TRANSLATION_LOCALES.length} locales, ${warnings.length} warnings).`,
	);
}
