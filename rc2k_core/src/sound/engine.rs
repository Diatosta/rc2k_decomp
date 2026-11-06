//! A car's sound object brought up to date for the turn, its whole-number part (converted
//! 2026-10-21 from `sound::car_engine_state` 0x40D8E2; its floats, the pitch and volume
//! inputs, the distance and the gear cue's pitch, stay the hook's x87 for now), and the
//! engine sounds' choices (the same day from `sound::revs_sound` 0x40DD19,
//! `sound::throttle_sound` 0x40DD96, `sound::speed_sound` 0x40DE09 and
//! `sound::car_slot_sounds` 0x40E0F6).
//!
//! The object's dwords the engine reads and writes (a reading of the code, names by use) are
//! an `Engine`, taken from the sound object and put back by the game crate: `new_revs` a new
//! revs reading (`revs`) came, `reading_times` / `reading_revs` the last two readings';
//! `throttle`, `smooth_throttle` following it, `gap_time` the gap, `direction` its direction
//! (0 level, 1 up, 2 down), `turns_up` / `turns_down` the turns going each way; `state` the
//! engine's state (0 no revs, 1 up to 1000, 2, 3 cut), `last_state` the last; `gear` and
//! `last_gear`, `no_gear` 1 in neutral; `countdown` a hold-off counted down, `wait_flag` and
//! `last_direction`; the back-fires `backfire_*`; `drop_sound` / `drop_start` the timed slot of
//! the throttle's drop, `gear_slot_start` / `gear_notice` the gear change's.

use crate::Invalid;

/// A sound object's dwords the engine works on, by name: what goes in and what comes out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Engine {
    /// +0x10: a new revs reading came.
    pub new_revs: u32,
    /// +0x14: the revs.
    pub revs: u32,
    /// +0x18: the throttle.
    pub throttle: u32,
    /// +0x20: the physics' gear-changed flag.
    pub gear_event: u32,
    /// +0x24: 1 in neutral.
    pub no_gear: u32,
    /// +0x94 / +0x98: the times of the last two readings.
    pub reading_times: [u32; 2],
    /// +0x9C / +0xA0: their revs.
    pub reading_revs: [u32; 2],
    /// +0xE8: the gear.
    pub gear: u32,
    /// +0xEC: the gear when it last changed.
    pub last_gear: u32,
    /// +0xF0: set with a gear change.
    pub gear_notice: u32,
    /// +0xF4: counted down to 0.
    pub countdown: u32,
    /// +0xF8: the back-fire wait flag.
    pub wait_flag: u32,
    /// +0xFC: the last direction.
    pub last_direction: u32,
    /// +0x10C: the engine's state.
    pub state: u32,
    /// +0x110: the last state.
    pub last_state: u32,
    /// +0x118: follows the throttle.
    pub smooth_throttle: u32,
    /// +0x11C: the gap.
    pub gap_time: u32,
    /// +0x128: the turns going up.
    pub turns_up: u32,
    /// +0x12C: the direction.
    pub direction: u32,
    /// +0x130: the turns going down.
    pub turns_down: u32,
    /// +0x1C4: the sound of the timed slot the throttle's drop arms.
    pub drop_sound: u32,
    /// +0x1C8: that slot's start flag.
    pub drop_start: u32,
    /// +0x1E0: the back-fire slot's start flag.
    pub backfire_start: u32,
    /// +0x1E4: that slot's playing flag.
    pub backfire_playing: u32,
    /// +0x1EC: that slot's volume.
    pub backfire_volume: u32,
    /// +0x1F4: the back-fires' wait, counted down.
    pub backfire_wait: u32,
    /// +0x1FC: the gear change slot's start flag.
    pub gear_slot_start: u32,
}

/// The registers the original leaves that callers may see: eax, ecx, edx.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Left {
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
}

/// The request's place for the timed sounds (converted 2026-10-21 from
/// `sound::car_gear_sounds` 0x40E24F): the object's +0x2C .. +0x34 less its +0x38 ..
/// +0x40, each `fsub` stored as a float. One rounding of an exact `f64` difference: the
/// x87's at any precision.
pub fn place_behind(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|n| (f64::from(a[n]) - f64::from(b[n])) as f32)
}

/// The gear change's pitch factor (the same function's): the request's factor (+0x20)
/// times the object's +0x210, stored as a float (the product exact in `f64`).
pub fn gear_pitch(factor: f32, pitch: f32) -> f32 {
    (f64::from(factor) * f64::from(pitch)) as f32
}

/// Kind bits: the gear change's cue, the back-fires.
pub const GEAR_CUE: u32 = 0x2000;
pub const BACK_FIRES: u32 = 0x1000;

/// The revs extrapolated, worked out before anything is written: (the readings' gap held
/// to 1000, the quotient, the remainder), None without one (a new reading, the revs held,
/// or a gap of 0). `hold` (`dword_5BEC94`) asked only without a new reading. Refuses a
/// quotient that does not fit (the original's `idiv` traps).
pub fn extrapolation(
    o: &Engine,
    clock: u32,
    hold: impl FnOnce() -> u32,
) -> Result<Option<(u32, u32, u32)>, Invalid> {
    if o.new_revs == 1 || hold() == 1 {
        return Ok(None);
    }
    let gap = o.reading_times[1].wrapping_sub(o.reading_times[0]);
    if gap == 0 {
        return Ok(None);
    }
    let gap = if gap as i32 > 1000 { 1000 } else { gap };
    let since = clock.wrapping_sub(o.reading_times[1]) as i32 as i64;
    let rise = o.reading_revs[1].wrapping_sub(o.reading_revs[0]) as i32 as i64;
    let product = since * rise;
    let quotient = product / gap as i32 as i64;
    if quotient != quotient as i32 as i64 {
        return Err(Invalid(
            "a revs quotient that does not fit: the original's idiv traps",
        ));
    }
    Ok(Some((
        gap,
        quotient as i32 as u32,
        (product % gap as i32 as i64) as u32,
    )))
}

/// The readings: a new one kept (its time `clock`), else the revs extrapolated (at least
/// 1000). `revs` is `extrapolation`'s; `hold` asked as there.
pub fn readings(
    o: &mut Engine,
    clock: u32,
    revs: Option<(u32, u32, u32)>,
    hold: impl FnOnce() -> u32,
    left: &mut Left,
) {
    if o.new_revs == 1 {
        o.reading_times[0] = o.reading_times[1];
        o.reading_times[1] = clock;
        o.reading_revs[0] = o.reading_revs[1];
        left.eax = o.revs;
        o.reading_revs[1] = left.eax;
        o.new_revs = 0;
    } else if let Some((gap, quotient, remainder)) = revs {
        left.ecx = gap;
        left.edx = remainder;
        left.eax = quotient.wrapping_add(o.reading_revs[1]);
        if (left.eax as i32) < 1000 {
            left.eax = 1000;
        }
        o.revs = left.eax;
    } else if hold() != 1 {
        // The readings' gap was 0.
        left.ecx = 0;
    }
}

/// The state, the throttle followed, the gear and the back-fires (after the pitch
/// inputs). `kind` reads the sound kind's flags (+0) where the original does. Returns
/// the gear change's cue: Some(clock & 0xF) when its pitch (+0x210) is to be set.
pub fn state(o: &mut Engine, kind: impl Fn() -> u32, clock: u32, left: &mut Left) -> Option<u32> {
    o.last_state = o.state;
    o.state = match o.revs {
        0 => 0,
        1..=1000 => 1,
        _ => 2,
    };
    if o.gear_event != 0 {
        o.gear_event = 0;
        o.state = 3;
    }
    left.ecx = o.throttle;
    let followed = o.smooth_throttle;
    left.eax = left.ecx.wrapping_sub(followed);
    o.gap_time = left.eax;
    if left.eax == 0 {
        o.direction = 0;
        if o.turns_up == 1 {
            o.turns_up = 2;
        }
        if o.turns_down == 1 {
            o.turns_down = 2;
        }
    } else if (left.ecx as i32) < (followed as i32) {
        o.turns_down = o.turns_down.wrapping_add(1);
        o.turns_up = 0;
        o.direction = 2;
        left.eax = o.smooth_throttle >> 7;
        o.smooth_throttle = o.smooth_throttle.wrapping_sub(left.eax).wrapping_sub(1);
        if (o.smooth_throttle as i32) < (left.ecx as i32) {
            o.smooth_throttle = left.ecx;
        }
    } else {
        o.turns_up = o.turns_up.wrapping_add(1);
        o.turns_down = 0;
        o.direction = 1;
        left.eax = 0x100u32.wrapping_sub(o.smooth_throttle) >> 6;
        o.smooth_throttle = o.smooth_throttle.wrapping_add(left.eax).wrapping_add(1);
        if (o.smooth_throttle as i32) > (left.ecx as i32) {
            o.smooth_throttle = left.ecx;
        }
    }
    let last = o.last_direction;
    o.last_direction = o.direction;
    if last.wrapping_add(o.direction) == 3 {
        o.countdown = o.countdown.wrapping_add(5);
    }
    let mut cue = None;
    left.eax = o.gear;
    if left.eax != o.last_gear {
        o.last_gear = left.eax;
        o.countdown = o.countdown.wrapping_add(7);
        if kind() & GEAR_CUE != 0 && left.eax as i32 > 1 && o.gap_time as i32 >= 0x32 {
            o.gear_slot_start = 1;
            o.gear_notice = 1;
            left.eax = clock & 0xF;
            cue = Some(left.eax);
        }
    }
    left.eax = (left.eax == 0) as u32;
    o.no_gear = left.eax;
    if kind() & BACK_FIRES != 0 {
        if o.direction == 1 {
            o.backfire_start = 0;
            o.backfire_playing = 0;
            o.backfire_wait = 0;
            o.backfire_volume = 0;
        } else if o.smooth_throttle <= 0xF0 {
            let wait = o.backfire_wait.wrapping_sub(1);
            o.backfire_wait = wait;
            if (wait as i32) < 0 {
                let over = o.revs.wrapping_sub(0x7D0);
                left.ecx = (if (over as i32) < 0 { 0 } else { over }) >> 9;
                left.eax = (clock & 3).wrapping_add(left.ecx).wrapping_add(1);
                o.backfire_wait = left.eax;
                o.backfire_start = 1;
                left.eax = o.backfire_volume.wrapping_add(8);
                if left.eax as i32 > 0x20 {
                    left.eax = 0x20;
                }
                o.backfire_volume = left.eax;
            }
        }
        if o.direction == 1 {
            o.wait_flag = 0;
        } else if o.wait_flag != 1 && o.direction == 2 && (o.gap_time as i32) < -0x14 {
            if o.revs >= 0xFA0 {
                left.eax = kind() & 0xFFF;
                o.drop_sound = left.eax;
                o.drop_start = 1;
            }
            o.wait_flag = 1;
        }
    }
    if o.countdown != 0 {
        o.countdown = o.countdown.wrapping_sub(1);
    }
    cue
}

/// `+0x104`: the throttle as a float over 256: `from_int`'s bits with the exponent less 8
/// (none for 0).
pub fn over_256(float_bits: u32) -> u32 {
    if float_bits & 0x7FFF_FFFF != 0 {
        float_bits.wrapping_sub(0x400_0000)
    } else {
        float_bits
    }
}

/// `sound::revs_sound`: the sound of a car whose object's +0 is `object`, from the base
/// sound of its set: a computer's car (+0 negative) has two (bit 0 of +0), a player's
/// one each (+0 is the car's).
pub fn revs_sound_number(object: u32, base: u32) -> u32 {
    let n = if (object as i32) < 0 {
        object & 1
    } else {
        object
    };
    n.wrapping_add(base)
}

/// A computer's car (its object's +0 negative): no throttle or speed sound.
pub fn computers(object: u32) -> bool {
    (object as i32) < 0
}

/// `sound::revs_sound` and `sound::throttle_sound`: the sound is stopped in engine state
/// 0 (no revs) or 3 (cut).
pub fn revs_stopped(state: u32) -> bool {
    state == 0 || state == 3
}

/// `sound::speed_sound`: the sound is stopped in engine state 0 or 1, in neutral (+0x24
/// 1), while held off (+0xF4) or off the ground (+0xAC); each asked in that order, only
/// while the ones before did not stop it.
pub fn speed_stopped(
    state: u32,
    neutral: impl FnOnce() -> u32,
    held_off: impl FnOnce() -> u32,
    airborne: impl FnOnce() -> u32,
) -> bool {
    state == 0 || state == 1 || neutral() == 1 || held_off() != 0 || airborne() != 0
}

/// `sound::car_slot_sounds`: the object's timed slots played at the car's place, in
/// order (the first four only for a car whose +0xC is 0); then the one at +0x220 at the
/// place +0x214.
pub fn placed_slots(own_view: bool) -> Vec<u32> {
    let mut slots = Vec::new();
    if own_view {
        slots.extend([0x164, 0x14C, 0x17C, 0x194]);
    }
    slots.extend([0x260, 0x1AC]);
    slots
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revs() {
        let mut o = Engine {
            reading_times: [100, 120],
            reading_revs: [2000, 2500],
            ..Engine::default()
        };
        let r = extrapolation(&o, 140, || 0).unwrap();
        assert_eq!(r, Some((20, 500, 0)));
        let mut left = Left::default();
        readings(&mut o, 140, r, || 0, &mut left);
        assert_eq!((o.revs, left.ecx), (3000, 20));
        assert_eq!(extrapolation(&o, 140, || 1).unwrap(), None);
        o.reading_revs[0] = 0;
        o.reading_revs[1] = 0x7FFF_FFFF;
        assert!(extrapolation(&o, 0x8000_0005, || 0).is_err());
        o.new_revs = 1;
        o.revs = 4321;
        readings(&mut o, 7, None, || unreachable!(), &mut left);
        assert_eq!(
            (o.reading_times[1], o.reading_revs[1], o.new_revs, left.eax),
            (7, 4321, 0, 4321)
        );
    }

    #[test]
    fn states() {
        let mut o = Engine {
            revs: 4500,
            throttle: 0x10,
            smooth_throttle: 0x80,
            last_direction: 1,
            gear: 3,
            backfire_wait: 0,
            ..Engine::default()
        };
        let mut left = Left::default();
        let cue = state(&mut o, || BACK_FIRES | GEAR_CUE | 0x123, 0x37, &mut left);
        // Going down: 0x80 - (0x80 >> 7) - 1, still above the throttle.
        assert_eq!(o.smooth_throttle, 0x7E);
        assert_eq!((o.direction, o.state), (2, 2));
        // Up then down: +5, then the gear's +7, less one.
        assert_eq!(o.countdown, 11);
        // The gap -0x70 is not at least 0x32: no cue.
        assert_eq!(cue, None);
        // A back-fire: (0x37 & 3) + ((4500 - 2000) >> 9) + 1 = 3 + 4 + 1.
        assert_eq!(
            (o.backfire_wait, o.backfire_start, o.backfire_volume),
            (8, 1, 8)
        );
        assert_eq!((o.drop_sound, o.drop_start, o.wait_flag), (0x123, 1, 1));
        let mut o = Engine {
            throttle: 0x80,
            gear: 2,
            ..Engine::default()
        };
        assert_eq!(state(&mut o, || GEAR_CUE, 0x2B, &mut left), Some(0xB));
        assert_eq!((o.gear_slot_start, o.gear_notice, o.no_gear), (1, 1, 0));
        assert_eq!(over_256(0x4380_0000), 0x3F80_0000);
        assert_eq!(over_256(0x8000_0000), 0x8000_0000);
    }

    #[test]
    fn engine_sounds() {
        assert_eq!(revs_sound_number(0x8000_0003, 10), 11);
        assert_eq!(revs_sound_number(5, 10), 15);
        assert!(computers(0xFFFF_FFFF) && !computers(0));
        assert!(revs_stopped(3) && !revs_stopped(1));
        assert!(speed_stopped(1, || unreachable!(), || 0, || 0));
        assert!(!speed_stopped(2, || 0, || 0, || 0));
        assert!(speed_stopped(2, || 0, || 0, || 1));
        assert_eq!(placed_slots(false), [0x260, 0x1AC]);
        assert_eq!(placed_slots(true).len(), 6);
    }

    #[test]
    fn gear_floats() {
        assert_eq!(
            place_behind([3.0, 1.5, -2.0], [1.0, 2.0, 0.25]),
            [2.0, -0.5, -2.25]
        );
        assert_eq!(gear_pitch(0.75, 0.875), 0.65625);
        assert_eq!(
            place_behind([1.0e30, 1.0, 0.0], [-1.0e-30, 0.0, 0.0])[0],
            1.0e30
        );
    }
}
