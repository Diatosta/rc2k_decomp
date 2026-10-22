//! The fatal-error reporter.

use crate::framework::GameHook;

pub mod raise;
pub mod report;
pub mod show_report_at_exit;

pub static HOOKS: &[&dyn GameHook] = &[&report::HOOK];
