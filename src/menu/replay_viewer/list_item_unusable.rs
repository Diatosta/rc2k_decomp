//! Whether the highlighted item of the viewer's list cannot be chosen: its first byte's bits 6
//! and 7 exactly 0x40 or 0x80, or its kind (low nibble) 5.

use crate::race::viewer_list::ViewerList;

pub fn list_item_unusable(
    list_state: &crate::menu::list_state::ListState,
    list: &ViewerList,
) -> bool {
    rc2k_core::menu::list_menu::unusable(list.item(list_state.choice.get()).first.get())
}
