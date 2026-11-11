//! The race's start and its panel (from `race::countdown_lights`
//! 0x45B008 and `race::car_label` 0x45B1B7; a car's start place
//! from `race::start_car` 0x4464AD).

/// The start lights of one view, from the milliseconds since the countdown began.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lights {
    /// `start_lights_lit`: the hundreds, at most 5.
    pub shown: u32,
    /// The lights lit: the hundreds, -1 from 6.
    pub lit: u32,
    /// The icon's width (`hud::draw_icon`).
    pub width: u32,
    /// The icon's alpha.
    pub alpha: u32,
    /// How far the icon drops below its row.
    pub drop: u32,
    /// On a whole hundred (with lights lit).
    pub whole: bool,
}

/// The time held to 1500 ms; the hundreds and the rest.
pub fn steps(since: u32) -> (u32, u32) {
    let t = since.min(0x5DC);
    (t / 100, t % 100)
}

/// The lights at `since` (None: negative, nothing more is done): the rest x 2 taken from
/// 0x100 for the width and the alpha (none on a whole hundred); with five shown, the width
/// 0x80 x that (held 0 .. 0x100) / 0x30 and the drop half what that took away.
pub fn lights(since: u32) -> Option<Lights> {
    if (since as i32) < 0 {
        return None;
    }
    let (q, rem) = steps(since);
    let shown = q.min(5);
    let lit = if q < 6 { q } else { u32::MAX };
    let whole = rem == 0 && lit != u32::MAX;
    let rest = 0x100 - if whole { 0 } else { rem * 2 };
    let (width, drop) = if shown == 5 {
        let v = (rest as i32).clamp(0, 0x100);
        let width = (v as i64 * 0x80 / 0x30) as i32;
        (width as u32, (v.wrapping_sub(width) >> 1) as u32)
    } else {
        (rest, 0)
    };
    Some(Lights {
        shown,
        lit,
        width,
        alpha: rest,
        drop,
        whole,
    })
}

/// " CAR nn" (each digit + '0' as a byte), or seven spaces for none.
pub fn car_label(number: u32, none: bool) -> [u8; 7] {
    if none {
        return [b' '; 7];
    }
    [
        b' ',
        b'C',
        b'A',
        b'R',
        b' ',
        ((number / 10) as u8).wrapping_add(b'0'),
        ((number % 10) as u8).wrapping_add(b'0'),
    ]
}

/// `race::start_car`: the car's checkpoint on the line: the first with a check (`first`),
/// less the car's number / 2 but in modes 1 and 3.
pub fn start_checkpoint(first: u32, mode: u32, car: u32) -> u32 {
    if mode != 1 && mode != 3 {
        first.wrapping_sub(car >> 1)
    } else {
        first
    }
}

/// `race::start_car`: whether the car starts to the side: with `session` set, or in
/// mode 0 with other than one player (each asked only where the original reads it).
pub fn starts_aside(
    switch: u32,
    mode: impl FnOnce() -> u32,
    players: impl FnOnce() -> u32,
) -> bool {
    switch != 0 || (mode() == 0 && players() != 1)
}

/// `race::start_car`: how far to the side: 0x14000 to the right for an odd car, to the
/// left for an even one.
pub fn side_offset(car: u32) -> u32 {
    let low = car & 1;
    (low + low).wrapping_sub(1).wrapping_mul(0x14000)
}

/// `race::start_car`: the checks the car starts with: the checkpoint's check count less
/// one, held to 0 ..= 100 (signed).
pub fn start_checks(count: u32) -> u32 {
    (count.wrapping_sub(1) as i32).clamp(0, 100) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start() {
        assert_eq!(lights(u32::MAX), None);
        let l = lights(300).unwrap();
        assert_eq!(
            (l.shown, l.lit, l.whole, l.width, l.alpha),
            (3, 3, true, 0x100, 0x100)
        );
        let l = lights(550).unwrap();
        assert_eq!(
            (l.shown, l.width, l.drop),
            (
                5,
                0x9C * 0x80 / 0x30,
                ((0x9C - 0x9C * 0x80 / 0x30) >> 1) as u32
            )
        );
        assert_eq!(lights(2000).unwrap().lit, u32::MAX);
        assert_eq!(&car_label(42, false), b" CAR 42");
    }

    #[test]
    fn car_starts() {
        assert_eq!(start_checkpoint(10, 0, 5), 8);
        assert_eq!(start_checkpoint(10, 3, 5), 10);
        assert_eq!(start_checkpoint(1, 2, 7), u32::MAX - 1);
        assert!(starts_aside(1, || unreachable!(), || unreachable!()));
        assert!(starts_aside(0, || 0, || 2) && !starts_aside(0, || 0, || 1));
        assert!(!starts_aside(0, || 1, || unreachable!()));
        assert_eq!(side_offset(3), 0x14000);
        assert_eq!(side_offset(2), 0x14000u32.wrapping_neg());
        assert_eq!(
            [0, 1, 50, 200, 0x8000_0000].map(start_checks),
            [0, 0, 49, 100, 100]
        );
    }
}
