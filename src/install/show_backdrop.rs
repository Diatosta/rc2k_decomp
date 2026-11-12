//! `install::show_backdrop` (0x41525D): three frames (`screens::run_frame`) of the
//! installer's backdrop alone (`install::backdrop_frame`), as a page opens and closes.
//!
//! Caller `install::run_page` (twice).

use crate::errors::Refused;
use crate::install::installer::run_frame;

/// The backdrop's frame run three times.
pub fn show_backdrop(system: &crate::system::owner::System) -> Result<(), Refused> {
    for _ in 0..3 {
        run_frame(
            system,
            Some(&crate::install::backdrop_frame::backdrop_frame),
        )?;
    }
    Ok(())
}
