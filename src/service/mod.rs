//! The service area's logic: what happens between the stages besides drawing.

use crate::framework::GameHook;

pub mod load_setup_values;
pub mod state;

pub static HOOKS: &[&dyn GameHook] = &[&load_setup_values::HOOK];
