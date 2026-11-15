/**
 * The QSL mark: the name in a 5×7 bit font, one field color per letter. The
 * footer draws it flat; the home page's machine carries it on its lid.
 */
import type { FieldColor } from './wire';

/** "QSL", one string per row; letters are separated by a blank column. */
export const MARK_GLYPHS = [
	'01110 01111 10000',
	'10001 10000 10000',
	'10001 10000 10000',
	'10001 01110 10000',
	'10101 00001 10000',
	'10010 00001 10000',
	'01101 11110 11111',
] as const;

/** Width of the mark in cells, gaps included. */
export const MARK_WIDTH = MARK_GLYPHS[0].length;

/** Share of a cell that its tile covers; the rest is the gap between tiles. */
export const MARK_TILE = 0.78;

export interface MarkCell {
	/** Column, counted from the left edge, gaps included. */
	readonly x: number;
	/** Row, counted from the top. */
	readonly y: number;
	/** Whether the bit is set. */
	readonly on: boolean;
	readonly color: FieldColor;
}

const LETTER_COLORS: readonly FieldColor[] = ['f1', 'f2', 'f3'];
/** A letter is five columns wide, plus the blank column after it. */
const LETTER_PITCH = 6;

/** Every cell of the mark, row by row; the blank columns between letters are left out. */
export const MARK_CELLS: readonly MarkCell[] = MARK_GLYPHS.flatMap((row, y) =>
	[...row].flatMap((char, x) =>
		char === ' ' ? [] : [{ x, y, on: char === '1', color: LETTER_COLORS[Math.floor(x / LETTER_PITCH)] }],
	),
);
