//! `menu::run_list`: calls every function of a screen's list (a screen has four such lists,
//! which its loop and its frame callback run).
//!
//! The functions are the typed tables' screen functions (`menu::table_routes::screen_fn`),
//! which take no arguments.

use crate::errors::Refused;
use crate::menu::fn_list::FnList;
use crate::menu::table_routes::screen_fn;
use crate::owners::Owners;

/// Runs the functions of `list` in order.
pub fn run_list(o: Owners<'static>, list: FnList) -> Result<(), Refused> {
    for f in list.functions() {
        screen_fn(o, *f)?;
    }
    Ok(())
}
