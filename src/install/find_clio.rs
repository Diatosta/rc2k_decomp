//! Whether the Clio's car is among the game's files: `files\b\obj0570.ro3` there sets the
//! has-Clio flag, which makes the front end offer the car. Called once, by `start`.

use crate::errors::Refused;
use crate::install::outside::Outside;

pub fn find_clio(
    install: &crate::install::owner::Install,
    outside: &mut dyn Outside,
) -> Result<(), Refused> {
    let vars = &install.vars;
    if outside.file_exists(crate::install::store::CLIO_FILE)? {
        vars.has_clio.set(1);
    }
    Ok(())
}
