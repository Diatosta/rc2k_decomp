//! The not-enough-room page's "go on anyway" action: `ignore_room` 1, the answer 1 and the page
//! left.

pub fn ignore_room(
    install: &crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
) {
    install.flags.ignore_room.set(true);
    list_state.list_answer.set(1);
    list_state.list_leave.set(1);
}
