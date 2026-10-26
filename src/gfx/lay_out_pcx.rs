//! `gfx::lay_out_pcx` (0x411A82): works out from a PCX header how the picture is to be
//! decoded, and converts its palette.
//!
//! In:  nothing
//! Out: every register preserved
//! From the header at `dword_51B5F4`: the planes and the bytes of a plane's row; the
//! picture's width and height (right - left + 1, bottom - top + 1, as words), and those cut
//! to the picture buffer's size (`g_pictureWidth`, `g_pictureHeight`) and to the limits
//! `dword_51B974` / `dword_51B978`. Pixels are 8 bits when bit 8 of `g_pictureFlags` is set,
//! else 32. The stride from one decoded row to the next comes from `g_pictureRowBytes`:
//! 0xFFFFFFFF = a screen's width of pixels (800), 0xFFFFFFFE = the buffer's width of
//! pixels, 0xFFFFFFFD = the decoded row itself, anything else = that many bytes. The rows
//! go to `g_pictureBuffer`. Last, `palette_to_dwords` on the file's palette.
//! All of it lands in the variables listed in `gfx/pcx.rs`. Three callers, the PCX readers.
//! The high half of the two sizes is what the instruction before left there (16-bit moves
//! into eax), which the Rust version follows.
//! The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write};
use crate::gfx::palette_to_dwords::palette_to_dwords;
use crate::gfx::pcx::{
    DECODED_HEIGHT_RVA, DECODED_ROW_BYTES_RVA, DECODED_ROW_DWORDS_RVA, DECODED_WIDTH_RVA,
    FILE_HEIGHT_RVA, FILE_PALETTE_RVA, FILE_WIDTH_RVA, HEADER_RVA, HEIGHT_LIMIT_RVA,
    LINE_BYTES_RVA, PALETTE_RVA, PIXEL_BITS_RVA, PIXEL_BYTES_RVA, PIXELS_RVA, PLANES_RVA,
    ROW_GAP_RVA, STRIDE_RVA, WIDTH_LIMIT_RVA,
};
use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

/// Converted: `rc2k_core::gfx::pcx::lay_out`, on the header and the variables in game memory.
/// Returns the layout it stored.
pub unsafe fn lay_out_pcx() -> rc2k_core::gfx::pcx::Layout {
    unsafe {
        let target = rc2k_core::gfx::pcx::Target {
            width: game_read(WIDTH_RVA),
            height: game_read(HEIGHT_RVA),
            width_limit: game_read(WIDTH_LIMIT_RVA),
            height_limit: game_read(HEIGHT_LIMIT_RVA),
            flags: game_read(FLAGS_RVA),
            row_bytes: game_read(ROW_BYTES_RVA),
        };
        let l = rc2k_core::gfx::pcx::lay_out(&game_read(HEADER_RVA), &target);
        for (at, value) in [
            (PLANES_RVA, l.planes),
            (LINE_BYTES_RVA, l.line_bytes),
            (FILE_WIDTH_RVA, l.file_width),
            (DECODED_WIDTH_RVA, l.width),
            (FILE_HEIGHT_RVA, l.file_height),
            (DECODED_HEIGHT_RVA, l.height),
            (PIXEL_BITS_RVA, l.pixel_bits),
            (PIXEL_BYTES_RVA, l.pixel_bytes),
            (DECODED_ROW_BYTES_RVA, l.row_bytes),
            (DECODED_ROW_DWORDS_RVA, l.row_dwords),
            (STRIDE_RVA, l.stride),
            (ROW_GAP_RVA, l.row_gap),
        ] {
            game_write(at, value);
        }
        game_write(PIXELS_RVA, game_read::<u32>(PICTURE_RVA));
        // Neither is null: both are the game's own tables.
        let _ = palette_to_dwords(
            game_addr(FILE_PALETTE_RVA) as *const u8,
            game_addr(PALETTE_RVA) as *mut u32,
        );
        l
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { lay_out_pcx() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test: headers and settings of the test's own in the game's variables, which are
// put back ----
