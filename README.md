# ternary-gate

**The sound of silence. When to shut up, and when to let it through.**

A noise gate is the simplest dynamics processor and maybe the most musical: if the signal is loud enough, let it through. If it's too quiet, mute it. The threshold is the line between "music" and "noise." Below the line, silence. Above it, sound.

But real gates aren't binary switches. They have *attack* (how fast they open — too fast and you get clicks, too slow and you miss the transient), *hold* (how long they stay open after the signal drops — prevents stuttering on decaying sounds), and *release* (how fast they close — too fast and you get a chopped-off tail, too slow and noise leaks in). These three parameters turn a crude on/off switch into a musical instrument.

## What's Inside

- **`NoiseGate`** — configurable gate with threshold, attack, hold, and release
- **`process(signal)`** — apply the gate to a ternary signal. Returns gated output
- **`sidechain(gate, control_signal, audio_signal)`** — gate the audio based on a *different* signal. The foundation of ducking and pumping effects
- **`hysteresis(open_threshold, close_threshold)`** — the gate opens at one level, closes at a lower one. Prevents chatter at the threshold boundary
- **`GateState`** — current state: `Closed`, `Attacking`, `Open`, `Holding`, `Releasing`

## Quick Example

```rust
use ternary_gate::*;

let mut gate = NoiseGate::new()
    .threshold(0.5)   // open when RMS > 0.5
    .attack(2)        // 2 ticks to fully open
    .hold(4)          // stay open 4 ticks after signal drops
    .release(8);      // 8 ticks to fully close

let noisy = vec![0, 0, 1, 1, -1, 0, 0, 0, 0, 0];
let gated = gate.process(&noisy);
// [0, 0, 1, 1, -1, ?, ?, ?, 0, 0]
// Opens when signal hits, stays open through hold, gradually closes during release

// Sidechain: duck music when voice is present
let voice = vec![0, 0, 1, 1, 1, 0, 0]; // voice active in middle
let music = vec![1, 1, 1, 1, 1, 1, 1]; // constant music
let ducked = sidechain(&gate, &voice, &music);
// Music ducks down when voice is present
```

## The Deeper Truth

**Gating is the most important ternary effect because ternary already IS gated.** In continuous audio, gating removes everything below a threshold. In ternary, the 0 state is already "below threshold" — it IS silence. So gating a ternary signal means deciding when the ±1 values should become 0, and when the 0 values should become ±1. It's threshold detection at the most fundamental level.

The sidechain is the secret weapon. In electronic music, sidechain compression (ducking the bass when the kick hits) creates the "breathing" effect that defines entire genres. In ternary, sidechain gating does the same thing with three states: one signal controls when the other signal is allowed to speak. This is the foundation of all call-and-response patterns in music — when one voice speaks, the others listen.

**Use cases:**
- **Noise removal** — clean up signals by gating out the noise floor
- **Rhythmic effects** — gated synths create stuttering, chopping patterns (trance gates)
- **Sidechain ducking** — make room for vocals or drums in a mix
- **Dynamic control** — turn continuous textures into rhythmic patterns
- **Education** — the simplest dynamics processor, fully transparent

## See Also

- **ternary-vu** — meter the signal to know where to set the threshold
- **ternary-envelope** — envelopes and gates work together (gate opens → envelope shapes)
- **ternary-compressor** — (future) smooth dynamics control instead of hard gating
- **ternary-rack** — wire gates into the signal chain

## Install

```bash
cargo add ternary-gate
```

## License

MIT
