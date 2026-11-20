//! The not-enough-room page: at the start `ignore_room` is set to 0; its "go on anyway" item
//! sets it to 1, answers 1 and leaves the page.

/// `state` which call of the list action this is (0 the start, negative the end).
pub(super) fn room_page(install: &crate::install::owner::Install, state: u32) {
    if state == 0 {
        install.flags.ignore_room.set(false);
    }
}

/// The "go on anyway" item's action.
pub(super) fn ignore_room(
    install: &crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
) {
    install.flags.ignore_room.set(true);
    list_state.list_answer.set(1);
    list_state.list_leave.set(1);
}
