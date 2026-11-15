/**
 * The animated background of the home page hero: a grid of bit cells with
 * packets of bits gliding along each row, like frames moving down a wire.
 * Ones are filled squares, zeros are outlines. Near the pointer the cells
 * grow and the static grid lights up. Packets fade out over "quiet zones",
 * the boxes of the text drawn on top, so the field flows around the words
 * instead of through them.
 *
 * Everything is drawn by one fragment shader on a full-screen triangle; there
 * is no geometry per cell and no dependency beyond WebGL 1.
 */

/** RGB triple in 0..1. */
export type Rgb = readonly [number, number, number];

/** Colors and strength of the field, read from the page's CSS custom properties. */
export interface SignalPalette {
	readonly fields: readonly [Rgb, Rgb, Rgb, Rgb];
	readonly grid: Rgb;
	/** Overall opacity of the packets. */
	readonly strength: number;
}

/**
 * Parse a CSS hex color (`#rgb` or `#rrggbb`) into 0..1 channels.
 *
 * @throws TypeError for anything else.
 */
export function parseHexColor(value: string): Rgb {
	const hex = value.trim().replace(/^#/, '');
	const full = hex.length === 3 ? [...hex].map((c) => c + c).join('') : hex;
	if (!/^[0-9a-f]{6}$/i.test(full)) throw new TypeError(`Expected a hex color, got "${value}"`);
	const channel = (offset: number) => parseInt(full.slice(offset, offset + 2), 16) / 255;
	return [channel(0), channel(2), channel(4)];
}

/** A box in CSS pixels relative to the canvas, where packets fade out. */
export interface QuietZone {
	readonly left: number;
	readonly top: number;
	readonly right: number;
	readonly bottom: number;
}

/** Most quiet zones the shader takes; extra zones are ignored. */
export const MAX_QUIET_ZONES = 6;

/** Where unused zone slots go: far enough away that their feather never reaches the canvas. */
const UNUSED_ZONE = -1e6;

/**
 * Pack quiet zones into the shader's `uQuiet` array: four device-pixel
 * values per zone (left, top, right, bottom), unused slots parked off-canvas.
 */
export function quietZoneUniform(zones: readonly QuietZone[], scale: number): Float32Array {
	const data = new Float32Array(MAX_QUIET_ZONES * 4).fill(UNUSED_ZONE);
	zones.slice(0, MAX_QUIET_ZONES).forEach(({ left, top, right, bottom }, i) => {
		data.set([left * scale, top * scale, right * scale, bottom * scale], i * 4);
	});
	return data;
}

/** Device pixels per CSS pixel to render at: sharp enough, capped to keep the shader cheap. */
export function renderScale(devicePixelRatio: number): number {
	return Math.min(Math.max(devicePixelRatio, 1), 1.5);
}

export const VERTEX_SHADER = /* glsl */ `
attribute vec2 aPosition;
void main() {
	gl_Position = vec4(aPosition, 0.0, 1.0);
}
`;

export const FRAGMENT_SHADER = /* glsl */ `
precision highp float;

uniform vec2 uResolution;   // canvas size in device pixels
uniform float uTime;        // seconds
uniform vec2 uPointer;      // device pixels, origin top-left
uniform float uPointerMix;  // 0..1, eased presence of the pointer
uniform float uPitch;       // cell size in device pixels
uniform vec3 uField0;
uniform vec3 uField1;
uniform vec3 uField2;
uniform vec3 uField3;
uniform vec3 uGrid;
uniform float uStrength;
uniform vec4 uQuiet[${MAX_QUIET_ZONES}]; // device pixels: left, top, right, bottom

float hash11(float p) {
	p = fract(p * 0.1031);
	p *= p + 33.33;
	p *= p + p;
	return fract(p);
}

float hash21(vec2 p) {
	vec3 p3 = fract(vec3(p.xyx) * 0.1031);
	p3 += dot(p3, p3.yzx + 33.33);
	return fract((p3.x + p3.y) * p3.z);
}

// 1 inside a quiet zone, fading to 0 over three cells outside it.
float quietness(vec2 px) {
	float q = 0.0;
	for (int i = 0; i < ${MAX_QUIET_ZONES}; i++) {
		vec4 box = uQuiet[i];
		vec2 d = max(box.xy - px, px - box.zw);
		float outside = length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
		q = max(q, 1.0 - smoothstep(0.0, uPitch * 3.0, outside));
	}
	return q;
}

vec3 fieldColor(float index) {
	if (index < 1.0) return uField0;
	if (index < 2.0) return uField1;
	if (index < 3.0) return uField2;
	return uField3;
}

void main() {
	vec2 px = vec2(gl_FragCoord.x, uResolution.y - gl_FragCoord.y);
	vec2 uv = px / uResolution;
	float aa = 1.25 / uPitch;

	// Pointer influence, in cells.
	float dist = length(px - uPointer) / uPitch;
	float near = uPointerMix * exp(-dist * dist / 70.0);

	// Static grid: tiny dots that light up around the pointer.
	vec2 g = fract(px / uPitch) - 0.5;
	float gridDot = 1.0 - smoothstep(0.07 - aa, 0.07 + aa, max(abs(g.x), abs(g.y)));
	float gridA = gridDot * (0.16 + 0.75 * near);

	// Each row is a lane; packets glide along it at the lane's own speed.
	float row = floor(px.y / uPitch);
	float busy = step(0.3, hash11(row * 5.7 + 2.0));
	float speed = mix(1.6, 7.5, hash11(row * 3.1 + 7.0));
	float u = px.x / uPitch - uTime * speed - hash11(row + 1.0) * 500.0;
	float cell = floor(u);
	vec2 local = vec2(fract(u), fract(px.y / uPitch)) - 0.5;

	float slot = 28.0;
	float packet = floor(cell / slot);
	float index = cell - packet * slot;
	float packetLength = floor(mix(5.0, 20.0, hash21(vec2(packet, row))));
	float sent = step(hash21(vec2(packet * 1.7, row + 3.0)), 0.5) * step(index, packetLength - 1.0) * busy;
	float one = step(0.45, hash21(vec2(cell, row * 13.0)));

	float size = 0.3 + 0.1 * near;
	float d = max(abs(local.x), abs(local.y));
	float body = 1.0 - smoothstep(size - aa, size + aa, d);
	float ring = body * smoothstep(size - 0.11 - aa, size - 0.11 + aa, d);

	// Packets stay off the text and thin out towards the bottom edge.
	float quiet = quietness(px);
	float calm = (1.0 - quiet) * smoothstep(1.0, 0.45, uv.y);
	float packetA = sent * mix(ring * 0.5, body, one) * (0.55 + 0.45 * near) * uStrength * calm;
	gridA *= smoothstep(1.0, 0.5, uv.y) * (1.0 - 0.5 * quiet);

	vec3 color = fieldColor(floor(hash21(vec2(packet, row * 2.0)) * 4.0));
	float alpha = clamp(packetA + gridA * (1.0 - packetA), 0.0, 1.0);
	vec3 premultiplied = color * packetA + uGrid * gridA * (1.0 - packetA);
	gl_FragColor = vec4(premultiplied, alpha);
}
`;

/** Controls returned by `createSignalField`. */
export interface SignalField {
	setPalette(palette: SignalPalette): void;
	/** Boxes, in CSS pixels relative to the canvas, that packets avoid. */
	setQuietZones(zones: readonly QuietZone[]): void;
	/** Stop animating and release the WebGL context. */
	destroy(): void;
}

/** Options for `createSignalField`. */
export interface SignalFieldOptions {
	readonly palette: SignalPalette;
	/** Cell size in CSS pixels. */
	readonly pitch?: number;
	/** Draw one still frame instead of animating. */
	readonly still?: boolean;
}

function compile(gl: WebGLRenderingContext, type: number, source: string): WebGLShader {
	const shader = gl.createShader(type);
	if (!shader) throw new Error('Could not create a shader');
	gl.shaderSource(shader, source);
	gl.compileShader(shader);
	if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
		throw new Error(gl.getShaderInfoLog(shader) ?? 'Shader compilation failed');
	}
	return shader;
}

/**
 * Start rendering the signal field into `canvas`.
 *
 * The loop runs only while the canvas is on screen and the tab is visible.
 * Returns `null` when WebGL is unavailable, so the caller can keep a static
 * fallback.
 */
export function createSignalField(
	canvas: HTMLCanvasElement,
	options: SignalFieldOptions,
): SignalField | null {
	const gl = canvas.getContext('webgl', { antialias: false, premultipliedAlpha: true, alpha: true });
	if (!gl) return null;

	let program: WebGLProgram;
	try {
		const created = gl.createProgram();
		if (!created) return null;
		program = created;
		gl.attachShader(program, compile(gl, gl.VERTEX_SHADER, VERTEX_SHADER));
		gl.attachShader(program, compile(gl, gl.FRAGMENT_SHADER, FRAGMENT_SHADER));
		gl.linkProgram(program);
		if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return null;
	} catch {
		return null;
	}
	gl.useProgram(program);

	// One triangle that covers the viewport.
	gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
	gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
	const position = gl.getAttribLocation(program, 'aPosition');
	gl.enableVertexAttribArray(position);
	gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);

	const uniform = (name: string) => gl.getUniformLocation(program, name);
	const u = {
		resolution: uniform('uResolution'),
		time: uniform('uTime'),
		pointer: uniform('uPointer'),
		pointerMix: uniform('uPointerMix'),
		pitch: uniform('uPitch'),
		fields: [0, 1, 2, 3].map((i) => uniform(`uField${i}`)),
		grid: uniform('uGrid'),
		strength: uniform('uStrength'),
		quiet: uniform('uQuiet'),
	};

	const pitch = options.pitch ?? 14;
	let scale = renderScale(window.devicePixelRatio);
	let pointer = { x: -1e4, y: -1e4, target: 0, mix: 0 };
	let frame = 0;
	let visible = true;
	let destroyed = false;
	let quietZones: readonly QuietZone[] = [];
	const start = performance.now();

	const setPalette = (palette: SignalPalette) => {
		palette.fields.forEach((rgb, i) => gl.uniform3fv(u.fields[i], rgb));
		gl.uniform3fv(u.grid, palette.grid);
		gl.uniform1f(u.strength, palette.strength);
	};
	setPalette(options.palette);

	const resize = () => {
		scale = renderScale(window.devicePixelRatio);
		const width = Math.max(1, Math.round(canvas.clientWidth * scale));
		const height = Math.max(1, Math.round(canvas.clientHeight * scale));
		if (canvas.width !== width || canvas.height !== height) {
			canvas.width = width;
			canvas.height = height;
			gl.viewport(0, 0, width, height);
		}
		gl.uniform2f(u.resolution, width, height);
		gl.uniform1f(u.pitch, pitch * scale);
		gl.uniform4fv(u.quiet, quietZoneUniform(quietZones, scale));
	};

	const draw = (now: number) => {
		pointer.mix += (pointer.target - pointer.mix) * 0.08;
		gl.uniform1f(u.time, options.still ? 40 : (now - start) / 1000);
		gl.uniform2f(u.pointer, pointer.x * scale, pointer.y * scale);
		gl.uniform1f(u.pointerMix, pointer.mix);
		gl.drawArrays(gl.TRIANGLES, 0, 3);
	};

	const loop = (now: number) => {
		frame = 0;
		if (destroyed) return;
		draw(now);
		if (!options.still && visible && !document.hidden) frame = requestAnimationFrame(loop);
	};
	const wake = () => {
		if (!frame && !destroyed) frame = requestAnimationFrame(loop);
	};

	const onPointerMove = (event: PointerEvent) => {
		const rect = canvas.getBoundingClientRect();
		pointer = { ...pointer, x: event.clientX - rect.left, y: event.clientY - rect.top, target: 1 };
	};
	const onPointerLeave = () => {
		pointer.target = 0;
	};

	const resizeObserver = new ResizeObserver(() => {
		resize();
		wake();
	});
	resizeObserver.observe(canvas);
	const intersectionObserver = new IntersectionObserver(([entry]) => {
		visible = entry.isIntersecting;
		if (visible) wake();
	});
	intersectionObserver.observe(canvas);
	const onVisibility = () => {
		if (!document.hidden) wake();
	};

	if (!options.still) {
		window.addEventListener('pointermove', onPointerMove, { passive: true });
		document.documentElement.addEventListener('pointerleave', onPointerLeave);
	}
	document.addEventListener('visibilitychange', onVisibility);
	canvas.addEventListener('webglcontextlost', () => destroy());

	function destroy() {
		destroyed = true;
		cancelAnimationFrame(frame);
		resizeObserver.disconnect();
		intersectionObserver.disconnect();
		window.removeEventListener('pointermove', onPointerMove);
		document.documentElement.removeEventListener('pointerleave', onPointerLeave);
		document.removeEventListener('visibilitychange', onVisibility);
	}

	resize();
	wake();

	return {
		setPalette(palette) {
			setPalette(palette);
			wake();
		},
		setQuietZones(zones) {
			quietZones = zones;
			gl.uniform4fv(u.quiet, quietZoneUniform(zones, scale));
			wake();
		},
		destroy,
	};
}
