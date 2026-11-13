//! Flips the flag of the highlighted menu item; if it is then not 1, the chosen gear ratio goes
//! back to the installed value. Refuses a highlighted item past the flag table.

use crate::service::setup_values::Change;

pub const CHANGE: Change = Change { setup: 0 };
