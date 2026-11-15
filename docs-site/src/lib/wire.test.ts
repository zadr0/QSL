import { describe, expect, it } from 'vitest';

import { hexBytes } from './hex';
import { bitLength, bitsToBytes, encodeSample, fieldWidths, SAMPLE_FIELDS } from './wire';

const initial = SAMPLE_FIELDS.map((field) => field.initial);

describe('bitLength', () => {
	it.each([
		[0, 0],
		[1, 1],
		[7, 3],
		[8, 4],
		[31, 5],
		[1023, 10],
	])('bitLength(%i) = %i', (input, expected) => {
		expect(bitLength(input)).toBe(expected);
	});

	it('rejects negative and fractional input', () => {
		expect(() => bitLength(-1)).toThrow(RangeError);
		expect(() => bitLength(1.5)).toThrow(RangeError);
	});
});

describe('encodeSample', () => {
	// kind=5, channel=17, value=513: frames cross-checked with the Rust generator.
	it('encodes the default representation little-endian in native widths', () => {
		expect(hexBytes(bitsToBytes(encodeSample(initial, 'default')))).toBe('05 11 01 02');
	});

	it('packs compact fields least significant bit first', () => {
		expect(hexBytes(bitsToBytes(encodeSample(initial, 'compact')))).toBe('8d 01 02');
	});

	it('pads every compact field to a byte boundary with alignment', () => {
		expect(hexBytes(bitsToBytes(encodeSample(initial, 'aligned')))).toBe('05 11 01 02');
	});

	it('uses native widths by default and range widths when compact', () => {
		expect(fieldWidths('default')).toEqual([8, 8, 16]);
		expect(fieldWidths('aligned')).toEqual([3, 5, 10]);
	});

	it('uses 3 + 5 + 10 bits in compact mode', () => {
		expect(fieldWidths('compact')).toEqual([3, 5, 10]);
		const bits = encodeSample(initial, 'compact');
		expect(bits).toHaveLength(24);
		expect(bits.slice(18).every((bit) => bit.field === null && bit.value === 0)).toBe(true);
	});

	it('encodes the maximum values', () => {
		expect(hexBytes(bitsToBytes(encodeSample([7, 31, 1023], 'compact')))).toBe('ff ff 03');
	});

	it('marks alignment padding after every field', () => {
		const fields = encodeSample(initial, 'aligned').map((bit) => bit.field);
		expect(fields).toEqual([
			...[0, 0, 0, null, null, null, null, null],
			...[1, 1, 1, 1, 1, null, null, null],
			...Array<number>(10).fill(2),
			...Array<null>(6).fill(null),
		]);
	});

	it('rejects values outside the declared ranges', () => {
		expect(() => encodeSample([8, 0, 0], 'compact')).toThrow(RangeError);
		expect(() => encodeSample([0, 0], 'compact')).toThrow(RangeError);
	});
});
