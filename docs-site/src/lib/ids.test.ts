import { describe, expect, it } from 'vitest';

import { uniqueId } from './ids';

describe('uniqueId', () => {
	it('returns a new ID with the given prefix on every call', () => {
		const ids = [uniqueId('qsl-wire'), uniqueId('qsl-wire'), uniqueId('qsl-bitmap')];
		expect(new Set(ids).size).toBe(ids.length);
		expect(ids[0]).toMatch(/^qsl-wire-\d+$/);
		expect(ids[2]).toMatch(/^qsl-bitmap-\d+$/);
	});
});
