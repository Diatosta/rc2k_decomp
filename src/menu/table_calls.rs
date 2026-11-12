//! The calls the menus' items (their keys' and actions' functions) make: the item function
//! of the typed tables (`rc2k_core::menu::tables::ItemFn`), run by the Rust function that
//! replaces it (`table_routes.rs`). Callers: `menu::run_item_action` (`call_action`: the
//! item's value and the menu) and `menu::navigation`'s moves (`call_key`). The screens'
//! draw lists run their typed functions directly (`menu::run_list`).
//!
//! A refused input (`BadInput`) ends the program, saying which function refused and why; a
//! fatal error is raised from Rust.

use crate::errors::Refused;
use crate::framework::log;
use crate::menu::menus_store::{MenuId, MenuItem};
use crate::menu::table_routes::item_fn;
use crate::owners::Owners;
use rc2k_core::fatal::Context;
use rc2k_core::menu::tables::ItemFn;

/// A menu item's callback for a direction (`menu::navigation`). Of the item functions the
/// tables name as key callbacks only `RaiseControllerSensitivity` reads a register: edi,
/// which is the item here; the rest get none (`tables_data.rs`' `on_key`
/// against `item_fn`'s reads).
pub fn call_key(o: Owners<'static>, function: ItemFn, item: &MenuItem) -> Result<(), Refused> {
    match function {
        // edi the item: its +0x2C, the left link.
        ItemFn::RaiseControllerSensitivity => {
            crate::menu::controller_items::raise_controller_sensitivity(
                o.menus,
                o.input,
                item.fixed().links[2],
            )
        }
        f => item_fn(o, f, &ItemCall::default()),
    }
}

/// What a menu item's function is given (the original's registers at
/// the call: eax the item's value, ebp the item (not kept), edi the menu; the screen loop's others,
/// which no function reads).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemCall {
    pub value: u32,
    pub menu: Option<MenuId>,
}

/// A menu item's action (`menu::run_item_action`): the item's function with the value
/// and the menu. A refusal ends the program here.
pub fn call_action(o: Owners<'static>, function: ItemFn, value: u32, menu: MenuId) {
    let given = ItemCall {
        value,
        menu: Some(menu),
    };
    if let Err(e) = item_fn(o, function, &given) {
        refused(&format!("{function:?}"), e, &Context::default());
    }
}

/// A refusal of a function called directly in its place (`what` names it): a refused input
/// ends the program, a fatal error is raised.
pub fn refused(what: &str, e: Refused, r: &Context) {
    match e {
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
