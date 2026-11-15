/**
 * Encoder for the demo schema shown by `PackingStrip.astro`:
 *
 * ```qsl
 * &Sample[safety] { u8(0..7) kind; u8(0..31) channel; u16(0..1023) value; }
 * ```
 *
 * It reproduces the three representations the QSL generator emits for this
 * schema. It is a model of this one schema, not a general QSL encoder.
 * `wire.test.ts` pins the outputs to frames produced by the Rust generator.
 */

/** Which representation to produce. */
export type WireMode = 'default' | 'compact' | 'aligned';

/** CSS color slot of a field, see `--qsl-f1` ... `--qsl-f4` in theme.css. */
export type FieldColor = 'f1' | 'f2' | 'f3' | 'f4';

/** One field of the demo schema. */
export interface SampleField {
	readonly name: string;
	readonly type: 'u8' | 'u16';
	readonly min: number;
	readonly max: number;
	/** Value shown before the reader changes anything. */
	readonly initial: number;
	/** Width of the Rust primitive, used by the default encoding. */
	readonly nativeBits: number;
	readonly color: FieldColor;
}

/** One bit of an encoded frame. */
export interface WireBit {
	/** Index into `SAMPLE_FIELDS`, or `null` for padding. */
	readonly field: number | null;
	readonly value: 0 | 1;
}

/** Fields of `Sample` in declaration order, which is also the order on the wire. */
export const SAMPLE_FIELDS: readonly SampleField[] = [
	{ name: 'kind', type: 'u8', min: 0, max: 7, initial: 5, nativeBits: 8, color: 'f1' },
	{ name: 'channel', type: 'u8', min: 0, max: 31, initial: 17, nativeBits: 8, color: 'f2' },
	{ name: 'value', type: 'u16', min: 0, max: 1023, initial: 513, nativeBits: 16, color: 'f3' },
];

/** Schema attribute list for each mode, as written in the `.qsl` source. */
export const MODE_ATTRIBUTES: Readonly<Record<WireMode, string>> = {
	default: 'safety',
	compact: 'safety,compact',
	aligned: 'safety,compact,alignment',
};

/** Number of bits needed to represent `n` (0 for 0), i.e. Rust's `u64::BITS - n.leading_zeros()`. */
export function bitLength(n: number): number {
	if (!Number.isInteger(n) || n < 0)
		throw new RangeError(`bitLength expects a non-negative integer, got ${n}`);
	return n === 0 ? 0 : Math.floor(Math.log2(n)) + 1;
}

/** Bits each field occupies on the wire in the given mode, before padding. */
export function fieldWidths(mode: WireMode): number[] {
	return SAMPLE_FIELDS.map((field) =>
		mode === 'default' ? field.nativeBits : bitLength(field.max - field.min),
	);
}

/**
 * Encode one `Sample` value.
 *
 * - `default`: each field little-endian in its native width.
 * - `compact`: each field stores `x - min` in `bit_length(max - min)` bits,
 *   packed least significant bit first across byte boundaries.
 * - `aligned`: like `compact`, but every field is padded to a byte boundary.
 *
 * @param values One value per entry of `SAMPLE_FIELDS`, in order.
 * @returns Bits in write order; the length is always a multiple of 8.
 * @throws RangeError if a value is missing, not an integer or out of range.
 */
export function encodeSample(values: readonly number[], mode: WireMode): WireBit[] {
	if (values.length !== SAMPLE_FIELDS.length) {
		throw new RangeError(`expected ${SAMPLE_FIELDS.length} values, got ${values.length}`);
	}
	const widths = fieldWidths(mode);
	const bits: WireBit[] = [];
	const padToByte = () => {
		while (bits.length % 8 !== 0) bits.push({ field: null, value: 0 });
	};

	SAMPLE_FIELDS.forEach((field, index) => {
		const value = values[index];
		if (!Number.isInteger(value) || value < field.min || value > field.max) {
			throw new RangeError(`${field.name} must be an integer in ${field.min}..${field.max}, got ${value}`);
		}
		const stored = mode === 'default' ? value : value - field.min;
		for (let bit = 0; bit < widths[index]; bit++) {
			bits.push({ field: index, value: ((stored >> bit) & 1) as 0 | 1 });
		}
		if (mode === 'aligned') padToByte();
	});

	padToByte();
	return bits;
}

/** Group bits (least significant first) into byte values. */
export function bitsToBytes(bits: readonly WireBit[]): number[] {
	const bytes: number[] = [];
	for (let offset = 0; offset < bits.length; offset += 8) {
		bytes.push(bits.slice(offset, offset + 8).reduce((byte, bit, i) => byte | (bit.value << i), 0));
	}
	return bytes;
}
