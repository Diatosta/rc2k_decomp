//! Race-block pieces that sit together because code reads across them by address: each group keeps
//! its pieces and the bytes between them in a fixed layout, as one static with its first values.
//! Each piece's store is a field here.
//!
//! - `ViewerAndKinds`: the viewer's panel, the bytes after it, the kinds' tables. The effects read
//!   the marks' settings as one record from `ViewerAndKinds.marks_settings.settings_text` (in the
//!   panel) to +0x26A8 (in the kinds' tables: `effects::init_marks`).
//! - `LightsAndGear`: the light records, a dword, the gear figures. `scene::load_object` reads a
//!   kind's record past the 34 (an unbounded kind) into the gear figures.
//! - `PanelAndOverlay`: the hud's panel and the race's overlay. The hud reads its records from
//!   `PANEL.split_6` on (+0xF4 at the panel's end: `hud::draw_hud`'s layouts). Two lead bytes
//!   come before the panel.
//! - `ViewAndRecords`: the view variables and the 52 view records of 0x5C after them
//!   (`ViewAndRecords.view_vars.view_records[0]`'s targets). The records are one array: with the
//!   variables overrunning onto the first records' fields, the third chase view read half of
//!   record 6 from the wrong memory.

use crate::memory::session::{GameThread, PlainBytes, PlainCells, PlainDword};
use crate::physics::gear_store::GearFigures;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::kind_store::KindTables;
use crate::race::light_records_store::LightRecords;
use crate::race::overlay_store::OverlayBlock;
use crate::race::panel_store::Panel;
use crate::race::view_vars_store::{ViewRecord, ViewVars};
use crate::race::viewer_screen_store::ViewerPanelBlock;

/// The viewer's panel, the bytes after it and the kinds' tables.
#[derive(rc2k_derive::Cells)]
pub struct ViewerAndKinds {
    /// +0x0: `race::viewer_screen_store`.
    pub viewer_panel: GameThread<ViewerPanelBlock>,
    /// +0x668: `race::effects_settings_store`.
    pub marks_settings: GameThread<EffectsSettings>,
    /// +0x2DD0: `race::kind_store`.
    pub kind_tables: KindTables,
}

/// The light records, a dword and the gear figures. A drawing store (`session::DrawShared`): the
/// light records and the dword between on plain cells; the gear figures, which the drawing reaches
/// only by reading on past the records, on the physics' cells.
#[derive(rc2k_derive::Cells)]
pub struct LightsAndGear {
    /// +0x0: `race::light_records_store`.
    pub light_records: LightRecords,
    /// +0x28E8: a dword between them, by game address.
    pub between: PlainDword,
    /// +0x28EC: `physics::gear_store`.
    pub gear: GearFigures,
}

/// The hud's panel and the race's overlay.
#[derive(rc2k_derive::Cells)]
pub struct PanelAndOverlay {
    /// Two lead bytes before the panel.
    pub lead: PlainBytes<2>,
    /// +0x0: `race::panel_store`.
    pub panel: Panel,
    /// +0x10E: `race::overlay_store`.
    pub overlay: OverlayBlock,
}

/// The view variables and the 52 view records after them.
#[derive(rc2k_derive::Cells)]
pub struct ViewAndRecords {
    /// +0x0: `race::view_vars_store`.
    pub view_vars: ViewVars,
    /// The 52 view records, after the view variables (named through `ViewerAndKinds` handles).
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

    #[test]
    fn groups_have_their_homes() {
        let race = crate::test_support::own::race();
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
