//! The effects' owner (step 11, owned state): the effects' state in one struct, handed by
//! reference to the code that runs them in place of global stores.
//!
//! `effects()` is the boundary: the callers that do not take the owner yet fetch it there
//! and pass it down. It rises as their callers take the owner, until the program's top holds
//! it (`notes/memory_layer_plan.md`, step 11).

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
        if self.setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings"));
        }
        Ok(&crate::race::effects_settings_store::EFFECTS_SETTINGS)
    }
}

/// The one instance. Reached through `effects()`; `moved` registers its stores' places.
pub static EFFECTS: Effects = Effects {
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
};

/// The effects' state, for a caller that does not take it as a parameter yet.
pub fn effects() -> &'static Effects {
    &EFFECTS
}
