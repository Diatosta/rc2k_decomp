//! `menu::go_back`: sets three variables of the menu system: `loop_flags.event_left` = 0,
//! `loop_flags.leave_screen` = 1, `draw_work.leave_result` = -1.
//!
//! It is the action of the back button of several menus and the last step of other actions (seven
//! callers, ten menu tables), so it is read as "leave this menu"; what each of the three variables
//! means has not been worked out. The menus and draw lists run its `entry` (`menu::table_calls`);

use crate::menu::owner::Menus;

pub fn go_back(menus: &Menus) {
    crate::menu::leaving::with_leaving(menus, rc2k_core::menu::leaving::Leaving::go_back);
}
