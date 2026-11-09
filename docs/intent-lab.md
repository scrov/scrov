# Intent Lab

**Live:** https://scrov.app/lab/intent/ · **Source:** [`public/lab/intent/`](../public/lab/intent/) · **Tests:** [`scripts/intent-core.test.ts`](../scripts/intent-core.test.ts)

Intent Lab is a browser prototype of brain–computer interface (BCI) cursor control. A real decoder turns a neural stream into velocities that are noisy, delayed, binned and sometimes missing. The app then has to turn those into a cursor that someone can aim with, with no tactile feedback at all. Intent Lab simulates that decoder so anyone can feel the problem with a mouse, a touchscreen or a keyboard, and so every design choice can be measured instead of argued.

## The problem

- **The input is a stream, not a pointer.** A decoder reports a velocity every few milliseconds. It has noise when you mean to move and noise when you mean to stay still, a bias that wanders, and dropped packets.
- **Holding still is the hard part.** Selecting by dwelling means holding the cursor on a target, which is exactly when noise is most visible.
- **Every stage adds lag.** Bin size, decoder delay, filtering, animation and waiting for the next display refresh all add up. A 15 ms budget doesn't leave room for guesswork.
- **There is no physical feedback.** No trackpad friction and no key travel. How the cursor moves on screen is all the feedback the user gets.

## Signal path

```
input event ──► main thread ──► Worker: bin clock ──► DecoderSim ──► postMessage
(timestamped)                   every binMs            gain + noise + drift
                                                       + delay + dropout
     ┌──────────────────────────────────────────────────────────┘
     ▼
integrate velocity ──► filter (position) ──► spring (k, ζ) ──► canvas ──► selection
   raw position          None / Exp / 1€       drawn cursor              on the drawn cursor
```

- **Mouse:** Pointer Lock with `pointerrawupdate` where available, giving raw, unbounded movement. The system cursor is hidden, so the decoded cursor is the only one on screen.
- **Touch and pen:** a drag is intent and a quick tap is a click.
- **Keyboard:** the arrow keys or WASD produce constant-speed intent and Space clicks. The whole task can be done without a pointer.

## Design decisions

1. **The decoder runs in a Worker, on its own clock.** That mirrors a real system, where decoding is not paced by the UI. The bin timer corrects itself: it aims for the next bin boundary, so jitter doesn't accumulate. Every timestamp is absolute (`performance.timeOrigin + performance.now()`), because a worker's `performance.now()` starts from a different origin than the page's.
2. **Filter position, not velocity.** The One Euro filter raises its cutoff as the signal speeds up. On decoded velocity, the derivative of the noise is large, so the filter would open up exactly while the user is trying to hold still. On integrated position, the speed term tracks real reaches, and the cutoff stays low during a dwell.
3. **A spring between the filter and the screen, with numbers shown.** Stiffness `k` and damping ratio `ζ` are tunable. The panel shows settle time and overshoot, computed by simulating the step response, so they are exact for every `ζ`, not only for the underdamped textbook case. The Designer brief notes that a spring coefficient changes perceptual response time. Here that change is something you can run and compare.
4. **Selection follows the cursor the user sees.** Dwell and click hit-test the drawn (sprung) position, not the hidden raw position. Otherwise the user would be aiming one thing while the system judged another.
5. **Dwell disarms after a selection (the Midas touch).** After a dwell selection, that cell won't select again until the cursor leaves it. Resting never fires twice, which is an error-recovery rule that costs nothing.
6. **Pausing resets the decoder.** Testing found a real bug here. When control paused (for example, Tab moved focus off the stage), bins still sitting in the delay queue were replayed on resume, carrying old movement and old timestamps. That produced ~300 ms latency outliers. `DecoderSim.reset()` now runs whenever the decoder stops, and a regression test covers it.
7. **Measure honestly.**
   - **Stream → frame** is the page's own share, from the worker posting a bin to the frame that draws it.
   - **Intent → frame** is from the input event to the drawn frame, including the simulated decoder delay.
   - Both are reported as p50 and p99, not as means.
   - "Drawn" means submitted to the browser, not lit on the display.
   - On a 60 Hz display, the wait for the next frame alone is up to 16.7 ms, which is more than the whole 15 ms budget. Getting under it takes a faster refresh rate, drawing as soon as data arrives (a low-latency `desynchronized` canvas), or native frame timing (`CADisplayLink` / `CVDisplayLink`) with prediction.
8. **Accessible as built.**
   - Keyboard-only control works.
   - Results and targets are announced through a live region.
   - Correct and missed selections are shown by color and also announced in words.
   - Controls are real form elements with labels.
   - Text meets 4.5:1 contrast in both dark and light themes.

## Metrics

- **Bitrate (bits/s)** = log₂(N − 1) × max(correct − missed, 0) ÷ seconds, for an N-cell grid. Missed selections cancel correct ones, so guessing doesn't pay. This is the "achieved bitrate" used to report intracortical BCI performance on grid and keyboard tasks (see Nuyujukian et al., *IEEE TBME*, 2015).
- **Median time to select:** from when a target appears to its correct selection. This catches the lag that the filter and spring add, which no timestamp can show.
- **Stream → frame** and **intent → frame:** p50 and p99, defined above.
- **Dropped bins** and **display rate:** measured, not assumed.

Every run ends with a results JSON (settings plus metrics) that can be copied, so comparisons can be reproduced.

## Results

Fill this in from runs on real hardware. Headless test browsers don't give representative timing.

| Preset | Bitrate (median of 3 runs) | Median time to select | Stream → frame p99 | Display |
|---|---|---|---|---|
| Raw decoder | [—] bits/s | [—] ms | [—] ms | [—] Hz |
| Smoothed | [—] bits/s | [—] ms | [—] ms | [—] Hz |
| Smoothed + spring | [—] bits/s | [—] ms | [—] ms | [—] Hz |

Use the same grid, mode and decoder settings for every row, and keep the copied JSON as the record.

## Verification

- **18 unit tests** (`bun test scripts/intent-core.test.ts`) cover:
  - the seeded PRNG, and the normal distribution's mean and variance;
  - the bitrate formula and its edge cases;
  - percentile interpolation;
  - One Euro: passes a constant through, cuts jitter at rest, and lags less with higher β;
  - the spring: converges, gives the same result at any frame rate, has no overshoot at ζ = 1, and settles in about 5.83/√k;
  - the decoder: a pure delay when there's no noise, unbiased noise, dropped packets, reset, and reproducibility.
- **Browser checks** in headless Chromium:
  - layout at phone, tablet and desktop sizes, in dark and light;
  - no horizontal overflow, no console errors, and both fonts load;
  - contrast and tap-target audits;
  - a keyboard-only 30 s run, from the first Tab to the results JSON.

## Limits and next steps

- `DecoderSim` is a stand-in for a decoder's output statistics, not a neural model.
- **Next:** draw on arrival with a `desynchronized` canvas and compare stream → frame against frame-paced drawing.
- **Next:** port the stage to SwiftUI + Metal with `CADisplayLink`.
- **Next:** move `DecoderSim` to Rust/WASM.
- **Next:** add a text-entry task and undo as error recovery.

## Files

| File | Role |
|---|---|
| `public/lab/intent/core.js` | Pure logic: PRNG, bitrate, percentiles, One Euro, spring, `DecoderSim` |
| `public/lab/intent/core.d.ts` | Types for the core, checked by `tsc --noEmit` |
| `public/lab/intent/decoder.worker.js` | Bin clock and decoder, off the main thread |
| `public/lab/intent/lab.js` | Input, signal path, task, drawing, metrics |
| `public/lab/intent/index.html`, `lab.css` | Page and styles (tokens from `DESIGN.md`) |
| `scripts/intent-core.test.ts` | Unit tests |

References: Casiez, Roussel & Vogel, "1€ Filter: A Simple Speed-based Low-pass Filter for Noisy Input in Interactive Systems", CHI 2012. Nuyujukian et al., "A High-Performance Keyboard Neural Prosthesis Enabled by Task Optimization", IEEE TBME, 2015.
