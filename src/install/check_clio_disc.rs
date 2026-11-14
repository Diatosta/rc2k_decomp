//! The Clio update: sets the has-Clio flag if `files\b\obj0570.ro3` is there and, for a start
//! from the update's disc with a "normal" or better installation recorded, sets up the
//! installer's script so that the third page's action is `install::install_clio`
//! (otherwise "CLIO: Requires normal or full installation" and a fatal error). Called once, by
//! `start`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::install::check_at_start::set_script_entry;
use crate::install::outside::Outside;
use crate::install::runs_from_source::runs_from_source;

const CAR_NAME_LEN: usize = 0x1B;

const CLIO_STEP: u32 = 0x41675B; // `install::install_clio`

pub fn check_clio_disc(
    g: &crate::install::owner::Install,
    outside: &mut dyn Outside,
) -> Result<(), Refused> {
    let install = &g.vars;
    if !outside.file_exists(&install.clio_file.text())? {
        return Ok(());
    }
    install.has_clio.set(1);
    if g.flags.installing.get()
        || outside.file_exists(&g.texts.record_path.text())?
        || runs_from_source(g, outside)?
        || g.names.status.get() != 1
        || g.names.source.get(0) == 0
        || outside.file_exists(&g.names.installer_data.text())?
    {
        return Ok(());
    }
    // Base directory B is an empty text whose size is not known (section 13: the
    // installer writes it elsewhere): read as a text at its place.
    let base_b = crate::files::cstring(&g.texts.base_b, "the installer's base")?;
    let Some(a) = outside.normalise_path(&base_b)? else {
        return Ok(());
    };
    g.paths.path_a.write_at(0, &a, "a path")?;
    outside.set_directory(&g.names.source.text());
    let Some(b) = outside.normalise_path(&base_b)? else {
        return Ok(());
    };
    g.paths.path_b.write_at(0, &b, "a path")?;
    outside.read_record()?;
    if crate::files::state::MEDIA_MODE.get() < 2 {
        outside.log(b"CLIO: Requires normal or full installation");
        return Err(FatalError {
            code: 0x9000_2105,
            message: "This update disc requires normal or full main installation\nPlease update the main installation first and try again",
        }
        .into());
    }
    let mut name = install.clio_car_name.read();
    let Some(end) = name
        .iter()
        .position(|&c| c == b'(')
        .filter(|end| end + 1 < CAR_NAME_LEN)
    else {
        return Err(BadInput("the Clio's name has no ( any more").into());
    };
    set_script_entry(g, 0x1038);
    name[..end].make_ascii_uppercase();
    name[end] = b'\n';
    name[end + 1] = 0;
    install.clio_car_name.write(&name);
    g.pages.action_pages[2].action.set(CLIO_STEP);
    Ok(())
}
