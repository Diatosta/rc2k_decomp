//! The race's stores that sit inside its groups, as views from the owner.

use crate::errors::BadInput;
use crate::math::engine_tables::EngineTablesRef;
use crate::physics::gear_store::GearFigures;
use crate::race::blocks_store::CarBlocks;
use crate::race::checkpoints_store::CheckpointsRef;
use crate::race::effects_settings_store::{CarEffects, EffectsSettings};
use crate::race::kind_store::KindTables;
use crate::race::light_records_store::LightRecordsRef;
use crate::race::owner::Race;
use crate::race::replay_work_store::ReplayWork;
use crate::race::viewer_buffers_store::ViewerBuffers;
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::scene::faces_store::FacesRef;
use crate::scene::polygon_store::PolygonTablesRef;

#[allow(dead_code, reason = "the call sites move to it in the M6 rounds")]
impl Race {
    /// The ten cars' blocks.
    pub fn car_blocks(&self) -> &CarBlocks {
        self.shared.car_blocks.get()
    }

    /// The cars' drawing state.
    pub fn car_parts(&self) -> &crate::race::car_parts_store::CarParts {
        self.shared.car_parts.get()
    }

    /// Car record `n` (`car_records[n]`, the name of one of the effects' car records).
    /// Refused past the eight and for a name that is not a record's start.
    pub fn car_record(&self, n: usize) -> Result<&CarEffects, BadInput> {
        self.shared
            .work
            .get()
            .car_record_number(n)
            .map(|k| &self.effects_settings().cars[k])
    }

    /// The eight car records.
    pub fn car_records(&self) -> Result<[&CarEffects; 8], BadInput> {
        let first = self.car_record(0)?;
        let mut out = [first; 8];
        for (n, record) in out.iter_mut().enumerate() {
            *record = self.car_record(n)?;
        }
        Ok(out)
    }

    /// The viewer's panel.
    pub fn viewer_panel(&self) -> &ViewerPanelBlock {
        &self.replay.viewer_and_kinds.viewer_panel
    }

    /// The viewer's buffers.
    pub fn viewer_buffers(&self) -> &ViewerBuffers {
        self.shared.viewer_buffers()
    }

    /// The gear figures.
    pub fn gear(&self) -> &GearFigures {
        self.physics.gear()
    }

    /// The replay's work area.
    pub fn replay_work(&self) -> std::cell::Ref<'_, ReplayWork> {
        self.replay.replay_work()
    }

    /// The replay's work area, to write: no other borrow may be held.
    pub fn replay_work_mut(&self) -> std::cell::RefMut<'_, ReplayWork> {
        self.replay.replay_work_mut()
    }

    /// The effects' settings.
    pub fn effects_settings(&self) -> &EffectsSettings {
        &self.replay.viewer_and_kinds.marks_settings
    }

    /// The scene's faces.
    pub fn faces(&self) -> FacesRef<'_> {
        self.shared.buffers.faces()
    }

    /// The surface kinds' tables.
    pub fn kind_tables(&self) -> &'static KindTables {
        &crate::race::kind_store::KIND_TABLES
    }

    /// The checkpoints.
    pub fn checkpoints(&self) -> CheckpointsRef<'_> {
        self.shared.checkpoints()
    }

    /// The light records.
    pub fn light_records(&self) -> LightRecordsRef<'_> {
        LightRecordsRef::Store(&self.frame.light_records)
    }

    /// The engine tables.
    pub fn engine_tables(&self) -> EngineTablesRef<'_> {
        self.shared.buffers.engine_tables()
    }

    /// The polygon tables.
    pub fn polygon_tables(&self) -> PolygonTablesRef<'_> {
        self.shared.buffers.polygon_tables()
    }
}
