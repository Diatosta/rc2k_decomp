//! `install::check_clio_disc` (0x416653): the Clio update.
//!
//! In:  nothing
//! Out: every register preserved; or it does not return (fatal error 0x90002105)
//! `g_hasClio` (0x529DB4) = 1 if `files\b\obj0570.ro3` is there (`sub_47C63F` reads it:
//! the extra car), and nothing at all if not. The rest is for a start from the update's
//! disc: only if the installer is not running, there is no `var\ral.zog` here, the
//! registry records an installation (status 1, a source) which is not where the game was
//! started (`install::runs_from_source`), and `files\d\gfx\install.bun` is not here. Then
//! base directory B (`unk_5295B4`) goes through `normalise_path` to `byte_95E8F8`, the
//! installation's source is made the current directory, B goes again to `byte_95E9F8`,
//! and the installation's record is read. An installation below "normal" (`g_mediaMode`
//! under 2) is "CLIO: Requires normal or full installation" in the log and the fatal
//! error. Otherwise the installer's script is set up (as in `check_at_start`, with
//! string 0x1038), the car's name `aRenaultClioSpo` is put in capitals up to its `(`,
//! where a line feed ends it, and `off_51DD48` becomes `sub_41675B`.
//! One caller, `start`, which calls it directly (2026-10-23).

use crate::errors::{BadInput, FatalError, Refused};
use crate::install::check_at_start::set_script_entry;
use crate::install::outside::Outside;
use crate::install::runs_from_source::runs_from_source;
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;

const CAR_NAME_LEN: usize = 0x1B;

// ---- game data ----

const CLIO_STEP: u32 = 0x41675B; // sub_41675B

// ---- Rust version ----

pub fn check_clio_disc(outside: &mut dyn Outside) -> Result<(), Refused> {
    let install = &crate::install::store::INSTALL;
    if !outside.file_exists(&install.clio_file.text())? {
        return Ok(());
    }
    install.has_clio.set(1);
    if INSTALLER_RUN.installing.get() == 1
        || outside.file_exists(&crate::install::installer::INSTALL_TEXTS.record_path.text())?
        || runs_from_source(outside)?
        || INSTALLER_NAMES.status.get() != 1
        || INSTALLER_NAMES.source.get(0) == 0
        || outside.file_exists(&INSTALLER_NAMES.installer_data.text())?
    {
        return Ok(());
    }
    // `base_b` (`unk_5295B4`) is an empty text whose size is not known (section 13: the
    // installer writes it elsewhere): read as a text at its place.
    let base_b = crate::files::cstring(
        crate::pointers::GamePtr::to(&crate::install::installer::INSTALL_TEXTS.base_b),
        "the installer's base",
    )?;
    let Some(a) = outside.normalise_path(&base_b)? else {
        return Ok(());
    };
    INSTALLER_PATHS.path_a.write_at(0, &a, "a path")?;
    outside.set_directory(&INSTALLER_NAMES.source.text());
    let Some(b) = outside.normalise_path(&base_b)? else {
        return Ok(());
    };
    INSTALLER_PATHS.path_b.write_at(0, &b, "a path")?;
    outside.read_record()?;
    if crate::files::state::MEDIA_MODE.load(std::sync::atomic::Ordering::Relaxed) < 2 {
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
    set_script_entry(0x1038);
    name[..end].make_ascii_uppercase();
    name[end] = b'\n';
    name[end + 1] = 0;
    install.clio_car_name.write(&name);
    INSTALLER_PAGES.step.set_u32(0, CLIO_STEP);
    Ok(())
}
