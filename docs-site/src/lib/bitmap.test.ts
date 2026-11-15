import { describe, expect, it } from 'vitest';

import { encodeBitmapFrame, frameSlots, readOrder, type Presence } from './bitmap';
import { hexBytes } from './hex';

const hex = (presence: Presence) => hexBytes(encodeBitmapFrame(presence).map(({ value }) => value));

describe('encodeBitmapFrame', () => {
	it('matches the documented frame with both optionals present', () => {
		expect(hex({ a: true, b: true })).toBe('c0 09 07 34 12');
	});

	it.each([
		[{ a: false, b: false }, '00 09'],
		[{ a: true, b: false }, '80 09 34 12'],
		[{ a: false, b: true }, '40 09 07'],
	])('encodes presence %o as %s', (presence, expected) => {
		expect(hex(presence)).toBe(expected);
	});
});

describe('readOrder', () => {
	it.each([
		[{ a: true, b: true }, ['b', 'a']],
		[{ a: true, b: false }, ['a']],
		[{ a: false, b: true }, ['b']],
		[{ a: false, b: false }, []],
	])('visits the present fields of %o in descending marker order', (presence, expected) => {
		expect(readOrder(presence)).toEqual(expected);
	});
});

describe('frameSlots', () => {
	const states: Presence[] = [
		{ a: true, b: true },
		{ a: true, b: false },
		{ a: false, b: true },
		{ a: false, b: false },
	];

	it('always lists the five bytes of the full frame', () => {
		for (const presence of states) {
			expect(frameSlots(presence).map(({ role }) => role)).toEqual(['mask', 'id', 'b', 'a', 'a']);
		}
	});

	it.each(states)('keeps exactly the encoded frame present for %o', (presence) => {
		const present = frameSlots(presence).filter((slot) => slot.present);
		expect(present.map(({ role, value }) => ({ role, value }))).toEqual(encodeBitmapFrame(presence));
	});
});
