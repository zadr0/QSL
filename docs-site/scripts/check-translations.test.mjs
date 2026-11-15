import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { compareShapes, pageShape, splitFrontmatter, unresolvedImports } from './check-translations.mjs';

const english = `---
title: Memory
description: Reuse buffers.
---

import ByteBars from '../../components/ByteBars.astro';

## Reading into existing memory {#reading-into-existing-memory}

See [methods](/reference/methods/#reuse) and <a href="/guides/safety/">safety</a>.

<LandingSection title="Usage" id="usage">

\`\`\`rust
// ## not a heading
let x = 1;
\`\`\`

</LandingSection>

### RT slot pool {#rt-slot-pool}
`;

describe('splitFrontmatter', () => {
	it('reads top-level keys and returns the body', () => {
		const { frontmatter, body } = splitFrontmatter(english);
		expect(frontmatter).toEqual({ title: 'Memory', description: 'Reuse buffers.' });
		expect(body.startsWith('\nimport ByteBars')).toBe(true);
	});

	it('treats a page without frontmatter as all body', () => {
		expect(splitFrontmatter('# Title\n')).toEqual({ frontmatter: {}, body: '# Title\n' });
	});
});

describe('pageShape', () => {
	it('collects headings, anchors, code, components and links, skipping code contents', () => {
		const shape = pageShape(english);
		expect(shape.headings).toEqual(['h2', 'h3']);
		expect(shape.anchors).toEqual(['reading-into-existing-memory', 'usage', 'rt-slot-pool']);
		expect(shape.codeBlocks).toEqual(['rust']);
		expect(shape.components).toEqual(['LandingSection']);
		expect(shape.links).toEqual(['/guides/safety/', '/reference/methods/#reuse']);
	});

	it('ignores components and links inside inline code', () => {
		const shape = pageShape('A `CachedFrame<N>` and `[x](y)` are text.\n');
		expect(shape.components).toEqual([]);
		expect(shape.links).toEqual([]);
	});
});

describe('compareShapes', () => {
	it('accepts a translation that only changes the text', () => {
		const translated = english
			.replace('title: Memory', 'title: Память')
			.replace('## Reading into existing memory', '## Чтение в существующую память')
			.replace('title="Usage"', 'title="Использование"');
		expect(compareShapes(pageShape(english), pageShape(translated))).toEqual([]);
	});

	it('reports a changed anchor, a translated link and a dropped code block', () => {
		const broken = english
			.replace('{#rt-slot-pool}', '{#pul-slotov}')
			.replace('/guides/safety/', '/ru/guides/safety/')
			.replace(/```rust[\s\S]*?```\n/, '');
		const problems = compareShapes(pageShape(english), pageShape(broken));
		expect(problems).toHaveLength(3);
		expect(problems.join('\n')).toMatch(/anchors differ[\s\S]*links differ/);
	});

	it('requires a title and a description', () => {
		const shape = pageShape(english.replace('description: Reuse buffers.\n', ''));
		expect(compareShapes(pageShape(english), shape)).toContain('missing frontmatter "description"');
	});
});

describe('unresolvedImports', () => {
	const page = fileURLToPath(new URL('../src/content/docs/ru/internals/architecture.mdx', import.meta.url));

	it('accepts an import one folder deeper than English', () => {
		const source = "import CrateMap from '../../../../components/CrateMap.astro';\n";
		expect(unresolvedImports(source, page)).toEqual([]);
	});

	it('reports an import path copied unchanged from English', () => {
		const source = "import CrateMap from '../../../components/CrateMap.astro';\n";
		expect(unresolvedImports(source, page)).toEqual(['../../../components/CrateMap.astro']);
	});

	it('ignores package imports', () => {
		const source = "import { Aside } from '@astrojs/starlight/components';\n";
		expect(unresolvedImports(source, page)).toEqual([]);
	});
});
