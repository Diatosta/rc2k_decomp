//! `menu::run_item_action` (0x474683): calls the action of the highlighted item of a menu.
//!
//! It goes through the menu's items (0x5C bytes each after a 4-byte head, ended by -1) until one's
//! type (+0x10) is the highlighted item `loop_flags.item`. For that one it stores the item's dword at
//! +0x54 in `draw_work.action_mode`, calls the item's action (+0x44) with that value in eax and the item in
//! ebp, and then sets `loop_flags.item` to the item's type again, whatever the action did to it. No
//! item is found: nothing happens. The first item is looked at without being checked for the end
//! mark. Refuses a menu without an end in its first 256 items.

use crate::menu::menus_store::{MenuId, MenuItem};
use crate::menu::owner::Menus;

// ---- game data ----

/// What the highlighted item's action left for the screen loop to decide on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActionResult {
    /// The action asks to leave the screen (and, with its question, to be asked about it).
    pub ask_leave: bool,
    /// The event the screen belongs to is over.
    pub event_over: bool,
    /// The action leaves the screen.
    pub leave_screen: bool,
    /// The event flow behind the screen was left.
    pub event_left: bool,
    /// The item's value, which the screen loop tells a plain action (2) from others by.
    pub action_mode: u32,
}

/// The highlighted item's action, with its value. (The original hands
/// the action the screen loop's other registers too, which no action reads; what the
/// action leaves is nobody's.)
pub fn run_item_action(
    menus: &Menus,
    menu: MenuId,
) -> Result<ActionResult, crate::errors::Refused> {
    let screens = &menus.screens;
    let items = menu.items();
    let highlighted: u32 = screens.loop_flags.item.get();
    // The logic is in `rc2k_core::menu::items::highlighted_item`. It asks for item n's type only
    // while item n is there, and for the x of one past the last: the end mark after it.
    let found = rc2k_core::menu::items::highlighted_item(
        highlighted,
        |n| {
            items
                .get(n as usize)
                .map_or(MenuItem::END, |i| i.fixed().id)
        },
        |n| {
            items
                .get(n as usize)
                .map_or(MenuItem::END, |i| i.fixed().x as u32)
        },
    )?;
    if let Some(n) = found {
        let item = &items[n as usize];
        let value = item.fixed().action_value;
        screens.draw_work.action_mode.set(value);
        crate::menu::table_calls::call_action(menus, item.fixed().action, value, menu);
        screens.loop_flags.item.set(item.fixed().id);
    }
    Ok(ActionResult {
        ask_leave: screens.loop_flags.ask_leave.is_set(),
        event_over: screens.event.event_over.is_set(),
        leave_screen: screens.loop_flags.leave_screen.is_set(),
        event_left: screens.loop_flags.event_left.is_set(),
        action_mode: screens.draw_work.action_mode.get(),
    })
}
