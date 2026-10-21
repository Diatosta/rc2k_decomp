//! The panel's values of a view (converted 2026-10-21 from `race::panel_values`
//! 0x45AB5A). Floats as the float pass keeps them: the x87's stack in `f64`, the stored
//! distance rounded to `f32` (exact at the game's double precision).

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
/// (`flt_5C5D98`) over the metres in a mile (`g_metresInMile`), doubled (`fild`, `fdiv`,
/// `fdiv`, `fadd st, st`, `fstp`).
pub fn distance_shown(travelled: i32, unit: f32, miles: f32) -> f32 {
    let v = travelled as f64 / unit as f64 / miles as f64;
    (v + v) as f32
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
}
