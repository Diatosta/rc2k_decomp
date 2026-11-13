//! The not-enough-room page's "go on anyway" action: `ignore_room` 1, the answer 1 and the page
//! left. Called by `install::item_action`.

pub fn ignore_room(install: &crate::install::owner::Install) {
    install.run.ignore_room.set(1);
    install.pages.list_answer.set(1);
    install.pages.list_leave.set(1);
}
