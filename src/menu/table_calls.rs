//! The calls the menus' items (their keys' and actions' functions) make: the item function of the
//! typed tables (`rc2k_core::menu::tables::ItemFn`), run by the Rust function that replaces it
//! (`table_routes.rs`). The screens' draw lists run their typed functions directly
//! (`menu::run_list`).
//!
//! A refused input (`BadInput`) ends the program, saying which function refused and why; a
//! fatal error is raised from Rust.

use crate::errors::Refused;
use crate::menu::table_routes::item_fn;
use crate::owners::Owners;
use crate::program_log::log;
use rc2k_core::fatal::Context;
use rc2k_core::menu::tables::ItemFn;

/// A menu item's callback for a direction (`menu::navigation`).
pub fn call_key(o: Owners<'static>, function: ItemFn) -> Result<(), Refused> {
    item_fn(o, function)
}

/// A menu item's action (`menu::run_item_action`). A refusal ends the program here.
pub fn call_action(o: Owners<'static>, function: ItemFn) {
    if let Err(e) = item_fn(o, function) {
        refused(&format!("{function:?}"), e, &Context::default());
    }
}

/// A refusal of a function called directly in its place (`what` names it): a refused input
/// ends the program, a fatal error is raised.
pub fn refused(what: &str, e: Refused, r: &Context) {
    match e {
        Refused::Recoverable(why) => crate::error_dialog::recover::back(why.0),
        Refused::BadInput(why) => {
            crate::standalone::finish(&format!("menu::table_calls: {what} refused: {}", why.0), 1)
        }
        Refused::Fatal(error) => {
            log(&format!(
                "menu::table_calls: {what}: raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            crate::fatal::raise::raise(error, r);
        }
    }
}
