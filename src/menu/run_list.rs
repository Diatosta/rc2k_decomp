//! `menu::run_list` (0x468202): calls every function of a screen's list (a screen has four
//! such lists, which its loop and its frame callback run).
//!
//! The functions are the typed tables' screen functions (`menu::table_routes::screen_fn`),
//! which take no register: the registers the original hands each (esi the place in the list,
//! the rest as the one before left them) are nobody's.

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
