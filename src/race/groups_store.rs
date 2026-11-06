//! Race-block pieces that left the block together (2026-10-29, phase 13): the census of
//! accesses by address saw the original reach across them, so each group keeps its
//! pieces and the bytes between them in the image's layout, one static with the image's
//! first values. Each piece's store is a field here (`race::store::LEFT` lists the groups).
//!
//! - `CARS_AND_REPLAY` (IDA 0x70F3EC .. 0x719FD8): the cars' blocks and the replay's work.
//!   The replay's state is 0xA04 dwords from car block 8 (0x7172EC), running into the
//!   replay's work (`drive::flush_replay`, `drive::load_replay_frame`).
//! - `VIEWER_AND_KINDS` (IDA 0x5C291C .. 0x5C5CE0): the viewer's panel, the block's bytes
//!   after it, the kinds' tables. The effects read the marks' settings as one record from
//!   `asc_5C2F84` (in the panel) to +0x26A8 (in the kinds' tables: `effects::init_marks`).
//! - `LIGHTS_AND_GEAR` (IDA 0x5C906C .. 0x5CC198): the light records, a dword, the gear
//!   figures. `scene::load_object` reads a kind's record past the 34 (an unbounded kind,
//!   as the original) into the gear figures.
//! - `PANEL_AND_OVERLAY` (IDA 0x6D13FE .. 0x6D1870): the hud's panel and the race's
//!   overlay. The hud reads its records from `PANEL.split_6` on (+0xF4 at the panel's end:
//!   `hud::draw_hud`'s layouts). The panel starts 2 bytes past a dword: two lead bytes keep
//!   the overlay's dwords aligned (the group's home is 2 bytes in).
//! - `VIEW_AND_RECORDS` (IDA 0x60626C .. 0x6077B0): the view variables and the 52 view
//!   records of 0x5C after them (0x606500 .., `off_606320`'s targets). Until 2026-11-05 the
//!   variables ran on to 0x60674C over the first records' fields, and record 6 (0x606728)
//!   crossed their end (seen 2026-10-29: the third chase view read half its record from the
//!   wrong memory while the variables had moved alone); the records are now one array.

use crate::physics::gear_store::GearFigures;
use crate::race::blocks_store::CarBlocks;
use crate::race::kind_store::KindTables;
use crate::race::light_records_store::LightRecords;
use crate::race::marks_settings_store::MarksSettings;
use crate::race::overlay_store::OverlayBlock;
#[cfg(test)]
use crate::race::owner::race;
use crate::race::panel_store::Panel;
use crate::race::replay_work_store::ReplayWork;
use crate::race::view_vars_store::{ViewRecord, ViewVars};
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::session::{Bytes, Dword};
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
    /// +0x668 (0x5C2F84): `race::marks_settings_store`.
    pub marks_settings: MarksSettings,
    /// +0x2DD0 (0x5C56EC): `race::kind_store`.
    pub kind_tables: KindTables,
}

/// IDA 0x5C906C .. 0x5CC198.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LightsAndGear {
    /// +0x0 (0x5C906C): `race::light_records_store`.
    pub light_records: LightRecords,
    /// +0x28E8 (0x5CB954): a dword between them, by IDA address.
    pub between: Dword,
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
            == race::marks_settings_store::START - race::viewer_screen_store::START
    );
    assert!(race::viewer_screen_store::END == race::marks_settings_store::START);
    assert!(race::marks_settings_store::END == race::kind_store::START);
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
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<
            { CARS_AND_REPLAY_RANGE.1 - CARS_AND_REPLAY_RANGE.0 },
        >(CARS_AND_REPLAY_RANGE.0))
    }
}
/// The viewer's panel .. the kinds' tables, with the image's first values.
impl ViewerAndKinds {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<
            { VIEWER_AND_KINDS_RANGE.1 - VIEWER_AND_KINDS_RANGE.0 },
        >(VIEWER_AND_KINDS_RANGE.0))
    }
}
/// The light records and the gear figures, with the image's first values.
impl LightsAndGear {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<
            { LIGHTS_AND_GEAR_RANGE.1 - LIGHTS_AND_GEAR_RANGE.0 },
        >(LIGHTS_AND_GEAR_RANGE.0))
    }
}

/// The hud's panel and the race's overlay, with the image's first values (the two lead
/// bytes the block's before the panel).
impl PanelAndOverlay {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<
            { 2 + PANEL_AND_OVERLAY_RANGE.1 - PANEL_AND_OVERLAY_RANGE.0 },
        >(PANEL_AND_OVERLAY_RANGE.0 - 2))
    }
}

/// The view variables and the view records, with the image's first values.
impl ViewAndRecords {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<
            { VIEW_AND_RECORDS_RANGE.1 - VIEW_AND_RECORDS_RANGE.0 },
        >(VIEW_AND_RECORDS_RANGE.0))
    }
}

/// The panel and the overlay without the two lead bytes: the group's home in `moved::MOVED`.
pub static PANEL_WINDOW: crate::race::store::Window<PanelAndOverlay> =
    crate::race::store::Window::new(&crate::race::owner::RACE.frame.panel_and_overlay, 2);

/// Where each group is (`moved::MOVED`'s homes).
#[cfg(test)]
pub fn cars_and_replay_home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::owner::race().replay.cars_and_replay,
    )) as usize
}
#[cfg(test)]
pub fn viewer_and_kinds_home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::owner::race().replay.viewer_and_kinds,
    )) as usize
}
#[cfg(test)]
pub fn lights_and_gear_home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(&race().frame.lights_and_gear)) as usize
}
#[cfg(test)]
pub fn view_and_records_home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(&race().frame.view_and_records)) as usize
}
#[cfg(test)]
pub fn panel_and_overlay_home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(&race().frame.panel_and_overlay.panel))
        as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        let (a, b) = CARS_AND_REPLAY_RANGE;
        crate::gfx::first_values_are_the_games(
            &crate::race::owner::race().replay.cars_and_replay,
            a,
            b,
        );
        let (a, b) = VIEWER_AND_KINDS_RANGE;
        crate::gfx::first_values_are_the_games(
            &crate::race::owner::race().replay.viewer_and_kinds,
            a,
            b,
        );
        let (a, b) = LIGHTS_AND_GEAR_RANGE;
        crate::gfx::first_values_are_the_games(&race().frame.lights_and_gear, a, b);
        let (a, b) = VIEW_AND_RECORDS_RANGE;
        crate::gfx::first_values_are_the_games(&race().frame.view_and_records, a, b);
        let (a, b) = PANEL_AND_OVERLAY_RANGE;
        crate::gfx::first_values_are_the_games(
            &race().frame.panel_and_overlay.panel,
            a,
            race_panel_end(),
        );
        crate::gfx::first_values_are_the_games(
            &race().frame.panel_and_overlay.overlay,
            race_panel_end(),
            b,
        );
    }

    fn race_panel_end() -> usize {
        crate::race::panel_store::END
    }

    #[test]
    fn groups_have_their_homes() {
        for (start, home) in [
            (CARS_AND_REPLAY_RANGE.0, cars_and_replay_home()),
            (VIEWER_AND_KINDS_RANGE.0, viewer_and_kinds_home()),
            (LIGHTS_AND_GEAR_RANGE.0, lights_and_gear_home()),
            (PANEL_AND_OVERLAY_RANGE.0 + 2, panel_and_overlay_home() + 2),
            (VIEW_AND_RECORDS_RANGE.0, view_and_records_home()),
        ] {
            assert_eq!(home % 4, 0);
            assert_eq!(crate::moved::place(start), home);
        }
        assert_eq!(
            crate::moved::place(0x5C562C),
            viewer_and_kinds_home() + (0x5C562C - VIEWER_AND_KINDS_RANGE.0)
        );
    }
}
