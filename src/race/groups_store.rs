//! Race-block pieces that left the block together (2026-10-29, phase 13): the census of
//! accesses by address saw the original reach across them, so each group keeps its
//! pieces and the bytes between them in the image's layout, one static with the image's
//! first values. Each piece's store is a field here.
//!
//! - `VIEWER_AND_KINDS` (IDA 0x5C291C .. 0x5C5CE0): the viewer's panel, the block's bytes
//!   after it, the kinds' tables. The effects read the marks' settings as one record from
//!   `ViewerAndKinds.marks_settings.settings_text` (in the panel) to +0x26A8 (in the kinds' tables: `effects::init_marks`).
//! - `LIGHTS_AND_GEAR` (IDA 0x5C906C .. 0x5CC198): the light records, a dword, the gear
//!   figures. `scene::load_object` reads a kind's record past the 34 (an unbounded kind,
//!   as the original) into the gear figures.
//! - `PANEL_AND_OVERLAY` (IDA 0x6D13FE .. 0x6D1870): the hud's panel and the race's
//!   overlay. The hud reads its records from `PANEL.split_6` on (+0xF4 at the panel's end:
//!   `hud::draw_hud`'s layouts). Two lead bytes come before the panel.
//! - `VIEW_AND_RECORDS` (IDA 0x60626C .. 0x6077B0): the view variables and the 52 view
//!   records of 0x5C after them (0x606500 .., `ViewAndRecords.view_vars.view_records[0]`'s targets). Until 2026-11-05 the
//!   variables ran on to 0x60674C over the first records' fields, and record 6 (0x606728)
//!   crossed their end (seen 2026-10-29: the third chase view read half its record from the
//!   wrong memory while the variables had moved alone); the records are now one array.

use crate::physics::gear_store::GearFigures;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::kind_store::KindTables;
use crate::race::light_records_store::LightRecords;
use crate::race::overlay_store::OverlayBlock;
#[cfg(test)]
use crate::race::owner::race;
use crate::race::panel_store::Panel;
use crate::race::view_vars_store::{ViewRecord, ViewVars};
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::session::{GameThread, PlainBytes, PlainCells, PlainDword};

/// IDA 0x5C291C .. 0x5C5CE0.
#[derive(rc2k_derive::Cells)]
pub struct ViewerAndKinds {
    /// +0x0 (0x5C291C): `race::viewer_screen_store`.
    pub viewer_panel: GameThread<ViewerPanelBlock>,
    /// +0x668 (0x5C2F84): `race::effects_settings_store`.
    pub marks_settings: GameThread<EffectsSettings>,
    /// +0x2DD0 (0x5C5708): `race::kind_store`.
    pub kind_tables: KindTables,
}

/// IDA 0x5C906C .. 0x5CC198. A drawing store (`session::DrawShared`): the light records and
/// the dword between on plain cells; the gear figures, which the drawing reaches only by
/// reading on past the records, on the physics' cells.
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
#[derive(rc2k_derive::Cells)]
pub struct PanelAndOverlay {
    /// Two lead bytes (not the game's) before the panel.
    pub lead: PlainBytes<2>,
    /// +0x0 (0x6D13FE): `race::panel_store`.
    pub panel: Panel,
    /// +0x10E (0x6D150C): `race::overlay_store`.
    pub overlay: OverlayBlock,
}

/// IDA 0x60626C .. 0x6077B0.
#[derive(rc2k_derive::Cells)]
pub struct ViewAndRecords {
    /// +0x0 (0x60626C): `race::view_vars_store`.
    pub view_vars: ViewVars,
    /// +0x294 (0x606500): the 52 view records (named through `off_606320`).
    pub records: [ViewRecord; 52],
}

// SAFETY (the groups): every field is a cell over a plain integer (`session`'s cells and
// records of them): any bit pattern is a valid value.

/// The viewer's panel, the marks' settings and the kinds' tables as the game starts.
impl ViewerAndKinds {
    pub const fn new() -> Self {
        ViewerAndKinds {
            viewer_panel: GameThread::new(ViewerPanelBlock::new()),
            marks_settings: GameThread::new(EffectsSettings::new()),
            kind_tables: KindTables::new(),
        }
    }
}

/// The light records and the gear figures as the game starts.
impl LightsAndGear {
    pub const fn new() -> Self {
        LightsAndGear {
            light_records: LightRecords::new(),
            between: PlainDword::new(0),
            gear: GearFigures::new(),
        }
    }
}

/// The hud's panel and the race's overlay as the game starts.
impl PanelAndOverlay {
    pub const fn new() -> Self {
        PanelAndOverlay {
            lead: PlainBytes::new([0; 2]),
            panel: Panel::new(),
            overlay: OverlayBlock::new(),
        }
    }
}

/// The view variables and the 52 view records as the game starts.
impl ViewAndRecords {
    pub const fn new() -> Self {
        ViewAndRecords {
            view_vars: ViewVars::<PlainCells>::new(),
            records: ViewRecord::first_records(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_have_their_homes() {
        let race = race();
        for home in [
            core::ptr::from_ref(&race.replay.viewer_and_kinds).addr(),
            core::ptr::from_ref(race.frame.lights_and_gear.get()).addr(),
            core::ptr::from_ref(&race.frame.panel_and_overlay.get().panel).addr(),
            core::ptr::from_ref(race.frame.view_and_records.get()).addr(),
        ] {
            assert_eq!(home % 4, 0);
        }
    }
}
