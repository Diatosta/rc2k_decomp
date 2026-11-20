//! The marks (bits 6 and 7) of the viewer's items cleared: each item's first byte is masked with
//! 0x3F. The viewer's items have no choices, so an item of a kind that has some is refused.

use crate::race::viewer_list_store::ViewerList;
use rc2k_core::menu::list_menu::{has_choices, unmarked};

pub fn list_clear_marks(list: &ViewerList) {
    for item in &list.items {
        let kind = unmarked(item.first.get());
        item.first.set(kind);
        if has_choices(kind) {
            crate::fatal::raise::end(
                "menu::replay_viewer::list_clear_marks",
                crate::errors::BadInput("an item with choices in a list that has none"),
            );
        }
    }
}
