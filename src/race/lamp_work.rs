//! A view's own lamps and flares (frame plan F7): the sixteen lamps the view gathers, the
//! flares' six tables, the counts and the lamps wanted. One per view slot
//! (`race::view_scratch::ViewScratch::lamps`). In the original these were the level grid's
//! `g_lamps` (0x63E05C), `flare_place_x` (0x63F18C) and the other five flare tables
//! (`race::flares_store`, 0x63F58C ..), the race view's `g_lampsWanted`
//! (0x63DB58) and `g_lampCount` (0x63DC48) and the frame work's `g_flareCount` (0x64158C),
//! shared by every view in turn. A race view now keeps its own lamps from frame to frame: the
//! lamps it starts a drawing from are the ones its own last drawing left (`ViewScratch::begin`
//! leaves them alone), where the original's went on from the previous view drawn. The original
//! also counted the lamps' depth entries (`g_lampEntryCount`,
//! 0x641590), which nothing read.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells)]
pub struct LampWork {
    /// `g_lamps`: 16 lamps of 0x10 bytes (+0 the point, +4 the strength, +8 the colour), each
    /// view's own.
    pub lamps: [PlainDword; 64],
    /// The flares' places' x (`LEVEL_GRID.flare_place_x`), each view's own.
    pub flare_place_x: [PlainDword; 256],
    /// The flares' y, z, colour, size and object number (`race::flares_store::FlareTables`),
    /// each view's own.
    pub flares: crate::race::flares_store::ViewFlares,
    /// `g_lampsWanted` (`RACE_VIEW.lamps_wanted`), each view's own.
    pub lamps_wanted: PlainDword,
    /// `g_lampCount` (`RACE_VIEW.lamp_count`), each view's own.
    pub lamp_count: PlainDword,
    /// `g_flareCount` (`FRAME_WORK.flare_count`), each view's own.
    pub flare_count: PlainDword,
}

impl LampWork {
    /// A view's lamps as the game starts them: none gathered or wanted, no flares.
    pub const fn new() -> Self {
        Self {
            lamps: ZeroCells::ZERO,
            flare_place_x: ZeroCells::ZERO,
            flares: crate::race::flares_store::ViewFlares::new(),
            lamps_wanted: PlainDword::ZERO,
            lamp_count: PlainDword::ZERO,
            flare_count: PlainDword::ZERO,
        }
    }
}
