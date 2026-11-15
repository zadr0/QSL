// @ts-check
/**
 * Sätteri hast plugin that rewrites root-relative links in docs content so
 * they include the site base path and the locale of the page being rendered.
 *
 * It covers links written in Markdown syntax (`[text](/start/)`) and string
 * `href` attributes of MDX elements and components such as `<LinkCard>`.
 * Two cases are left alone:
 * - attributes written as expressions (`href={url}`);
 * - `<a>` tags written as raw HTML inside `.md` files, which reach the hast
 *   tree as unparsed `raw` nodes. `npm run test:links` reports them.
 *
 * The rewriting rules are in `src/lib/links.mjs`.
 */
import { fileURLToPath } from 'node:url';

import { localeFromContentPath, localizeHref } from '../lib/links.mjs';

/** @typedef {NonNullable<import('@astrojs/markdown-satteri').SatteriProcessorOptions['hastPlugins']>[number]} HastPluginEntry */

/**
 * @typedef {object} Options
 * @property {string} base Deployment base path, e.g. `/QSL`.
 * @property {readonly string[]} localePrefixes URL segments of the locales.
 */

/**
 * Create the plugin for `markdown.processor` in `astro.config.mjs`.
 *
 * @param {Options} options
 * @returns {HastPluginEntry}
 */
export function localizeLinks({ base, localePrefixes }) {
	return ({ fileURL }) => {
		const locale = fileURL ? localeFromContentPath(fileURLToPath(fileURL), localePrefixes) : undefined;

		/**
		 * The localized form of `href`, or `undefined` when it must stay as written.
		 *
		 * @param {unknown} href
		 * @returns {string | undefined}
		 */
		const localize = (href) => {
			if (typeof href !== 'string') return undefined;
			const localized = localizeHref(href, { base, locale });
			return localized === href ? undefined : localized;
		};

		/**
		 * The string `href` attribute of an MDX element, if it has one.
		 *
		 * @param {ReadonlyArray<{ type: string; name?: string; value?: unknown }>} attributes
		 */
		const jsxHref = (attributes) =>
			attributes.find((attribute) => attribute.type === 'mdxJsxAttribute' && attribute.name === 'href')
				?.value;

		return {
			name: 'qsl-localize-links',
			element: {
				filter: ['a'],
				visit(node, ctx) {
					const href = localize(node.properties.href);
					if (href) ctx.setProperty(node, 'href', href);
				},
			},
			mdxJsxFlowElement: {
				filter: [],
				visit(node, ctx) {
					const href = localize(jsxHref(node.attributes));
					if (href) ctx.setProperty(node, 'href', href);
				},
			},
			mdxJsxTextElement: {
				filter: [],
				visit(node, ctx) {
					const href = localize(jsxHref(node.attributes));
					if (href) ctx.setProperty(node, 'href', href);
				},
			},
		};
	};
}
