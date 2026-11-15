/**
 * DOM helpers for the client scripts of the components.
 */

/**
 * Return the first element under `root` that matches `selector`.
 *
 * Components only query markup they rendered themselves, so a miss means the
 * template and the script are out of sync. Failing loudly makes that visible
 * in development instead of silently skipping an update.
 */
export function requireElement<T extends Element = HTMLElement>(root: ParentNode, selector: string): T {
	const element = root.querySelector<T>(selector);
	if (!element) throw new Error(`Element not found: ${selector}`);
	return element;
}
