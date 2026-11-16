//! The language page: at the end with a chosen answer the language is chosen and loaded; the
//! items past the languages (up to 9) are hidden. Called by `install::run_page`
//! (`install::run_pages` does not show the page with at most one language or this version
//! already installed).

use crate::errors::Refused;
use crate::install::installer::load_language;
use crate::memory::list_handle::ListMenu;
use rc2k_core::install::pages;

/// `state`: which call of the list action this is (0 the start, negative the end).
pub fn language_page(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
    state: u32,
    list: ListMenu,
) -> Result<(), Refused> {
    if (state as i32) < 0 {
        let answer = o.menus.list.list_answer.get() as u32;
        if answer != 0 {
            crate::install::choose_language::choose_language(install, answer)?;
            load_language(o)?;
        }
    }
    // hide the items past the languages
    let languages = install.names.language_count.get();
    let (_, hidden) = pages::hidden_languages(languages);
    for k in 0..hidden {
        // From the item after the languages found.
        let item = list.item(languages.wrapping_add(k));
        item.set_first(item.first() | pages::HIDDEN);
    }
    Ok(())
}
