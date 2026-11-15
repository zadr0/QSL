// @ts-check
import { satteri } from '@astrojs/markdown-satteri';
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import {latexMath} from './src/plugins/math.mjs';

import { sidebar } from './config/sidebar.mjs';
import { BASE_PATH, DEFAULT_LOCALE, LOCALE_PREFIXES, LOCALES, SITE_URL } from './config/site.mjs';
import { localizeLinks } from './src/plugins/localize-links.mjs';

// https://astro.build/config
export default defineConfig({
	site: SITE_URL,
	server: { host: '127.0.0.1', port: 4322 },
	vite: { server: { strictPort: true } },
	base: BASE_PATH,
	trailingSlash: 'always',
	redirects: Object.fromEntries(LOCALE_PREFIXES.flatMap(locale => [
		[`/${locale}/reference/complete/`, `${BASE_PATH.replace(/\/$/, "")}/${locale}/reference/attributes/`],
		[`/${locale}/reference/glossary/`, `${BASE_PATH.replace(/\/$/, "")}/${locale}/start/introduction/`],
	])),
	markdown: {
		processor: satteri({
			// `## Heading {#id}` pins an anchor, so links keep working when a
			// translation changes the heading text.
			features: { headingAttributes: true, math: true },
            mdastPlugins: [latexMath()],
			hastPlugins: [localizeLinks({ base: BASE_PATH, localePrefixes: LOCALE_PREFIXES })],
		}),
	},
	integrations: [
		starlight({
			title: 'QSL',
			description: 'QSL compiles data schemas into Rust structs and binary codecs at build time.',
			defaultLocale: DEFAULT_LOCALE,
			locales: LOCALES,
			favicon: '/favicon.svg',
			social: [],
			customCss: [
				'@fontsource-variable/geologica/shrp.css',
				'@fontsource-variable/jetbrains-mono/wght.css',
				'@fontsource-variable/literata/opsz.css',
				'./src/styles/theme.css',
				'./src/styles/demo.css',
                'katex/dist/katex.min.css',
			],
			components: {
				Footer: './src/components/Footer.astro',
				Head: './src/components/Head.astro',
				Header: './src/components/Header.astro',
				Hero: './src/components/Hero.astro',
				LanguageSelect: './src/components/LanguageSelect.astro',
				SiteTitle: './src/components/SiteTitle.astro',
				ThemeSelect: './src/components/ThemeSelect.astro',
			},
			sidebar,
		}),
	],
});
