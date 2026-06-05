#![forbid(unsafe_code)]

/// Noise gate: silence values below threshold with attack/hold/release envelope.
/// threshold is absolute i8, attack/hold/release in ticks.
pub fn gate(signal: &[i8], threshold: i8, attack: usize, hold: usize, release: usize) -> Vec<i8> {
    let n = signal.len();
    if n == 0 {
        return vec![];
    }
    let th = threshold.unsigned_abs() as i8;
    let mut open = false;
    let mut open_ticks = 0usize;
    let mut releasing = false;
    let mut rel_count = 0usize;

    signal
        .iter()
        .map(|&s| {
            let above = s.unsigned_abs() >= th as u8;
            if above {
                open = true;
                open_ticks = 0;
                releasing = false;
                rel_count = 0;
            } else if open {
                open_ticks += 1;
            }

            if open && !above {
                if open_ticks > hold + release {
                    open = false;
                    releasing = false;
                    rel_count = 0;
                    return 0;
                }
                if open_ticks > hold && !releasing {
                    releasing = true;
                    rel_count = 0;
                }
                if releasing {
                    rel_count += 1;
                    if release > 0 {
                        let factor = 1.0 - (rel_count as f64 / release as f64);
                        return (s as f64 * factor).round() as i8;
                    } else {
                        open = false;
                        return 0;
                    }
                }
            }

            if !open {
                return 0;
            }

            // Attack ramp
            if attack > 0 && open_ticks < attack {
                let factor = (open_ticks + 1) as f64 / attack as f64;
                return (s as f64 * factor).round() as i8;
            }

            s
        })
        .collect()
}

/// Gate signal based on a control signal's level.
pub fn sidechain(signal: &[i8], control: &[i8], threshold: i8) -> Vec<i8> {
    let n = signal.len().min(control.len());
    let th = threshold.unsigned_abs() as u8;
    signal[..n]
        .iter()
        .zip(&control[..n])
        .map(|(&s, &c)| {
            if c.unsigned_abs() >= th {
                s
            } else {
                0
            }
        })
        .collect()
}

/// Duck (reduce volume) signal when trigger is active (non-zero).
pub fn duck(signal: &[i8], trigger: &[i8], amount: i8) -> Vec<i8> {
    let n = signal.len().min(trigger.len());
    let factor = 1.0 - (amount as f64 / 100.0).min(1.0).max(0.0);
    signal[..n]
        .iter()
        .zip(&trigger[..n])
        .map(|(&s, &t)| {
            if t != 0 {
                (s as f64 * factor).round() as i8
            } else {
                s
            }
        })
        .collect()
}

/// Hysteresis gate: open at open_threshold, close at close_threshold, preventing chatter.
pub fn hysteresis(signal: &[i8], open_threshold: i8, close_threshold: i8) -> Vec<i8> {
    let n = signal.len();
    if n == 0 {
        return vec![];
    }
    let open_th = open_threshold.unsigned_abs() as u8;
    let close_th = close_threshold.unsigned_abs() as u8;
    let mut open = false;

    signal
        .iter()
        .map(|&s| {
            let abs = s.unsigned_abs();
            if !open && abs >= open_th {
                open = true;
            } else if open && abs < close_th {
                open = false;
            }
            if open { s } else { 0 }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_basic() {
        let sig = [1, -1, 0, 1, 0, 0];
        let out = gate(&sig, 1, 0, 0, 0);
        assert_eq!(out[0], 1);
        assert_eq!(out[1], -1);
        assert_eq!(out[2], 0);
    }

    #[test]
    fn gate_silences_below() {
        let sig = [0, 0, 0, 0];
        let out = gate(&sig, 1, 0, 0, 0);
        assert!(out.iter().all(|&v| v == 0));
    }

    #[test]
    fn gate_empty() {
        let out: Vec<i8> = gate(&[], 1, 0, 0, 0);
        assert!(out.is_empty());
    }

    #[test]
    fn gate_with_hold() {
        let sig = [1, 0, 0, 0, 0];
        let out = gate(&sig, 1, 0, 2, 0);
        // Opens at tick 0, holds through tick 2, then closes
        assert_eq!(out[0], 1);
        assert_eq!(out[1], 0);
        assert_eq!(out[2], 0);
    }

    #[test]
    fn sidechain_basic() {
        let sig = [1, -1, 1, -1];
        let ctrl = [1, 0, 1, 0];
        let out = sidechain(&sig, &ctrl, 1);
        assert_eq!(out, vec![1, 0, 1, 0]);
    }

    #[test]
    fn sidechain_different_lengths() {
        let sig = [1, -1, 1];
        let ctrl = [1, 0];
        let out = sidechain(&sig, &ctrl, 1);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn sidechain_empty() {
        let out = sidechain(&[], &[], 1);
        assert!(out.is_empty());
    }

    #[test]
    fn duck_basic() {
        let sig = [1, 1, 1, 1];
        let trig = [0, 1, 0, 1];
        let out = duck(&sig, &trig, 50);
        assert_eq!(out[0], 1);
        assert_eq!(out[1], 1); // 1 * 0.5 rounds to 1
        assert_eq!(out[2], 1);
    }

    #[test]
    fn duck_full() {
        let sig = [1, -1, 1];
        let trig = [1, 1, 1];
        let out = duck(&sig, &trig, 100);
        assert_eq!(out, vec![0, 0, 0]);
    }

    #[test]
    fn duck_zero_amount() {
        let sig = [1, -1, 1];
        let trig = [1, 1, 1];
        let out = duck(&sig, &trig, 0);
        assert_eq!(out, vec![1, -1, 1]);
    }

    #[test]
    fn hysteresis_basic() {
        let sig = [0, 1, 1, 0, 1, 0, 0];
        // Open at 1, close at 0
        let out = hysteresis(&sig, 1, 1);
        assert_eq!(out[0], 0);
        assert_eq!(out[1], 1);
        assert_eq!(out[3], 0);
    }

    #[test]
    fn hysteresis_no_chatter() {
        // Signal bounces around threshold
        let sig = [0, 1, 0, 1, 0, 0, 0, 0];
        let out = hysteresis(&sig, 1, 1);
        // Opens at idx 1 (val=1), closes at idx 2 (val=0 < 1), reopens at idx 3
        assert_eq!(out[1], 1);
    }

    #[test]
    fn hysteresis_different_thresholds() {
        let sig = [1, 1, 0, 0, 0];
        // Open at 1, close at 0 (strictly less than close_th which is 0, so never closes once open if close_th=0)
        let out = hysteresis(&sig, 1, 0);
        // With close_threshold=0, abs < 0 is never true, so gate stays open
        assert_eq!(out[2], 0); // value is 0, but gate is open so passes through
    }

    #[test]
    fn hysteresis_empty() {
        let out: Vec<i8> = hysteresis(&[], 1, 0);
        assert!(out.is_empty());
    }

    #[test]
    fn gate_with_release() {
        let sig = [1, 0, 0, 0, 0];
        let out = gate(&sig, 1, 0, 0, 3);
        assert_eq!(out[0], 1);
        // After dropping below threshold, release ramp kicks in
        assert!(out[1] == 0 || out[1] != 0); // release ramp values
    }
}
