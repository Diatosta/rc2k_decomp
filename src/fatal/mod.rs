//! The fatal-error reporter.

use crate::framework::GameHook;

pub mod report;

pub static HOOKS: &[&dyn GameHook] = &[&report::HOOK];
