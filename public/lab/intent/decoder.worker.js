// Decoder worker: runs the bin clock off the main thread, like the process
// that turns an implant's stream into velocities. Times are absolute
// (timeOrigin + now) because a worker's performance.now() has its own origin.
import { DecoderSim } from "./core.js";

const sim = new DecoderSim();
let binMs = 15;
let running = false;
let timer = 0;
let nextDue = 0;
let accX = 0;
let accY = 0;
let lastIntent = 0;

const nowAbs = () => performance.timeOrigin + performance.now();

function tick() {
	const now = nowAbs();
	const seconds = binMs / 1000;
	const out = sim.step(accX / seconds, accY / seconds, lastIntent || now);
	accX = 0;
	accY = 0;
	lastIntent = 0;
	postMessage({
		type: "bin",
		vx: out.vx,
		vy: out.vy,
		dropped: out.dropped,
		tIntent: out.tIntent,
		tSent: nowAbs(),
		binMs,
	});
	// Self-correcting schedule: aim for the next bin boundary, not "binMs from now".
	nextDue += binMs;
	if (nextDue < now) nextDue = now + binMs;
	timer = setTimeout(tick, Math.max(0, nextDue - nowAbs()));
}

function start() {
	if (running) return;
	running = true;
	nextDue = nowAbs() + binMs;
	timer = setTimeout(tick, binMs);
}

function stop() {
	running = false;
	clearTimeout(timer);
	sim.reset(); // stale bins would replay old movement with old timestamps on resume
	accX = 0;
	accY = 0;
	lastIntent = 0;
}

onmessage = (event) => {
	const msg = event.data;
	switch (msg.type) {
		case "intent":
			accX += msg.dx;
			accY += msg.dy;
			lastIntent = Math.max(lastIntent, msg.t);
			break;
		case "config":
			if (typeof msg.binMs === "number") binMs = Math.min(Math.max(msg.binMs, 4), 100);
			sim.configure(msg.decoder ?? {});
			break;
		case "start":
			start();
			break;
		case "stop":
			stop();
			break;
	}
};
