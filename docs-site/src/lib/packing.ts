/**
 * Slot model behind the animated frame on the home page (`PackingStrip.astro`).
 *
 * The animation shows one `Sample` value moving between encodings. Instead of
 * re-rendering the bits, every bit that appears in any encoding owns a fixed
 * slot. An encoding only decides, for each slot, whether it carries a data
 * bit, a padding bit or nothing, and at which wire position it sits. The
 * component renders the slots once and lets CSS transitions move them.
 *
 * Slots, in order:
 * - for each field of `SAMPLE_FIELDS`, `nativeBits` slots, bit 0 (least
 *   significant) first;
 * - `TRAILING_PAD_SLOTS` slots for the padding that completes the last byte
 *   of the compact encoding.
 */
import { bitsToBytes, encodeSample, fieldWidths, SAMPLE_FIELDS, type WireMode } from './wire';

/** What a slot shows in a given encoding. */
export type SlotRole = 'data' | 'pad' | 'hidden';

/** Identity of a slot; constant across encodings. */
export interface Slot {
	/** Index into `SAMPLE_FIELDS`, or `null` for a trailing padding slot. */
	readonly field: number | null;
	/** Bit index within the field, or index among the trailing padding slots. */
	readonly bit: number;
}

/** A slot's appearance in one encoding. */
export interface SlotState {
	readonly role: SlotRole;
	/**
	 * Wire position in write order. Hidden slots get a fractional position at
	 * the seam where they collapse, so they shrink in place instead of jumping.
	 */
	readonly position: number;
	readonly value: 0 | 1;
}

/** One encoding of the sample, ready to render. */
export interface PackingLayout {
	/** One state per entry of `SLOTS`, in the same order. */
	readonly slots: readonly SlotState[];
	/** Number of wire positions in use; always a multiple of 8. */
	readonly bitCount: number;
}

/** Padding slots needed after the last field of the compact encoding. */
export const TRAILING_PAD_SLOTS = (8 - (fieldWidths('compact').reduce((a, b) => a + b, 0) % 8)) % 8;

/** Every slot, in render order. */
export const SLOTS: readonly Slot[] = [
	...SAMPLE_FIELDS.flatMap((field, index) =>
		Array.from({ length: field.nativeBits }, (_, bit): Slot => ({ field: index, bit })),
	),
	...Array.from({ length: TRAILING_PAD_SLOTS }, (_, bit): Slot => ({ field: null, bit })),
];

/** Slot index of bit `bit` of field `field`, or of trailing padding slot `bit`. */
function slotIndex(field: number | null, bit: number): number {
	if (field === null) return SLOTS.length - TRAILING_PAD_SLOTS + bit;
	let offset = 0;
	for (let index = 0; index < field; index++) offset += SAMPLE_FIELDS[index].nativeBits;
	return offset + bit;
}

/**
 * Lay out the sample's slots for one encoding.
 *
 * Data bits come from `encodeSample`, so the visible slots, read in position
 * order, are exactly the encoded frame. Padding that follows a field (the
 * `aligned` encoding) uses that field's spare high slots; padding at the end
 * of the frame uses the trailing slots.
 *
 * @param values One value per entry of `SAMPLE_FIELDS`.
 * @throws RangeError for the same inputs as `encodeSample`.
 */
export function layoutSlots(values: readonly number[], mode: WireMode): PackingLayout {
	const bits = encodeSample(values, mode);
	const states: Array<SlotState | undefined> = Array.from({ length: SLOTS.length }, () => undefined);
	const nextBit = SAMPLE_FIELDS.map(() => 0);
	let lastField: number | null = null;
	let trailing = 0;

	bits.forEach((wireBit, position) => {
		if (wireBit.field !== null) {
			lastField = wireBit.field;
			states[slotIndex(wireBit.field, nextBit[wireBit.field]++)] = {
				role: 'data',
				position,
				value: wireBit.value,
			};
			return;
		}
		const ownerHasRoom = lastField !== null && nextBit[lastField] < SAMPLE_FIELDS[lastField].nativeBits;
		const isFinalPadding = bits.slice(position).every((bit) => bit.field === null);
		const index =
			ownerHasRoom && !(mode === 'compact' && isFinalPadding)
				? slotIndex(lastField, nextBit[lastField as number]++)
				: slotIndex(null, trailing++);
		states[index] = { role: 'pad', position, value: 0 };
	});

	// Hidden slots collapse at the seam right after the last visible slot before them.
	let seam = -0.5;
	const slots = states.map((state): SlotState => {
		if (state) {
			seam = state.position + 0.5;
			return state;
		}
		return { role: 'hidden', position: seam, value: 0 };
	});
	return { slots, bitCount: bits.length };
}

/** Wire positions covered by one field's data bits. */
export interface FieldSpan {
	/** Position of the field's first bit. */
	readonly from: number;
	/** Number of data bits. */
	readonly span: number;
}

/** Everything the strip shows for one encoding. */
export interface FrameView extends PackingLayout {
	/** The encoded frame. */
	readonly bytes: readonly number[];
	/** One entry per field of `SAMPLE_FIELDS`. */
	readonly fields: readonly FieldSpan[];
}

/**
 * Describe one encoding of the sample for rendering: slot layout, frame bytes
 * and the span of each field.
 *
 * @throws RangeError for the same inputs as `encodeSample`.
 */
export function frameView(values: readonly number[], mode: WireMode): FrameView {
	const layout = layoutSlots(values, mode);
	const fields = SAMPLE_FIELDS.map((_, field): FieldSpan => {
		const positions = layout.slots
			.filter((state, index) => SLOTS[index].field === field && state.role === 'data')
			.map((state) => state.position);
		return { from: Math.min(...positions), span: positions.length };
	});
	return { ...layout, bytes: bitsToBytes(encodeSample(values, mode)), fields };
}
