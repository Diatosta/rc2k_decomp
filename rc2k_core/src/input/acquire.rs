//! A device record's controller acquired: which, its settings, and the properties'
//! figures (from `input::acquire_device` 0x408473 and
//! `input::begin_force` 0x40945D's gain).
//!
//! The record's settings: the kind (2 when 0, 3 for a controller of more than three axes),
//! then each of the configuration's values that is set (the kind 1 .. 3, the axes' code and
//! the axis pick not negative, the strength and the two zones up to 100). Its axes' ranges
//! and dead zones go by the axis pick's bytes; the saturation leaves the dead and end
//! zones out.

/// What becomes of a record asked to be acquired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acquire {
    /// A controller not listed, or not attached: nothing.
    Nothing,
    /// The record still has a device: fatal error 0x8023831D.
    HasDevice,
    /// Set up and acquired.
    Acquire,
}

/// The fatal error of a record that still has a device.
pub const HAS_DEVICE: u32 = 0x8023_831D;

/// `controller` of `count` listed; `attached` (its entry's +0x18) asked only for a listed
/// one, `device` (the record's +4) only for an attached one.
pub fn acquire(
    controller: u32,
    count: u32,
    attached: impl FnOnce() -> u32,
    device: impl FnOnce() -> u32,
) -> Acquire {
    if controller >= count || attached() != 1 {
        Acquire::Nothing
    } else if device() != 0 {
        Acquire::HasDevice
    } else {
        Acquire::Acquire
    }
}

/// The configuration's values (`dword_4F68E0` .. `dword_4F68F8`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub kind: u32,
    pub axis_code: u32,
    pub dead_zone: u32,
    pub end_zone: u32,
    pub axis_pick: u32,
    pub strength: u32,
}

/// A record's settings (its dwords +8, +0x18, +0xC, +0x10, +0x14, +0x1C).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    pub kind: u32,
    pub axis_code: u32,
    pub strength: u32,
    pub dead_zone: u32,
    pub end_zone: u32,
    pub axis_pick: u32,
}

/// The settings a record leaves with; `axes` (the controller's, asked only for a kind of
/// 0).
pub fn settings(mut s: Settings, axes: impl FnOnce() -> u32, c: &Config) -> Settings {
    if s.kind == 0 {
        s.kind = if axes() > 3 { 3 } else { 2 };
    }
    if (1..=3).contains(&(c.kind as i32)) {
        s.kind = c.kind;
    }
    if (c.axis_code as i32) >= 0 {
        s.axis_code = c.axis_code;
    }
    if c.strength <= 100 {
        s.strength = c.strength;
    }
    if c.dead_zone <= 100 {
        s.dead_zone = c.dead_zone;
    }
    if c.end_zone <= 100 {
        s.end_zone = c.end_zone;
    }
    if (c.axis_pick as i32) >= 0 {
        s.axis_pick = c.axis_pick;
    }
    s
}

/// An axis's offset in the joystick's state from one of the axis pick's bytes.
pub fn axis_offset(pick: u8) -> u32 {
    (pick as u32 & 7) * 4
}

/// Whether a record of this kind has two axes set (kinds up to 2, unsigned).
pub fn two_axes(kind: u32) -> bool {
    kind <= 2
}

/// The dead zone's property: the zone x 100 (wrapping).
pub fn dead_zone_value(dead_zone: u32) -> u32 {
    dead_zone.wrapping_mul(100)
}

/// The saturation: (100 - dead - end) x 10000 / (100 - dead) when both are above 0
/// (compared signed, divided unsigned); None: not set.
pub fn saturation(dead_zone: u32, end_zone: u32) -> Option<u32> {
    let rest = 100u32.wrapping_sub(dead_zone);
    if (rest as i32) <= 0 {
        return None;
    }
    let live = rest.wrapping_sub(end_zone);
    ((live as i32) > 0).then(|| live.wrapping_mul(10000) / rest)
}

/// `input::begin_force`: the gain for a strength (x 100, wrapping); None (a strength of
/// 0): the self-centring alone.
pub fn gain(strength: u32) -> Option<u32> {
    (strength != 0).then(|| strength.wrapping_mul(100))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn which() {
        assert_eq!(
            acquire(2, 2, || unreachable!(), || unreachable!()),
            Acquire::Nothing
        );
        assert_eq!(acquire(0, 2, || 0, || unreachable!()), Acquire::Nothing);
        assert_eq!(acquire(0, 2, || 1, || 5), Acquire::HasDevice);
        assert_eq!(acquire(1, 2, || 1, || 0), Acquire::Acquire);
    }

    #[test]
    fn configured() {
        let s = Settings {
            strength: 50,
            ..Settings::default()
        };
        let none = Config {
            kind: 0,
            axis_code: u32::MAX,
            dead_zone: 200,
            end_zone: 101,
            axis_pick: u32::MAX,
            strength: 101,
        };
        assert_eq!(settings(s, || 4, &none), Settings { kind: 3, ..s });
        assert_eq!(settings(s, || 3, &none).kind, 2);
        let all = Config {
            kind: 1,
            axis_code: 0x102,
            dead_zone: 10,
            end_zone: 5,
            axis_pick: 0,
            strength: 80,
        };
        assert_eq!(
            settings(Settings { kind: 3, ..s }, || unreachable!(), &all),
            Settings {
                kind: 1,
                axis_code: 0x102,
                strength: 80,
                dead_zone: 10,
                end_zone: 5,
                axis_pick: 0
            }
        );
        assert_eq!(
            settings(Settings { kind: 7, ..s }, || 2, &Config { kind: 4, ..none }).kind,
            7
        );
    }

    #[test]
    fn figures() {
        assert_eq!(axis_offset(0x13), 12);
        assert!(two_axes(2) && !two_axes(3));
        assert_eq!(saturation(10, 5), Some(85 * 10000 / 90));
        assert_eq!(saturation(100, 0), None);
        assert_eq!(saturation(50, 50), None);
        assert_eq!(saturation(200, 0), None);
        assert_eq!(gain(0), None);
        assert_eq!(gain(80), Some(8000));
    }
}
