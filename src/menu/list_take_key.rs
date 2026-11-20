//! `menu::list_take_key`: a list menu's key: the pending key taken (`input::take_key`); none: a
//! millisecond's sleep (`clock::sleep`). A key: the key repeat count is reset unless it is the
//! last key again and the time since the last key (less 6) is negative, then one more; the key
//! is kept as the last key.

/// The key, 0 for none.
pub fn list_take_key(
    list_state: &crate::menu::list_state::ListState,
    input: &crate::input::owner::Input,
) -> u8 {
    let key = crate::input::take_key::take_key(input);
    if key == 0 {
        crate::clock::sleep::sleep(1);
    } else {
        let repeated = key == list_state.read().last_key;
        let (mut frame, mut moved) = (0, 0);
        if repeated {
            (frame, moved) = (list_state.read().frame_clock, list_state.read().move_clock);
        }
        list_state.write().repeats =
            rc2k_core::menu::list_menu::repeats(list_state.read().repeats, repeated, frame, moved);
        list_state.write().last_key = key;
    }
    key
}
