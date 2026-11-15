# QSL documentation discoverability

## Implemented

The current documentation has unique descriptive titles and descriptions, self-canonical URLs, reciprocal RU/EN hreflang, a server-rendered English home at en.html, SoftwareSourceCode/WebPage JSON-LD, Open Graph/Twitter cards and a 1200×630 preview. The sitemap contains current canonical pages only. Historical copied reports and the offline duplicate use noindex; preserved archives are not put in the sitemap. No rating, speedup, review or ranking is fabricated.

Website links to repository source resolve to tagged GitHub files rather than non-existent paths outside the Pages documentation root. Historical reports are served as explicit website copies; their original artifacts remain unchanged. Rebuild with `python reference/build-site.py .`, then run `python reference/verify-site.py` and `python reference/verify-seo.py`.

## GitHub Pages and robots.txt

The project is under https://zadr0.github.io/QSL/. Crawlers consult https://zadr0.github.io/robots.txt, not /QSL/robots.txt. The host-root file currently returns 404; no disallow rule was found. A misleading project-local replacement is deliberately not used. `reference/robots-root.example.txt` is an example for the host owner or a future custom domain; it is not a deployed host-root policy. [Google robots location rules](https://developers.google.com/crawling/docs/robots-txt/create-robots-txt).

Sitemap: https://zadr0.github.io/QSL/sitemap.xml. It is linked from HTML. Search Console submission requires an owned, verified property and is not claimed performed without that account access. IndexNow uses a project-scoped keyLocation and is submitted only after the key and URLs are publicly deployed. An accepted request means receipt, not indexing or a ranking guarantee. [IndexNow protocol](https://www.indexnow.org/documentation).

## Discovery boundaries

GitHub description/topics, a versioned Release and crates.io homepage/category metadata connect the same project identity. Search engines choose whether and when to crawl, index and rank it. Technical SEO does not prove demand or top search positions. [Sitemaps](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview), [localized pages](https://developers.google.com/search/docs/specialty/international/localized-versions), [SoftwareSourceCode vocabulary](https://schema.org/SoftwareSourceCode).
