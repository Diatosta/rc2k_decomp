//! The panel's values of a view (converted from `race::panel_values`
//! 0x45AB5A; its motion Converted from `race::panel_motion` 0x45AE1D). Floats as
//! the float pass keeps them: the x87's stack in `f64`, the stored values rounded to `f32`
//! (exact at the game's double precision; `fldpi`'s 64-bit pi is kept exactly in
//! `times_pi`). The overlay's stage parts Converted from `race::init_overlay`
//! 0x45A9DE (their miles stay the x87's, asked through a closure).

use crate::math::geometry::fistp;

/// A float field shown as a whole number (`fld` / `fistp`: the car's +0x718).
pub fn shown_whole(v: f32) -> i32 {
    fistp(v)
}

/// The distance along the line since the start: the car's checkpoint's distance less the
/// start checkpoint's (none when below), halved.
pub fn travelled(car: u32, start: u32) -> u32 {
    car.saturating_sub(start) >> 1
}

/// The distance shown: `travelled` (a whole number) over the distance unit
/// (`k65536`) over the metres in a mile (`g_metresInMile`), doubled (`fild`, `fdiv`,
/// `fdiv`, `fadd st, st`, `fstp`).
pub fn distance_shown(travelled: i32, unit: f32, miles: f32) -> f32 {
    let v = travelled as f64 / unit as f64 / miles as f64;
    (v + v) as f32
}

/// The x87's `fldpi` (rounded to the nearest): pi = this x 2^-62.
const FLDPI: u64 = 0xC90F_DAA2_2168_C235;

/// `fild a; fldpi; fmulp` at the x87's double precision: a whole number times the x87's
/// 64-bit pi, rounded once to 53 bits (to the nearest, ties to even).
pub fn times_pi(a: i32) -> f64 {
    if a == 0 {
        return 0.0;
    }
    let p = a.unsigned_abs() as u128 * FLDPI as u128;
    let drop = 128 - p.leading_zeros() - 53;
    let mut m = p >> drop;
    let rest = p & ((1u128 << drop) - 1);
    let half = 1u128 << (drop - 1);
    if rest > half || (rest == half && m & 1 == 1) {
        m += 1;
    }
    let v = m as f64 * 2f64.powi(drop as i32 - 62);
    if a < 0 { -v } else { v }
}

/// A view's motion value on the panel: a whole number x `half` x `small` (`fild`, `fmul`,
/// `fmul`, `fstp`).
pub fn motion(v: i32, half: f32, small: f32) -> f32 {
    (v as f64 * half as f64 * small as f64) as f32
}

/// The heading's whole number from the view block's +0x9C0: (-(it)) >> 14.
pub fn heading_turn(word: u32) -> u32 {
    word.wrapping_neg() >> 14
}

/// The heading on the panel: the turn x pi x `half` / `big` (`fild`, `fldpi`, `fmulp`,
/// `fmul`, `fdiv`, `fstp`).
pub fn heading(turn: i32, half: f32, big: f32) -> f32 {
    (times_pi(turn) * half as f64 / big as f64) as f32
}

/// The time the panel's numbers have been shown past 800 ms (`g_sceneClock` less
/// `shown_since` less 800), 0 when below.
pub fn shown_for(clock: u32, since: u32) -> u32 {
    let t = clock.wrapping_sub(since).wrapping_sub(0x320);
    if (t as i32) < 0 { 0 } else { t }
}

/// The overlay's parts' room (`g_overlayPartMiles` and the two lists after it).
pub const MOST_OVERLAY_PARTS: u32 = 0x40;

/// The overlay's six sizes (`dword_6D1486` .. `dword_6D149E`, the fourth after a gap).
pub const OVERLAY_SIZES: [u32; 6] = [0x40, 0x80, 0x50, 0x40, 0x80, 0x50];

/// The car block dwords `race::init_overlay` clears: +0xF78 .. +0xF94, +0xF98, +0xFA0.
pub const OVERLAY_CLEARED: [u32; 10] = [
    0xF78, 0xF7C, 0xF80, 0xF84, 0xF88, 0xF8C, 0xF90, 0xF94, 0xF98, 0xFA0,
];

/// `race::init_overlay`: the parts the overlay is cut into: with two checkpoint kinds or
/// more (signed) the stretches found (`found`, asked only then; Some(that) for
/// `g_overlayParts`), and of those, the count when one or more (signed); else one part of
/// a mile. Refused past the room.
pub fn overlay_parts(
    kinds: u32,
    found: impl FnOnce() -> u32,
) -> Result<(Option<u32>, Option<u32>), crate::Invalid> {
    if (kinds.wrapping_sub(2) as i32) < 0 {
        return Ok((None, None));
    }
    let count = found();
    if (count.wrapping_sub(1) as i32) < 0 {
        return Ok((Some(count), None));
    }
    if count > MOST_OVERLAY_PARTS {
        return Err(crate::Invalid(
            "more than 0x40 parts: the original writes past its lists",
        ));
    }
    Ok((Some(count), Some(count)))
}

/// A part's length as the overlay halves it: the two checkpoints' distances' difference,
/// shifted right (logical: a negative one comes out huge).
pub fn half_length(end: u32, start: u32) -> u32 {
    end.wrapping_sub(start) >> 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        assert_eq!(shown_whole(2.5), 2);
        assert_eq!(shown_whole(3.5), 4);
        assert_eq!(shown_whole(f32::NAN), i32::MIN);
        assert_eq!(travelled(10, 4), 3);
        assert_eq!(travelled(4, 10), 0);
        assert_eq!(distance_shown(65536 * 8, 65536.0, 2.0), 8.0);
    }

    #[test]
    fn motion_and_heading() {
        assert_eq!(times_pi(1), std::f64::consts::PI);
        assert_eq!(times_pi(-1), -std::f64::consts::PI);
        assert_eq!(times_pi(0), 0.0);
        // Against the long product for a spread of whole numbers.
        for a in [3i32, 7, 0x3FFFF, 12345, 1 << 17, i32::MAX] {
            let exact = a as f64 * std::f64::consts::PI;
            let ulp = exact.abs() * f64::EPSILON;
            assert!((times_pi(a) - exact).abs() <= ulp, "{a}");
        }
        assert_eq!(motion(0x2_0000, 0.5, 1.0 / 65536.0), 1.0);
        assert_eq!(heading_turn(0xFFFF_C000), 1);
        assert_eq!(heading(2, 0.5, 1.0), std::f64::consts::PI as f32);
        assert_eq!(shown_for(5000, 1000), 3200);
        assert_eq!(shown_for(1000, 900), 0);
    }

    #[test]
    fn overlay() {
        assert_eq!(overlay_parts(1, || unreachable!()), Ok((None, None)));
        assert_eq!(overlay_parts(2, || 0), Ok((Some(0), None)));
        assert_eq!(
            overlay_parts(2, || 0x8000_0001),
            Ok((Some(0x8000_0001), None))
        );
        assert!(overlay_parts(2, || 0x8000_0000).is_err());
        assert_eq!(overlay_parts(5, || 3), Ok((Some(3), Some(3))));
        assert!(overlay_parts(5, || 0x41).is_err());
        assert_eq!(half_length(10, 4), 3);
        assert_eq!(half_length(4, 10), 0x7FFF_FFFD);
    }
}
