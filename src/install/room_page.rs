//! The not-enough-room page: at the start `ignore_room` is set to 0.

/// `state` which call of the list action this is (0 the start, negative the end).
pub(super) fn room_page(install: &crate::install::owner::Install, state: u32) {
    if state == 0 {
        install.flags.ignore_room.set(false);
    }
}
