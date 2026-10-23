//! `gfx::read_picture_file` (0x4119F4): reads a picture file, PNG or PCX, into the picture
//! buffer as 32-bit pixels.
//!
//! In:  eax = path, which may be written to (see `files::swap_missing_extension`)
//! Out: carry = failed; every register preserved
//! A path ending in `.pcx` goes to `gfx::read_pcx`. Any other first has its ending swapped
//! if the file is missing, and then goes to `gfx::read_png` and, if that fails, to
//! `read_pcx` (which is how a `.png` path whose file is missing ends up read as PCX).
//! That is with the setting `dword_51B97C` at 0, as it is in the owner's game. With another
//! value a path ending in `.png` is not read at all (the value 2: success, nothing done) or
//! replaced by `files\d\gfx\chatzone.pcx`, and other paths go to `read_pcx` alone.
//! 14 callers. Refuses a path of fewer than four characters (the original looks at the four
//! bytes before the text's end) and what the readers refuse. The name is a reading.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::files::swap_missing_extension::swap_missing_extension;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::gfx::read_pcx::read_pcx;
use crate::gfx::read_png::read_png;
use crate::strings::text_length::text_length;

// ---- game data ----

const PNG_SETTING_RVA: usize = rva(0x51B97C); // dword_51B97C: a setting; 0 = PNG files are read
const CHATZONE_PATH_RVA: usize = rva(0x5AB400); // "files\d\gfx\chatzone.pcx"

// ---- Rust version ----

/// The path's last four characters. Converted: `rc2k_core::gfx::loading::ending`.
unsafe fn ending(path: *const u8) -> Result<[u8; 4], BadInput> {
    let text = unsafe { std::ffi::CStr::from_ptr(path as *const std::ffi::c_char) }.to_bytes();
    // As the original measures it.
    unsafe { text_length(path) }?;
    Ok(rc2k_core::gfx::loading::ending(text)?)
}

/// Returns whether a picture was read. Converted: `rc2k_core::gfx::loading::reader`.
pub unsafe fn read_picture_file(path: *mut u8) -> Result<bool, Refused> {
    use rc2k_core::gfx::loading::{Reader, reader};
    unsafe {
        if &ending(path)? == b".pcx" {
            return Ok(read_pcx(path)?);
        }
        swap_missing_extension(path)?;
        match reader(ending(path)?, game_read::<u32>(PNG_SETTING_RVA)) {
            Reader::PngThenPcx => Ok(read_png(path)? || read_pcx(path)?),
            Reader::Pcx => Ok(read_pcx(path)?),
            Reader::Nothing => Ok(true),
            Reader::Chatzone => Ok(read_pcx(game_addr(CHATZONE_PATH_RVA) as *const u8)?),
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let path = r.eax as *mut u8;
    let worked = match unsafe { read_picture_file(path) } {
        Ok(worked) => worked,
        // The standalone has no original to hand a refusal to: the picture is not read.
        Err(_) if crate::framework::standalone() => false,
        Err(e) => return Err(e),
    };
    r.set_carry(!worked);
    Ok(())
}

// ---- self-test: a PNG and a PCX file of the test's own, asked for by several paths, with
// the setting at each of its values ----
