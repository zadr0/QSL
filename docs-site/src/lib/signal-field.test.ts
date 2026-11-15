import { describe, expect, it } from 'vitest';

import { MAX_QUIET_ZONES, parseHexColor, quietZoneUniform, renderScale } from './signal-field';

describe('parseHexColor', () => {
	it('reads six- and three-digit hex colors', () => {
		expect(parseHexColor('#ff0080')).toEqual([1, 0, 128 / 255]);
		expect(parseHexColor(' #0f0 ')).toEqual([0, 1, 0]);
	});

	it.each(['', 'red', '#12345', 'rgb(0 0 0)'])('rejects %j', (value) => {
		expect(() => parseHexColor(value)).toThrow(TypeError);
	});
});

describe('renderScale', () => {
	it.each([
		[0.5, 1],
		[1, 1],
		[1.25, 1.25],
		[2, 1.5],
		[3, 1.5],
	])('maps a device pixel ratio of %f to %f', (ratio, expected) => {
		expect(renderScale(ratio)).toBe(expected);
	});
});

describe('quietZoneUniform', () => {
	it('scales each zone to device pixels and parks unused slots off-canvas', () => {
		const data = quietZoneUniform([{ left: 10, top: 20, right: 110, bottom: 60 }], 1.5);
		expect(data).toHaveLength(MAX_QUIET_ZONES * 4);
		expect([...data.slice(0, 4)]).toEqual([15, 30, 165, 90]);
		expect(data.slice(4).every((value) => value <= -1e6)).toBe(true);
	});

	it('ignores zones beyond the maximum', () => {
		const zone = { left: 1, top: 1, right: 2, bottom: 2 };
		const data = quietZoneUniform(
			Array.from({ length: MAX_QUIET_ZONES + 2 }, () => zone),
			1,
		);
		expect(data).toHaveLength(MAX_QUIET_ZONES * 4);
		expect(data.every((value) => value > 0)).toBe(true);
	});
});
