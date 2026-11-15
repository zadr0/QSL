/**
 * Encoder for the optional-field bitmap demo shown by `BitmapDemo.astro`:
 *
 * ```qsl
 * &P[safety,optionals=bitmap,opt_table] { u8 id; u16? a; u8? b; }
 * ```
 *
 * Bitmap v2 rules for this schema:
 * - one mask byte covers the two optional fields;
 * - marker 1 (`a`) is the most significant bit, marker 2 (`b`) the next one;
 * - the mask comes first, then required fields, then present optional values
 *   in descending marker order (`b` before `a`);
 * - integers are little-endian.
 *
 * With every field present the frame is `c0 09 07 34 12`, as documented in
 * the QSL repository. `bitmap.test.ts` covers all four presence states.
 */

/** Which optional fields are present. */
export interface Presence {
	readonly a: boolean;
	readonly b: boolean;
}

/** What a frame byte belongs to. */
export type ByteRole = 'mask' | 'id' | 'a' | 'b';

/** One byte of an encoded frame and the part of the message it belongs to. */
export interface FrameByte {
	readonly value: number;
	readonly role: ByteRole;
}

/** Field values used by the demo. */
export const BITMAP_VALUES = { id: 0x09, a: 0x1234, b: 0x07 } as const;

/** Mask bit of each optional field. */
const MASK_BIT = { a: 0x80, b: 0x40 } as const;

/** Optional fields in the order a reader visits them. */
export function readOrder(presence: Presence): Array<'a' | 'b'> {
	const order: Array<'a' | 'b'> = [];
	if (presence.b) order.push('b');
	if (presence.a) order.push('a');
	return order;
}

/** Encode one frame of `P` for the given presence state. */
export function encodeBitmapFrame(presence: Presence): FrameByte[] {
	const mask = (presence.a ? MASK_BIT.a : 0) | (presence.b ? MASK_BIT.b : 0);
	const frame: FrameByte[] = [
		{ value: mask, role: 'mask' },
		{ value: BITMAP_VALUES.id, role: 'id' },
	];
	for (const field of readOrder(presence)) {
		if (field === 'b') {
			frame.push({ value: BITMAP_VALUES.b, role: 'b' });
		} else {
			frame.push({ value: BITMAP_VALUES.a & 0xff, role: 'a' }, { value: BITMAP_VALUES.a >> 8, role: 'a' });
		}
	}
	return frame;
}

/** Marker number of each optional field, as written in the schema. */
export const MARKERS = { a: 1, b: 2 } as const;

/** One byte position the demo can show; see `frameSlots`. */
export interface FrameSlot {
	readonly role: ByteRole;
	/** Byte value when present, 0 otherwise. */
	readonly value: number;
	readonly present: boolean;
}

/**
 * Every byte any presence state can produce, in frame order, with the ones
 * the given state leaves out marked as absent. The demo renders these slots
 * once and only toggles them, so bytes slide in and out instead of jumping.
 *
 * The present slots, in order, are exactly `encodeBitmapFrame(presence)`.
 */
export function frameSlots(presence: Presence): FrameSlot[] {
	const full = encodeBitmapFrame({ a: true, b: true });
	const frame = encodeBitmapFrame(presence);
	let next = 0;
	return full.map(({ role }) => {
		const byte = frame[next];
		if (byte?.role !== role) return { role, value: 0, present: false };
		next += 1;
		return { role, value: byte.value, present: true };
	});
}
