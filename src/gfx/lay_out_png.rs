//! `gfx::lay_out_png` (0x41CCF7): sets the layout variables for a picture `decode_png` has
//! just decoded.
//!
//! In:  nothing
//! Out: every register preserved
//! The decoded width and height are the info block's (`buf`, 0x93E8B0, a `PngInfo`); pixels
//! are 8 bits when bit 8 of `g_pictureFlags` is set, else 32; a row follows the one before
//! without a gap; the rows are at `g_pictureBuffer`. Last, `palette_to_dwords` on the info
//! block's palette, whether the file had one or not. The planes, the bytes of a plane's row
//! and the file's own size, which `lay_out_pcx` sets, are left alone. One caller,
//! `gfx::read_png`. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, game_ptr, game_read, game_write};
use crate::gfx::palette_to_dwords::palette_to_dwords;
use crate::gfx::pcx::{
    DECODED_HEIGHT_RVA, DECODED_ROW_BYTES_RVA, DECODED_ROW_DWORDS_RVA, DECODED_WIDTH_RVA,
    PALETTE_RVA, PIXEL_BITS_RVA, PIXEL_BYTES_RVA, PIXELS_RVA, ROW_GAP_RVA, STRIDE_RVA,
};
use crate::gfx::picture::{FLAGS_RVA, PICTURE_RVA};
use crate::gfx::png::{INFO_RVA, PngInfo};

// ---- Rust version ----

pub unsafe fn lay_out_png() {
    unsafe {
        let info: *mut PngInfo = game_ptr(INFO_RVA);
        // Converted: `rc2k_core::gfx::png::layout`.
        let layout = rc2k_core::gfx::png::layout(
            (&raw const (*info).width).read_unaligned(),
            (&raw const (*info).height).read_unaligned(),
            game_read(FLAGS_RVA),
        );
        for (at, value) in [
            DECODED_WIDTH_RVA,
            DECODED_HEIGHT_RVA,
            PIXEL_BITS_RVA,
            PIXEL_BYTES_RVA,
            DECODED_ROW_BYTES_RVA,
            DECODED_ROW_DWORDS_RVA,
            STRIDE_RVA,
            ROW_GAP_RVA,
        ]
        .into_iter()
        .zip(layout)
        {
            game_write(at, value);
        }
        game_write(PIXELS_RVA, game_read::<u32>(PICTURE_RVA));
        // Neither is null: both are the game's own tables.
        let _ = palette_to_dwords(
            (&raw const (*info).palette).cast(),
            game_addr(PALETTE_RVA) as *mut u32,
        );
    }
}

// ---- self-test: info blocks and settings of the test's own in the game's variables, which
// are put back ----
