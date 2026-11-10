//! The mud amounts the wheels add (converted 2026-10-21 from
//! `physics::turn_wheel_angles` 0x449EE9; despite the name it adds the mud thrown and splashed
//! per car texture, `mud_thrown_in` / `mud_splashes_in`, for `race::update_mud_texture`).
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision.

use super::x87::{below, loaded};

/// The car kinds' table (`dword_5C5B48`): 0x22 entries.
pub const KINDS: u32 = 0x22;

/// Whether the car's kind (`g_car` +0x24) turns its wheels: below 0x22 and its entry
/// (`kinds`, asked then) not 0.
pub fn turns_wheels(kind: u32, kinds: impl FnOnce(u32) -> u32) -> bool {
    kind < KINDS && kinds(kind) != 0
}

/// One half: `speed` (None with the switch `dword_71A22C` set: 0) held at `zero` or more,
/// times `f1` and `f2` (kept in `dword_62400C`), added to the four angles.
pub fn add_turn(speed: Option<f32>, zero: f32, f1: f32, f2: f32, angles: &mut [f32; 4]) -> f32 {
    let s = match speed {
        Some(s) if below(s as f64, zero as f64) => loaded(zero),
        Some(s) => s,
        None => 0.0,
    };
    let step = (s as f64 * f1 as f64 * f2 as f64) as f32;
    for a in angles.iter_mut() {
        *a = (*a as f64 + step as f64) as f32;
    }
    step
}

/// The second half's speed: the drive speed less the car's (stored: a float).
pub fn slip(drive: f32, speed: f32) -> f32 {
    (drive as f64 - speed as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns() {
        assert!(turns_wheels(3, |_| 1));
        assert!(!turns_wheels(3, |_| 0));
        assert!(!turns_wheels(0x22, |_| panic!("not asked")));
        let mut a = [1.0f32; 4];
        assert_eq!(add_turn(Some(2.0), 0.0, 0.5, 3.0, &mut a), 3.0);
        assert_eq!(a, [4.0; 4]);
        assert_eq!(add_turn(Some(-2.0), 0.0, 0.5, 3.0, &mut a), 0.0);
        assert_eq!(add_turn(None, 0.0, 0.5, 3.0, &mut a), 0.0);
        assert_eq!(a, [4.0; 4]);
        assert_eq!(slip(5.0, 1.5), 3.5);
    }
}
