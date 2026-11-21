//! The one door from the race to the error dialog: the notices the race and the replay viewer
//! show, and the going back to the main menu after a trouble.

use crate::errors::Refused;

/// Shows the player the text of the own-text `id`, with `with` filled in.
pub fn notice(id: u32, with: &str) {
    crate::error_dialog::notice(&crate::error_dialog::text_with(id, with));
}

/// Runs `f`; a trouble that can be gone back from comes out as `Err(reason)`.
pub fn guarded<T>(f: impl FnOnce() -> Result<T, Refused>) -> Result<Result<T, Refused>, String> {
    crate::error_dialog::recover::guarded(f)
}

/// Leaves everything up to the nearest `guarded` call; `reason` is what the player is told.
pub fn back(reason: &str) -> ! {
    crate::error_dialog::recover::back(reason)
}
