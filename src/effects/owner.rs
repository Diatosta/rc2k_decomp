//! The effects' state in one struct, held by the race (`Race::effects`) and handed down by
//! reference to the code that draws the effects.

use crate::effects::mask_store::Masks;
use crate::effects::owner_groups::{Blocks, Clipping, FrameState, Kinds, Setup};
use crate::effects::paths::Paths;
use crate::effects::pong::Pong;
use crate::effects::rain::Weather;
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::BadInput;

/// The effects' state.
pub struct Effects {
    pub windscreen_flakes: WindscreenFlakes,
    pub pong: Pong,
    pub weather: Weather,
    pub particles: Particles,
    pub ring: Ring,
    pub windscreen_drops: WindscreenDrops,
    pub paths: Paths,
    pub masks: Masks,
    pub clipping: Clipping,
    pub blocks: Blocks,
    pub frame: FrameState,
    pub kinds: Kinds,
    pub setup: Setup,
}

impl Effects {
    /// The marks' settings the race handed `effects::init_marks`; refused while there are none.
    pub fn settings(
        &self,
    ) -> Result<&'static crate::race::effects_settings_store::EffectsSettings, BadInput> {
        match self.setup.marks.get() {
            Some(marks) if self.setup.settings.is_set.get() != 0 => Ok(marks),
            _ => Err(BadInput("no marks settings")),
        }
    }

    /// The car blocks the race handed `init_marks` with its settings record; refused before
    /// the first (no car record names a block then).
    pub fn car_blocks(&self) -> Result<&'static crate::race::blocks_store::CarBlocks, BadInput> {
        self.setup
            .car_blocks
            .get()
            .ok_or(BadInput("a car record without its block"))
    }

    /// The car record a handle names (`effects_settings_store::car_handle`), none for 0; a
    /// value that names no record ends the program. None before the race's record is handed
    /// over (no handle is set then).
    pub fn car_record(
        &self,
        handle: u32,
    ) -> Option<&'static crate::race::effects_settings_store::CarEffects> {
        self.setup.marks.get().and_then(|m| m.car_or_none(handle))
    }

    /// Whether `car`'s whole matrix's block is set.
    pub fn has_whole(&self, car: &crate::race::effects_settings_store::CarEffects) -> bool {
        self.car_blocks().is_ok_and(|b| car.has_whole(b))
    }

    /// Whether `car`'s part matrices' block is set.
    pub fn has_parts(&self, car: &crate::race::effects_settings_store::CarEffects) -> bool {
        self.car_blocks().is_ok_and(|b| car.has_parts(b))
    }
}

impl Effects {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            windscreen_flakes: WindscreenFlakes::new(),
            pong: Pong::new(),
            weather: Weather::new(),
            particles: Particles::new(),
            ring: Ring::new(),
            windscreen_drops: WindscreenDrops::new(),
            paths: Paths::new(),
            masks: Masks::new(),
            clipping: Clipping::new(),
            blocks: Blocks::new(),
            frame: FrameState::new(),
            kinds: Kinds::new(),
            setup: Setup::new(),
        }
    }
}
