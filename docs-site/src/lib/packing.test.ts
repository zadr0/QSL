import { describe, expect, it } from 'vitest';

import { frameView, layoutSlots, SLOTS, TRAILING_PAD_SLOTS, type PackingLayout } from './packing';
import { encodeSample, SAMPLE_FIELDS, type WireMode } from './wire';

const initial = SAMPLE_FIELDS.map((field) => field.initial);
const modes: WireMode[] = ['default', 'compact', 'aligned'];

/** Visible slots read in wire order, as `{ field, value }` pairs. */
function wireOrder(layout: PackingLayout) {
	return layout.slots
		.map((state, index) => ({ state, slot: SLOTS[index] }))
		.filter(({ state }) => state.role !== 'hidden')
		.sort((a, b) => a.state.position - b.state.position)
		.map(({ state, slot }) => ({ field: state.role === 'pad' ? null : slot.field, value: state.value }));
}

describe('SLOTS', () => {
	it('has one slot per native bit plus the compact trailing padding', () => {
		expect(TRAILING_PAD_SLOTS).toBe(6);
		expect(SLOTS).toHaveLength(8 + 8 + 16 + 6);
	});
});

describe('layoutSlots', () => {
	it.each(modes)('shows exactly the encoded frame in %s mode', (mode) => {
		for (const values of [initial, [0, 0, 0], [7, 31, 1023], [2, 9, 300]]) {
			expect(wireOrder(layoutSlots(values, mode))).toEqual(encodeSample(values, mode));
		}
	});

	it.each(modes)('uses each wire position once in %s mode', (mode) => {
		const { slots, bitCount } = layoutSlots(initial, mode);
		const positions = slots.filter((s) => s.role !== 'hidden').map((s) => s.position);
		expect([...positions].sort((a, b) => a - b)).toEqual(Array.from({ length: bitCount }, (_, i) => i));
	});

	it('reports the frame size in bits', () => {
		expect(layoutSlots(initial, 'default').bitCount).toBe(32);
		expect(layoutSlots(initial, 'compact').bitCount).toBe(24);
		expect(layoutSlots(initial, 'aligned').bitCount).toBe(32);
	});

	it('hides the unused high bits in compact mode and adds trailing padding', () => {
		const roles = layoutSlots(initial, 'compact').slots.map((s) => s.role);
		expect(roles.filter((r) => r === 'data')).toHaveLength(3 + 5 + 10);
		expect(roles.filter((r) => r === 'pad')).toHaveLength(TRAILING_PAD_SLOTS);
		expect(roles.filter((r) => r === 'hidden')).toHaveLength(32 - 18);
	});

	it('pads each field from its own spare slots in aligned mode', () => {
		const { slots } = layoutSlots(initial, 'aligned');
		const kindSlots = slots.slice(0, 8).map((s) => s.role);
		expect(kindSlots).toEqual(['data', 'data', 'data', 'pad', 'pad', 'pad', 'pad', 'pad']);
		expect(slots.slice(-TRAILING_PAD_SLOTS).every((s) => s.role === 'hidden')).toBe(true);
	});

	it('collapses hidden slots at the seam after the last visible slot', () => {
		const { slots } = layoutSlots(initial, 'compact');
		// kind keeps bits 0..2 at positions 0..2; its hidden bits 3..7 collapse at 2.5.
		expect(slots.slice(3, 8).every((s) => s.position === 2.5)).toBe(true);
	});

	it('rejects out-of-range values', () => {
		expect(() => layoutSlots([8, 0, 0], 'compact')).toThrow(RangeError);
	});
});

describe('frameView', () => {
	it('returns the frame bytes and where each field sits', () => {
		expect(frameView(initial, 'compact')).toMatchObject({
			bytes: [0x8d, 0x01, 0x02],
			bitCount: 24,
			fields: [
				{ from: 0, span: 3 },
				{ from: 3, span: 5 },
				{ from: 8, span: 10 },
			],
		});
		expect(frameView(initial, 'aligned').fields).toEqual([
			{ from: 0, span: 3 },
			{ from: 8, span: 5 },
			{ from: 16, span: 10 },
		]);
		expect(frameView(initial, 'default')).toMatchObject({
			bytes: [0x05, 0x11, 0x01, 0x02],
			fields: [
				{ from: 0, span: 8 },
				{ from: 8, span: 8 },
				{ from: 16, span: 16 },
			],
		});
	});
});
