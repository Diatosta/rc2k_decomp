//! `install::ignore_room` (0x4142DA): the not-enough-room page's "go on anyway" action:
//! `ignore_room` 1, the answer (`list_answer`) 1 and the page left.
//!
//! An item action (kind 6). Caller `install::item_action` through a page's item.

pub fn ignore_room(install: &'static crate::install::owner::Install) {
    install.run.ignore_room.set(1);
    install.pages.list_answer.set(1);
    install.pages.list_leave.set(1);
}
