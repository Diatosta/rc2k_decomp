//! `menu::list_take_key`: a list menu's key: the pending key taken (`input::take_key`); none: a
//! millisecond's sleep (`system::sleep`). A key: the key repeat count is reset unless it is the
//! last key again and the time since the last key (less 6) is negative, then one more; the key
//! is kept as the last key.

/// The key, 0 for none.
pub fn list_take_key(
    list_state: &crate::menu::list_state::ListState,
    input: &crate::input::owner::Input,
) -> u8 {
    let key = crate::input::take_key::take_key(input);
    if key == 0 {
        crate::system::sleep::sleep(1);
    } else {
        let repeated = key == list_state.last_key.get();
        let (mut frame, mut moved) = (0, 0);
        if repeated {
            (frame, moved) = (list_state.frame_clock.get(), list_state.move_clock.get());
        }
        list_state.repeats.set(rc2k_core::menu::list_menu::repeats(
            list_state.repeats.get(),
            repeated,
            frame,
            moved,
        ));
        list_state.last_key.set(key);
    }
    key
}
