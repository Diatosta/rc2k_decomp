//! The race cars' lights and parts' sounds: whether a car's lights are drawn, its light word,
//! the light at a place and the face corners' colours.

use crate::Invalid;

/// Whether a race car's lights are drawn: it has lights, it is drawn (any car in modes 0 and 2,
/// else the shown car), and the shown car is not standing nor hidden; each asked only when needed.
pub fn lights_drawn(
    has_lights: bool,
    mode: u32,
    shown: bool,
    standing: impl FnOnce() -> bool,
    hidden: impl FnOnce() -> bool,
) -> bool {
    has_lights
        && (mode == 2 || mode == 0 || shown)
        && !(shown && standing())
        && !(shown && hidden())
}

/// A nibble-wise step: `prev`'s bits in `keep` and `bits`.
fn set(prev: u32, keep: u32, bits: u32) -> u32 {
    (prev & keep) | bits
}

/// A drawn car's light word from `prev`: 0x21; for the shown car with beams 0x12, then 0x14
/// unless the kind is 1, then 0x11 unless it is 2 or 3; then 0x100 in the third nibble, 0x200
/// with `high` (the shown car only).
pub fn light_word(
    prev: u32,
    shown: bool,
    beams: impl FnOnce() -> bool,
    kind: impl FnOnce() -> u32,
    high: u32,
) -> u32 {
    let mut w = set(prev, 0xFFFF_FF0F, 0x20);
    w = set(w, 0xFFFF_FFF0, 1);
    if shown && beams() {
        w = set(w, 0xFFFF_FF0F, 0x10);
        w = set(w, 0xFFFF_FFF0, 2);
        let kind = kind();
        if kind != 1 {
            w = set(w, 0xFFFF_FFF0, 4);
            if kind != 2 && kind != 3 {
                w = set(w, 0xFFFF_FFF0, 1);
            }
        }
    }
    w = set(w, 0xFFFF_F0FF, 0x100);
    if high != 0 && (high != 1 || shown) {
        w = set(w, 0xFFFF_F0FF, 0x200);
    }
    w
}

/// The light word's bit for lights on (set in every drawn car's).
pub const LIGHTS_ON: u32 = 0x20;

/// Whether a car's lights are drawn, from its light word.
pub fn lights_on(word: u32) -> bool {
    word & LIGHTS_ON != 0
}

/// The depth entries one car's lights may add (its own and its flares).
pub const ENTRIES_A_CAR: u32 = 25;

/// What the car lights refuse: no race cars or more than eight, and a depth list without room
/// for every car's entries.
pub fn check_car_lights(cars: u32, count: u32, room: u32) -> Result<(), Invalid> {
    if cars == 0 || cars > 8 {
        return Err(Invalid("no race cars or more than eight"));
    }
    if count > room - ENTRIES_A_CAR * cars {
        return Err(Invalid("the depth list has no room for the cars' lights"));
    }
    Ok(())
}

/// The count from which a car part's sound wakes.
pub const WAKE_COUNT: u32 = 10;

/// What happens to one part slot's sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartWake {
    /// Already awake (flag set) or not yet counted to 10.
    Left,
    /// Flag set to 1.
    Woken,
    /// Flag set, and the count exactly 10: the car's sound object may wake too.
    WokenNow,
}

/// A part slot by its wake flag and its count.
pub fn part_wake(flag: u32, count: u32) -> PartWake {
    if flag != 0 || count < WAKE_COUNT {
        PartWake::Left
    } else if count == WAKE_COUNT {
        PartWake::WokenNow
    } else {
        PartWake::Woken
    }
}

/// Two colours added byte by byte, each held to 0xFF: four bytes with MMX, else the low three.
pub fn add_colours(a: u32, b: u32, mmx: bool) -> u32 {
    let bytes = if mmx { 4 } else { 3 };
    let mut out = a;
    for k in 0..bytes {
        let s = ((a >> (8 * k)) & 0xFF) + ((b >> (8 * k)) & 0xFF);
        out = (out & !(0xFF << (8 * k))) | (s.min(0xFF) << (8 * k));
    }
    out
}

/// Two colours multiplied byte by byte, (a x b) >> 8.
pub fn multiply_colours(a: u32, b: u32) -> u32 {
    let mut out = 0;
    for k in 0..4 {
        let p = (((a >> (8 * k)) & 0xFF) * ((b >> (8 * k)) & 0xFF)) >> 8;
        out |= p << (8 * k);
    }
    out
}

/// A whole number held to 0 ..= 0xFF (signed).
pub fn clamp_byte(v: u32) -> u32 {
    (v as i32).clamp(0, 0xFF) as u32
}

/// Whether car n's headlights light the place: with one car chosen that car (or all with the
/// all-cars setting); else the players' cars; outside one player and modes 0 and 2 only the
/// drawn view's car.
pub fn headlights_light(
    n: u32,
    one_chosen: bool,
    chosen: impl FnOnce() -> u32,
    all: bool,
    players: impl Fn() -> u32,
    mode: impl FnOnce() -> u32,
    view_car: impl FnOnce() -> u32,
) -> bool {
    if one_chosen {
        n == chosen() || all
    } else if !all && n >= players() {
        false
    } else if players() == 1 || matches!(mode(), 0 | 2) {
        true
    } else {
        n == view_car()
    }
}

/// The strength of a car's headlights at the place, an index into the car light table: from the
/// spread ahead, the height off the beam and the car's marks.
pub fn headlight_index(spread: u32, side: u32, marks: u32) -> u32 {
    let mut beam = clamp_byte(((spread >> 2).wrapping_sub(0x100)).wrapping_neg());
    beam = beam.wrapping_add(beam);
    let side = (side as i32).wrapping_abs() as u32;
    let f = clamp_byte(((0x10_0000u32.wrapping_sub(side)) as i32 >> 12) as u32);
    let mut strength = f.wrapping_mul(beam) >> 8;
    strength = strength.wrapping_mul(marks) >> 8;
    clamp_byte(strength)
}

/// The grey of a lamp at the place: ((0x200000 - distance) x strength) >> 17, held to
/// 0 ..= 0xFF, in the three colour bytes.
pub fn lamp_grey(distance: u32, strength: u32) -> (u32, u32) {
    let product = (0x20_0000u32.wrapping_sub(distance) as i32 as i64) * (strength as i32 as i64);
    let grey = clamp_byte(((product as u32) as i32 >> 0x11) as u32);
    (
        (grey << 16 | grey << 8 | grey) & 0xFF_FFFF,
        (product >> 32) as u32,
    )
}

/// The lamps there are at most (the light colour refuses past them).
pub const MOST_LIGHT_LAMPS: u32 = 16;

/// The light colour's refusals: no cars, more than eight, a null block, more than 16 lamps.
pub fn check_light_colour(
    cars: u32,
    block: impl Fn(u32) -> u32,
    lamps: impl FnOnce() -> u32,
) -> Result<(), Invalid> {
    if cars == 0 || cars > 8 {
        return Err(Invalid("no race cars or more than eight"));
    }
    if (0..cars).any(|n| block(n) == 0) {
        return Err(Invalid("a null car block"));
    }
    if lamps() > MOST_LIGHT_LAMPS {
        return Err(Invalid("more than 16 lamps"));
    }
    Ok(())
}

/// A corner's colour, its light and with the face's flag 4 the two extra lights.
pub fn corner_colour(light: u32, lit: bool, extra: impl FnOnce() -> (u32, u32), mmx: bool) -> u32 {
    if !lit {
        return light;
    }
    let (a, b) = extra();
    add_colours(add_colours(light, a, mmx), b, mmx)
}

/// The face's switches flag (+2) for the extra lights.
pub const EXTRA_LIGHTS: u16 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours() {
        assert_eq!(
            add_colours(0x80_F0_10_20, 0x90_20_F0_01, true),
            0xFF_FF_FF_21
        );
        assert_eq!(
            add_colours(0x80_F0_10_20, 0x90_20_F0_01, false),
            0x80_FF_FF_21
        );
        assert_eq!(
            multiply_colours(0xFF_80_40_FF, 0x80_80_FF_02),
            0x7F_40_3F_01
        );
        assert_eq!(clamp_byte(0x1234), 0xFF);
        assert_eq!(clamp_byte(u32::MAX), 0);
        assert!(headlights_light(
            2,
            true,
            || 2,
            false,
            || unreachable!(),
            || 0,
            || 0
        ));
        assert!(headlights_light(
            3,
            true,
            || 2,
            true,
            || unreachable!(),
            || 0,
            || 0
        ));
        assert!(!headlights_light(3, false, || 0, false, || 2, || 0, || 0));
        assert!(headlights_light(
            1,
            false,
            || 0,
            false,
            || 2,
            || 0,
            || unreachable!()
        ));
        assert!(headlights_light(1, false, || 0, false, || 2, || 1, || 1));
        assert!(!headlights_light(1, false, || 0, false, || 2, || 1, || 0));
        assert_eq!(headlight_index(0x400, 0, 0x100), 0);
        assert_eq!(headlight_index(0, 0, 0x100), 0xFF);
        // spread 0: 0x100 less nothing held to 0xFF, doubled; side 0: 0xFF.
        assert_eq!(
            headlight_index(0, 0, 0x20),
            (((0xFF * 0x1FE) >> 8) * 0x20) >> 8
        );
        assert_eq!(lamp_grey(0x20_0000, 5), (0, 0));
        assert_eq!(lamp_grey(0, 0x100), (0xFF_FFFF, 0));
        assert_eq!(lamp_grey(0x10_0000, 0x2), (0x10_1010, 0));
        assert_eq!(lamp_grey(0x40_0000, 0x7FFF_FFFF), (0x10_1010, 0xFFF0_0000));
        assert!(check_light_colour(0, |_| 1, || 0).is_err());
        assert!(check_light_colour(1, |_| 0, || 0).is_err());
        assert!(check_light_colour(1, |_| 1, || 17).is_err());
        assert!(check_light_colour(1, |_| 1, || 16).is_ok());
        assert_eq!(corner_colour(5, false, || unreachable!(), true), 5);
        assert_eq!(corner_colour(5, true, || (1, 2), true), 8);
    }

    #[test]
    fn drawn() {
        assert!(lights_drawn(
            true,
            0,
            false,
            || unreachable!(),
            || unreachable!()
        ));
        assert!(!lights_drawn(
            true,
            1,
            false,
            || unreachable!(),
            || unreachable!()
        ));
        assert!(!lights_drawn(
            false,
            0,
            true,
            || unreachable!(),
            || unreachable!()
        ));
        assert!(!lights_drawn(true, 1, true, || true, || unreachable!()));
        assert!(!lights_drawn(true, 1, true, || false, || true));
        assert!(lights_drawn(true, 1, true, || false, || false));
    }

    #[test]
    fn words() {
        assert_eq!(
            light_word(0, false, || unreachable!(), || unreachable!(), 0),
            0x121
        );
        assert_eq!(
            light_word(0xF000_0FFF, false, || unreachable!(), || unreachable!(), 1),
            0xF000_0121
        );
        assert_eq!(light_word(0, true, || true, || 1, 1), 0x212);
        assert_eq!(light_word(0, true, || true, || 2, 0), 0x114);
        assert_eq!(light_word(0, true, || true, || 7, 2), 0x211);
        assert_eq!(light_word(0, true, || false, || unreachable!(), 0), 0x121);
    }

    #[test]
    fn car_lights() {
        assert!(lights_on(0x121) && !lights_on(0x111));
        assert!(check_car_lights(0, 0, 0x1000).is_err());
        assert!(check_car_lights(9, 0, 0x1000).is_err());
        assert!(check_car_lights(8, 0x1000 - 200, 0x1000).is_ok());
        assert!(check_car_lights(8, 0x1000 - 199, 0x1000).is_err());
    }

    #[test]
    fn parts() {
        assert_eq!(part_wake(1, 20), PartWake::Left);
        assert_eq!(part_wake(0, 9), PartWake::Left);
        assert_eq!(part_wake(0, 10), PartWake::WokenNow);
        assert_eq!(part_wake(0, 11), PartWake::Woken);
    }
}
