//! The 1999 layout of the marks' stores, as the differential tests of the kept x87 assembly
//! need it. The stores hold their fields in a natural order with nothing between them; the
//! assembly was recorded against RAL.EXE's own bytes, where unnamed bytes sit between the
//! fields. Here each store's runs of fields are listed with the address (`START` plus `at`)
//! the assembly reads them at, so a test copies the fields in from the buffer standing for
//! the marks' block, runs the Rust version on them, and copies them back. The bytes between the
//! runs stay as the buffer had them: nothing typed reads them. Test code only; it goes with the
//! recorded vectors.

use crate::effects::asm_test::Operand;
use crate::effects::asm_test::START as BLOCK;
use core::mem::offset_of;

/// A run of fields that were next to each other in the 1999 layout.
pub struct Run {
    /// Its offset from the store's first byte in the 1999 layout.
    pub at: usize,
    /// Its offset in the typed store.
    pub to: usize,
    /// Its length.
    pub len: usize,
}

/// A store with a 1999 layout.
pub trait Laid: Sized {
    /// RAL.EXE's address of the store's first byte.
    const START: usize;
    /// The runs of fields, in order.
    const RUNS: &'static [Run];
}

/// Runs `f` on a copy of the store `T` made from the bytes the test buffer `mk` (standing
/// for the marks' block) holds at the 1999 places of its fields, and puts the copy's fields
/// back into the buffer afterwards. Bytes of the copy no run covers are 0.
pub fn with_laid<T: Laid, R>(mk: Operand, f: impl FnOnce(&T) -> R) -> R {
    with_laid_at(mk, T::START, f)
}

/// `with_laid` for a store that has several copies in the 1999 layout, the one at `start`.
pub fn with_laid_at<T: Laid, R>(mk: Operand, start: usize, f: impl FnOnce(&T) -> R) -> R {
    let place = |r: &Run| mk.at((start + r.at).wrapping_sub(BLOCK), r.len);
    // Every run must lie in the buffer before any is read.
    for r in T::RUNS {
        place(r);
    }
    // On the heap: some stores are near 100 KB, and a test thread's stack is small.
    // SAFETY: a store is cells of integers and floats, for which zero bytes are a value.
    let mut local = unsafe { Box::<T>::new_zeroed().assume_init() };
    let base = (&raw mut *local).cast::<u8>();
    for r in T::RUNS {
        // SAFETY: the run lies in the buffer (checked) and in the store (`runs_cover_the_stores`).
        unsafe { core::ptr::copy_nonoverlapping(place(r), base.add(r.to), r.len) };
    }
    let out = f(&local);
    for r in T::RUNS {
        // SAFETY: as above.
        unsafe { core::ptr::copy_nonoverlapping(base.add(r.to).cast_const(), place(r), r.len) };
    }
    out
}

use crate::effects::blocks::CornerSlopes;
use crate::effects::frame_state::{DrawWork, ViewVars};
use crate::effects::kind_store::QuadWork;
use crate::effects::mask_store::{KindPictures, MaskBlocks, PictureCopy};
use crate::effects::pong::{PongRules, PongState};
use crate::effects::rain::{RainRules, RainState};
use crate::effects::ring_store::RingWork;
use crate::effects::setup::MarksClock;
use crate::effects::snow::{SnowCurveVars, SnowLight, SnowState, SnowTables};
use crate::effects::tilt_shape::ShapeTilt;
use crate::effects::tuning::{MarksFloats, MarksKindFloats};
use crate::effects::view_vectors::KeptView;
use crate::effects::windscreen_drop_state::{WindscreenDropState, WindscreenDropWork};
use crate::effects::windscreen_drop_vars::WindscreenDropVars;
use crate::effects::windscreen_flakes_store::{
    WindscreenFlakeState, WindscreenFlakeTables, WindscreenFlakeTiming,
};

/// A copy of the store `T` made from the buffer as `with_laid` does, but zero where the buffer
/// does not reach (a block reads only the fields it uses), and never put back.
pub fn copy_of<T: Laid>(mk: Operand) -> &'static T {
    // SAFETY: a store is cells of integers and floats, for which zero bytes are a value.
    let mut local = unsafe { Box::<T>::new_zeroed().assume_init() };
    let base = (&raw mut *local).cast::<u8>();
    for r in T::RUNS {
        let bytes = mk.bytes_or_zero((T::START + r.at).wrapping_sub(BLOCK), r.len);
        // SAFETY: the run lies in the store (`runs_cover_the_stores`).
        unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), base.add(r.to), r.len) };
    }
    Box::leak(local)
}

impl Laid for CornerSlopes {
    const START: usize = 0x920734;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        to: offset_of!(CornerSlopes, slopes),
        len: 0x2400,
    }];
}

/// The first of the three quads; the others are at 0x933ED0 and 0x934BD0 (`with_laid_at`).
impl Laid for QuadWork {
    const START: usize = 0x9331B4;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(QuadWork, place),
            len: 0xC,
        },
        Run {
            at: 0x10,
            to: offset_of!(QuadWork, light_turned),
            len: 0x30,
        },
        Run {
            at: 0x4C,
            to: offset_of!(QuadWork, sides),
            len: 0x18,
        },
        Run {
            at: 0x70,
            to: offset_of!(QuadWork, frame),
            len: 0x88,
        },
    ];
}

impl Laid for DrawWork {
    const START: usize = 0x8EB871;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(DrawWork, profile_work),
            len: 0x2C,
        },
        Run {
            at: 0x44,
            to: offset_of!(DrawWork, half),
            len: 0x38,
        },
        Run {
            at: 0xD0,
            to: offset_of!(DrawWork, sides),
            len: 0x8,
        },
        Run {
            at: 0xE0,
            to: offset_of!(DrawWork, step),
            len: 0xC,
        },
        Run {
            at: 0xF0,
            to: offset_of!(DrawWork, near),
            len: 0x4,
        },
        Run {
            at: 0xF8,
            to: offset_of!(DrawWork, uv_k),
            len: 0x8,
        },
    ];
}

impl Laid for ViewVars {
    const START: usize = 0x8F3975;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(ViewVars, heading),
            len: 0x18,
        },
        Run {
            at: 0x1C,
            to: offset_of!(ViewVars, first_end_half_width),
            len: 0x8,
        },
        Run {
            at: 0x28,
            to: offset_of!(ViewVars, second_end_half_width),
            len: 0x8,
        },
        Run {
            at: 0x34,
            to: offset_of!(ViewVars, turned_points),
            len: 0x24,
        },
    ];
}

impl Laid for MaskBlocks {
    const START: usize = 0x934D6E;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(MaskBlocks, seen),
            len: 0x18,
        },
        Run {
            at: 0x48,
            to: offset_of!(MaskBlocks, mask),
            len: 0x14,
        },
        Run {
            at: 0x8C,
            to: offset_of!(MaskBlocks, bmask),
            len: 0x30,
        },
    ];
}

impl Laid for PictureCopy {
    const START: usize = 0x934E29;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(PictureCopy, list),
            len: 0x10,
        },
        Run {
            at: 0x40,
            to: offset_of!(PictureCopy, copy),
            len: 0x40,
        },
    ];
}

impl Laid for KindPictures {
    const START: usize = 0x934EA8;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        to: offset_of!(KindPictures, numbers),
        len: 0x88,
    }];
}

impl Laid for PongState {
    const START: usize = 0x768658;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(PongState, box_sides),
            len: 0x38,
        },
        Run {
            at: 0x3C,
            to: offset_of!(PongState, ball_way),
            len: 0x8,
        },
        Run {
            at: 0x48,
            to: offset_of!(PongState, ball_size),
            len: 0x8,
        },
        Run {
            at: 0x54,
            to: offset_of!(PongState, saved_bat),
            len: 0x8,
        },
        Run {
            at: 0x60,
            to: offset_of!(PongState, bat),
            len: 0x8,
        },
        Run {
            at: 0x78,
            to: offset_of!(PongState, bat_size),
            len: 0x8,
        },
        Run {
            at: 0x84,
            to: offset_of!(PongState, ball_centre),
            len: 0x8,
        },
        Run {
            at: 0x90,
            to: offset_of!(PongState, bat_centre),
            len: 0x8,
        },
        Run {
            at: 0x9C,
            to: offset_of!(PongState, difference),
            len: 0x8,
        },
        Run {
            at: 0xA8,
            to: offset_of!(PongState, steps),
            len: 0xDC,
        },
    ];
}

impl Laid for PongRules {
    const START: usize = 0x769182;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(PongRules, start),
            len: 0x20,
        },
        Run {
            at: 0x24,
            to: offset_of!(PongRules, walls),
            len: 0x3C,
        },
        Run {
            at: 0x78,
            to: offset_of!(PongRules, cell_step),
            len: 0x24,
        },
        Run {
            at: 0xA0,
            to: offset_of!(PongRules, setup_places),
            len: 0x10,
        },
    ];
}

impl Laid for RainState {
    const START: usize = 0x8BC405;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(RainState, drawn),
            len: 0x60,
        },
        Run {
            at: 0x78,
            to: offset_of!(RainState, motion),
            len: 0x84,
        },
        Run {
            at: 0x100,
            to: offset_of!(RainState, drift),
            len: 0x4,
        },
        Run {
            at: 0x108,
            to: offset_of!(RainState, drops),
            len: 0x1804,
        },
        Run {
            at: 0x1924,
            to: offset_of!(RainState, edge_a),
            len: 0x8,
        },
        Run {
            at: 0x1930,
            to: offset_of!(RainState, edge_b),
            len: 0x8,
        },
        Run {
            at: 0x193C,
            to: offset_of!(RainState, ends),
            len: 0x18,
        },
        Run {
            at: 0x1958,
            to: offset_of!(RainState, depth),
            len: 0x50,
        },
        Run {
            at: 0x1A38,
            to: offset_of!(RainState, turned),
            len: 0x18,
        },
    ];
}

impl Laid for RainRules {
    const START: usize = 0x8C5F91;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(RainRules, rhw),
            len: 0x28,
        },
        Run {
            at: 0x40,
            to: offset_of!(RainRules, target_scale),
            len: 0x20,
        },
    ];
}

impl Laid for RingWork {
    const START: usize = 0x935620;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(RingWork, list),
            len: 0x10,
        },
        Run {
            at: 0x40,
            to: offset_of!(RingWork, cursor),
            len: 0x4,
        },
        Run {
            at: 0x48,
            to: offset_of!(RingWork, block),
            len: 0x4,
        },
        Run {
            at: 0x50,
            to: offset_of!(RingWork, top),
            len: 0x4,
        },
        Run {
            at: 0x58,
            to: offset_of!(RingWork, fade),
            len: 0x24,
        },
        Run {
            at: 0x90,
            to: offset_of!(RingWork, rate),
            len: 0x8,
        },
        Run {
            at: 0x9C,
            to: offset_of!(RingWork, near),
            len: 0x8,
        },
        Run {
            at: 0xA8,
            to: offset_of!(RingWork, count),
            len: 0x4,
        },
        Run {
            at: 0xF8,
            to: offset_of!(RingWork, away),
            len: 0x18,
        },
    ];
}

impl Laid for MarksClock {
    const START: usize = 0x764FC8;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        to: offset_of!(MarksClock, started),
        len: 0x30,
    }];
}

impl Laid for SnowTables {
    const START: usize = 0x8C6000;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        to: offset_of!(SnowTables, randoms),
        len: 0xC000,
    }];
}

impl Laid for SnowState {
    const START: usize = 0x8D2000;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x30,
            to: offset_of!(SnowState, wide_turn),
            len: 0x124,
        },
        Run {
            at: 0x15C,
            to: offset_of!(SnowState, flag),
            len: 0x34,
        },
        Run {
            at: 0x194,
            to: offset_of!(SnowState, fall),
            len: 0x4,
        },
        Run {
            at: 0x19C,
            to: offset_of!(SnowState, drawn),
            len: 0xC,
        },
        Run {
            at: 0x1F4,
            to: offset_of!(SnowState, depth),
            len: 0x8,
        },
        Run {
            at: 0x208,
            to: offset_of!(SnowState, point),
            len: 0x3C,
        },
        Run {
            at: 0x2D4,
            to: offset_of!(SnowState, turned),
            len: 0xC,
        },
    ];
}

impl Laid for SnowCurveVars {
    const START: usize = 0x8DA410;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(SnowCurveVars, curve_vars),
            len: 0x1C,
        },
        Run {
            at: 0x20,
            to: offset_of!(SnowCurveVars, records),
            len: 0x1000,
        },
    ];
}

impl Laid for SnowLight {
    const START: usize = 0x8DB430;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(SnowLight, light_place),
            len: 0x110,
        },
        Run {
            at: 0x118,
            to: offset_of!(SnowLight, side),
            len: 0x10,
        },
    ];
}

impl Laid for ShapeTilt {
    const START: usize = 0x811BC3;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        to: offset_of!(ShapeTilt, tilt),
        len: 0x28,
    }];
}

impl Laid for MarksFloats {
    const START: usize = 0x765140;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(MarksFloats, pull_limit),
            len: 0x4,
        },
        Run {
            at: 0xC,
            to: offset_of!(MarksFloats, burst_direction_k),
            len: 0x8,
        },
        Run {
            at: 0x18,
            to: offset_of!(MarksFloats, half),
            len: 0x8,
        },
        Run {
            at: 0x24,
            to: offset_of!(MarksFloats, fifth),
            len: 0xC,
        },
        Run {
            at: 0x34,
            to: offset_of!(MarksFloats, three),
            len: 0x4,
        },
        Run {
            at: 0x4C,
            to: offset_of!(MarksFloats, sixteen),
            len: 0x8,
        },
        Run {
            at: 0x58,
            to: offset_of!(MarksFloats, hundred_twenty_eight),
            len: 0x10,
        },
        Run {
            at: 0x7C,
            to: offset_of!(MarksFloats, quarter),
            len: 0x1C,
        },
        Run {
            at: 0x9C,
            to: offset_of!(MarksFloats, fixed_16_scale),
            len: 0x4,
        },
        Run {
            at: 0xA4,
            to: offset_of!(MarksFloats, seven_hundred),
            len: 0x8,
        },
        Run {
            at: 0xB0,
            to: offset_of!(MarksFloats, two_thousand),
            len: 0x4,
        },
        Run {
            at: 0xD4,
            to: offset_of!(MarksFloats, over_65536),
            len: 0x4,
        },
        Run {
            at: 0xDC,
            to: offset_of!(MarksFloats, spread_scale),
            len: 0x8,
        },
    ];
}

impl Laid for MarksKindFloats {
    const START: usize = 0x765238;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        to: offset_of!(MarksKindFloats, other),
        len: 0x6C,
    }];
}

impl Laid for KeptView {
    const START: usize = 0x769236;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(KeptView, first_sine_cosine),
            len: 0x28,
        },
        Run {
            at: 0x34,
            to: offset_of!(KeptView, kept_a),
            len: 0x1C,
        },
        Run {
            at: 0x60,
            to: offset_of!(KeptView, last),
            len: 0x44,
        },
        Run {
            at: 0xB4,
            to: offset_of!(KeptView, step),
            len: 0x28,
        },
    ];
}

impl Laid for WindscreenDropState {
    const START: usize = 0x814710;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(WindscreenDropState, live),
            len: 0x20,
        },
        Run {
            at: 0x24,
            to: offset_of!(WindscreenDropState, depth),
            len: 0x4,
        },
    ];
}

impl Laid for WindscreenDropWork {
    const START: usize = 0x814772;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(WindscreenDropWork, drift),
            len: 0x8,
        },
        Run {
            at: 0x50,
            to: offset_of!(WindscreenDropWork, curve),
            len: 0x47C,
        },
        Run {
            at: 0x4FC,
            to: offset_of!(WindscreenDropWork, setup_axes),
            len: 0x38,
        },
        Run {
            at: 0x538,
            to: offset_of!(WindscreenDropWork, cell),
            len: 0x4E0,
        },
    ];
}

impl Laid for WindscreenDropVars {
    const START: usize = 0x816192;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(WindscreenDropVars, scale),
            len: 0x2C,
        },
        Run {
            at: 0x44,
            to: offset_of!(WindscreenDropVars, kept_place),
            len: 0x30,
        },
        Run {
            at: 0x8C,
            to: offset_of!(WindscreenDropVars, drift),
            len: 0x18,
        },
        Run {
            at: 0xA8,
            to: offset_of!(WindscreenDropVars, windscreen_flakes_cleared),
            len: 0x18044,
        },
        Run {
            at: 0x180F4,
            to: offset_of!(WindscreenDropVars, axes_k),
            len: 0x18,
        },
        Run {
            at: 0x18110,
            to: offset_of!(WindscreenDropVars, side),
            len: 0x18,
        },
        Run {
            at: 0x18134,
            to: offset_of!(WindscreenDropVars, turn),
            len: 0x24,
        },
        Run {
            at: 0x18164,
            to: offset_of!(WindscreenDropVars, adjugate),
            len: 0x24,
        },
        Run {
            at: 0x18194,
            to: offset_of!(WindscreenDropVars, decay),
            len: 0x1C,
        },
    ];
}

impl Laid for WindscreenFlakeTiming {
    const START: usize = 0x89CA30;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        to: offset_of!(WindscreenFlakeTiming, frames),
        len: 0xC,
    }];
}

impl Laid for WindscreenFlakeTables {
    const START: usize = 0x89CACA;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(WindscreenFlakeTables, curve),
            len: 0x438,
        },
        Run {
            at: 0x450,
            to: offset_of!(WindscreenFlakeTables, axes),
            len: 0x6C,
        },
        Run {
            at: 0x4C8,
            to: offset_of!(WindscreenFlakeTables, corners),
            len: 0x30,
        },
        Run {
            at: 0x51C,
            to: offset_of!(WindscreenFlakeTables, points),
            len: 0x184,
        },
    ];
}

impl Laid for WindscreenFlakeState {
    const START: usize = 0x8BC18A;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: offset_of!(WindscreenFlakeState, done),
            len: 0xD0,
        },
        Run {
            at: 0xD4,
            to: offset_of!(WindscreenFlakeState, scaled_setting),
            len: 0x8,
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runs are the whole store, in order, with nothing between them.
    fn covered<T: Laid>() {
        let mut next = 0;
        for r in T::RUNS {
            assert_eq!(r.to, next, "{}", std::any::type_name::<T>());
            next += r.len;
        }
        assert_eq!(next, size_of::<T>(), "{}", std::any::type_name::<T>());
    }

    #[test]
    fn runs_cover_the_stores() {
        covered::<CornerSlopes>();
        covered::<QuadWork>();
        covered::<DrawWork>();
        covered::<ViewVars>();
        covered::<MaskBlocks>();
        covered::<PictureCopy>();
        covered::<KindPictures>();
        covered::<PongState>();
        covered::<PongRules>();
        covered::<RainState>();
        covered::<RainRules>();
        covered::<RingWork>();
        covered::<MarksClock>();
        covered::<SnowTables>();
        covered::<SnowState>();
        covered::<SnowCurveVars>();
        covered::<SnowLight>();
        covered::<ShapeTilt>();
        covered::<MarksFloats>();
        covered::<MarksKindFloats>();
        covered::<KeptView>();
        covered::<WindscreenDropState>();
        covered::<WindscreenDropWork>();
        covered::<WindscreenDropVars>();
        covered::<WindscreenFlakeTiming>();
        covered::<WindscreenFlakeTables>();
        covered::<WindscreenFlakeState>();
    }
}
