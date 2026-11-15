# Local QSL documentation

Run from the docs-site directory:

~~~text
npm run native:build
npm run build
npm start
~~~

Open http://127.0.0.1:4322/ru/ (English: /en/). The local server listens on loopback only and serves the built site; rebuild after content changes. Native examples invoke the local generated Rust code and analyzer. They are demonstrations, not browser-timing benchmarks. All calculators are in the final Visualizers / Tests navigation group. Archived Criterion measurements are separate.

Nine locales: English, Russian, Simplified Chinese, Japanese, Korean, Spanish, Brazilian Portuguese, German and French.

~~~text
npm test
npm run check
npm run build
npm run test:links
node scripts/check-local-contract.mjs
~~~

The original structural translation comparator is stricter than route coverage: independently edited localized chapter shapes can differ. The local contract check verifies that every route exists in every locale and checks formulas, component separation and native examples.
