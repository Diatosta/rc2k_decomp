//! The calls the menus' items (their keys' and actions' functions) and the screens' draw
//! lists make (typed tables, 2026-10-22): the function named by its RAL address, as the
//! tables hold it, run by its Rust version (`table_routes.rs`; `crate::routes`, for
//! the few others, went 2026-10-24: nothing could reach them by address). Callers: `menu::run_list` and the frame
//! callbacks (`call_listed`: screen functions, no registers), `menu::run_item_action`
//! (`call_action`: an `ItemCall`, the item's value, the item and the menu),
//! `menu::navigation`'s moves (`call_key`).
//!
//! A refused input (`BadInput`) ends the program, saying which function refused and why
//! (2026-10-23); a fatal error is raised from Rust. An address in no table: the empty
//! functions are done here (`standalone::answers`); anything else ends the program, "not
//! ported" (2026-10-24).

use crate::errors::{BadInput, Refused};
use crate::framework::log;
use crate::menu::menus_store::{MenuId, MenuItem};
use crate::menu::owner::Menus;
use crate::menu::table_routes::{item_fn, screen_fn};
use rc2k_core::fatal::Context;
use rc2k_core::menu::tables::{ItemFn, ScreenFn};

/// A function of a screen's function list (`menu::run_list`, the frame callbacks): a
/// screen function of the typed tables, which takes no register. The lists the screens
/// put in place are all the tables' (`rc2k_core::menu::tables::LISTS`, every one a list
/// of screen functions); anything else in one is refused.
pub fn call_listed(menus: &Menus, function: u32) -> Result<(), Refused> {
    match ScreenFn::from_ral(function) {
        Some(f) => screen_fn(menus, f),
        None => Err(BadInput("a listed function that is none of the screens'").into()),
    }
}

/// A menu item's callback for a direction (`menu::navigation`): an item function of the
/// typed tables. Of those the tables name as key callbacks only
/// `RaiseControllerSensitivity` reads a register: edi, which is the item here; the rest
/// get none (2026-10-24, `tables_data.rs`' `on_key` against `item_fn`'s reads).
pub fn call_key(menus: &Menus, function: u32, item: &MenuItem) -> Result<(), Refused> {
    match ItemFn::from_ral(function) {
        // edi the item: its +0x2C, the left link.
        Some(ItemFn::RaiseControllerSensitivity) => {
            crate::menu::controller_items::raise_controller_sensitivity(item.links[2].get())
        }
        Some(f) => item_fn(menus, f, &ItemCall::default()),
        None => Err(BadInput("a key callback that is none of the items' functions").into()),
    }
}

/// What a menu item's function is given (typed, 2026-10-24; the original's registers at
/// the call: eax the item's value, ebp the item (not kept), edi the menu; the screen loop's others,
/// which no function reads).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemCall {
    pub value: u32,
    pub menu: Option<MenuId>,
}

impl ItemCall {
    /// The registers the original hands the function, for a refusal's report.
    pub fn regs(&self) -> Context {
        Context {
            eax: self.value,
            edi: self.menu.map_or(0, MenuId::code),
            ..Context::default()
        }
    }
}

/// A menu item's action (`menu::run_item_action`): the item's function with the value,
/// the item and the menu. A refusal ends the program here, as `call`'s.
pub fn call_action(menus: &Menus, function: u32, value: u32, _item: &MenuItem, menu: MenuId) {
    let given = ItemCall {
        value,
        menu: Some(menu),
    };
    let ral = function;
    let result = if let Some(f) = ItemFn::from_ral(ral) {
        Some((Named::Item(f), item_fn(menus, f, &given)))
    } else {
        ScreenFn::from_ral(ral).map(|f| (Named::Screen(f), screen_fn(menus, f)))
    };
    match result {
        Some((_, Ok(()))) => {}
        Some((name, Err(e))) => refused(&format!("{name:?} ({ral:#x})"), e, &given.regs()),
        None => {
            if !crate::standalone::answers(ral as usize) {
                crate::standalone::not_ported(ral as usize, "a menu or draw list");
            }
        }
    }
}

/// A refusal of a function called by its address or directly in its place (`what` names
/// it): a refused input ends the program, a fatal error is raised.
pub fn refused(what: &str, e: Refused, r: &Context) {
    match e {
        Refused::BadInput(why) => {
            crate::standalone::finish(&format!("menu::table_calls: {what} refused: {}", why.0), 1)
        }
        Refused::Fatal(error) => {
            log(&format!(
                "menu::table_calls: {what}: the Rust version raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            crate::fatal::raise::raise(error, r);
        }
    }
}

/// The function a call ran, for the log.
enum Named {
    Item(ItemFn),
    Screen(ScreenFn),
}

impl std::fmt::Debug for Named {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Named::Item(x) => x.fmt(f),
            Named::Screen(x) => x.fmt(f),
        }
    }
}
