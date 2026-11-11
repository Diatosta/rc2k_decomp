//! The race's stores that sit inside its groups, as views from the owner.

use crate::physics::gear_store::GearFigures;
use crate::race::blocks_store::CarBlocks;
use crate::race::checkpoints_store::Checkpoints;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::kind_store::KindTables;
use crate::race::light_records_store::LightRecords;
use crate::race::owner::Race;
use crate::race::replay_work_store::ReplayWork;
use crate::race::viewer_buffers_store::ViewerBuffers;
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::scene::engine_tables_store::EngineTables;
use crate::scene::faces_store::Faces;
use crate::scene::polygon_store::PolygonTables;
use crate::screens::text_buffers_store::TextBuffers;

#[allow(dead_code, reason = "the call sites move to it in the M6 rounds")]
impl Race {
    /// The ten cars' blocks.
    pub fn car_blocks(&self) -> &CarBlocks {
        &self.replay.cars_and_replay.car_blocks
    }

    /// The viewer's panel.
    pub fn viewer_panel(&self) -> &ViewerPanelBlock {
        &self.replay.viewer_and_kinds.viewer_panel
    }

    /// The viewer's buffers.
    pub fn viewer_buffers(&self) -> &ViewerBuffers {
        &self.shared.buffers.viewer
    }

    /// The gear figures.
    pub fn gear(&self) -> &GearFigures {
        &self.frame.lights_and_gear.gear
    }

    /// The replay's work area.
    pub fn replay_work(&self) -> &ReplayWork {
        &self.replay.cars_and_replay.replay_work
    }

    /// The effects' settings.
    pub fn effects_settings(&self) -> &EffectsSettings {
        &self.replay.viewer_and_kinds.marks_settings
    }

    /// The scene's faces.
    pub fn faces(&self) -> &Faces {
        &self.shared.buffers.faces
    }

    /// The surface kinds' tables.
    pub fn kind_tables(&self) -> &KindTables {
        &self.replay.viewer_and_kinds.kind_tables
    }

    /// The checkpoints.
    pub fn checkpoints(&self) -> &Checkpoints {
        &self.shared.buffers.checkpoints
    }

    /// The screens' text buffers.
    pub fn text_buffers(&self) -> &TextBuffers {
        &self.shared.buffers.text
    }

    /// The light records.
    pub fn light_records(&self) -> &LightRecords {
        &self.frame.lights_and_gear.light_records
    }

    /// The engine tables.
    pub fn engine_tables(&self) -> &EngineTables {
        &self.shared.buffers.engine_tables
    }

    /// The polygon tables.
    pub fn polygon_tables(&self) -> &PolygonTables {
        &self.shared.buffers.polygon_tables
    }
}
