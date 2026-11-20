//! The recorded layout of the marks' stores, as the differential tests of the kept x87
//! assembly need it. The stores hold their fields in a natural order with nothing between
//! them; the assembly was recorded against a layout with unnamed bytes between the
//! fields. Here each store's runs of fields are listed with the address (`START` plus `at`)
//! the assembly reads them at, so a test copies the fields in from the buffer standing for
//! the marks' block, runs the Rust version on them, and copies them back. The bytes between
//! the runs stay as the buffer had them: nothing typed reads them. Test code only.

use crate::effects::asm_test::Operand;
use crate::effects::asm_test::START as BLOCK;
use crate::memory::cells::Cells;
use crate::test_support::paths::{field_range, field_start, set_field_range};

/// A run of fields that were next to each other in the recorded layout.
pub struct Run {
    /// Its offset from the store's first byte in the recorded layout.
    pub at: usize,
    /// The path of the field it begins with in the typed store; the run takes this field's
    /// cells and the next fields' until `len` bytes of cells are covered.
    pub field: &'static str,
    /// Its length in bytes of cells.
    pub len: usize,
}

/// A store with a recorded layout.
pub trait Laid: Cells + Sized {
    /// The address of the store's first byte in the recorded layout.
    const START: usize;
    /// The runs of fields, in order.
    const RUNS: &'static [Run];
}

use crate::effects::blocks_store::CornerSlopes;
use crate::effects::frame_state_store::{DrawWork, ViewVars};
use crate::effects::kind_store::QuadWork;
use crate::effects::mask_store::{MaskBlocks, PictureCopy};
use crate::effects::pong_store::{PongRules, PongState};
use crate::effects::rain_store::RainState;
use crate::effects::ring_store::RingWork;
use crate::effects::setup_store::MarksClock;
use crate::effects::snow_store::{SnowCurveVars, SnowLight, SnowState, SnowTables};
use crate::effects::tilt_shape_store::ShapeTilt;
use crate::effects::tuning_store::{MarksFloats, MarksKindFloats};
use crate::effects::view_vectors_store::KeptView;
use crate::effects::windscreen_drop_state_store::{WindscreenDropState, WindscreenDropWork};
use crate::effects::windscreen_drop_vars_store::WindscreenDropVars;
use crate::effects::windscreen_flakes_store::{
    WindscreenFlakeState, WindscreenFlakeTables, WindscreenFlakeTiming,
};

/// A store with every cell 0.
fn zeroed<T: Cells>() -> Box<T> {
    // On the heap: some stores are near 100 KB, and a test thread's stack is small.
    crate::memory::cells::boxed_zeroed_cells()
}

/// Runs `f` on a copy of the store `T` made from the bytes the test buffer `mk` (standing
/// for the marks' block) holds at the recorded places of its fields, and puts the copy's fields
/// back into the buffer afterwards. Cells no run covers are 0.
pub fn with_laid<T: Laid, R>(mk: Operand, f: impl FnOnce(&T) -> R) -> R {
    with_laid_at(mk, T::START, f)
}

/// `with_laid` for a store that has several copies in the recorded layout, the one at `start`.
pub(super) fn with_laid_at<T: Laid, R>(mk: Operand, start: usize, f: impl FnOnce(&T) -> R) -> R {
    let place = |r: &Run| (start + r.at).wrapping_sub(BLOCK);
    let local = zeroed::<T>();
    for r in T::RUNS {
        // `read` checks that the run lies in the buffer.
        let bytes = mk.read(place(r), r.len);
        assert!(set_field_range(&*local, r.field, &bytes), "{}", r.field);
    }
    let out = f(&local);
    for r in T::RUNS {
        mk.put(place(r), &field_range(&*local, r.field, r.len));
    }
    out
}

/// A copy of the store `T` made from the buffer as `with_laid` does, but zero where the buffer
/// does not reach (a block reads only the fields it uses), and never put back.
pub(super) fn copy_of<T: Laid>(mk: Operand) -> &'static T {
    let local = zeroed::<T>();
    for r in T::RUNS {
        let bytes = mk.bytes_or_zero((T::START + r.at).wrapping_sub(BLOCK), r.len);
        assert!(set_field_range(&*local, r.field, &bytes), "{}", r.field);
    }
    Box::leak(local)
}

impl Laid for CornerSlopes {
    const START: usize = 0x920734;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        field: ".slopes",
        len: 0x2400,
    }];
}

/// The first of the three quads; the others are reached by `with_laid_at`.
impl Laid for QuadWork {
    const START: usize = 0x9331B4;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".place",
            len: 0xC,
        },
        Run {
            at: 0x10,
            field: ".light_turned",
            len: 0x30,
        },
        Run {
            at: 0x4C,
            field: ".sides",
            len: 0x18,
        },
        Run {
            at: 0x70,
            field: ".frame",
            len: 0x88,
        },
    ];
}

impl Laid for DrawWork {
    const START: usize = 0x8EB871;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".profile_work",
            len: 0x2C,
        },
        Run {
            at: 0x44,
            field: ".half",
            len: 0x38,
        },
        Run {
            at: 0xD0,
            field: ".sides",
            len: 0x8,
        },
        Run {
            at: 0xE0,
            field: ".step",
            len: 0xC,
        },
        Run {
            at: 0xF0,
            field: ".near",
            len: 0x4,
        },
        Run {
            at: 0xF8,
            field: ".uv_k",
            len: 0x8,
        },
    ];
}

impl Laid for ViewVars {
    const START: usize = 0x8F3975;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".heading",
            len: 0x18,
        },
        Run {
            at: 0x1C,
            field: ".first_end_half_width",
            len: 0x8,
        },
        Run {
            at: 0x28,
            field: ".second_end_half_width",
            len: 0x8,
        },
        Run {
            at: 0x34,
            field: ".turned_points",
            len: 0x24,
        },
    ];
}

impl Laid for MaskBlocks {
    const START: usize = 0x934D6E;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".seen",
            len: 0x18,
        },
        Run {
            at: 0x48,
            field: ".mask",
            len: 0x14,
        },
        Run {
            at: 0x8C,
            field: ".bmask",
            len: 0x30,
        },
    ];
}

impl Laid for PictureCopy {
    const START: usize = 0x934E29;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".list",
            len: 0x10,
        },
        Run {
            at: 0x40,
            field: ".copy",
            len: 0x40,
        },
    ];
}

impl Laid for PongState {
    const START: usize = 0x768658;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".box_sides",
            len: 0x38,
        },
        Run {
            at: 0x3C,
            field: ".ball_way",
            len: 0x8,
        },
        Run {
            at: 0x48,
            field: ".ball_size",
            len: 0x8,
        },
        Run {
            at: 0x54,
            field: ".saved_bat",
            len: 0x8,
        },
        Run {
            at: 0x60,
            field: ".bat",
            len: 0x8,
        },
        Run {
            at: 0x78,
            field: ".bat_size",
            len: 0x8,
        },
        Run {
            at: 0x84,
            field: ".ball_centre",
            len: 0x8,
        },
        Run {
            at: 0x90,
            field: ".bat_centre",
            len: 0x8,
        },
        Run {
            at: 0x9C,
            field: ".difference",
            len: 0x8,
        },
        Run {
            at: 0xA8,
            field: ".steps",
            len: 0xDC,
        },
    ];
}

impl Laid for PongRules {
    const START: usize = 0x769182;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".start",
            len: 0x20,
        },
        Run {
            at: 0x24,
            field: ".walls",
            len: 0x3C,
        },
        Run {
            at: 0x78,
            field: ".cell_step",
            len: 0x24,
        },
        Run {
            at: 0xA0,
            field: ".setup_places",
            len: 0x10,
        },
    ];
}

impl Laid for RainState {
    const START: usize = 0x8BC405;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".drawn",
            len: 0x60,
        },
        Run {
            at: 0x78,
            field: ".motion",
            len: 0x84,
        },
        Run {
            at: 0x100,
            field: ".drift",
            len: 0x4,
        },
        Run {
            at: 0x108,
            field: ".drops",
            len: 0x1804,
        },
        Run {
            at: 0x1924,
            field: ".edge_a",
            len: 0x8,
        },
        Run {
            at: 0x1930,
            field: ".edge_b",
            len: 0x8,
        },
        Run {
            at: 0x193C,
            field: ".ends",
            len: 0x18,
        },
        Run {
            at: 0x1958,
            field: ".depth",
            len: 0x50,
        },
        Run {
            at: 0x1A38,
            field: ".turned",
            len: 0x18,
        },
    ];
}

impl Laid for RingWork {
    const START: usize = 0x935620;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".list",
            len: 0x10,
        },
        Run {
            at: 0x40,
            field: ".cursor",
            len: 0x4,
        },
        Run {
            at: 0x48,
            field: ".block",
            len: 0x4,
        },
        Run {
            at: 0x50,
            field: ".top",
            len: 0x4,
        },
        Run {
            at: 0x58,
            field: ".fade",
            len: 0x24,
        },
        Run {
            at: 0x90,
            field: ".rate",
            len: 0x8,
        },
        Run {
            at: 0x9C,
            field: ".near",
            len: 0x8,
        },
        Run {
            at: 0xA8,
            field: ".count",
            len: 0x4,
        },
        Run {
            at: 0xF8,
            field: ".away",
            len: 0x18,
        },
    ];
}

impl Laid for MarksClock {
    const START: usize = 0x764FC8;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        field: ".started",
        len: 0x30,
    }];
}

impl Laid for SnowTables {
    const START: usize = 0x8C6000;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        field: ".randoms",
        len: 0xC000,
    }];
}

impl Laid for SnowState {
    const START: usize = 0x8D2000;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x30,
            field: ".wide_turn",
            len: 0x124,
        },
        Run {
            at: 0x15C,
            field: ".flag",
            len: 0x34,
        },
        Run {
            at: 0x194,
            field: ".fall",
            len: 0x4,
        },
        Run {
            at: 0x19C,
            field: ".drawn",
            len: 0xC,
        },
        Run {
            at: 0x1F4,
            field: ".depth",
            len: 0x8,
        },
        Run {
            at: 0x208,
            field: ".point",
            len: 0x3C,
        },
        Run {
            at: 0x2D4,
            field: ".turned",
            len: 0xC,
        },
    ];
}

impl Laid for SnowCurveVars {
    const START: usize = 0x8DA410;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".curve_vars",
            len: 0x1C,
        },
        Run {
            at: 0x20,
            field: ".records",
            len: 0x1000,
        },
    ];
}

impl Laid for SnowLight {
    const START: usize = 0x8DB430;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".light_place",
            len: 0x110,
        },
        Run {
            at: 0x118,
            field: ".side",
            len: 0x10,
        },
    ];
}

impl Laid for ShapeTilt {
    const START: usize = 0x811BC3;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        field: ".tilt",
        len: 0x28,
    }];
}

impl Laid for MarksFloats {
    const START: usize = 0x765140;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".pull_limit",
            len: 0x4,
        },
        Run {
            at: 0xC,
            field: ".burst_direction_k",
            len: 0x8,
        },
        Run {
            at: 0x18,
            field: ".half",
            len: 0x8,
        },
        Run {
            at: 0x24,
            field: ".fifth",
            len: 0xC,
        },
        Run {
            at: 0x34,
            field: ".three",
            len: 0x4,
        },
        Run {
            at: 0x4C,
            field: ".sixteen",
            len: 0x8,
        },
        Run {
            at: 0x58,
            field: ".hundred_twenty_eight",
            len: 0x10,
        },
        Run {
            at: 0x7C,
            field: ".quarter",
            len: 0x1C,
        },
        Run {
            at: 0x9C,
            field: ".fixed_16_scale",
            len: 0x4,
        },
        Run {
            at: 0xA4,
            field: ".seven_hundred",
            len: 0x8,
        },
        Run {
            at: 0xB0,
            field: ".two_thousand",
            len: 0x4,
        },
        Run {
            at: 0xD4,
            field: ".over_65536",
            len: 0x4,
        },
        Run {
            at: 0xDC,
            field: ".spread_scale",
            len: 0x8,
        },
    ];
}

impl Laid for MarksKindFloats {
    const START: usize = 0x765238;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        field: ".other",
        len: 0x6C,
    }];
}

impl Laid for KeptView {
    const START: usize = 0x769236;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".first_sine_cosine",
            len: 0x28,
        },
        Run {
            at: 0x34,
            field: ".kept_a",
            len: 0x1C,
        },
        Run {
            at: 0x60,
            field: ".last",
            len: 0x44,
        },
        Run {
            at: 0xB4,
            field: ".step",
            len: 0x28,
        },
    ];
}

impl Laid for WindscreenDropState {
    const START: usize = 0x814710;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".live",
            len: 0x20,
        },
        Run {
            at: 0x24,
            field: ".depth",
            len: 0x4,
        },
    ];
}

impl Laid for WindscreenDropWork {
    const START: usize = 0x814772;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".drift",
            len: 0x8,
        },
        Run {
            at: 0x50,
            field: ".curve",
            len: 0x47C,
        },
        Run {
            at: 0x4FC,
            field: ".setup_axes",
            len: 0x38,
        },
        Run {
            at: 0x538,
            field: ".cell",
            len: 0x4E0,
        },
    ];
}

impl Laid for WindscreenDropVars {
    const START: usize = 0x816192;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".scale",
            len: 0x2C,
        },
        Run {
            at: 0x44,
            field: ".kept_place",
            len: 0x30,
        },
        Run {
            at: 0x8C,
            field: ".drift",
            len: 0x18,
        },
        Run {
            at: 0xA8,
            field: ".windscreen_flakes_cleared",
            len: 0x18044,
        },
        Run {
            at: 0x180F4,
            field: ".axes_k",
            len: 0x18,
        },
        Run {
            at: 0x18110,
            field: ".side",
            len: 0x18,
        },
        Run {
            at: 0x18134,
            field: ".turn",
            len: 0x24,
        },
        Run {
            at: 0x18164,
            field: ".adjugate",
            len: 0x24,
        },
        Run {
            at: 0x18194,
            field: ".decay",
            len: 0x1C,
        },
    ];
}

impl Laid for WindscreenFlakeTiming {
    const START: usize = 0x89CA30;
    const RUNS: &'static [Run] = &[Run {
        at: 0x0,
        field: ".frames",
        len: 0xC,
    }];
}

impl Laid for WindscreenFlakeTables {
    const START: usize = 0x89CACA;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".curve",
            len: 0x438,
        },
        Run {
            at: 0x450,
            field: ".axes",
            len: 0x6C,
        },
        Run {
            at: 0x4C8,
            field: ".corners",
            len: 0x30,
        },
        Run {
            at: 0x51C,
            field: ".points",
            len: 0x184,
        },
    ];
}

impl Laid for WindscreenFlakeState {
    const START: usize = 0x8BC18A;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            field: ".unused_done",
            len: 0xD0,
        },
        Run {
            at: 0xD4,
            field: ".scaled_setting",
            len: 0x8,
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runs are the whole store, in order, with nothing between them: each begins where
    /// the last ended, at its field, and the last ends where the cells do.
    fn covered<T: Laid>() {
        let name = std::any::type_name::<T>();
        let store = zeroed::<T>();
        let mut next = 0;
        for r in T::RUNS {
            assert_eq!(
                field_start(&*store, r.field),
                Some(next),
                "{name} {}",
                r.field
            );
            next += r.len;
        }
        assert_eq!(next, field_range(&*store, "", usize::MAX).len(), "{name}");
    }

    #[test]
    fn runs_cover_the_stores() {
        covered::<CornerSlopes>();
        covered::<QuadWork>();
        covered::<DrawWork>();
        covered::<ViewVars>();
        covered::<MaskBlocks>();
        covered::<PictureCopy>();
        covered::<PongState>();
        covered::<PongRules>();
        covered::<RainState>();
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
