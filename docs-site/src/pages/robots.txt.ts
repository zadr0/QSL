import type { APIRoute } from 'astro';

import { BASE_PATH, SITE_URL } from '../../config/site.mjs';

export const GET: APIRoute = () =>
	new Response(
		`User-agent: *\nAllow: /\n\nSitemap: ${new URL(`${BASE_PATH}/sitemap-index.xml`, SITE_URL).href}\n`,
		{ headers: { 'Content-Type': 'text/plain; charset=utf-8' } },
	);
