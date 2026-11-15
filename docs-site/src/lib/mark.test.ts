import { describe, expect, it } from 'vitest';

import { MARK_CELLS, MARK_GLYPHS, MARK_WIDTH } from './mark';

describe('QSL mark', () => {
	it('has rows of equal width', () => {
		expect(MARK_GLYPHS.every((row) => row.length === MARK_WIDTH)).toBe(true);
	});

	it('leaves out the blank columns between letters', () => {
		expect(MARK_CELLS).toHaveLength(MARK_GLYPHS.length * 15);
		expect(MARK_CELLS.some(({ x }) => x === 5 || x === 11)).toBe(false);
	});

	it('gives each letter its own field color', () => {
		const colorAt = (x: number) => MARK_CELLS.find((cell) => cell.x === x)?.color;
		expect([colorAt(0), colorAt(6), colorAt(12)]).toEqual(['f1', 'f2', 'f3']);
	});
});
