//! `install::room_page` (0x4142C7): the not-enough-room page. At the start (ebx 0)
//! `dword_51BA04` (ignore the room) is set to 0.
//!
//! Takes the state.
//! Caller `install::run_page`.

/// `state` which call of the list action this is (0 the start, negative the end).
pub fn room_page(install: &'static crate::install::owner::Install, state: u32) {
    if state == 0 {
        install.run.ignore_room.set(0);
    }
}
