//! `install::finish_page` (0x414347): at the page's end (state negative) with the answer
//! (`list_answer`) 2, `script_mark` becomes 5 and the answer 1 (the readme is shown).
//!
//! Takes the state.
//! Caller `install::run_page`.

/// `state` which call of the list action this is (0 the start, negative the end).
pub fn finish_page(install: &crate::install::owner::Install, state: u32) {
    // Converted: `rc2k_core::install::pages::shows_readme`.
    if (state as i32) < 0
        && rc2k_core::install::pages::shows_readme(install.pages.list_answer.get())
    {
        install
            .names
            .script
            .set(crate::install::vars::SCRIPT_MARK, 5);
        install.pages.list_answer.set(1);
    }
}
