//! `menu::list_take_key`: a list menu's key: the pending key taken (`input::take_key`); none: a
//! millisecond's sleep (`system::sleep`). A key: the key repeat count is reset unless it is the
//! last key again and the time since the last key (less 6) is negative, then one more; the key
//! is kept as the last key.
//!
//! Callers: `race::replay_viewer`.

/// The key, 0 for none.
pub fn list_take_key(
    install: &crate::install::owner::Install,
    input: &crate::input::owner::Input,
) -> u8 {
    let key = crate::input::take_key::take_key(input);
    if key == 0 {
        crate::system::sleep::sleep(1);
    } else {
        let pages = &install.pages;
        let repeated = key == pages.last_key.get();
        let (mut frame, mut moved) = (0, 0);
        if repeated {
            (frame, moved) = (pages.frame_clock.get(), pages.move_clock.get());
        }
        pages.repeats.set(rc2k_core::menu::list_menu::repeats(
            pages.repeats.get(),
            repeated,
            frame,
            moved,
        ));
        pages.last_key.set(key);
    }
    key
}
