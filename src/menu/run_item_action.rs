//! `menu::run_item_action` (0x474683): calls the action of the highlighted item of a menu.
//!
//! In:  edi = the menu
//! Out: every register preserved
//! It goes through the menu's items (0x5C bytes each after a 4-byte head, ended by -1) until
//! one's type (+0x10) is the highlighted item `dword_755525`. For that one it stores the
//! item's dword at +0x54 in `dword_755A38`, calls the item's action (+0x44) with that value
//! in eax and the item in ebp, and then sets `dword_755525` to the item's type again,
//! whatever the action did to it. No item is found: nothing happens.
//! The first item is looked at without being checked for the end mark.
//! The action is the game's and can run for as long as an event lasts, so the thunk is not
//! gated and shadow mode runs the original alone. 89 callers: every screen loop.
//! Refuses a menu without an end in its first 256 items.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write, rva};
use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};
use crate::menu::screen_loop::Carried;
use crate::service::state::MENU_ITEM_RVA;

// ---- game data ----

const ACTION_VALUE_RVA: usize = rva(0x755A38); // dword_755A38
const TYPE_OFFSET: u32 = rc2k_core::menu::items::ITEM_TYPE;
const ACTION_OFFSET: u32 = 0x44;
const VALUE_OFFSET: u32 = 0x54;

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

// ---- Rust version ----

/// `carried` are the screen loop's registers that the action gets as they are (the
/// original's every register but eax = the value, ebp = the item, edi = the menu: the
/// flows started from items hand them on to their screens); what the action leaves is
/// nobody's (the original keeps every register).
pub unsafe fn run_item_action(menu: u32, carried: &Carried) -> Result<(), crate::errors::Refused> {
    let first = menu.wrapping_add(MENU_HEAD_LEN);
    let item_at = |n: u32| first.wrapping_add(n.wrapping_mul(ITEM_LEN));
    unsafe {
        let highlighted: u32 = game_read(MENU_ITEM_RVA);
        // Converted: `rc2k_core::menu::items::highlighted_item`.
        let found = rc2k_core::menu::items::highlighted_item(
            highlighted,
            |n| dword(item_at(n) + TYPE_OFFSET),
            |n| dword(item_at(n)),
        )?;
        if let Some(n) = found {
            let item = item_at(n);
            let value = dword(item + VALUE_OFFSET);
            game_write(ACTION_VALUE_RVA, value);
            crate::menu::table_calls::call_action(
                dword(item + ACTION_OFFSET),
                value,
                item,
                menu,
                carried,
            );
            game_write(MENU_ITEM_RVA, dword(item + TYPE_OFFSET));
        }
    }
    Ok(())
}
