import { describe, expect, it } from 'vitest';

import { hexByte, hexBytes } from './hex';

describe('hexByte', () => {
	it.each([
		[0, '00'],
		[0x09, '09'],
		[0x8d, '8d'],
		[0xff, 'ff'],
	])('formats %i as %s', (value, expected) => {
		expect(hexByte(value)).toBe(expected);
	});

	it.each([-1, 256, 1.5, Number.NaN])('rejects %s', (value) => {
		expect(() => hexByte(value)).toThrow(RangeError);
	});
});

describe('hexBytes', () => {
	it('joins bytes with spaces', () => {
		expect(hexBytes([0x8d, 0x01, 0x02])).toBe('8d 01 02');
		expect(hexBytes([])).toBe('');
	});
});
