import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

/**
 * UI strings used by the custom components in `src/components/`.
 * Every key is optional: a missing translation falls back to English
 * (`src/content/i18n/en.json`).
 */
const componentStrings = z
	.object({
		'qsl.nav.label': z.string(),
		'qsl.nav.docs': z.string(),
		'qsl.nav.reference': z.string(),
		'qsl.nav.api': z.string(),
		'qsl.theme.toLight': z.string(),
		'qsl.theme.toDark': z.string(),
		'qsl.footer.license': z.string(),
		'qsl.hero.pause': z.string(),
		'qsl.wire.encoding': z.string(),
		'qsl.wire.values': z.string(),
		'qsl.wire.mode.default': z.string(),
		'qsl.wire.mode.compact': z.string(),
		'qsl.wire.mode.aligned': z.string(),
		'qsl.wire.bytes': z.string(),
		'qsl.wire.bitOrder': z.string(),
		'qsl.wire.padding': z.string(),
		'qsl.bitmap.present': z.string(),
		'qsl.bitmap.mask': z.string(),
		'qsl.bitmap.readOrder': z.string(),
		'qsl.bitmap.none': z.string(),
		'qsl.bitmap.marker': z.string(),
	})
	.partial();

/** Content collections: Starlight pages and the UI strings of each language. */
export const collections = {
	docs: defineCollection({ loader: docsLoader(), schema: docsSchema({ extend: z.object({ qslResources: z.array(z.object({ label: z.string(), url: z.url() })).default([]) }) }) }),
	i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema({ extend: componentStrings }) }),
};
