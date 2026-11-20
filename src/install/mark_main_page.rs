//! The main page's items marked by the installation's state: not this version, the update item is
//! unusable; no installation, the item after it; with the update script's title word the update
//! item is usable and the uninstall item hidden.

use crate::memory::list_handle::ListMenu;
use rc2k_core::install::pages;

pub(super) fn mark_main_page(install: &crate::install::owner::Install, list: ListMenu) {
    let title = list.item(1).text_id();
    let marks = pages::main_page_marks(
        install.run.same_version.get() == 1,
        install.names.status.get() != 0,
        title,
    );
    for (offset, or, and) in marks {
        let item = list.item_at_offset(offset);
        item.set_first((item.first() | or) & and);
    }
}
