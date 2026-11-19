//! The advanced page's "delete all saved data": the player's own files (`var` and `savedata`,
//! what stays is in `rc2k_core::saves::wipe`) removed, and the program kept from writing them
//! back while it closes. Meant for the platforms where the folders are not at hand (a phone, the
//! browser's storage, a handheld).
//!
//! The deletion is made when the question is answered yes, and once more as the program
//! ends: a file written in between (the options, the window's mode, the debug window's layout)
//! would bring the settings back.

use crate::errors::Refused;
use crate::owners::Owners;
use rc2k_core::saves::wipe::{Outcome, delete_player_data};
use rc2k_core::strings::own::id;
use std::sync::atomic::{AtomicBool, Ordering};

/// Seconds the question keeps its Yes out of reach, so that a key pressed in passing cannot
/// confirm it.
const YES_HELD_SECS: u32 = 5;

/// The data was deleted: nothing may save again.
static DELETED: AtomicBool = AtomicBool::new(false);

/// Whether the player's data was deleted in this run (the program is closing and saves nothing).
pub fn deleted() -> bool {
    DELETED.load(Ordering::Relaxed)
}

/// The player's files removed now.
fn delete_now() -> Outcome {
    let out = delete_player_data(
        &crate::files::host_path(b"var"),
        &crate::files::host_path(b"savedata"),
    );
    crate::logfile::program_log::log(&format!(
        "saves: all saved data deleted ({} files removed, {} could not be)",
        out.removed, out.failed
    ));
    out
}

/// What the program does as it ends after a deletion: the files made since are removed too.
pub fn delete_again_at_end() {
    if deleted() {
        delete_now();
    }
}

/// The row chosen: the question (No is chosen to begin with and is Escape's answer; Yes opens after a few seconds), then the
/// deletion, a line saying so, and the program closes as when the window is closed.
pub fn choose(o: Owners<'static>) -> Result<(), Refused> {
    if !crate::error_dialog::ask_yes(id::DELETE_TITLE, id::DELETE_ASK, YES_HELD_SECS) {
        return Ok(());
    }
    DELETED.store(true, Ordering::Relaxed);
    let out = delete_now();
    let said = if out.failed == 0 {
        id::DELETE_DONE
    } else {
        id::DELETE_FAILED
    };
    crate::error_dialog::notice(&crate::error_dialog::text_with(said, ""));
    o.system.session.flags.quitting.set(true);
    Ok(())
}
