//! The main page's items marked by the installation's state: not this version, the update item
//! is unusable; no installation, the item after it; registration not offered, its item hidden;
//! with the update script's title word the update item is usable and the others hidden. A page
//! action, run by `install::run_page`.

use crate::memory::list_handle::ListMenu;
use rc2k_core::install::pages;

pub fn mark_main_page(install: &crate::install::owner::Install, list: ListMenu) {
    let title = list.item(1).text_id();
    let marks = pages::main_page_marks(
        install.run.same_version.get() == 1,
        install.names.status.get() != 0,
        install.flags.offer_registration.get(),
        title,
    );
    for (offset, or, and) in marks {
        let item = list.item_at_offset(offset);
        item.set_first((item.first() | or) & and);
    }
}
