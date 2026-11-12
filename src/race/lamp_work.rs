//! A view's own lamps and flares (frame plan F7): the sixteen lamps the view gathers, the
//! flares' six tables, the counts and the lamps wanted. One per view slot
//! (`race::view_scratch::ViewScratch::lamps`). In the original these were the level grid's
//! `g_lamps` (0x63E05C), `flare_place_x` (0x63F18C) and the other five flare tables
//! (`race::flares_store`, 0x63F58C ..), the race view's `g_lampsWanted`
//! (0x63DB58) and `g_lampCount` (0x63DC48) and the frame work's `g_flareCount` (0x64158C),
//! shared by every view in turn. Each race view starts from the lamps the one before it left
//! (`PUBLISHED`). The original also counted the lamps' depth entries (`g_lampEntryCount`,
//! 0x641590), which nothing read.

use crate::race::level_grid_store::{LevelGrid, first_of};
use crate::race::view_store::RaceView;
use crate::session::Dword;
use core::mem::offset_of;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LampWork {
    /// `g_lamps`: 16 lamps of 0x10 bytes (+0 the point, +4 the strength, +8 the colour), each
    /// view's own.
    pub lamps: [Dword; 64],
    /// The flares' places' x (`LEVEL_GRID.flare_place_x`), each view's own.
    pub flare_place_x: [Dword; 256],
    /// The flares' y, z, colour, size and object number (`race::flares_store::FlareTables`),
    /// each view's own.
    pub flares: crate::race::flares_store::FlareTables,
    /// `g_lampsWanted` (`RACE_VIEW.lamps_wanted`), each view's own.
    pub lamps_wanted: Dword,
    /// `g_lampCount` (`RACE_VIEW.lamp_count`), each view's own.
    pub lamp_count: Dword,
    /// `g_flareCount` (`FRAME_WORK.flare_count`), each view's own.
    pub flare_count: Dword,
}

/// The image's first value of the race view's dword at `at`: what the view's lamps start from.
const fn first_dword(at: usize) -> Dword {
    Dword::new(u32::from_le_bytes(crate::race::store::first_values::<4>(
        crate::race::view_store::START + at,
    )))
}

impl LampWork {
    /// With the image's first values.
    pub const fn new() -> Self {
        let lamps_wanted = first_dword(offset_of!(RaceView, lamps_wanted));
        let lamp_count = first_dword(offset_of!(RaceView, lamp_count));
        let flare_count = Dword::new(0);
        Self {
            lamps: first_of::<_, 0x100>(offset_of!(LevelGrid, lamps)),
            flare_place_x: first_of::<_, 0x400>(offset_of!(LevelGrid, flare_place_x)),
            flares: crate::race::flares_store::FlareTables::new(),
            lamps_wanted,
            lamp_count,
            flare_count,
        }
    }
}

/// The lamps the last race view left (`ViewScratch::begin`): each view starts from them and
/// leaves its own at its end (`draw_view_scene`). The original's lamps went from view to view in
/// one store, and a view reads the previous one's before it gathers its own: its view light
/// (`race::view_light`, which the effects tint by) and what it draws before (the sky).
pub static PUBLISHED: LampWork = LampWork::new();
