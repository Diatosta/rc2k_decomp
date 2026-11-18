//! The state the 3D sounds are placed by: the viewed car checked, the car file's six light figures
//! scaled, the view's turn taken, and the sound switches from car block 0's words. What each value
//! is, is a reading. The matrices, turns and float sums stay with the caller.

use crate::Invalid;

/// Refused: a viewed car past the eight blocks, or one whose block is null (`nonnull`, asked
/// only for one of the eight).
pub fn check_viewed(viewed: u32, nonnull: impl FnOnce() -> bool) -> Result<(), Invalid> {
    if viewed > 7 {
        return Err(Invalid("a viewed car past the eight blocks"));
    }
    if !nonnull() {
        return Err(Invalid("the viewed car's block is null"));
    }
    Ok(())
}

/// A light record figure times 65536 over 1000. Refused: a quotient that does not fit 32 bits.
pub fn light_figure(v: u32) -> Result<i32, Invalid> {
    let q = (i64::from(v as i32) << 16) / 1000;
    if q != i64::from(q as i32) {
        return Err(Invalid(
            "a light record figure whose times 65536 / 1000 leaves 32 bits",
        ));
    }
    Ok(q as i32)
}

/// The six figures (two points, `CameraWork.listener_points[0]` ..) with the second and fifth
/// less `less`.
pub fn listener_points(f: [i32; 6], less: u32) -> [i32; 6] {
    [
        f[0],
        f[1].wrapping_sub(less as i32),
        f[2],
        f[3],
        f[4].wrapping_sub(less as i32),
        f[5],
    ]
}

/// The cells of the view turn matrix (rows of four) whose nine figures are copied, in order, into
/// the listener's matrix.
pub const TURN_FIGURES: [usize; 9] = [0, 1, 2, 4, 5, 6, 8, 9, 10];

/// The viewed car ahead: its +0xABC 0x10 or more (unsigned).
pub fn car_ahead(word: u32) -> bool {
    word >= 0x10
}

/// The figure with `RaceVars.settings_words[0]` set, else 0.
pub fn first_figure(switch: u32, figure: impl FnOnce() -> u32) -> u32 {
    if switch == 0 { 0 } else { figure() }
}

/// The sound switches from car block 0's words a and b with `FrameWork.mirror_wanted` (`on`), in
/// store order (`ViewerAndKinds.marks_settings.windscreen_drops_rate_factor`, `E8`, `C0`, `C4`,
/// `E4`, `D0`, `C8`, `CC`).
pub fn switches(on: bool, a: u32, b: u32) -> [u32; 8] {
    [
        if on { a << 2 } else { 0 },
        a.wrapping_add(a),
        a.wrapping_add(a),
        if on { b >> 3 } else { 0 },
        b << 3,
        if (a != 0 || b != 0) && on { 0 } else { 1 },
        1,
        0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listener() {
        assert!(check_viewed(8, || panic!()).is_err());
        assert!(check_viewed(3, || false).is_err());
        assert!(check_viewed(3, || true).is_ok());
        assert_eq!(light_figure(1000), Ok(65536));
        assert_eq!(light_figure((-1000i32) as u32), Ok(-65536));
        assert!(light_figure(0x7FFF_FFFF).is_err());
        assert_eq!(listener_points([1, 2, 3, 4, 5, 6], 1), [1, 1, 3, 4, 4, 6]);
        assert!(car_ahead(0x10) && !car_ahead(0xF));
        assert_eq!(first_figure(0, || panic!()), 0);
        assert_eq!(switches(true, 1, 0x10), [4, 2, 2, 2, 0x80, 0, 1, 0]);
        assert_eq!(switches(false, 0, 0)[5], 1);
    }
}
