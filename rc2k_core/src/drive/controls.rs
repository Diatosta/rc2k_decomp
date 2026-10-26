//! The work car's controls from its input words (converted 2026-10-21 from
//! `drive::read_car_controls` 0x43A35A, `drive::pedal_inputs` 0x43A004,
//! `drive::reset_request` 0x43A0D3 and `drive::device_axes` 0x409430): the keys newly
//! pressed and the switches they give, the pedals stepped by keys or set by a device, the
//! reset key held long enough, and a device's first reading's curve.

/// What the keys word gives (`read_car_controls`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Keys {
    /// The bits not held before (`dword_71A44C`).
    pub pressed: u32,
    /// The handbrake's input (`dword_61389C`): bit 2 held times 0x60.
    pub hand: u32,
    /// Bit 0 newly pressed (`dword_6138A0`; a reading: gear up).
    pub up: u32,
    /// Bit 1 newly pressed (`dword_6138A4`; a reading: gear down).
    pub down: u32,
}

/// The keys word read against the one held before.
pub fn read_keys(held_before: u32, keys: u32) -> Keys {
    let pressed = !held_before & keys;
    Keys {
        pressed,
        hand: (keys & 4).wrapping_mul(0x60),
        up: ((pressed & 1) * 0x100) >> 8,
        down: ((pressed & 2) * 0x80) >> 8,
    }
}

/// The bits a player's newly pressed keys act on: bit 3 flips `dword_71A05C` (a
/// reading: the lights), bit 5 changes the camera's view.
pub const FLIP_BIT: u32 = 8;
pub const VIEW_BIT: u32 = 0x20;

/// The flipped switch.
pub fn flipped(v: u32) -> u32 {
    v.wrapping_add(1) & 1
}

/// Whether the computer drives the car: its switch (`dword_71C3E0`), or race mode 2 with
/// the grades' mode 2 (each asked only when the one before does not decide).
pub fn computer_drives(
    switch: u32,
    mode: impl FnOnce() -> u32,
    grades: impl FnOnce() -> u32,
) -> bool {
    switch != 0 || (mode() == 2 && grades() == 2)
}

/// A pedal's top and step.
pub const PEDAL_TOP: u32 = 0x100;
pub const PEDAL_STEP: u32 = 0x10;

/// `pedal_inputs`' pedal: by keys stepped toward the top (input set) or 0, held there by
/// the sign of the steps; else the input itself.
pub fn pedal(by_keys: bool, input: u32, pedal: u32) -> u32 {
    if !by_keys {
        return input;
    }
    if input != 0 {
        let p = pedal.wrapping_add(PEDAL_STEP);
        if (p.wrapping_sub(PEDAL_TOP) as i32) >= 0 {
            PEDAL_TOP
        } else {
            p
        }
    } else {
        let p = pedal.wrapping_sub(PEDAL_STEP);
        if (p as i32) < 0 { 0 } else { p }
    }
}

/// The keys' bits that step the throttle and the brake.
pub const THROTTLE_KEY: u32 = 0x4000;
pub const BRAKE_KEY: u32 = 0x2000;

/// `pedal_inputs`' handbrake: the input, but 0 when that is `PEDAL_TOP` or less and the
/// switch (`unk_71A400`, asked only then) is set.
pub fn handbrake(input: u32, switch: impl FnOnce() -> u32) -> u32 {
    if input <= PEDAL_TOP && switch() != 0 {
        0
    } else {
        input
    }
}

/// When the reset key counts: the start timer past 0x320 (by the sign).
pub const RESET_FROM: u32 = 0x320;
/// The reset key's bit, the steps it is held for and the notice shown meanwhile.
pub const RESET_KEY: u32 = 0x80;
pub const RESET_STEPS: u32 = 100;
pub const RESET_NOTICE: u32 = 0xCD;
/// The time penalty, less the stuck count (+0x1E8).
pub const RESET_PENALTY: u32 = 0x2BC;

/// Whether the start timer is past `RESET_FROM`.
pub fn reset_counts(timer: u32) -> bool {
    (timer.wrapping_sub(RESET_FROM) as i32) >= 0
}

/// Whether the reset key is held: its bit, outside race mode 2 and with the car slow
/// (`slow`, asked last).
pub fn reset_held(keys: u32, mode: u32, slow: impl FnOnce() -> bool) -> bool {
    keys & RESET_KEY != 0 && mode != 2 && slow()
}

/// One step of the reset key held: (the notice is shown now, the count after, the car is
/// reset now).
pub fn reset_step(count: u32) -> (bool, u32, bool) {
    let next = count.wrapping_add(1);
    (count == 0, next, next == RESET_STEPS)
}

/// The penalty added to the car's time at a reset.
pub fn reset_penalty(stuck: u32) -> u32 {
    RESET_PENALTY.wrapping_sub(stuck)
}

/// `device_axes`' curve on the first reading: squared / 256 (an arithmetic shift of the
/// low dword) with its sign kept.
pub fn device_curve(x: u32) -> u32 {
    let sign = ((x as i32 >> 31) * 2 + 1) as u32;
    (((x.wrapping_mul(x) as i32) >> 8) as u32).wrapping_mul(sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys() {
        let k = read_keys(0b0101, 0b0111);
        assert_eq!(
            k,
            Keys {
                pressed: 2,
                hand: 0x180,
                up: 0,
                down: 1
            }
        );
        assert_eq!(flipped(1), 0);
        assert_eq!(flipped(0), 1);
        assert!(computer_drives(1, || unreachable!(), || unreachable!()));
        assert!(computer_drives(0, || 2, || 2));
        assert!(!computer_drives(0, || 1, || unreachable!()));
    }

    #[test]
    fn pedals() {
        assert_eq!(pedal(false, 77, 5), 77);
        assert_eq!(pedal(true, 1, 0xF0), 0x100);
        assert_eq!(pedal(true, 1, 0xE0), 0xF0);
        assert_eq!(pedal(true, 0, 0x8), 0);
        assert_eq!(pedal(true, 0, 0x18), 0x8);
        assert_eq!(handbrake(0x100, || 1), 0);
        assert_eq!(handbrake(0x101, || unreachable!()), 0x101);
        assert_eq!(handbrake(5, || 0), 5);
    }

    #[test]
    fn reset() {
        assert!(!reset_counts(0x31F));
        assert!(reset_counts(0x320));
        assert!(reset_held(0x80, 1, || true));
        assert!(!reset_held(0x80, 2, || unreachable!()));
        assert_eq!(reset_step(0), (true, 1, false));
        assert_eq!(reset_step(99), (false, 100, true));
        assert_eq!(reset_penalty(0x2C), 0x290);
    }

    #[test]
    fn curve() {
        assert_eq!(device_curve(0x100), 0x100);
        assert_eq!(device_curve((-0x100i32) as u32), (-0x100i32) as u32);
        assert_eq!(device_curve(0x10), 1);
    }
}
