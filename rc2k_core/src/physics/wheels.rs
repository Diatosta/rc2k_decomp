//! The mud amounts the wheels add: the mud thrown and splashed per car texture. Floats: the
//! x87's stack values are `f64`, rounded to `f32` where stored.

use super::x87::{below, loaded};

/// The car kinds' table size.
pub const KINDS: u32 = 0x22;

#[cfg(test)]
/// Whether the car's kind collects mud: below 0x22 and its entry (asked then) not 0.
pub(crate) fn gets_mud(kind: u32, kinds: impl FnOnce(u32) -> u32) -> bool {
    kind < KINDS && kinds(kind) != 0
}

/// One half: `speed` (`None` with all wheels off: 0) held at `zero` or more, times `f1` and
/// `f2`, added to the four amounts.
pub fn add_mud(speed: Option<f32>, zero: f32, f1: f32, f2: f32, amounts: &mut [f32; 4]) -> f32 {
    let s = match speed {
        Some(s) if below(s as f64, zero as f64) => loaded(zero),
        Some(s) => s,
        None => 0.0,
    };
    let step = (s as f64 * f1 as f64 * f2 as f64) as f32;
    for a in amounts.iter_mut() {
        *a = (*a as f64 + step as f64) as f32;
    }
    step
}

/// The second half's speed: the drive speed less the car's.
pub fn slip(drive: f32, speed: f32) -> f32 {
    (drive as f64 - speed as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns() {
        assert!(gets_mud(3, |_| 1));
        assert!(!gets_mud(3, |_| 0));
        assert!(!gets_mud(0x22, |_| panic!("not asked")));
        let mut a = [1.0f32; 4];
        assert_eq!(add_mud(Some(2.0), 0.0, 0.5, 3.0, &mut a), 3.0);
        assert_eq!(a, [4.0; 4]);
        assert_eq!(add_mud(Some(-2.0), 0.0, 0.5, 3.0, &mut a), 0.0);
        assert_eq!(add_mud(None, 0.0, 0.5, 3.0, &mut a), 0.0);
        assert_eq!(a, [4.0; 4]);
        assert_eq!(slip(5.0, 1.5), 3.5);
    }
}
