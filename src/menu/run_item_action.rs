//! `menu::run_item_action` (0x474683): calls the action of the highlighted item of a menu.
//!
//! In:  edi = the menu
//! Out: every register preserved
//! It goes through the menu's items (0x5C bytes each after a 4-byte head, ended by -1) until one's
//! type (+0x10) is the highlighted item `dword_755525`. For that one it stores the item's dword at
//! +0x54 in `dword_755A38`, calls the item's action (+0x44) with that value in eax and the item in
//! ebp, and then sets `dword_755525` to the item's type again, whatever the action did to it. No
//! item is found: nothing happens. The first item is looked at without being checked for the end
//! mark. Refuses a menu without an end in its first 256 items.

use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::screen_loop::Carried;

// ---- game data ----

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
        let highlighted: u32 = LOOP_FLAGS.item.get();
        // Converted: `rc2k_core::menu::items::highlighted_item`.
        let found = rc2k_core::menu::items::highlighted_item(
            highlighted,
            |n| dword(item_at(n) + TYPE_OFFSET),
            |n| dword(item_at(n)),
        )?;
        if let Some(n) = found {
            let item = item_at(n);
            let value = dword(item + VALUE_OFFSET);
            DRAW_WORK.action_value.set(value);
            crate::menu::table_calls::call_action(
                dword(item + ACTION_OFFSET),
                value,
                item,
                menu,
                carried,
            );
            LOOP_FLAGS.item.set(dword(item + TYPE_OFFSET));
        }
    }
    Ok(())
}
