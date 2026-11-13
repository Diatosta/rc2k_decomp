//! Three frames of the installer's backdrop alone (`install::backdrop_frame`), as a page opens
//! and closes. Called by `install::run_page`.

use crate::errors::Refused;
use crate::install::installer::run_frame;

pub fn show_backdrop(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    for _ in 0..3 {
        run_frame(
            o,
            Some(&|| crate::install::backdrop_frame::backdrop_frame(o)),
        )?;
    }
    Ok(())
}
