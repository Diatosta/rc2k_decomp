//! The marks' paths and shapes state in one group (step 11, owned state): the splines'
//! stores, the waves, the profiles, the morph, spread and line work, and the block points.
//! It is the `paths` field of `effects::owner::Effects`.

use crate::effects::lines_store::LinesWork;
use crate::effects::morph_object_points_store::MorphWork;
use crate::effects::profiles_store::{ProfileRings, ProfileTables};
use crate::effects::splines_store::{
    SplineConstants, SplineHead, SplineKinds, SplinePath, SplineTables, SplineWork,
};
use crate::effects::spread_point_store::SpreadRandom;
use crate::effects::waves_store::{MarksSines, WaveLevels, WaveState};
use crate::memory::owned::Owned;

/// The paths' state.
pub struct Paths {
    pub spline_constants: SplineConstants,
    pub spline_kinds: SplineKinds,
    pub spline_path: SplinePath,
    pub spline_tables: SplineTables,
    pub spline_work: SplineWork,
    pub spline_head: SplineHead,
    pub wave: WaveState,
    pub wave_levels: WaveLevels,
    pub sines: MarksSines,
    pub profile_rings: ProfileRings,
    pub profile_tables: ProfileTables,
    pub morph_work: MorphWork,
    pub spread_random: SpreadRandom,
    pub lines_work: LinesWork,
    /// The points of the eight blocks (Rust's memory, not a place of the image).
    pub block_points: [Owned<Vec<[u32; 3]>>; 8],
}

impl Paths {
    pub const fn new() -> Self {
        Paths {
            spline_constants: SplineConstants::new(),
            spline_kinds: SplineKinds::new(),
            spline_path: SplinePath::new(),
            spline_tables: SplineTables::new(),
            spline_work: SplineWork::new(),
            spline_head: SplineHead::new(),
            wave: WaveState::new(),
            wave_levels: WaveLevels::new(),
            sines: MarksSines::new(),
            profile_rings: ProfileRings::new(),
            profile_tables: ProfileTables::new(),
            morph_work: MorphWork::new(),
            spread_random: SpreadRandom::new(),
            lines_work: LinesWork::new(),
            block_points: [const { Owned::new("no block points") }; 8],
        }
    }
}
