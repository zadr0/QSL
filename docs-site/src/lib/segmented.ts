/**
 * Positions the sliding highlight of a `.qsl-segmented` control (styles in
 * `src/styles/demo.css`) under its checked option.
 *
 * The first call places the highlight without animation and marks the
 * control `[data-ready]`, which turns on the slide for later calls.
 */
export function placeSegmentedThumb(control: HTMLElement): void {
	const label = control.querySelector<HTMLElement>('input:checked + label');
	const thumb = control.querySelector<HTMLElement>('.qsl-segmented__thumb');
	if (!label || !thumb) return;
	// Options may wrap onto a second row on narrow screens, so track both axes.
	thumb.style.setProperty('--x', `${label.offsetLeft}px`);
	thumb.style.setProperty('--y', `${label.offsetTop}px`);
	thumb.style.setProperty('--w', `${label.offsetWidth}px`);
	thumb.style.setProperty('--h', `${label.offsetHeight}px`);
	if (control.dataset.ready === undefined) {
		// Commit the first position before transitions are enabled.
		void thumb.offsetWidth;
		control.dataset.ready = '';
	}
}
