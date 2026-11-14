import { describe, expect, test } from "bun:test";
import {
	bitsPerSecond,
	createRng,
	DecoderSim,
	gaussian,
	OneEuro,
	percentile,
	springOvershoot,
	springSettleMs,
	springStep,
} from "../public/lab/intent/core.js";

describe("createRng / gaussian", () => {
	test("is deterministic for a seed and stays in [0, 1)", () => {
		const a = createRng(42);
		const b = createRng(42);
		for (let i = 0; i < 1000; i++) {
			const x = a();
			expect(x).toBe(b());
			expect(x).toBeGreaterThanOrEqual(0);
			expect(x).toBeLessThan(1);
		}
	});

	test("gaussian has mean ~0 and standard deviation ~1", () => {
		const rng = createRng(3);
		const n = 40_000;
		let sum = 0;
		let sumSq = 0;
		for (let i = 0; i < n; i++) {
			const g = gaussian(rng);
			sum += g;
			sumSq += g * g;
		}
		const mean = sum / n;
		expect(Math.abs(mean)).toBeLessThan(0.03);
		expect(Math.abs(Math.sqrt(sumSq / n - mean * mean) - 1)).toBeLessThan(0.03);
	});
});

describe("bitsPerSecond", () => {
	test("matches log2(N - 1) * (correct - incorrect) / seconds", () => {
		expect(bitsPerSecond(36, 20, 2, 60)).toBeCloseTo((Math.log2(35) * 18) / 60, 10);
	});

	test("never goes negative and rejects degenerate tasks", () => {
		expect(bitsPerSecond(36, 2, 5, 60)).toBe(0);
		expect(bitsPerSecond(1, 10, 0, 60)).toBe(0);
		expect(bitsPerSecond(36, 10, 0, 0)).toBe(0);
	});
});

describe("percentile", () => {
	const values = Array.from({ length: 100 }, (_, i) => i + 1);

	test("interpolates between ranks", () => {
		expect(percentile(values, 0)).toBe(1);
		expect(percentile(values, 50)).toBeCloseTo(50.5, 10);
		expect(percentile(values, 100)).toBe(100);
		expect(percentile([5, 1, 3], 50)).toBe(3);
	});

	test("returns NaN for no data", () => {
		expect(percentile([], 50)).toBeNaN();
	});
});

describe("OneEuro", () => {
	test("passes a constant signal through unchanged", () => {
		const f = new OneEuro();
		for (let i = 0; i < 50; i++) expect(f.filter(10, 1 / 60)).toBeCloseTo(10, 10);
	});

	test("cuts jitter at rest", () => {
		const rng = createRng(9);
		const f = new OneEuro({ minCutoff: 1, beta: 0.008 });
		let rawVar = 0;
		let outVar = 0;
		for (let i = 0; i < 2000; i++) {
			const raw = 5 * gaussian(rng);
			const out = f.filter(raw, 1 / 60);
			if (i > 100) {
				rawVar += raw * raw;
				outVar += out * out;
			}
		}
		expect(outVar).toBeLessThan(rawVar * 0.25);
	});

	test("a higher beta lags less behind a fast ramp", () => {
		const slow = new OneEuro({ minCutoff: 1, beta: 0 });
		const fast = new OneEuro({ minCutoff: 1, beta: 0.02 });
		let x = 0;
		let lagSlow = 0;
		let lagFast = 0;
		for (let i = 0; i < 60; i++) {
			x += 1000 / 60; // 1000 px/s reach
			lagSlow = x - slow.filter(x, 1 / 60);
			lagFast = x - fast.filter(x, 1 / 60);
		}
		expect(lagFast).toBeLessThan(lagSlow / 3);
	});
});

describe("spring", () => {
	test("converges to the target", () => {
		let s = { x: 0, v: 0 };
		for (let i = 0; i < 240; i++) s = springStep(s, 100, 400, 0.9, 1 / 120);
		expect(s.x).toBeCloseTo(100, 3);
		expect(Math.abs(s.v)).toBeLessThan(1e-3);
	});

	test("is frame-rate independent", () => {
		let a = { x: 0, v: 0 };
		let b = { x: 0, v: 0 };
		for (let i = 0; i < 30; i++) a = springStep(a, 1, 400, 0.7, 1 / 60);
		for (let i = 0; i < 60; i++) b = springStep(b, 1, 400, 0.7, 1 / 120);
		expect(a.x).toBeCloseTo(b.x, 3);
	});

	test("critically damped never overshoots, underdamped does", () => {
		expect(springOvershoot(400, 1)).toBeLessThan(0.001);
		expect(springOvershoot(400, 0.3)).toBeGreaterThan(0.3);
	});

	test("settle time matches theory and falls as stiffness rises", () => {
		// Critically damped 2% settling time is ~5.83 / sqrt(k).
		expect(springSettleMs(400, 1)).toBeGreaterThan(0.9 * (5833 / 20));
		expect(springSettleMs(400, 1)).toBeLessThan(1.1 * (5833 / 20));
		expect(springSettleMs(1200, 1)).toBeLessThan(springSettleMs(300, 1));
	});
});

describe("DecoderSim", () => {
	test("with no noise it is a pure delay of lagBins bins", () => {
		const d = new DecoderSim({ noise: 0, lagBins: 2, dropout: 0, drift: false });
		const out = [10, 20, 30, 40, 50].map((v, i) => d.step(v, -v, i));
		expect(out.map((o) => o.vx)).toEqual([0, 0, 10, 20, 30]);
		expect(out.map((o) => o.vy)).toEqual([0, 0, -10, -20, -30]);
		expect(out.map((o) => o.tIntent)).toEqual([null, null, 0, 1, 2]);
	});

	test("noise is unbiased without drift", () => {
		const d = new DecoderSim({ seed: 11, noise: 0.5, lagBins: 0, dropout: 0, drift: false });
		let sum = 0;
		const n = 20_000;
		for (let i = 0; i < n; i++) sum += d.step(200, 0).vx;
		expect(Math.abs(sum / n - 200)).toBeLessThan(5);
	});

	test("dropped packets hold the last value and carry no intent time", () => {
		const d = new DecoderSim({ noise: 0, lagBins: 0, dropout: 1, drift: false });
		const out = d.step(100, 0, 5);
		expect(out.dropped).toBe(true);
		expect(out.vx).toBe(0);
		expect(out.tIntent).toBeNull();
	});

	test("reset drops queued bins so a pause never replays stale intent", () => {
		const d = new DecoderSim({ noise: 0, lagBins: 2, dropout: 0, drift: false });
		d.step(500, 0, 1);
		d.step(500, 0, 2);
		d.reset();
		const out = [d.step(0, 0, 10), d.step(0, 0, 11), d.step(0, 0, 12)];
		expect(out.map((o) => o.vx)).toEqual([0, 0, 0]);
		expect(out.map((o) => o.tIntent)).toEqual([null, null, 10]);
	});

	test("is reproducible for a seed", () => {
		const run = () => {
			const d = new DecoderSim({ seed: 5 });
			return Array.from({ length: 50 }, () => d.step(300, 100).vx);
		};
		expect(run()).toEqual(run());
	});
});
