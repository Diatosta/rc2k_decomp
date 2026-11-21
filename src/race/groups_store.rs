//! Race-block pieces that sit together because code reads across them by address: each group keeps
//! its pieces and the bytes between them in a fixed layout, as one static with its first values.
//! Each piece's store is a field here.
//!
//! - `ViewerAndKinds`: the viewer's panel and the marks' settings.
//! - `LightsAndGear`: the light records. `scene::load_object` reads a kind's record past the 34
//!   (an unbounded kind) on into the gear figures (`LightRecords::group_at`).
//! - `ViewAndRecords`: the view variables and the 52 view records of 0x5C after them
//!   (`ViewAndRecords.view_vars.view_records[0]`'s targets). The records are one array: with the
//!   variables overrunning onto the first records' fields, the third chase view read half of
//!   record 6 from the wrong memory.

use crate::memory::session::{GameThread, PlainCells};
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::light_records_store::LightRecords;
use crate::race::view_vars_store::{ViewRecord, ViewVars};
use crate::race::viewer_screen_store::ViewerPanelBlock;

/// The viewer's panel and the marks' settings.
#[derive(rc2k_derive::Cells)]
pub struct ViewerAndKinds {
    /// `race::viewer_screen_store`.
    pub viewer_panel: GameThread<ViewerPanelBlock>,
    /// `race::effects_settings_store`.
    pub marks_settings: GameThread<EffectsSettings>,
}

/// The light records. A drawing store (`session::DrawShared`) on plain cells.
#[derive(rc2k_derive::Cells)]
pub struct LightsAndGear {
    /// `race::light_records_store`.
    pub light_records: LightRecords,
}

/// The view variables and the 52 view records after them.
#[derive(rc2k_derive::Cells)]
pub struct ViewAndRecords {
    /// `race::view_vars_store`.
    pub view_vars: ViewVars,
    /// The 52 view records, after the view variables (named through `ViewerAndKinds` handles).
    pub records: [ViewRecord; 52],
}

// SAFETY (the groups): every field is a cell over a plain integer (`session`'s cells and
// records of them): any bit pattern is a valid value.

/// The viewer's panel and the marks' settings as the game starts.
impl ViewerAndKinds {
    pub const fn new() -> Self {
        ViewerAndKinds {
            viewer_panel: GameThread::new(ViewerPanelBlock::new()),
            marks_settings: GameThread::new(EffectsSettings::new()),
        }
    }
}

/// The light records as the game starts.
impl LightsAndGear {
    pub const fn new() -> Self {
        LightsAndGear {
            light_records: LightRecords::new(),
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
            core::ptr::from_ref(race.frame.view_and_records.get()).addr(),
        ] {
            assert_eq!(home % 4, 0);
        }
    }
}
