//! A view's own lamps and flares (frame plan F7): the sixteen lamps the view gathers, the
//! flares' six tables, the counts and the lamps wanted. One per view slot
//! (`race::view_scratch::ViewScratch::lamps`). In the original these were the level grid's
//! `g_lamps` (0x63E05C), `flare_place_x` (0x63F18C) and the other five flare tables
//! (`race::flares_store`, 0x63F58C ..), the race view's `g_lampsWanted`
//! (0x63DB58) and `g_lampCount` (0x63DC48) and the frame work's `g_flareCount` (0x64158C),
//! shared by every view in turn. Each race view starts from the lamps the one before it left
//! (`PUBLISHED`). The original also counted the lamps' depth entries (`g_lampEntryCount`,
//! 0x641590), which nothing read.

use crate::cells::ZeroCells;
use crate::session::Dword;

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
    pub flares: crate::race::flares_store::ViewFlares,
    /// `g_lampsWanted` (`RACE_VIEW.lamps_wanted`), each view's own.
    pub lamps_wanted: Dword,
    /// `g_lampCount` (`RACE_VIEW.lamp_count`), each view's own.
    pub lamp_count: Dword,
    /// `g_flareCount` (`FRAME_WORK.flare_count`), each view's own.
    pub flare_count: Dword,
}

impl LampWork {
    /// A view's lamps as the game starts them: none gathered or wanted, no flares.
    pub const fn new() -> Self {
        Self {
            lamps: ZeroCells::ZERO,
            flare_place_x: ZeroCells::ZERO,
            flares: crate::race::flares_store::ViewFlares::new(),
            lamps_wanted: Dword::ZERO,
            lamp_count: Dword::ZERO,
            flare_count: Dword::ZERO,
        }
    }
}

/// The lamps the last race view left (`ViewScratch::begin`): each view starts from them and
/// leaves its own at its end (`draw_view_scene`). The original's lamps went from view to view in
/// one store, and a view reads the previous one's before it gathers its own: its view light
/// (`race::view_light`, which the effects tint by) and what it draws before (the sky).
///
/// Shared between the parts drawn beside each other: every race view's scene publishes at its
/// end and every one reads at its start, so with two players beside each other a view starts
/// from the lamps of the last scene to end, whichever that was. Atomic cells, copied field by
/// field (`publish`, `take_into`) as `LampWork`'s were.
pub struct PublishedLamps {
    lamps: [Dword; 64],
    flare_place_x: [Dword; 256],
    place_y: [Dword; 256],
    place_z: [Dword; 256],
    other_light: [Dword; 256],
    size: [Dword; 256],
    object_number: [Dword; 256],
    lamps_wanted: Dword,
    lamp_count: Dword,
    flare_count: Dword,
}

/// `to` set from `from`, cell by cell.
fn copy_cells(to: &[Dword], from: &[Dword]) {
    for (to, from) in to.iter().zip(from) {
        to.set(from.get());
    }
}

impl PublishedLamps {
    const fn new() -> Self {
        Self {
            lamps: ZeroCells::ZERO,
            flare_place_x: ZeroCells::ZERO,
            place_y: ZeroCells::ZERO,
            place_z: ZeroCells::ZERO,
            other_light: ZeroCells::ZERO,
            size: ZeroCells::ZERO,
            object_number: ZeroCells::ZERO,
            lamps_wanted: Dword::ZERO,
            lamp_count: Dword::ZERO,
            flare_count: Dword::ZERO,
        }
    }

    /// `lamps`, a view's at its end, as the lamps the next view starts from.
    pub fn publish(&self, lamps: &LampWork) {
        copy_cells(&self.lamps, &lamps.lamps);
        copy_cells(&self.flare_place_x, &lamps.flare_place_x);
        copy_cells(&self.place_y, &lamps.flares.place_y);
        copy_cells(&self.place_z, &lamps.flares.place_z);
        copy_cells(&self.other_light, &lamps.flares.other_light);
        copy_cells(&self.size, &lamps.flares.size);
        copy_cells(&self.object_number, &lamps.flares.object_number);
        self.lamps_wanted.set(lamps.lamps_wanted.get());
        self.lamp_count.set(lamps.lamp_count.get());
        self.flare_count.set(lamps.flare_count.get());
    }

    /// The published lamps into a view's, as its drawing begins.
    pub fn take_into(&self, lamps: &LampWork) {
        copy_cells(&lamps.lamps, &self.lamps);
        copy_cells(&lamps.flare_place_x, &self.flare_place_x);
        copy_cells(&lamps.flares.place_y, &self.place_y);
        copy_cells(&lamps.flares.place_z, &self.place_z);
        copy_cells(&lamps.flares.other_light, &self.other_light);
        copy_cells(&lamps.flares.size, &self.size);
        copy_cells(&lamps.flares.object_number, &self.object_number);
        lamps.lamps_wanted.set(self.lamps_wanted.get());
        lamps.lamp_count.set(self.lamp_count.get());
        lamps.flare_count.set(self.flare_count.get());
    }
}

pub static PUBLISHED: PublishedLamps = PublishedLamps::new();
