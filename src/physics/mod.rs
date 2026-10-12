//! The cars' physics.

use crate::framework::GameHook;

pub mod apply_patches;

pub static HOOKS: &[&dyn GameHook] = &[&apply_patches::HOOK];
