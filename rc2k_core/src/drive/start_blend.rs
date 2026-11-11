//! The share used at a start (from `drive::start_blend` 0x43773D):
//! the checkpoints the work car is past the lead's start, held to 0 ..= 100, times a step,
//! added to a base and an offset, held to 0 ..= 1. It is the computer driver's skill share: 0 holds its wanted speed down to 30, 1 lets it drive the full
//! table speed (`drive_computer_car`).
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores: the same bits at the game's double precision.

use crate::physics::x87::{below, loaded};

/// What `start_blend` makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartBlend {
    /// The held count (kept in `dword_62400C`; the original's eax).
    pub count: u32,
    /// `flt_71A654`: the count times the step.
    pub share: f32,
    /// `flt_71A658`: base + share + offset, held to `low` ..= `high`.
    pub blend: f32,
}

/// `drive::start_blend`: `car` (`dword_71A070`) less `lead` (`dword_70F484`) held to
/// 0 ..= 100 by the signs; the share times `step` (`flt_604EC0`); the blend of `base`
/// (`flt_71A650`), the share and `offset` (`flt_71A64C`) held to `low` .. `high`
/// (`flt_5C5D90`, `flt_5C5D38`: 0, 1; below or no number raised, not below lowered).
pub fn start_blend(
    car: u32,
    lead: u32,
    step: f32,
    base: f32,
    offset: f32,
    low: f32,
    high: f32,
) -> StartBlend {
    let mut count = car.wrapping_sub(lead);
    if (count as i32) < 0 {
        count = 0;
    }
    if (count.wrapping_sub(100) as i32) >= 0 {
        count = 100;
    }
    let share = (count as i32 as f64 * step as f64) as f32;
    let mut blend = (base as f64 + share as f64 + offset as f64) as f32;
    if below(blend as f64, low as f64) {
        blend = loaded(low);
    }
    if !below(blend as f64, high as f64) {
        blend = loaded(high);
    }
    StartBlend {
        count,
        share,
        blend,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blends() {
        let b = start_blend(1050, 1000, 0.01, 0.1, 0.2, 0.0, 1.0);
        assert_eq!(b.count, 50);
        assert_eq!(b.share, 0.5);
        assert_eq!(b.blend, (0.1f64 as f32 as f64 + 0.5 + 0.2f32 as f64) as f32);
        assert_eq!(start_blend(990, 1000, 0.01, -1.0, 0.0, 0.0, 1.0).blend, 0.0);
        assert_eq!(
            start_blend(0x8000_0000, 1000, 0.01, 0.0, 0.0, 0.0, 1.0).count,
            100
        );
        assert_eq!(start_blend(1200, 1000, 0.01, 0.9, 0.0, 0.0, 1.0).blend, 1.0);
        assert_eq!(
            start_blend(1000, 1000, f32::NAN, 0.0, 0.0, 0.0, 1.0).blend,
            0.0
        );
    }
}
