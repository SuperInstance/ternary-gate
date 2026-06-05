# ternary-gate

**Noise gate with attack/hold/release envelope for ternary audio.**

A noise gate is the simplest dynamics processor: if the signal is above the threshold, let it through. If below, mute it. But real gates aren't binary — they have *attack* (how fast they open), *hold* (how long they stay open after signal drops), and *release* (how fast they close). These three parameters turn a crude on/off switch into a musical tool.

This crate implements a full noise gate for ternary signals in `{-1, 0, +1}`. The ternary constraint makes the threshold meaningful: with only three values, gating is a *structural* decision, not just a level decision.

## What's Inside

- **`gate(signal, threshold, attack, hold, release)`** — full envelope noise gate
  - `threshold`: absolute signal level to trigger opening
  - `attack`: ticks to ramp from closed to open
  - `hold`: ticks to stay open after signal drops below threshold
  - `release`: ticks to ramp from open to closed
- **`hard_gate(signal, threshold)`** — instant on/off, no envelope. Zero crossing protection
- **`duck(signal, sidechain, threshold, depth)`** — ducking: reduce signal when sidechain is above threshold

## Quick Example

```rust
use ternary_gate::*;

// A noisy signal with a few real hits
let signal = vec![0, 0, 0, 1, 1, -1, 1, 0, 0, 0, 0];

// Gate with threshold=1, attack=1, hold=2, release=2
let gated = gate(&signal, 1, 1, 2, 2);
// First 3 zeros: closed (muted)
// 1, 1, -1, 1: open (passed through)
// Hold keeps gate open for 2 more ticks after signal drops
// Release ramps down over 2 ticks
// Result: noise removed, signal preserved, no clicks

// Hard gate: instant on/off
let hard = hard_gate(&signal, 1);
// Only non-zero samples pass — more aggressive, potential clicks

// Ducking: reduce music when voice is present
let music = vec![1, 1, 1, 1, 1, 1];
let voice = vec![0, 0, 1, 1, 0, 0]; // voice present in middle
let ducked = duck(&music, &voice, 1, 0.5);
// Music dips when voice is active
```

## The Insight

**Attack/hold/release is the difference between a gate and a click.** Without the envelope, gating produces harsh on/off transitions that sound like artifacts. With the envelope, the gate *breathes* — it opens smoothly, holds through brief gaps, and closes gently. In ternary, this is even more important because each state change is already discrete — the envelope is the only smoothing tool you have.

**Use cases:**
- **Audio processing** — clean up noisy ternary audio signals
- **Drum pattern cleanup** — gate out low-level noise between hits
- **Voice activation** — detect speech in noisy environments
- **Signal conditioning** — prepare ternary signals for further processing
- **Data stream filtering** — gate out uninteresting events in real-time streams

## See Also

- **ternary-echo** — what comes AFTER the gate in an audio chain
- **ternary-compressor** — (related) dynamics compression for ternary signals
- **ternary-vu** — meter signal levels for gate threshold calibration
- **ternary-bite** — signal degradation (the opposite aesthetic)

## Install

```bash
cargo add ternary-gate
```

## License

MIT
