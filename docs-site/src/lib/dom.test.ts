import { describe, expect, it } from 'vitest';

import { requireElement } from './dom';

/** Minimal stand-in for a DOM node; the tests run without a browser. */
function rootWith(elements: Record<string, Element>): ParentNode {
	return { querySelector: (selector: string) => elements[selector] ?? null } as unknown as ParentNode;
}

describe('requireElement', () => {
	it('returns the matching element', () => {
		const element = {} as Element;
		expect(requireElement(rootWith({ '[data-size]': element }), '[data-size]')).toBe(element);
	});

	it('throws with the selector when nothing matches', () => {
		expect(() => requireElement(rootWith({}), '[data-size]')).toThrow('Element not found: [data-size]');
	});
});
