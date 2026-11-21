//! The marks' paths and shapes state in one group (step 11, owned state): the splines'
//! stores, the waves, the profiles, the morph, spread and line work, and the block points.
//! It is the `paths` field of `effects::owner::Effects`.

use crate::effects::lines_store::LinesWork;
use crate::effects::splines_store::{
    SplineConstants, SplineHead, SplineKinds, SplinePath, SplineTables, SplineWork,
};
use crate::effects::waves_store::WaveLevels;
use crate::memory::owned::Owned;

/// The paths' state.
pub struct Paths {
    pub spline_constants: SplineConstants,
    pub spline_kinds: core::cell::RefCell<SplineKinds>,
    pub spline_path: SplinePath,
    pub spline_tables: core::cell::RefCell<SplineTables>,
    pub spline_work: core::cell::RefCell<SplineWork>,
    pub spline_head: core::cell::RefCell<SplineHead>,
    pub wave_levels: core::cell::RefCell<WaveLevels>,
    /// The block last taken for a settings block's points (`lines::set_up_blocks`).
    pub morph_taken: core::cell::Cell<u32>,
    /// The random (times 0x9D plus 0x47) of `effects::spread_point`.
    pub spread_random: core::cell::Cell<u32>,
    pub lines_work: core::cell::RefCell<LinesWork>,
    /// The points of the eight blocks (Rust's memory, not a place of the image).
    pub block_points: [Owned<Vec<[u32; 3]>>; 8],
}

impl Paths {
    pub const fn new() -> Self {
        Paths {
            spline_constants: SplineConstants::new(),
            spline_kinds: core::cell::RefCell::new(SplineKinds::new()),
            spline_path: SplinePath::new(),
            spline_tables: core::cell::RefCell::new(SplineTables::new()),
            spline_work: core::cell::RefCell::new(SplineWork::new()),
            spline_head: core::cell::RefCell::new(SplineHead::new()),
            wave_levels: core::cell::RefCell::new(WaveLevels::new()),
            morph_taken: core::cell::Cell::new(0),
            spread_random: core::cell::Cell::new(0),
            lines_work: core::cell::RefCell::new(LinesWork::new()),
            block_points: [const { Owned::new("no block points") }; 8],
        }
    }
}
