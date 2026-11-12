//! Race-block pieces that left the block together (2026-10-29, phase 13): the census of
//! accesses by address saw the original reach across them, so each group keeps its
//! pieces and the bytes between them in the image's layout, one static with the image's
//! first values. Each piece's store is a field here.
//!
//! - `CARS_AND_REPLAY` (IDA 0x70F3EC .. 0x719FD8): the cars' blocks and the replay's work.
//!   The replay's state is 0xA04 dwords from car block 8 (0x7172EC), running into the
//!   replay's work (`drive::flush_replay`, `drive::load_replay_frame`).
//! - `VIEWER_AND_KINDS` (IDA 0x5C291C .. 0x5C5CE0): the viewer's panel, the block's bytes
//!   after it, the kinds' tables. The effects read the marks' settings as one record from
//!   `ViewerAndKinds.marks_settings.settings_text` (in the panel) to +0x26A8 (in the kinds' tables: `effects::init_marks`).
//! - `LIGHTS_AND_GEAR` (IDA 0x5C906C .. 0x5CC198): the light records, a dword, the gear
//!   figures. `scene::load_object` reads a kind's record past the 34 (an unbounded kind,
//!   as the original) into the gear figures.
//! - `PANEL_AND_OVERLAY` (IDA 0x6D13FE .. 0x6D1870): the hud's panel and the race's
//!   overlay. The hud reads its records from `PANEL.split_6` on (+0xF4 at the panel's end:
//!   `hud::draw_hud`'s layouts). The panel starts 2 bytes past a dword: two lead bytes keep
//!   the overlay's dwords aligned (the group's home is 2 bytes in).
//! - `VIEW_AND_RECORDS` (IDA 0x60626C .. 0x6077B0): the view variables and the 52 view
//!   records of 0x5C after them (0x606500 .., `ViewAndRecords.view_vars.view_records[0]`'s targets). Until 2026-11-05 the
//!   variables ran on to 0x60674C over the first records' fields, and record 6 (0x606728)
//!   crossed their end (seen 2026-10-29: the third chase view read half its record from the
//!   wrong memory while the variables had moved alone); the records are now one array.

use crate::physics::gear_store::GearFigures;
use crate::race::blocks_store::CarBlocks;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::kind_store::KindTables;
use crate::race::light_records_store::LightRecords;
use crate::race::overlay_store::OverlayBlock;
#[cfg(test)]
use crate::race::owner::race;
use crate::race::panel_store::Panel;
use crate::race::replay_work_store::ReplayWork;
use crate::race::view_vars_store::{ViewRecord, ViewVars};
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::session::{Bytes, PlainDword};
use core::mem::{offset_of, size_of};

/// IDA 0x70F3EC .. 0x719FD8.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CarsAndReplay {
    /// +0x0 (0x70F3EC): `race::blocks_store`.
    pub car_blocks: CarBlocks,
    /// +0xA0D0 (0x7194BC): `race::replay_work_store`.
    pub replay_work: ReplayWork,
}

/// IDA 0x5C291C .. 0x5C5CE0.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewerAndKinds {
    /// +0x0 (0x5C291C): `race::viewer_screen_store`.
    pub viewer_panel: ViewerPanelBlock,
    /// +0x668 (0x5C2F84): `race::effects_settings_store`.
    pub marks_settings: EffectsSettings,
    /// +0x2DD0 (0x5C56EC): `race::kind_store`.
    pub kind_tables: KindTables,
}

/// IDA 0x5C906C .. 0x5CC198. A drawing store (`session::DrawShared`): the light records and
/// the dword between on plain cells; the gear figures, which the drawing reaches only by
/// reading on past the records, on the physics' cells.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LightsAndGear {
    /// +0x0 (0x5C906C): `race::light_records_store`.
    pub light_records: LightRecords,
    /// +0x28E8 (0x5CB954): a dword between them, by IDA address.
    pub between: PlainDword,
    /// +0x28EC (0x5CB958): `physics::gear_store`.
    pub gear: GearFigures,
}

/// IDA 0x6D13FE .. 0x6D1870, two bytes in.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PanelAndOverlay {
    /// Two lead bytes (not the game's) aligning the overlay.
    pub lead: Bytes<2>,
    /// +0x0 (0x6D13FE): `race::panel_store`.
    pub panel: Panel,
    /// +0x10E (0x6D150C): `race::overlay_store`.
    pub overlay: OverlayBlock,
}

/// IDA 0x60626C .. 0x6077B0.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewAndRecords {
    /// +0x0 (0x60626C): `race::view_vars_store`.
    pub view_vars: ViewVars,
    /// +0x294 (0x606500): the 52 view records (named through `off_606320`).
    pub records: [ViewRecord; 52],
}

/// The groups' IDA ranges, in order.
pub const CARS_AND_REPLAY_RANGE: (usize, usize) = (0x70F3EC, 0x719FD8);
pub const VIEWER_AND_KINDS_RANGE: (usize, usize) = (0x5C291C, 0x5C5CE0);
pub const LIGHTS_AND_GEAR_RANGE: (usize, usize) = (0x5C906C, 0x5CC198);
pub const PANEL_AND_OVERLAY_RANGE: (usize, usize) = (0x6D13FE, 0x6D1870);
pub const VIEW_AND_RECORDS_RANGE: (usize, usize) = (0x60626C, 0x6077B0);

const _: () = {
    use crate::{physics, race};
    assert!(size_of::<CarsAndReplay>() == CARS_AND_REPLAY_RANGE.1 - CARS_AND_REPLAY_RANGE.0);
    assert!(
        offset_of!(CarsAndReplay, replay_work)
            == race::replay_work_store::START - race::blocks_store::START
    );
    assert!(race::blocks_store::START == CARS_AND_REPLAY_RANGE.0);
    assert!(race::replay_work_store::END == CARS_AND_REPLAY_RANGE.1);
    assert!(size_of::<ViewerAndKinds>() == VIEWER_AND_KINDS_RANGE.1 - VIEWER_AND_KINDS_RANGE.0);
    assert!(
        offset_of!(ViewerAndKinds, marks_settings)
            == race::effects_settings_store::START - race::viewer_screen_store::START
    );
    assert!(race::viewer_screen_store::END == race::effects_settings_store::START);
    assert!(race::effects_settings_store::END == race::kind_store::START);
    assert!(
        offset_of!(ViewerAndKinds, kind_tables)
            == race::kind_store::START - race::viewer_screen_store::START
    );
    assert!(race::viewer_screen_store::START == VIEWER_AND_KINDS_RANGE.0);
    assert!(race::kind_store::END == VIEWER_AND_KINDS_RANGE.1);
    assert!(size_of::<LightsAndGear>() == LIGHTS_AND_GEAR_RANGE.1 - LIGHTS_AND_GEAR_RANGE.0);
    assert!(
        offset_of!(LightsAndGear, gear)
            == physics::gear_store::START - race::light_records_store::START
    );
    assert!(race::light_records_store::START == LIGHTS_AND_GEAR_RANGE.0);
    assert!(physics::gear_store::END == LIGHTS_AND_GEAR_RANGE.1);
    assert!(
        size_of::<PanelAndOverlay>() == 2 + PANEL_AND_OVERLAY_RANGE.1 - PANEL_AND_OVERLAY_RANGE.0
    );
    assert!(offset_of!(PanelAndOverlay, panel) == 2);
    assert!(
        offset_of!(PanelAndOverlay, overlay)
            == 2 + race::overlay_store::START - race::panel_store::START
    );
    assert!(race::panel_store::START == PANEL_AND_OVERLAY_RANGE.0);
    assert!(race::overlay_store::END == PANEL_AND_OVERLAY_RANGE.1);
    assert!(size_of::<ViewAndRecords>() == VIEW_AND_RECORDS_RANGE.1 - VIEW_AND_RECORDS_RANGE.0);
    assert!(
        offset_of!(ViewAndRecords, records)
            == race::view_vars_store::END - race::view_vars_store::START
    );
    assert!(race::view_vars_store::START == VIEW_AND_RECORDS_RANGE.0);
};

// SAFETY (the three): every field is a cell over a plain integer (`session`'s cells and
// records of them): any bit pattern is a valid value.

/// The cars' blocks and the replay's work, with the image's first values.
impl CarsAndReplay {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<
            { CARS_AND_REPLAY_RANGE.1 - CARS_AND_REPLAY_RANGE.0 },
        >(CARS_AND_REPLAY_RANGE.0))
    }
}
/// The viewer's panel .. the kinds' tables, with the image's first values.
impl ViewerAndKinds {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<
            { VIEWER_AND_KINDS_RANGE.1 - VIEWER_AND_KINDS_RANGE.0 },
        >(VIEWER_AND_KINDS_RANGE.0))
    }
}
/// The light records and the gear figures, with the image's first values.
impl LightsAndGear {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<
            { LIGHTS_AND_GEAR_RANGE.1 - LIGHTS_AND_GEAR_RANGE.0 },
        >(LIGHTS_AND_GEAR_RANGE.0))
    }
}

/// The hud's panel and the race's overlay, with the image's first values (the two lead
/// bytes the block's before the panel).
impl PanelAndOverlay {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<
            { 2 + PANEL_AND_OVERLAY_RANGE.1 - PANEL_AND_OVERLAY_RANGE.0 },
        >(PANEL_AND_OVERLAY_RANGE.0 - 2))
    }
}

/// The view variables and the view records, with the image's first values.
impl ViewAndRecords {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::view_vars_store::with_view_refs(
            crate::race::store::first_values::<
                { VIEW_AND_RECORDS_RANGE.1 - VIEW_AND_RECORDS_RANGE.0 },
            >(VIEW_AND_RECORDS_RANGE.0),
            VIEW_AND_RECORDS_RANGE.0,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        use crate::test_support::first_values::{
            assert_block_starts_as_the_games, assert_store_starts_as_the_games,
        };
        let race = race();
        assert_block_starts_as_the_games(&race.replay.cars_and_replay);
        assert_block_starts_as_the_games(&race.replay.viewer_and_kinds);
        assert_block_starts_as_the_games(race.frame.lights_and_gear.get());
        crate::test_support::first_values::assert_block_starts_as_the_games_but(
            &race.frame.view_and_records,
            &[(
                offset_of!(ViewAndRecords, view_vars) + offset_of!(ViewVars, view_records),
                "race.owner.RACE.frame.view_and_records.view_vars.view_records[0]/208",
            )],
        );
        // The panel and the overlay after it: one store from the panel's first byte.
        assert_store_starts_as_the_games("store:race.groups_store.PANEL_TAIL");
    }

    #[test]
    fn groups_have_their_homes() {
        let race = race();
        for home in [
            core::ptr::from_ref(&race.replay.cars_and_replay).addr(),
            core::ptr::from_ref(&race.replay.viewer_and_kinds).addr(),
            core::ptr::from_ref(race.frame.lights_and_gear.get()).addr(),
            core::ptr::from_ref(&race.frame.panel_and_overlay.panel).addr() + 2,
            core::ptr::from_ref(&race.frame.view_and_records).addr(),
        ] {
            assert_eq!(home % 4, 0);
        }
    }
}
