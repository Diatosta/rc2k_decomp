//! `menu::run_item_action` (0x474683): calls the action of the highlighted item of a menu.
//!
//! It goes through the menu's items (0x5C bytes each after a 4-byte head, ended by -1) until one's
//! type (+0x10) is the highlighted item `dword_755525`. For that one it stores the item's dword at
//! +0x54 in `dword_755A38`, calls the item's action (+0x44) with that value in eax and the item in
//! ebp, and then sets `dword_755525` to the item's type again, whatever the action did to it. No
//! item is found: nothing happens. The first item is looked at without being checked for the end
//! mark. Refuses a menu without an end in its first 256 items.

use crate::menu::menus_store::{MenuId, MenuItem};
use crate::menu::owner::Menus;

// ---- game data ----

/// The highlighted item's action, with its value. (The original hands
/// the action the screen loop's other registers too, which no action reads; what the
/// action leaves is nobody's.)
pub fn run_item_action(menus: &Menus, menu: MenuId) -> Result<(), crate::errors::Refused> {
    let screens = &menus.screens;
    let items = menu.items();
    let highlighted: u32 = screens.loop_flags.item.get();
    // Converted: `rc2k_core::menu::items::highlighted_item`. It asks for item n's type only
    // while item n is there, and for the x of one past the last: the end mark after it.
    let found = rc2k_core::menu::items::highlighted_item(
        highlighted,
        |n| items.get(n as usize).map_or(MenuItem::END, |i| i.id.get()),
        |n| items.get(n as usize).map_or(MenuItem::END, |i| i.x.get()),
    )?;
    if let Some(n) = found {
        let item = &items[n as usize];
        let value = item.action_value.get();
        screens.draw_work.action_value.set(value);
        crate::menu::table_calls::call_action(menus, item.action.get(), value, item, menu);
        screens.loop_flags.item.set(item.id.get());
    }
    Ok(())
}
