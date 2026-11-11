//! `menu::list_take_key` (0x41469A): a list menu's key: the pending key taken
//! (`input::take_key`, eax 0 first); none: a millisecond's sleep (`system::sleep`). A key:
//! the key repeat count is reset unless it is the last key again and the time since the
//! last key (less 6) is negative (`js`), then one more; the key is kept as the last key.
//!
//! Returns the key, 0 for none.
//!
//! Callers: `race::replay_viewer`.

/// The key, 0 for none.
pub fn list_take_key() -> u8 {
    let key = crate::input::take_key::take_key();
    if key == 0 {
        crate::system::sleep::sleep(1);
    } else {
        // The logic is in `rc2k_core::menu::list_menu::repeats`.
        let pages = &crate::install::owner::install().pages;
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
