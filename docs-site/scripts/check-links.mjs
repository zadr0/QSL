#!/usr/bin/env node
// @ts-check
/**
 * Checks every internal link and asset reference in the built site.
 *
 * For each HTML file in the output directory it reads `href` and `src`
 * attributes that point to this site (`/QSL/...` or `https://zadr0.github.io/QSL/...`),
 * resolves them to files, and for `#fragment` links checks that the target
 * page has an element with that `id`. Root-relative links outside the base
 * path (`/start/`) are reported too: on GitHub Pages they lead off the site,
 * which usually means a link in content was not localized.
 *
 * Usage: node scripts/check-links.mjs [outDir=dist]
 * Exit code 1 if any link is broken.
 */
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

import { BASE_PATH, SITE_URL } from '../config/site.mjs';
import { isRootRelative } from '../src/lib/links.mjs';

/** @typedef {{ page: string; link: string; reason: string }} BrokenLink */
/** @typedef {{ tag: string; url: string }} Link */

/**
 * Starlight gives its 404 page canonical and hreflang URLs of a `/404/` route
 * that is never built. The page is served with status 404, so crawlers ignore
 * them; only its visible links and assets are checked.
 */
const NOT_FOUND_PAGE = '404.html';

/**
 * Extract the `href` or `src` value of every tag in HTML.
 *
 * @param {string} html
 * @returns {Link[]}
 */
export function extractLinks(html) {
	const links = [];
	for (const match of html.matchAll(/<([a-zA-Z][\w-]*)\b[^>]*?\s(?:href|src)=(?:"([^"]*)"|'([^']*)')/g)) {
		links.push({ tag: match[1].toLowerCase(), url: decodeEntities(match[2] ?? match[3] ?? '') });
	}
	return links;
}

/**
 * Extract all `id` attribute values from HTML.
 *
 * @param {string} html
 * @returns {Set<string>}
 */
export function extractIds(html) {
	const ids = new Set();
	for (const match of html.matchAll(/\sid=(?:"([^"]*)"|'([^']*)')/g)) {
		ids.add(decodeEntities(match[1] ?? match[2] ?? ''));
	}
	return ids;
}

/**
 * Turn a link into a site path without the base, or `null` if it is not a
 * link to this site.
 *
 * @param {string} link
 * @param {{ base: string; site: string }} options
 * @returns {{ path: string; fragment: string } | null}
 */
export function toSitePath(link, { base, site }) {
	let url = link;
	if (url.startsWith(site)) url = url.slice(site.length);
	if (!url.startsWith(base)) return null;
	const rest = url.slice(base.length);
	if (rest !== '' && !rest.startsWith('/') && !rest.startsWith('#') && !rest.startsWith('?')) return null;
	const [withoutFragment, fragment = ''] = rest.split('#', 2);
	const [path] = withoutFragment.split('?', 1);
	return { path: path === '' ? '/' : path, fragment };
}

/**
 * Decode the entities Astro uses in attribute values.
 *
 * @param {string} text
 */
function decodeEntities(text) {
	return text.replaceAll('&amp;', '&').replaceAll('&quot;', '"').replaceAll('&#39;', "'");
}

/**
 * List every HTML file under `dir`.
 *
 * @param {string} dir
 * @returns {string[]}
 */
function listHtmlFiles(dir) {
	return readdirSync(dir, { recursive: true, encoding: 'utf8' })
		.filter((file) => file.endsWith('.html'))
		.map((file) => join(dir, file));
}

/**
 * Decode a percent-encoded URL part, or return `null` if it is malformed.
 *
 * @param {string} text
 * @returns {string | null}
 */
function decodeUrlPart(text) {
	try {
		return decodeURIComponent(text);
	} catch {
		return null;
	}
}

/**
 * Resolve a site path to a file in the output directory.
 *
 * @param {string} outDir
 * @param {string} sitePath
 * @returns {string | null}
 */
function resolveFile(outDir, sitePath) {
	const decoded = decodeUrlPart(sitePath);
	if (decoded === null) return null;
	const candidates = decoded.endsWith('/')
		? [join(outDir, decoded, 'index.html')]
		: [join(outDir, decoded), join(outDir, decoded, 'index.html'), join(outDir, `${decoded}.html`)];
	return candidates.find((file) => existsSync(file) && statSync(file).isFile()) ?? null;
}

/**
 * Check every HTML file in `outDir` and list the links that do not resolve.
 *
 * @param {string} outDir
 * @param {{base?:string,site?:string}} [options]
 * @returns {{ broken: BrokenLink[]; pageCount: number }}
 */
export function checkLinks(outDir, {base=BASE_PATH,site=SITE_URL}={}) {
	const pages = listHtmlFiles(outDir);
	/** @type {Map<string, Set<string>>} */
	const idsByFile = new Map();
	const idsOf = (/** @type {string} */ file) => {
		let ids = idsByFile.get(file);
		if (!ids) {
			ids = extractIds(readFileSync(file, 'utf8'));
			idsByFile.set(file, ids);
		}
		return ids;
	};

	/** @type {BrokenLink[]} */
	const broken = [];
	for (const page of pages) {
		const html = readFileSync(page, 'utf8');
		const pageName = relative(outDir, page).split(sep).join('/');
		for (const { tag, url: link } of extractLinks(html)) {
			if (pageName === NOT_FOUND_PAGE && tag === 'link') continue;
			if (link.startsWith('#')) {
				const fragment = decodeUrlPart(link.slice(1));
				if (fragment !== '' && (fragment === null || !idsOf(page).has(fragment))) {
					broken.push({ page: pageName, link, reason: 'missing anchor on the same page' });
				}
				continue;
			}
			const target = toSitePath(link, { base, site });
			if (!target) {
				if (isRootRelative(link)) broken.push({ page: pageName, link, reason: 'outside the base path' });
				continue;
			}
			const file = resolveFile(outDir, target.path);
			if (!file) {
				broken.push({ page: pageName, link, reason: 'no such file' });
				continue;
			}
			const fragment = decodeUrlPart(target.fragment);
			if (fragment !== '' && file.endsWith('.html') && (fragment === null || !idsOf(file).has(fragment))) {
				broken.push({ page: pageName, link, reason: 'missing anchor' });
			}
		}
	}
	return { broken, pageCount: pages.length };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
	const outDir = process.argv[2] ?? 'dist';
	if (!existsSync(outDir)) {
		console.error(`Output directory "${outDir}" not found. Run "npm run build" first.`);
		process.exit(1);
	}
	const { broken, pageCount } = checkLinks(outDir);
	const pages = `${pageCount} ${pageCount === 1 ? 'page' : 'pages'}`;
	if (broken.length > 0) {
		for (const { page, link, reason } of broken) console.error(`${page}: ${link} (${reason})`);
		console.error(`\n${broken.length} broken ${broken.length === 1 ? 'link' : 'links'} in ${pages}.`);
		process.exit(1);
	}
	console.log(`All internal links resolve (${pages} checked).`);
}
