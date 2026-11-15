/**
 * Element IDs for components that can appear more than once on a page.
 *
 * The counter lives in module scope and is shared by every page of a build,
 * so an ID is never reused within a page. The numbers depend on render order:
 * do not link to these IDs or use them in CSS.
 */
let counter = 0;

/** Return a new ID such as `qsl-wire-3`. */
export function uniqueId(prefix: string): string {
	counter += 1;
	return `${prefix}-${counter}`;
}
