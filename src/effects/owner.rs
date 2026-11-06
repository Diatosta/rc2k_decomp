//! The effects' owner (step 11, owned state): the effects' state in one struct, handed by
//! reference to the code that runs them in place of global stores. The pilot holds the
//! windscreen flakes; the other effects' stores join it one by one.
//!
//! `effects()` is the boundary: the callers that do not take the owner yet fetch it there
//! and pass it down. It rises as their callers take the owner, until the program's top holds
//! it (`notes/memory_layer_plan.md`, step 11).

use crate::effects::windscreen_flakes_store::WindscreenFlakes;

/// The effects' state.
pub struct Effects {
    pub windscreen_flakes: WindscreenFlakes,
}

/// The one instance. Reached through `effects()`; `moved` registers its stores' places.
pub static EFFECTS: Effects = Effects {
    windscreen_flakes: WindscreenFlakes::new(),
};

/// The effects' state, for a caller that does not take it as a parameter yet.
pub fn effects() -> &'static Effects {
    &EFFECTS
}
