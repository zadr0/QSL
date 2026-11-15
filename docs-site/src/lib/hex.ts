/**
 * Hex formatting for the wire diagrams: lowercase, two digits per byte, the
 * same as Rust's `{:02x}` and the frames quoted in the docs.
 */

/** Format one byte, e.g. `0x8d` as `"8d"`. Throws `RangeError` for non-bytes. */
export function hexByte(value: number): string {
	if (!Number.isInteger(value) || value < 0 || value > 0xff) {
		throw new RangeError(`Not a byte: ${value}`);
	}
	return value.toString(16).padStart(2, '0');
}

/** Format a byte sequence separated by spaces, e.g. `"8d 01 02"`. */
export function hexBytes(bytes: Iterable<number>): string {
	return Array.from(bytes, hexByte).join(' ');
}
