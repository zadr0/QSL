// @ts-check
import js from '@eslint/js';
import { defineConfig, globalIgnores } from 'eslint/config';
import astro from 'eslint-plugin-astro';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default defineConfig(
	globalIgnores(['dist/', '.astro/', 'node_modules/']),
	{
		linterOptions: { reportUnusedDisableDirectives: 'error' },
	},
	js.configs.recommended,
	tseslint.configs.strict,
	// `.astro` files get their globals from this preset: Node for the
	// frontmatter, browser for `<script>` blocks.
	astro.configs['flat/recommended'],
	astro.configs['flat/jsx-a11y-strict'],
	{
		// Config files, scripts, the Markdown plugin and tests run in Node.
		// Client code is TypeScript, where the type checker reports unknown globals.
		files: ['**/*.{js,mjs,cjs}'],
		ignores: ['**/*.astro/*'],
		languageOptions: {
			globals: globals.node,
		},
	},
);
