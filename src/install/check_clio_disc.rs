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
//! One caller, `start`.
//! Self-test: a redirected copy in worlds of the test's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::install::check_at_start::{INSTALLING_RVA, RECORD_RVA, set_script_entry};
use crate::install::check_test::MEDIA_MODE_RVA;
use crate::install::outside::{Game, Outside};
use crate::install::read_registry::{SOURCE_RVA, STATUS_RVA};
use crate::install::runs_from_source::runs_from_source;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const CLIO_FILE_RVA: usize = rva(0x529DB8); // "files\b\obj0570.ro3"
const HAS_CLIO_RVA: usize = rva(0x529DB4); // g_hasClio
const INSTALLER_DATA_RVA: usize = rva(0x51BEA1); // "files\d\gfx\install.bun"
const BASE_B_RVA: usize = rva(0x5295B4); // unk_5295B4
const B_BEFORE_RVA: usize = rva(0x95E8F8); // byte_95E8F8
const B_AFTER_RVA: usize = rva(0x95E9F8); // byte_95E9F8
const CAR_NAME_RVA: usize = rva(0x529F03); // "renault clio sport V6 (A7)"
const CAR_NAME_LEN: usize = 0x1B;
const STEP_RVA: usize = rva(0x51DD48); // off_51DD48
const CLIO_STEP_RVA: usize = rva(0x41675B); // sub_41675B

// ---- Rust version ----

pub unsafe fn check_clio_disc(outside: &mut dyn Outside) -> Result<(), Refused> {
    unsafe {
        if !outside.file_exists(game_addr(CLIO_FILE_RVA) as *const u8)? {
            return Ok(());
        }
        game_write(HAS_CLIO_RVA, 1u32);
        if game_read::<u32>(INSTALLING_RVA) == 1
            || outside.file_exists(game_addr(RECORD_RVA) as *const u8)?
            || runs_from_source(outside)?
            || game_read::<u8>(STATUS_RVA) != 1
            || game_read::<u8>(SOURCE_RVA) == 0
            || outside.file_exists(game_addr(INSTALLER_DATA_RVA) as *const u8)?
        {
            return Ok(());
        }
        let base_b = game_addr(BASE_B_RVA) as *const u8;
        if !outside.normalise_path(base_b, game_addr(B_BEFORE_RVA) as *mut u8)? {
            return Ok(());
        }
        outside.set_directory(game_addr(SOURCE_RVA) as *const u8);
        if !outside.normalise_path(base_b, game_addr(B_AFTER_RVA) as *mut u8)? {
            return Ok(());
        }
        outside.read_record()?;
        if game_read::<u32>(MEDIA_MODE_RVA) < 2 {
            outside.log(
                c"CLIO: Requires normal or full installation"
                    .as_ptr()
                    .cast(),
            );
            return Err(FatalError {
                code: 0x9000_2105,
                message: "This update disc requires normal or full main installation\nPlease update the main installation first and try again",
            }
            .into());
        }
        let name = std::slice::from_raw_parts_mut(game_addr(CAR_NAME_RVA) as *mut u8, CAR_NAME_LEN);
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
        game_write(STEP_RVA, game_addr(CLIO_STEP_RVA) as u32);
        Ok(())
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { check_clio_disc(&mut Game) }?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
