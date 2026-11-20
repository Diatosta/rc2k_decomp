//! The game's force-feedback effects on a pad with two rumble motors and no force feedback (an
//! Xbox pad, a DualSense): the effects running at a moment, each as strong as it is then, added
//! up onto the motors. A steady force or a ramp, and a slow shake, go to the heavy motor; a fast
//! shake goes to the light one. Conditions (a spring, a damper) resist a wheel's turning and have
//! nothing to shake: they are left out.

/// What an effect is, as far as the motors care.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// A steady force: its level.
    Constant,
    /// A force from one level to another over the effect's length.
    Ramp,
    /// A shake: its magnitude and offset, the period in microseconds its fourth figure.
    Periodic,
    /// A spring, damper, inertia or friction.
    Condition,
}

/// One effect that is running.
#[derive(Clone, Copy, Debug)]
pub struct Running<'a> {
    pub form: Form,
    /// DirectInput's figures for its form (levels -10000 .. 10000).
    pub params: &'a [i32],
    /// The effect's own gain, 0 .. 10000.
    pub gain: u32,
    /// Its length in microseconds (`u32::MAX`: for ever).
    pub length_us: u32,
    /// Attack level, attack time, fade level, fade time (levels 0 .. 10000, times in
    /// microseconds).
    pub envelope: Option<[u32; 4]>,
    /// How long it has been running, in microseconds.
    pub elapsed_us: u64,
}

/// A shake faster than this (its period shorter, in microseconds) is the light motor's.
pub(crate) const FAST_PERIOD_US: i32 = 100_000;

const FULL: i64 = 10_000;

impl Running<'_> {
    fn param(&self, n: usize) -> i64 {
        self.params.get(n).copied().unwrap_or(0) as i64
    }

    fn forever(&self) -> bool {
        self.length_us == u32::MAX
    }

    /// Its strength now, 0 .. 10000, and whether it is the light motor's; none once it is over.
    fn strength(&self) -> Option<(i64, bool)> {
        let t = self.elapsed_us as i64;
        let length = self.length_us as i64;
        if !self.forever() && t >= length {
            return None;
        }
        let (level, fast) = match self.form {
            Form::Condition => return Some((0, false)),
            Form::Constant => (self.param(0).abs(), false),
            Form::Ramp => {
                let (start, end) = (self.param(0), self.param(1));
                let level = if self.forever() || length == 0 {
                    start
                } else {
                    start + (end - start) * t / length
                };
                (level.abs(), false)
            }
            Form::Periodic => (
                self.param(0).abs() + self.param(1).abs(),
                self.param(3) < FAST_PERIOD_US as i64,
            ),
        };
        let mut level = level.min(FULL);
        if let Some([attack_level, attack_us, fade_level, fade_us]) = self.envelope {
            let (attack_level, attack_us) = (attack_level as i64, attack_us as i64);
            let (fade_level, fade_us) = (fade_level as i64, fade_us as i64);
            if t < attack_us {
                level = attack_level + (level - attack_level) * t / attack_us;
            } else if !self.forever() && fade_us > 0 && t > length - fade_us {
                level = fade_level + (level - fade_level) * (length - t) / fade_us;
            }
        }
        Some((
            level.clamp(0, FULL) * self.gain.min(FULL as u32) as i64 / FULL,
            fast,
        ))
    }
}

/// The motors' levels (heavy, light; 0 .. 65535) for the effects running, at the device's gain
/// (0 .. 10000).
pub fn motors<'a>(running: impl IntoIterator<Item = Running<'a>>, device_gain: u32) -> (u16, u16) {
    let (mut heavy, mut light) = (0i64, 0i64);
    for (level, fast) in running.into_iter().filter_map(|r| r.strength()) {
        if fast {
            light += level;
        } else {
            heavy += level;
        }
    }
    let scale = |v: i64| {
        let v = (v * device_gain.min(FULL as u32) as i64 / FULL).min(FULL);
        (v * 65535 / FULL) as u16
    };
    (scale(heavy), scale(light))
}

/// How a phone's one motor is told to pulse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pulse {
    /// 0.0 (barely) to 1.0 (as strong as the motor goes).
    pub strength: f32,
    /// How long the pulse lasts, in milliseconds.
    pub ms: u32,
}

/// The weakest rumble a phone pulses for, as a fraction of the motors' full level: lighter
/// ones are a buzz that says nothing and only drains the battery.
const PHONE_WEAKEST: f32 = 0.12;
/// The shortest and longest pulse (ms). A running rumble is pulsed again by the caller, so it
/// is a train of pulses: a full one lasts longer than the gap, so that the next starts before
/// it ends and the motor runs steadily; a light one is shorter than the gap, a rumble that
/// pulses.
const PHONE_SHORTEST_MS: u32 = 60;
const PHONE_LONGEST_MS: u32 = 160;
/// The least time between two pulses (ms), so that a rumble that changes every frame does not
/// start a pulse every frame.
pub const PHONE_PULSE_GAP_MS: u128 = 120;

/// The two motors' levels (0 .. 65535) mixed down to a phone's one motor: the stronger of the
/// two sets the strength, and the pulse lasts longer the stronger it is. A phone's motor is
/// felt much less than a pad's, so the strength follows a square-root curve: the game's typical
/// effects (a fraction of full) are felt, and full strength is still reached at full level.
/// None when the rumble is too light to be worth a pulse.
pub fn phone_pulse(heavy: u16, light: u16) -> Option<Pulse> {
    let level = f32::from(heavy.max(light)) / 65535.0;
    if level < PHONE_WEAKEST {
        return None;
    }
    let strength = level.sqrt();
    let ms = PHONE_SHORTEST_MS
        + ((PHONE_LONGEST_MS - PHONE_SHORTEST_MS) as f32 * strength).round() as u32;
    Some(Pulse { strength, ms })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn running(form: Form, params: &[i32]) -> Running<'_> {
        Running {
            form,
            params,
            gain: 10_000,
            length_us: u32::MAX,
            envelope: None,
            elapsed_us: 0,
        }
    }

    #[test]
    fn nothing_running_is_still() {
        assert_eq!(motors([], 10_000), (0, 0));
    }

    #[test]
    fn a_steady_force_is_the_heavy_motor_whatever_its_side() {
        assert_eq!(
            motors([running(Form::Constant, &[-5000])], 10_000),
            (32767, 0)
        );
        assert_eq!(
            motors([running(Form::Constant, &[5000])], 10_000),
            (32767, 0)
        );
    }

    #[test]
    fn shakes_go_by_their_speed() {
        let slow = [8000, 0, 0, 200_000];
        let fast = [4000, 1000, 0, 20_000];
        assert_eq!(motors([running(Form::Periodic, &slow)], 10_000), (52428, 0));
        assert_eq!(motors([running(Form::Periodic, &fast)], 10_000), (0, 32767));
    }

    #[test]
    fn springs_do_not_shake() {
        let spring = [0, 10000, 10000, 10000, 10000, 0];
        assert_eq!(motors([running(Form::Condition, &spring)], 10_000), (0, 0));
    }

    #[test]
    fn gains_and_the_sum_are_held_to_full() {
        let mut half = running(Form::Constant, &[10000]);
        half.gain = 5000;
        assert_eq!(motors([half], 10_000), (32767, 0));
        assert_eq!(motors([half], 5000), (16383, 0));
        let full = running(Form::Constant, &[10000]);
        assert_eq!(motors([full, full], 10_000), (65535, 0));
    }

    #[test]
    fn an_effect_ends_with_its_length() {
        let mut knock = running(Form::Constant, &[10000]);
        knock.length_us = 200_000;
        knock.elapsed_us = 199_999;
        assert_eq!(motors([knock], 10_000).0, 65535);
        knock.elapsed_us = 200_000;
        assert_eq!(motors([knock], 10_000), (0, 0));
    }

    #[test]
    fn a_ramp_moves_over_its_length() {
        let mut ramp = running(Form::Ramp, &[10000, 0]);
        ramp.length_us = 1_000_000;
        ramp.elapsed_us = 500_000;
        assert_eq!(motors([ramp], 10_000).0, 32767);
    }

    #[test]
    fn the_envelope_rises_and_fades() {
        let mut e = running(Form::Constant, &[10000]);
        e.length_us = 1_000_000;
        e.envelope = Some([0, 100_000, 0, 100_000]);
        e.elapsed_us = 50_000;
        assert_eq!(motors([e], 10_000).0, 32767);
        e.elapsed_us = 500_000;
        assert_eq!(motors([e], 10_000).0, 65535);
        e.elapsed_us = 950_000;
        assert_eq!(motors([e], 10_000).0, 32767);
    }

    #[test]
    fn a_phone_pulses_for_the_stronger_motor() {
        assert_eq!(phone_pulse(0, 0), None);
        assert_eq!(phone_pulse(3000, 2000), None);
        let light = phone_pulse(0, 20000).unwrap();
        let both = phone_pulse(20000, 5000).unwrap();
        assert_eq!(light, both);
        let full = phone_pulse(65535, 65535).unwrap();
        assert_eq!(full.strength, 1.0);
        assert_eq!(full.ms, PHONE_LONGEST_MS);
        assert!(light.ms >= PHONE_SHORTEST_MS && light.ms < full.ms);
    }
}
