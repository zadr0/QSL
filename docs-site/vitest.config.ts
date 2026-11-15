import { defineConfig } from 'vitest/config';

export default defineConfig({
	test: {
		include: ['src/**/*.test.{ts,mjs}', 'scripts/**/*.test.mjs'],
		environment: 'node',
	},
});
