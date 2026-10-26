//! `gfx::load_jade` (0x4075F3): loads the glyphs of the fatal error's own font,
//! `files\d\gfx\jade.bun`, and works out its colour.
//!
//! In:  nothing
//! Out: every register preserved
//! Up to 0x1400 bytes of the file to `unk_4F52C4` (the glyph table of the four font
//! objects at 0x51DE30 on); a file that cannot be read is fatal error 0x80F0B0D1, one
//! that fills all 0x1400 bytes 0x80F0B0D2. Then `dword_4F66C4` (those fonts' palette) =
//! white in the surface's format (`sub_407629`: `gfx::components_to_pixel` of 0xFFFFFF).
//! One caller, `system::main_loop`.
//! Refuses what `read_file` refuses.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{FatalError, Refused};
use crate::files::read_file::read_file;
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::pixel_format::surface_format;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const PATH_RVA: usize = rva(0x4F66C8); // "files\d\gfx\jade.bun"
const GLYPHS_RVA: usize = rva(0x4F52C4); // unk_4F52C4
const ROOM: u32 = rc2k_core::gfx::screenshot::JADE_ROOM;
const COLOUR_RVA: usize = rva(0x4F66C4); // dword_4F66C4

// ---- Rust version ----

pub unsafe fn load_jade(path: *const u8) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::gfx::screenshot::jade_read`.
        use rc2k_core::gfx::screenshot::{JADE_COLOUR, jade_read};
        let read = read_file(path, game_addr(GLYPHS_RVA) as *mut u8, ROOM, 0)?;
        jade_read(read.ok, read.bytes_read).map_err(|code| FatalError { code, message: "" })?;
        game_write(
            COLOUR_RVA,
            components_to_pixel(&surface_format(), JADE_COLOUR),
        );
        Ok(())
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_jade(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
