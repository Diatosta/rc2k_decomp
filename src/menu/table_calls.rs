//! The calls the menus' items (their keys' and actions' functions) and the screens' draw
//! lists make (typed tables, 2026-10-22): the function named by its RAL address, as the
//! game's memory still holds it, run by its Rust version (`table_routes.rs`, generated from
//! `rc2k_core::menu::tables`), no longer through its hook. Callers: `menu::run_list`,
//! `menu::run_item_action`, `menu::run_draw_list` and the screen loops through them,
//! `menu::navigation`'s moves, and every function the screen loops and the frame callback
//! call (`screen_loop::Real`, through `call_any`: their lists, and their own calls).
//!
//! A refused input (`BadInput`) ends the program, saying which function refused and why
//! (2026-10-23); a fatal error is raised from Rust. An address in no table goes to the
//! game, logged once: a platform hook's Rust side, SDL's and the empty functions are run
//! here (`platform::by_address`, `standalone::answers`); anything else reaches the
//! standalone's fault handler, which ends with "not ported".

use crate::errors::Refused;
use crate::framework::{IMAGE_BASE, Regs, call_game, game_addr, log};
use crate::menu::table_routes::{item_fn, screen_fn};
use rc2k_core::menu::tables::{ItemFn, ScreenFn};
use std::sync::Mutex;

/// Runs the function at `function` (an address in the game's memory) with `r`.
pub fn call(function: u32, r: &mut Regs) {
    run(function, r, true);
}

/// `call` for a caller that also calls functions in no table (the screen loops' and the
/// frame callback's `Real`, `crate::calls::call`): a function with a Rust version in
/// `crate::routes` runs it; any other goes to the game without a log line.
pub fn call_any(function: u32, r: &mut Regs) {
    run(function, r, false);
}

fn run(function: u32, r: &mut Regs, only_tables: bool) {
    let ral = (function as usize)
        .wrapping_sub(game_addr(0))
        .wrapping_add(IMAGE_BASE) as u32;
    let result = if let Some(f) = ItemFn::from_ral(ral) {
        Some((Named::Item(f), item_fn(f, r)))
    } else if let Some(f) = ScreenFn::from_ral(ral) {
        Some((Named::Screen(f), screen_fn(f, r)))
    } else {
        let at = (ral as usize).wrapping_sub(IMAGE_BASE);
        crate::routes::by_rva(at, r).map(|done| (Named::Routed, done))
    };
    match result {
        Some((_, Ok(()))) => {}
        Some((name, Err(Refused::BadInput(why)))) => crate::standalone::finish(
            &format!("menu::table_calls: {name:?} ({ral:#x}) refused: {}", why.0),
            1,
        ),
        Some((name, Err(Refused::Fatal(error)))) => {
            log(&format!(
                "menu::table_calls: {name:?} ({ral:#x}): the Rust version raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            unsafe { crate::fatal::raise::raise(error, r) };
        }
        None => {
            let at = ral as usize;
            if crate::platform::by_address(at, r) || crate::standalone::answers(at, r) {
                return;
            }
            if only_tables && first(ral) {
                log(&format!(
                    "menu::table_calls: {ral:#x} is in no menu or draw list; the game's runs"
                ));
            }
            unsafe { call_game(r, function as usize) };
        }
    }
}

/// The function a call ran, for the log.
enum Named {
    Item(ItemFn),
    Screen(ScreenFn),
    /// One of `crate::routes`' (named in the log by its address).
    Routed,
}

impl std::fmt::Debug for Named {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Named::Item(x) => x.fmt(f),
            Named::Screen(x) => x.fmt(f),
            Named::Routed => f.write_str("a routed function"),
        }
    }
}

/// True the first time for a function.
fn first(ral: u32) -> bool {
    static SEEN: Mutex<Vec<u32>> = Mutex::new(Vec::new());
    let mut seen = SEEN.lock().unwrap();
    if seen.contains(&ral) {
        return false;
    }
    seen.push(ral);
    true
}
