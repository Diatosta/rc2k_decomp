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

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
};
use crate::gfx::palette_to_dwords::palette_to_dwords;
use crate::gfx::pcx::{
    DECODED_HEIGHT_RVA, DECODED_ROW_BYTES_RVA, DECODED_ROW_DWORDS_RVA, DECODED_WIDTH_RVA,
    PALETTE_RVA, PIXEL_BITS_RVA, PIXEL_BYTES_RVA, PIXELS_RVA, ROW_GAP_RVA, STRIDE_RVA,
    layout_regions,
};
use crate::gfx::picture::{FLAGS_RVA, PICTURE_RVA};
use crate::gfx::png::{INFO_RVA, PngInfo};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::lay_out_png", rva(0x41CCF7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn lay_out_png() {
    unsafe {
        let info: *mut PngInfo = game_ptr(INFO_RVA);
        let width = (&raw const (*info).width).read_unaligned();
        game_write(DECODED_WIDTH_RVA, width);
        game_write(
            DECODED_HEIGHT_RVA,
            (&raw const (*info).height).read_unaligned(),
        );
        let bits: u32 = if game_read::<u32>(FLAGS_RVA) & 0x100 != 0 {
            8
        } else {
            32
        };
        game_write(PIXEL_BITS_RVA, bits);
        game_write(PIXEL_BYTES_RVA, bits >> 3);
        let row_bytes = (bits >> 3).wrapping_mul(width);
        game_write(DECODED_ROW_BYTES_RVA, row_bytes);
        game_write(DECODED_ROW_DWORDS_RVA, row_bytes >> 2);
        game_write(STRIDE_RVA, row_bytes);
        game_write(ROW_GAP_RVA, 0u32);
        game_write(PIXELS_RVA, game_read::<u32>(PICTURE_RVA));
        // Neither is null: both are the game's own tables.
        let _ = palette_to_dwords(
            (&raw const (*info).palette).cast(),
            game_addr(PALETTE_RVA) as *mut u32,
        );
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { lay_out_png() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &layout_regions(), &rust);
}

// ---- self-test: info blocks and settings of the test's own in the game's variables, which
// are put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = layout_regions();
    let inputs = [
        (game_addr(INFO_RVA) as u32, std::mem::size_of::<PngInfo>()),
        at(FLAGS_RVA),
        at(PICTURE_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    for (width, height, flags) in [
        (800u32, 600u32, 0x802u32),
        (1, 1, 0x802),
        (640, 480, 0x900),
        (0, 0, 0),
        (0xFFFF_FFFF, 7, 0x802),
    ] {
        let prepare = || unsafe {
            restore(&regions, &vec![0xAA; regions.iter().map(|r| r.1).sum()]);
            let info: *mut PngInfo = game_ptr(INFO_RVA);
            (&raw mut (*info).width).write_unaligned(width);
            (&raw mut (*info).height).write_unaligned(height);
            for n in 0..0x300usize {
                (&raw mut (*info).palette)
                    .cast::<u8>()
                    .add(n)
                    .write((n * 5 + width as usize) as u8);
            }
            game_write(FLAGS_RVA, flags);
        };
        check_state(
            &mut t,
            &format!("{width}x{height} flags {flags:#x}"),
            call_original,
            Regs::SENTINEL,
            &prepare,
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
