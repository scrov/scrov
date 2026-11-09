// Types for core.js, so `tsc --noEmit` and editors can check callers and tests.

export type Rng = () => number;

export function createRng(seed?: number): Rng;
export function gaussian(rng: Rng): number;
export function bitsPerSecond(targets: number, correct: number, incorrect: number, seconds: number): number;
export function percentile(values: readonly number[], p: number): number;
export function smoothingFactor(dt: number, cutoffHz: number): number;

export class ExpSmoother {
	constructor(alpha?: number);
	alpha: number;
	value: number | null;
	reset(): void;
	filter(x: number): number;
}

export interface OneEuroOptions {
	minCutoff?: number;
	beta?: number;
	dCutoff?: number;
}

export class OneEuro {
	constructor(options?: OneEuroOptions);
	minCutoff: number;
	beta: number;
	dCutoff: number;
	prevRaw: number | null;
	value: number | null;
	dx: number;
	reset(): void;
	filter(x: number, dt: number): number;
}

export interface SpringState {
	x: number;
	v: number;
}

export function springStep(
	state: SpringState,
	target: number,
	stiffness: number,
	dampingRatio: number,
	dt: number,
): SpringState;
export function springSettleMs(stiffness: number, dampingRatio: number, tolerance?: number): number;
export function springOvershoot(stiffness: number, dampingRatio: number): number;

export interface DecoderOptions {
	seed?: number;
	noise?: number;
	lagBins?: number;
	dropout?: number;
	drift?: boolean;
	gain?: number;
}

export interface DecodedBin {
	vx: number;
	vy: number;
	dropped: boolean;
	/** When the intent behind this bin happened; null when no fresh intent arrived. */
	tIntent: number | null;
}

export class DecoderSim {
	constructor(options?: DecoderOptions);
	noise: number;
	lagBins: number;
	dropout: number;
	drift: boolean;
	gain: number;
	configure(options?: Omit<DecoderOptions, "seed">): void;
	reset(): void;
	step(intentVx: number, intentVy: number, tIntent?: number): DecodedBin;
}
