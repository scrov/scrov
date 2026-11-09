// Intent Lab page: input → decoder worker → filter → spring → canvas, plus a
// timed grid task that reports bits per second and end-to-end latency.
import {
	bitsPerSecond,
	ExpSmoother,
	OneEuro,
	percentile,
	springOvershoot,
	springSettleMs,
	springStep,
} from "./core.js";

const $ = (id) => document.getElementById(id);
const stage = $("stage");
const canvas = $("canvas");
const ctx = canvas.getContext("2d");
const form = $("controls");
const overlay = $("overlay");
const announcer = $("announce");

const KEY_SPEED = 520; // px/s of intent while an arrow key is held
const BUDGET_MS = 15;
const MAX_SAMPLES = 20_000;
const nowAbs = () => performance.timeOrigin + performance.now();

const PRESETS = {
	raw: { filter: "none", spring: false },
	smooth: { filter: "oneEuro", spring: false },
	tuned: { filter: "oneEuro", spring: true },
};

/* ---------- settings ---------- */

function readSettings() {
	const el = form.elements;
	return {
		binMs: Number(el.binMs.value),
		lagBins: Number(el.lagBins.value),
		noise: Number(el.noise.value) / 100,
		dropout: Number(el.dropout.value) / 100,
		drift: el.drift.checked,
		filter: el.filter.value,
		minCutoff: Number(el.minCutoff.value),
		beta: Number(el.beta.value),
		alpha: Number(el.alpha.value),
		spring: el.spring.checked,
		stiffness: Number(el.stiffness.value),
		damping: Number(el.damping.value),
		showRaw: el.showRaw.checked,
		grid: Number(el.grid.value),
		mode: el.mode.value,
		dwellMs: Number(el.dwellMs.value),
		duration: Number(el.duration.value),
	};
}

const outputs = {
	binMs: (s) => `${s.binMs} ms`,
	lagBins: (s) => `${s.lagBins} bins · ${s.lagBins * s.binMs} ms`,
	noise: (s) => `${Math.round(s.noise * 100)}%`,
	dropout: (s) => `${Math.round(s.dropout * 100)}%`,
	minCutoff: (s) => `${s.minCutoff.toFixed(1)} Hz`,
	beta: (s) => s.beta.toFixed(3),
	alpha: (s) => s.alpha.toFixed(2),
	stiffness: (s) => `${s.stiffness}`,
	damping: (s) => s.damping.toFixed(2),
	grid: (s) => `${s.grid} × ${s.grid}`,
	dwellMs: (s) => `${s.dwellMs} ms`,
};

let s = readSettings();

/* ---------- state ---------- */

const geom = { w: 0, h: 0, gx: 0, gy: 0, size: 0, cell: 0, dpr: 1 };
const raw = { x: 0, y: 0 };
const target = { x: 0, y: 0 };
let springX = { x: 0, v: 0 };
let springY = { x: 0, v: 0 };
const render = { x: 0, y: 0 };
const oneEuro = { x: new OneEuro(), y: new OneEuro() };
const exp = { x: new ExpSmoother(), y: new ExpSmoother() };

const pending = [];
const latIntent = [];
const latStream = [];
const frameIntervals = [];
const binsSeen = { total: 0, dropped: 0 };

const run = {
	active: false,
	start: 0,
	end: 0,
	correct: 0,
	incorrect: 0,
	selectTimes: [],
	targetCell: -1,
	targetSince: 0,
	result: null,
};
const dwell = { cell: -1, ms: 0, disarmed: -1 };
let flash = { cell: -1, kind: "", until: 0 };

const input = { locked: false, touchId: null, touchX: 0, touchY: 0, tapStart: 0, tapMoved: 0, keys: new Set() };
let decoderOn = false;
let lastFrame = 0;
let lastMetrics = 0;
let theme = {};

/* ---------- worker ---------- */

const worker = new Worker(new URL("./decoder.worker.js", import.meta.url), { type: "module" });
worker.onmessage = (event) => {
	if (event.data.type === "bin") pending.push(event.data);
};

function sendConfig() {
	worker.postMessage({
		type: "config",
		binMs: s.binMs,
		decoder: { noise: s.noise, lagBins: s.lagBins, dropout: s.dropout, drift: s.drift },
	});
}

function sendIntent(dx, dy, timeStamp) {
	if (!decoderOn || (dx === 0 && dy === 0)) return;
	worker.postMessage({ type: "intent", dx, dy, t: performance.timeOrigin + timeStamp });
}

function updateDecoderState() {
	const want =
		!document.hidden &&
		(run.active || input.locked || input.touchId !== null || document.activeElement === stage);
	if (want !== decoderOn) {
		decoderOn = want;
		if (!want) pending.length = 0;
		worker.postMessage({ type: want ? "start" : "stop" });
	}
	overlay.hidden = want;
}

/* ---------- geometry and theme ---------- */

function readTheme() {
	const css = getComputedStyle(document.documentElement);
	const v = (name) => css.getPropertyValue(name).trim();
	theme = {
		bg: v("--surface"),
		cell: v("--surface-2"),
		line: v("--border"),
		strong: v("--border-strong"),
		text: v("--text"),
		subtle: v("--text-subtle"),
		primary: v("--primary"),
		accent: v("--accent"),
		success: v("--success"),
		error: v("--error"),
	};
}

function layout() {
	const rect = stage.getBoundingClientRect();
	const dpr = Math.min(window.devicePixelRatio || 1, 3);
	const oldW = geom.w || rect.width;
	const oldH = geom.h || rect.height;
	geom.w = rect.width;
	geom.h = rect.height;
	geom.dpr = dpr;
	canvas.width = Math.round(rect.width * dpr);
	canvas.height = Math.round(rect.height * dpr);
	geom.size = Math.min(rect.width, rect.height) - 32;
	geom.cell = geom.size / s.grid;
	geom.gx = (rect.width - geom.size) / 2;
	geom.gy = (rect.height - geom.size) / 2;
	// Keep the cursor at the same relative place when the stage resizes.
	const sx = rect.width / oldW;
	const sy = rect.height / oldH;
	for (const p of [raw, target, render]) {
		p.x = p.x ? p.x * sx : rect.width / 2;
		p.y = p.y ? p.y * sy : rect.height / 2;
	}
	springX = { x: render.x, v: 0 };
	springY = { x: render.y, v: 0 };
}

function cellAt(x, y) {
	const c = Math.floor((x - geom.gx) / geom.cell);
	const r = Math.floor((y - geom.gy) / geom.cell);
	if (c < 0 || r < 0 || c >= s.grid || r >= s.grid) return -1;
	return r * s.grid + c;
}

const clamp = (v, lo, hi) => Math.min(Math.max(v, lo), hi);
const describeCell = (cell) => `row ${Math.floor(cell / s.grid) + 1}, column ${(cell % s.grid) + 1}`;

/* ---------- task ---------- */

function say(text) {
	announcer.textContent = "";
	requestAnimationFrame(() => {
		announcer.textContent = text;
	});
}

function nextTarget(now) {
	const n = s.grid * s.grid;
	let cell = run.targetCell;
	while (cell === run.targetCell) cell = Math.floor(Math.random() * n);
	run.targetCell = cell;
	run.targetSince = now;
}

function select(cell, now) {
	if (!run.active || cell < 0) return;
	const hit = cell === run.targetCell;
	if (hit) {
		run.correct++;
		run.selectTimes.push(now - run.targetSince);
		nextTarget(now);
		say(`Correct. Next target ${describeCell(run.targetCell)}.`);
	} else {
		run.incorrect++;
		say(`Missed. Target is ${describeCell(run.targetCell)}.`);
	}
	flash = { cell, kind: hit ? "hit" : "miss", until: now + 260 };
	dwell.disarmed = cell; // re-arms only after the cursor leaves this cell
	dwell.ms = 0;
	updateScore();
}

function startRun() {
	const now = performance.now();
	Object.assign(run, {
		active: true,
		start: now,
		end: now + s.duration * 1000,
		correct: 0,
		incorrect: 0,
		selectTimes: [],
		targetCell: -1,
		result: null,
	});
	latIntent.length = 0;
	latStream.length = 0;
	binsSeen.total = 0;
	binsSeen.dropped = 0;
	dwell.disarmed = cellAt(render.x, render.y);
	nextTarget(now);
	$("results").hidden = true;
	$("start").textContent = "Restart run";
	stage.focus({ preventScroll: true });
	updateDecoderState();
	updateScore();
	say(`Run started, ${s.duration} seconds. Target ${describeCell(run.targetCell)}.`);
}

function finishRun(now) {
	run.active = false;
	const seconds = (now - run.start) / 1000;
	const stats = (arr) => ({
		p50: round(percentile(arr, 50), 1),
		p95: round(percentile(arr, 95), 1),
		p99: round(percentile(arr, 99), 1),
	});
	run.result = {
		lab: "scrov intent-lab",
		version: 1,
		date: new Date().toISOString(),
		seconds: round(seconds, 1),
		grid: s.grid,
		correct: run.correct,
		incorrect: run.incorrect,
		bitsPerSecond: round(bitsPerSecond(s.grid * s.grid, run.correct, run.incorrect, seconds), 3),
		medianSelectMs: Math.round(percentile(run.selectTimes, 50)) || null,
		latencyMs: { intentToFrame: stats(latIntent), streamToFrame: stats(latStream) },
		refreshHz: refreshHz(),
		droppedBinsPct: round((100 * binsSeen.dropped) / Math.max(binsSeen.total, 1), 1),
		settings: { ...s },
	};
	$("results-json").textContent = JSON.stringify(run.result, null, 2);
	$("results").hidden = false;
	$("start").textContent = "Start another run";
	updateDecoderState();
	updateScore();
	say(
		`Run complete. ${run.result.bitsPerSecond} bits per second, ${run.correct} correct, ${run.incorrect} missed.`,
	);
}

function resetAll() {
	run.active = false;
	run.targetCell = -1;
	run.correct = 0;
	run.incorrect = 0;
	run.selectTimes = [];
	run.result = null;
	latIntent.length = 0;
	latStream.length = 0;
	binsSeen.total = 0;
	binsSeen.dropped = 0;
	pending.length = 0;
	for (const p of [raw, target, render]) {
		p.x = geom.w / 2;
		p.y = geom.h / 2;
	}
	springX = { x: render.x, v: 0 };
	springY = { x: render.y, v: 0 };
	resetFilters();
	$("results").hidden = true;
	$("start").textContent = `Start ${s.duration} s run`;
	updateDecoderState();
	updateScore();
	say("Reset.");
}

const round = (v, digits) => (Number.isFinite(v) ? Number(v.toFixed(digits)) : null);

/* ---------- signal path ---------- */

function resetFilters() {
	oneEuro.x.reset();
	oneEuro.y.reset();
	exp.x.reset();
	exp.y.reset();
}

function applyFilter(dt) {
	if (s.filter === "oneEuro") {
		target.x = oneEuro.x.filter(raw.x, dt);
		target.y = oneEuro.y.filter(raw.y, dt);
	} else if (s.filter === "exp") {
		target.x = exp.x.filter(raw.x);
		target.y = exp.y.filter(raw.y);
	} else {
		target.x = raw.x;
		target.y = raw.y;
	}
}

function keyIntent(dt, timeStamp) {
	const k = input.keys;
	let vx = (k.has("right") ? 1 : 0) - (k.has("left") ? 1 : 0);
	let vy = (k.has("down") ? 1 : 0) - (k.has("up") ? 1 : 0);
	if (vx === 0 && vy === 0) return;
	const norm = Math.hypot(vx, vy);
	vx = (vx / norm) * KEY_SPEED * dt;
	vy = (vy / norm) * KEY_SPEED * dt;
	sendIntent(vx, vy, timeStamp);
}

function frame(t) {
	const dt = lastFrame ? Math.min((t - lastFrame) / 1000, 0.05) : 0;
	if (lastFrame) {
		frameIntervals.push(t - lastFrame);
		if (frameIntervals.length > 120) frameIntervals.shift();
	}
	lastFrame = t;
	keyIntent(dt, t);

	// 1. Integrate every decoded bin that arrived since the last frame.
	const drawnIntents = [];
	const drawnSent = [];
	for (const bin of pending) {
		const sec = bin.binMs / 1000;
		raw.x = clamp(raw.x + bin.vx * sec, 0, geom.w);
		raw.y = clamp(raw.y + bin.vy * sec, 0, geom.h);
		applyFilter(sec);
		binsSeen.total++;
		if (bin.dropped) binsSeen.dropped++;
		if (bin.tIntent !== null) drawnIntents.push(bin.tIntent);
		drawnSent.push(bin.tSent);
	}
	pending.length = 0;

	// 2. Spring the drawn cursor toward the filtered position.
	if (s.spring && dt > 0) {
		springX = springStep(springX, target.x, s.stiffness, s.damping, dt);
		springY = springStep(springY, target.y, s.stiffness, s.damping, dt);
	} else {
		springX = { x: target.x, v: 0 };
		springY = { x: target.y, v: 0 };
	}
	render.x = clamp(springX.x, 0, geom.w);
	render.y = clamp(springY.x, 0, geom.h);

	// 3. Selection: dwell on what the user sees, not on the hidden raw position.
	const now = performance.now();
	const cell = cellAt(render.x, render.y);
	if (cell !== dwell.disarmed) dwell.disarmed = -1;
	if (cell !== dwell.cell) {
		dwell.cell = cell;
		dwell.ms = 0;
	}
	if (run.active && s.mode === "dwell" && cell >= 0 && cell !== dwell.disarmed) {
		dwell.ms += dt * 1000;
		if (dwell.ms >= s.dwellMs) select(cell, now);
	}
	if (run.active && now >= run.end) finishRun(now);

	// 4. Draw, then timestamp the bins this frame just put on screen.
	draw(now, cell);
	const drawnAt = nowAbs();
	for (const ti of drawnIntents) pushSample(latIntent, drawnAt - ti);
	for (const ts of drawnSent) pushSample(latStream, drawnAt - ts);

	if (now - lastMetrics > 250) {
		lastMetrics = now;
		updateMetrics(now);
	}
	requestAnimationFrame(frame);
}

function pushSample(arr, v) {
	if (v < 0 || v > 2000) return;
	arr.push(v);
	if (arr.length > MAX_SAMPLES) arr.shift();
}

/* ---------- drawing ---------- */

function draw(now, hoverCell) {
	const { w, h, gx, gy, cell, dpr } = geom;
	ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
	ctx.fillStyle = theme.bg;
	ctx.fillRect(0, 0, w, h);

	const pad = Math.min(3, cell * 0.08);
	for (let r = 0; r < s.grid; r++) {
		for (let c = 0; c < s.grid; c++) {
			const idx = r * s.grid + c;
			const x = gx + c * cell + pad;
			const y = gy + r * cell + pad;
			const size = cell - pad * 2;
			ctx.fillStyle = idx === run.targetCell && run.active ? theme.primary : theme.cell;
			if (flash.cell === idx && now < flash.until) {
				ctx.fillStyle = flash.kind === "hit" ? theme.success : theme.error;
			}
			ctx.fillRect(x, y, size, size);
			if (idx === hoverCell) {
				ctx.strokeStyle = theme.text;
				ctx.lineWidth = 2;
				ctx.strokeRect(x + 1, y + 1, size - 2, size - 2);
			}
		}
	}

	if (s.showRaw) {
		ctx.strokeStyle = theme.subtle;
		ctx.lineWidth = 1.5;
		ctx.beginPath();
		ctx.arc(raw.x, raw.y, 5, 0, Math.PI * 2);
		ctx.stroke();
	}

	if (run.active && s.mode === "dwell" && dwell.ms > 0 && hoverCell >= 0) {
		const p = Math.min(dwell.ms / s.dwellMs, 1);
		ctx.strokeStyle = theme.accent;
		ctx.lineWidth = 3;
		ctx.beginPath();
		ctx.arc(render.x, render.y, 15, -Math.PI / 2, -Math.PI / 2 + p * Math.PI * 2);
		ctx.stroke();
	}

	ctx.fillStyle = theme.accent;
	ctx.strokeStyle = theme.bg;
	ctx.lineWidth = 2;
	ctx.beginPath();
	ctx.arc(render.x, render.y, 7, 0, Math.PI * 2);
	ctx.fill();
	ctx.stroke();
}

/* ---------- metrics ---------- */

function refreshHz() {
	const median = percentile(frameIntervals, 50);
	return Number.isFinite(median) && median > 0 ? Math.round(1000 / median) : null;
}

const fmtMs = (v) => (Number.isFinite(v) ? `${v.toFixed(1)} ms` : "—");

function updateScore() {
	$("m-score").textContent = `${run.correct} / ${run.incorrect}`;
	$("m-select").textContent = run.selectTimes.length
		? `${Math.round(percentile(run.selectTimes, 50))} ms`
		: "—";
}

function updateMetrics(now) {
	const seconds = run.active ? (now - run.start) / 1000 : run.result?.seconds ?? 0;
	const bps = run.result && !run.active
		? run.result.bitsPerSecond
		: bitsPerSecond(s.grid * s.grid, run.correct, run.incorrect, seconds);
	$("m-bps").textContent = bps.toFixed(2);
	$("clock").textContent = run.active
		? `${Math.max(0, (run.end - now) / 1000).toFixed(1)} s left`
		: `${s.duration}.0 s`;

	const i50 = percentile(latIntent, 50);
	const i99 = percentile(latIntent, 99);
	const s50 = percentile(latStream, 50);
	const s99 = percentile(latStream, 99);
	$("m-intent").textContent = `${fmtMs(i50)} / ${fmtMs(i99)}`;
	$("m-stream").textContent = `${fmtMs(s50)} / ${fmtMs(s99)}`;
	$("m-drop").textContent = `${((100 * binsSeen.dropped) / Math.max(binsSeen.total, 1)).toFixed(1)}%`;
	const hz = refreshHz();
	$("m-hz").textContent = hz ? `${hz} Hz · ${(1000 / hz).toFixed(1)} ms/frame` : "—";

	const fill = $("budget-fill");
	const within = Number.isFinite(s99) && s99 <= BUDGET_MS;
	fill.style.width = `${Number.isFinite(s99) ? Math.min(s99 / (BUDGET_MS * 2), 1) * 100 : 0}%`;
	fill.dataset.state = within ? "ok" : "over";
	$("budget-verdict").textContent = Number.isFinite(s99)
		? `${within ? "Within" : "Over"} the 15 ms budget at p99 (${s99.toFixed(1)} ms).`
		: "Take control of the stage to start measuring.";
}

/* ---------- controls ---------- */

function syncControls() {
	s = readSettings();
	for (const [name, fmt] of Object.entries(outputs)) {
		const out = $(`${name}-out`);
		if (out) out.textContent = fmt(s);
	}
	$("one-euro-fields").hidden = s.filter !== "oneEuro";
	$("exp-fields").hidden = s.filter !== "exp";
	form.elements.stiffness.disabled = !s.spring;
	form.elements.damping.disabled = !s.spring;
	form.elements.dwellMs.disabled = s.mode !== "dwell";
	const settle = springSettleMs(s.stiffness, s.damping);
	const over = springOvershoot(s.stiffness, s.damping);
	$("settle-out").textContent = s.spring
		? `Settles in ${settle} ms · ${Math.round(over * 100)}% overshoot`
		: "Spring off: the cursor jumps straight to the filtered position.";
	for (const f of [oneEuro.x, oneEuro.y]) {
		f.minCutoff = s.minCutoff;
		f.beta = s.beta;
	}
	exp.x.alpha = s.alpha;
	exp.y.alpha = s.alpha;
	if (!run.active) $("start").textContent = run.result ? "Start another run" : `Start ${s.duration} s run`;
	sendConfig();
}

function applyPreset(name) {
	const preset = PRESETS[name];
	if (!preset) return;
	form.elements.filter.value = preset.filter;
	form.elements.spring.checked = preset.spring;
	resetFilters();
	syncControls();
}

form.addEventListener("input", (event) => {
	const field = event.target;
	if (field.name === "preset") {
		applyPreset(field.value);
		return;
	}
	if (field.name === "filter" || field.name === "spring") {
		form.elements.preset.value = "custom";
		resetFilters();
	}
	if (field.name === "grid") {
		syncControls();
		layout();
		if (run.active) nextTarget(performance.now());
		return;
	}
	syncControls();
});
form.addEventListener("submit", (event) => event.preventDefault());

$("start").addEventListener("click", startRun);
$("reset").addEventListener("click", resetAll);
$("copy-results").addEventListener("click", async () => {
	const status = $("copy-status");
	try {
		await navigator.clipboard.writeText($("results-json").textContent);
		status.textContent = "Copied.";
	} catch {
		status.textContent = "Copy failed. Select the text and copy it.";
	}
	setTimeout(() => {
		status.textContent = "";
	}, 2500);
});

/* ---------- input ---------- */

const KEYMAP = {
	ArrowLeft: "left",
	ArrowRight: "right",
	ArrowUp: "up",
	ArrowDown: "down",
	a: "left",
	d: "right",
	w: "up",
	s: "down",
};

stage.addEventListener("keydown", (event) => {
	const dir = KEYMAP[event.key];
	if (dir) {
		input.keys.add(dir);
		event.preventDefault();
	} else if (event.key === " " || event.key === "Enter") {
		event.preventDefault();
		if (!event.repeat && s.mode === "click") select(cellAt(render.x, render.y), performance.now());
	}
});
stage.addEventListener("keyup", (event) => {
	const dir = KEYMAP[event.key];
	if (dir) input.keys.delete(dir);
});
stage.addEventListener("focus", updateDecoderState);
stage.addEventListener("blur", () => {
	input.keys.clear();
	updateDecoderState();
});

// Mouse: Pointer Lock gives raw, unbounded movement and hides the system cursor,
// so the only cursor on screen is the decoded one.
const RAW_MOVE = "onpointerrawupdate" in window ? "pointerrawupdate" : "pointermove";
stage.addEventListener(RAW_MOVE, (event) => {
	if (input.locked && event.pointerType === "mouse") sendIntent(event.movementX, event.movementY, event.timeStamp);
});

stage.addEventListener("pointerdown", (event) => {
	if (event.pointerType === "mouse") {
		if (!input.locked) {
			const request = stage.requestPointerLock?.();
			if (request && typeof request.catch === "function") request.catch(() => {});
		} else if (event.button === 0 && s.mode === "click") {
			select(cellAt(render.x, render.y), performance.now());
		}
		return;
	}
	// Touch and pen: a drag is intent, a quick tap is a click.
	if (input.touchId !== null) return;
	input.touchId = event.pointerId;
	input.touchX = event.clientX;
	input.touchY = event.clientY;
	input.tapStart = event.timeStamp;
	input.tapMoved = 0;
	stage.setPointerCapture(event.pointerId);
	updateDecoderState();
});
stage.addEventListener("pointermove", (event) => {
	if (event.pointerId !== input.touchId) return;
	const dx = event.clientX - input.touchX;
	const dy = event.clientY - input.touchY;
	input.touchX = event.clientX;
	input.touchY = event.clientY;
	input.tapMoved += Math.hypot(dx, dy);
	sendIntent(dx, dy, event.timeStamp);
});
const endTouch = (event) => {
	if (event.pointerId !== input.touchId) return;
	const tap = event.timeStamp - input.tapStart < 250 && input.tapMoved < 10;
	input.touchId = null;
	if (tap && event.type === "pointerup" && s.mode === "click") {
		select(cellAt(render.x, render.y), performance.now());
	}
	updateDecoderState();
};
stage.addEventListener("pointerup", endTouch);
stage.addEventListener("pointercancel", endTouch);

document.addEventListener("pointerlockchange", () => {
	input.locked = document.pointerLockElement === stage;
	updateDecoderState();
});
document.addEventListener("visibilitychange", updateDecoderState);
window.matchMedia("(prefers-color-scheme: light)").addEventListener("change", readTheme);
new ResizeObserver(layout).observe(stage);

/* ---------- boot ---------- */

readTheme();
syncControls();
layout();
resetAll();
announcer.textContent = "";
requestAnimationFrame(frame);
