// Intent Lab core: pure, deterministic building blocks. No DOM access, so the
// same module runs in the page, in the decoder worker and under `bun test`.

/** Seeded PRNG (mulberry32). Returns a function that yields numbers in [0, 1). */
export function createRng(seed = 1) {
	let a = seed >>> 0;
	return () => {
		a = (a + 0x6d2b79f5) >>> 0;
		let t = a;
		t = Math.imul(t ^ (t >>> 15), t | 1);
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

/** Standard normal sample (Box–Muller). */
export function gaussian(rng) {
	let u = 0;
	while (u === 0) u = rng();
	const v = rng();
	return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
}

/**
 * Achieved bitrate of an N-target selection task, in bits per second:
 * log2(N - 1) * max(correct - incorrect, 0) / seconds.
 * This is the metric used for intracortical BCI grid tasks in the literature.
 */
export function bitsPerSecond(targets, correct, incorrect, seconds) {
	if (targets < 2 || seconds <= 0) return 0;
	return (Math.log2(targets - 1) * Math.max(correct - incorrect, 0)) / seconds;
}

/** Linear-interpolated percentile (p in 0..100). NaN for an empty list. */
export function percentile(values, p) {
	if (values.length === 0) return Number.NaN;
	const sorted = [...values].sort((a, b) => a - b);
	const rank = (Math.min(Math.max(p, 0), 100) / 100) * (sorted.length - 1);
	const lo = Math.floor(rank);
	const hi = Math.ceil(rank);
	return sorted[lo] + (sorted[hi] - sorted[lo]) * (rank - lo);
}

/** Smoothing factor of a first-order low-pass at `cutoffHz`, sampled every `dt` seconds. */
export function smoothingFactor(dt, cutoffHz) {
	const r = 2 * Math.PI * cutoffHz * dt;
	return r / (r + 1);
}

/** Exponential smoothing for one channel. alpha = 1 passes the input through. */
export class ExpSmoother {
	constructor(alpha = 0.3) {
		this.alpha = alpha;
		this.value = null;
	}

	reset() {
		this.value = null;
	}

	filter(x) {
		this.value = this.value === null ? x : this.value + this.alpha * (x - this.value);
		return this.value;
	}
}

/**
 * One Euro filter for one channel (Casiez, Roussel & Vogel, CHI 2012).
 * Low cutoff while the signal is slow (kills jitter during a dwell), higher
 * cutoff as it speeds up (less lag during a reach).
 */
export class OneEuro {
	constructor({ minCutoff = 1, beta = 0.008, dCutoff = 1 } = {}) {
		this.minCutoff = minCutoff;
		this.beta = beta;
		this.dCutoff = dCutoff;
		this.reset();
	}

	reset() {
		this.prevRaw = null;
		this.value = null;
		this.dx = 0;
	}

	filter(x, dt) {
		if (this.value === null || dt <= 0) {
			this.prevRaw = x;
			this.value = x;
			this.dx = 0;
			return x;
		}
		const rawDx = (x - this.prevRaw) / dt;
		this.dx += smoothingFactor(dt, this.dCutoff) * (rawDx - this.dx);
		const cutoff = this.minCutoff + this.beta * Math.abs(this.dx);
		this.value += smoothingFactor(dt, cutoff) * (x - this.value);
		this.prevRaw = x;
		return this.value;
	}
}

/**
 * Advance a damped spring toward `target` by `dt` seconds.
 * `stiffness` is k (1/s²) and `dampingRatio` is ζ. Integrates with
 * semi-implicit Euler in sub-steps of at most 1/480 s, so it stays stable
 * at any frame rate. Returns a new { x, v }.
 */
export function springStep(state, target, stiffness, dampingRatio, dt) {
	const c = 2 * dampingRatio * Math.sqrt(stiffness);
	const steps = Math.max(1, Math.ceil(dt * 480));
	const h = dt / steps;
	let { x, v } = state;
	for (let i = 0; i < steps; i++) {
		v += (-stiffness * (x - target) - c * v) * h;
		x += v * h;
	}
	return { x, v };
}

/**
 * Time for a unit step to settle within `tolerance` of the target and stay
 * there, in milliseconds. Simulated, so it is exact for every damping ratio.
 */
export function springSettleMs(stiffness, dampingRatio, tolerance = 0.02) {
	const dt = 1 / 2000;
	let state = { x: 0, v: 0 };
	let lastOutside = 0;
	for (let t = dt; t <= 5; t += dt) {
		state = springStep(state, 1, stiffness, dampingRatio, dt);
		if (Math.abs(state.x - 1) > tolerance) lastOutside = t;
	}
	return Math.round(lastOutside * 1000);
}

/** Largest overshoot of a unit step response, as a fraction of the step (0 = none). */
export function springOvershoot(stiffness, dampingRatio) {
	const dt = 1 / 2000;
	let state = { x: 0, v: 0 };
	let peak = 0;
	for (let t = 0; t <= 5; t += dt) {
		state = springStep(state, 1, stiffness, dampingRatio, dt);
		peak = Math.max(peak, state.x);
	}
	return Math.max(0, peak - 1);
}

/**
 * Simulated velocity decoder. Each call to `step` is one bin: it takes the
 * intended velocity (px/s) and returns what a noisy decoder would report:
 *  - signal-dependent Gaussian noise plus a floor that is present at rest,
 *  - a slowly wandering bias (drift),
 *  - a fixed delay of `lagBins` bins,
 *  - dropped packets that hold the last output, like a lost radio packet.
 */
export class DecoderSim {
	constructor(options = {}) {
		this.rng = createRng(options.seed ?? 7);
		this.queue = [];
		this.last = { vx: 0, vy: 0 };
		this.bias = { x: 0, y: 0 };
		this.configure({ noise: 0.25, lagBins: 2, dropout: 0.02, drift: true, gain: 1, ...options });
	}

	configure({ noise, lagBins, dropout, drift, gain } = {}) {
		if (noise !== undefined) this.noise = noise;
		if (lagBins !== undefined) this.lagBins = Math.max(0, Math.round(lagBins));
		if (dropout !== undefined) this.dropout = dropout;
		if (drift !== undefined) this.drift = drift;
		if (gain !== undefined) this.gain = gain;
		if (!this.drift) this.bias = { x: 0, y: 0 };
		while (this.queue.length > this.lagBins + 1) this.queue.shift();
	}

	/** Forget queued bins and the held output, e.g. when control pauses. */
	reset() {
		this.queue = [];
		this.last = { vx: 0, vy: 0 };
	}

	step(intentVx, intentVy, tIntent = 0) {
		const speed = Math.hypot(intentVx, intentVy);
		const sigma = this.noise * (0.5 * speed + 300);
		if (this.drift) {
			const kick = this.noise * 6;
			this.bias.x = 0.997 * this.bias.x + kick * gaussian(this.rng);
			this.bias.y = 0.997 * this.bias.y + kick * gaussian(this.rng);
		}
		this.queue.push({
			vx: this.gain * intentVx + sigma * gaussian(this.rng) + this.bias.x,
			vy: this.gain * intentVy + sigma * gaussian(this.rng) + this.bias.y,
			tIntent,
		});
		if (this.queue.length <= this.lagBins) {
			return { vx: 0, vy: 0, dropped: false, tIntent: null };
		}
		const out = this.queue.shift();
		if (this.rng() < this.dropout) {
			return { vx: this.last.vx, vy: this.last.vy, dropped: true, tIntent: null };
		}
		this.last = { vx: out.vx, vy: out.vy };
		return { vx: out.vx, vy: out.vy, dropped: false, tIntent: out.tIntent };
	}
}
