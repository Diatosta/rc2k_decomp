//! The triggers' own rumble on a pad whose triggers have motors (an Xbox One or Series pad, a
//! DualSense): the right trigger, the accelerator, shakes under wheelspin and, faintly, with
//! the engine's load; the left trigger, the brake, shakes when the braking wheels lock. Both are
//! still while the pedal is not pressed, while the car is off the ground, and at the pad's
//! strength (the controller page's force feedback strength) scaled down like the motors.
//!
//! The slip is the driven wheels' speed less the car's forward speed. Its units are the
//! game's speed units; the dead zone and the range are chosen by feel, not measured.

/// A pedal at its full travel.
pub const PEDAL_FULL: u32 = 0x100;

/// A slip under this (the game's own wheel margin) is not felt.
pub const SLIP_DEAD: f32 = 1.5;
/// How far past the dead zone the wheelspin must go for the accelerator trigger at its full
/// shake: a normal wheelspin start (a slip of about 5) is felt at close to nine tenths of it.
pub const SLIP_RANGE: f32 = 4.0;
/// The same for the brake trigger and locked wheels, which never slip as far as spinning ones
/// do (a hard stop locks them at a slip of about 3): a hard stop is felt at two thirds or more.
pub const LOCK_RANGE: f32 = 1.9;
/// A car slower than this (forward, either way) does not lock its wheels: a stopped car
/// braking has nothing to feel.
pub const LOCK_MIN_SPEED: f32 = 3.0;
/// The engine's load at full throttle and full revs, as a share of the accelerator trigger's
/// range.
pub const ENGINE_SHARE: f32 = 0.15;

/// What the car is doing, as far as the triggers care.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Feel {
    /// The accelerator and brake pedals, 0 ..= `PEDAL_FULL`.
    pub throttle: u32,
    pub brake: u32,
    /// The driven wheels' speed less the car's forward speed.
    pub slip: f32,
    /// The car's forward speed (negative in reverse).
    pub forward: f32,
    /// The engine's revs as a share of its top revs.
    pub revs: f32,
    /// The driven wheels, or any wheel, are off the ground.
    pub airborne: bool,
    /// The engine's load is felt in the accelerator trigger (the player's engine vibration
    /// switch); off, the triggers carry only what the road and the tyres do.
    pub engine_load: bool,
}

/// The triggers' levels, 0 ..= 65535 as SDL takes them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Levels {
    pub brake: u16,
    pub throttle: u16,
}

impl Levels {
    pub const STILL: Levels = Levels {
        brake: 0,
        throttle: 0,
    };
}

/// 0 .. 1 for a slip: nothing within the dead zone, then rising to its full over `range`.
fn shake(slip: f32, range: f32) -> f32 {
    ((slip - SLIP_DEAD) / range).clamp(0.0, 1.0)
}

fn pedal(value: u32) -> f32 {
    value.min(PEDAL_FULL) as f32 / PEDAL_FULL as f32
}

fn level(share: f32, gain: u32) -> u16 {
    let gain = gain.min(10_000) as f32 / 10_000.0;
    (share.clamp(0.0, 1.0) * gain * 65535.0).round() as u16
}

/// The levels for what the car does, at the pad's strength `gain` (0 .. 10000).
pub fn levels(feel: &Feel, gain: u32) -> Levels {
    if feel.airborne || !feel.slip.is_finite() || !feel.forward.is_finite() {
        return Levels::STILL;
    }
    // Wheelspin is the driven wheels faster than the car along its way of travel; a lock the
    // other way. In reverse the signs turn round.
    let along = if feel.forward < 0.0 { -1.0 } else { 1.0 };
    let slip = feel.slip * along;
    let load = if feel.engine_load {
        ENGINE_SHARE * feel.revs.clamp(0.0, 1.0).powi(2)
    } else {
        0.0
    };
    let throttle = pedal(feel.throttle) * (shake(slip, SLIP_RANGE) + load).min(1.0);
    let brake = if feel.forward.abs() >= LOCK_MIN_SPEED {
        pedal(feel.brake) * shake(-slip, LOCK_RANGE)
    } else {
        0.0
    };
    Levels {
        brake: level(brake, gain),
        throttle: level(throttle, gain),
    }
}

/// How long the pad keeps a level unless it is sent again, and how often a running one is sent
/// again (ms).
pub const KEEP_MS: u32 = 1000;
pub const AGAIN_MS: u64 = 400;
/// How long after the last setting the triggers go still (ms): the race sets them every 40 ms,
/// so a game that has stopped (a pause, a question, the end of the race) has them still.
pub const STALE_MS: u64 = 250;

/// What goes to the pad and when: a level is sent when it changes and again before the pad
/// would end it, and the triggers are stilled when nothing sets them any more.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sender {
    want: Levels,
    set_at: Option<u64>,
    sent: Levels,
    sent_at: Option<u64>,
}

impl Sender {
    /// The levels the game wants now (`now` in ms).
    pub fn set(&mut self, levels: Levels, now: u64) {
        self.want = levels;
        self.set_at = Some(now);
    }

    /// What to send to the pad at `now`, if anything.
    pub fn poll(&mut self, now: u64) -> Option<Levels> {
        let fresh = self
            .set_at
            .is_some_and(|at| now.saturating_sub(at) < STALE_MS);
        let levels = if fresh { self.want } else { Levels::STILL };
        let due = levels != Levels::STILL
            && self
                .sent_at
                .is_none_or(|at| now.saturating_sub(at) >= AGAIN_MS);
        if levels != self.sent || due {
            self.sent = levels;
            self.sent_at = Some(now);
            Some(levels)
        } else {
            None
        }
    }

    /// Everything stopped (the pad given back or gone): the stilling to send, if the triggers
    /// were not still.
    pub fn stop(&mut self) -> Option<Levels> {
        let was = self.sent;
        *self = Self::default();
        (was != Levels::STILL).then_some(Levels::STILL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feel() -> Feel {
        Feel {
            throttle: PEDAL_FULL,
            brake: 0,
            slip: 0.0,
            forward: 20.0,
            revs: 0.0,
            airborne: false,
            engine_load: true,
        }
    }

    #[test]
    fn a_gripping_car_is_still_but_for_the_engine() {
        assert_eq!(levels(&feel(), 10_000), Levels::STILL);
        let mut loaded = feel();
        loaded.revs = 1.0;
        let l = levels(&loaded, 10_000);
        assert_eq!(l.brake, 0);
        assert_eq!(l.throttle, (0.15f32 * 65535.0).round() as u16);
    }

    #[test]
    fn without_the_engine_a_gripping_car_stays_still_at_any_revs() {
        let mut f = feel();
        f.revs = 1.0;
        f.engine_load = false;
        assert_eq!(levels(&f, 10_000), Levels::STILL);
        // The road's part is as it was: wheelspin shakes the accelerator, a lock the brake.
        f.slip = SLIP_DEAD + SLIP_RANGE;
        assert_eq!(levels(&f, 10_000).throttle, 65535);
        f.slip = 4.0;
        let spin = levels(&f, 10_000).throttle;
        f.engine_load = true;
        assert!(levels(&f, 10_000).throttle > spin);
        f.engine_load = false;
        f.throttle = 0;
        f.brake = PEDAL_FULL;
        f.slip = -(SLIP_DEAD + LOCK_RANGE);
        assert_eq!(levels(&f, 10_000).brake, 65535);
    }

    #[test]
    fn wheelspin_shakes_the_accelerator_by_the_pedal() {
        let mut f = feel();
        f.slip = SLIP_DEAD + SLIP_RANGE;
        assert_eq!(levels(&f, 10_000).throttle, 65535);
        f.throttle = PEDAL_FULL / 2;
        let half = levels(&f, 10_000).throttle;
        assert!((32766..=32768).contains(&half));
        f.throttle = 0;
        assert_eq!(levels(&f, 10_000).throttle, 0);
        // Within the dead zone nothing.
        f.throttle = PEDAL_FULL;
        f.slip = SLIP_DEAD;
        assert_eq!(levels(&f, 10_000).throttle, 0);
    }

    #[test]
    fn locked_wheels_shake_the_brake_only_when_moving() {
        let mut f = feel();
        f.throttle = 0;
        f.brake = PEDAL_FULL;
        f.slip = -(SLIP_DEAD + LOCK_RANGE);
        let l = levels(&f, 10_000);
        assert_eq!((l.brake, l.throttle), (65535, 0));
        f.forward = 1.0;
        assert_eq!(levels(&f, 10_000), Levels::STILL);
        // Wheelspin does not shake the brake.
        f.forward = 20.0;
        f.slip = SLIP_DEAD + SLIP_RANGE;
        assert_eq!(levels(&f, 10_000).brake, 0);
    }

    #[test]
    fn a_normal_start_and_a_hard_stop_are_clearly_felt() {
        let mut f = feel();
        f.revs = 0.4;
        f.forward = 1.0;
        f.slip = 5.0;
        let start = levels(&f, 10_000).throttle as f32 / 65535.0;
        assert!((0.7..=0.9).contains(&start), "{start}");
        let mut f = feel();
        f.throttle = 0;
        f.brake = PEDAL_FULL;
        f.slip = -2.8;
        let stop = levels(&f, 10_000).brake as f32 / 65535.0;
        assert!((0.6..=0.8).contains(&stop), "{stop}");
        // The engine's load alone stays a light background.
        let mut f = feel();
        f.revs = 1.0;
        f.slip = 1.8;
        let load = levels(&f, 10_000).throttle as f32 / 65535.0;
        assert!(load <= 0.25, "{load}");
    }

    #[test]
    fn reversing_turns_the_signs_round() {
        let mut f = feel();
        f.forward = -20.0;
        f.slip = -(SLIP_DEAD + SLIP_RANGE);
        assert_eq!(levels(&f, 10_000).throttle, 65535);
        f.throttle = 0;
        f.brake = PEDAL_FULL;
        f.slip = SLIP_DEAD + LOCK_RANGE;
        assert_eq!(levels(&f, 10_000).brake, 65535);
    }

    #[test]
    fn off_the_ground_and_bad_numbers_are_still() {
        let mut f = feel();
        f.slip = 100.0;
        f.airborne = true;
        assert_eq!(levels(&f, 10_000), Levels::STILL);
        f.airborne = false;
        f.slip = f32::NAN;
        assert_eq!(levels(&f, 10_000), Levels::STILL);
    }

    #[test]
    fn the_pads_strength_scales_both() {
        let mut f = feel();
        f.slip = 100.0;
        assert_eq!(levels(&f, 5000).throttle, 32768);
        assert_eq!(levels(&f, 0).throttle, 0);
        assert_eq!(levels(&f, 99_999).throttle, 65535);
    }

    #[test]
    fn the_sender_sends_changes_repeats_and_stills() {
        let mut s = Sender::default();
        assert_eq!(s.poll(0), None);
        let on = Levels {
            brake: 100,
            throttle: 0,
        };
        s.set(on, 1000);
        assert_eq!(s.poll(1000), Some(on));
        assert_eq!(s.poll(1040), None);
        s.set(on, 1040);
        assert_eq!(s.poll(1280), None);
        // Again before the pad's end of it.
        s.set(on, 1390);
        assert_eq!(s.poll(1400), Some(on));
        // Nothing sets it any more: stilled once.
        assert_eq!(s.poll(1400 + STALE_MS), Some(Levels::STILL));
        assert_eq!(s.poll(2000), None);
    }

    #[test]
    fn stopping_stills_what_was_sent() {
        let mut s = Sender::default();
        assert_eq!(s.stop(), None);
        let on = Levels {
            brake: 0,
            throttle: 7,
        };
        s.set(on, 0);
        assert_eq!(s.poll(0), Some(on));
        assert_eq!(s.stop(), Some(Levels::STILL));
        assert_eq!(s.poll(1), None);
    }
}
