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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::palette_to_dwords::palette_to_dwords;
use crate::gfx::pcx::{
    DECODED_HEIGHT_RVA, DECODED_ROW_BYTES_RVA, DECODED_ROW_DWORDS_RVA, DECODED_WIDTH_RVA,
    FILE_HEIGHT_RVA, FILE_PALETTE_RVA, FILE_WIDTH_RVA, HEADER_LEN, HEADER_RVA, HEIGHT_LIMIT_RVA,
    LINE_BYTES_RVA, PALETTE_RVA, PIXEL_BITS_RVA, PIXEL_BYTES_RVA, PIXELS_RVA, PLANES_RVA,
    ROW_GAP_RVA, STRIDE_RVA, WIDTH_LIMIT_RVA, layout_regions, reader_inputs, test_header,
    test_palette,
};
use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::lay_out_pcx", rva(0x411A82), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { lay_out_pcx() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &layout_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test: headers and settings of the test's own in the game's variables, which are
// put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = layout_regions();
    let inputs = [
        &reader_inputs()[..],
        &[
            (game_addr(HEADER_RVA) as u32, HEADER_LEN),
            (game_addr(FILE_PALETTE_RVA) as u32, 0x300),
            at(FLAGS_RVA),
        ],
    ]
    .concat();
    let saved = (capture(&regions), capture(&inputs));
    let palette = test_palette();

    // (left, top, width, height, planes, bytes in a plane's row)
    let headers = [
        (0u16, 0u16, 10u16, 4u16, 1u8, 10u16),
        (3, 5, 640, 480, 3, 640),
        (0, 0, 1024, 768, 3, 1024),
        (100, 100, 1, 1, 1, 2),
        (10, 10, 0, 0, 1, 0), // right and bottom one short of left and top
        (0, 0, 0xFFFF, 0xFFFF, 255, 0xFFFF),
    ];
    // (buffer width, buffer height, row length setting, flags, width limit, height limit)
    let settings = [
        (800u32, 600u32, 0xFFFF_FFFDu32, 0x800u32, 0x800u32, 0x800u32),
        (800, 600, 0xFFFF_FFFF, 0x800, 0x800, 0x800),
        (256, 256, 0xFFFF_FFFE, 0x900, 0x800, 0x800),
        (800, 600, 3200, 0x100, 64, 32),
        (0x10_0000, 0x10_0000, 0xFFFF_FFFD, 0, 0x20_0000, 0x20_0000),
    ];
    for (left, top, width, height, planes, line_bytes) in headers {
        for (buffer_width, buffer_height, row_setting, flags, width_limit, height_limit) in settings
        {
            let header = test_header(left, top, width, height, planes, line_bytes);
            let prepare = || unsafe {
                restore(&regions, &vec![0xAA; regions.iter().map(|r| r.1).sum()]);
                std::ptr::copy_nonoverlapping(
                    header.as_ptr(),
                    game_addr(HEADER_RVA) as *mut u8,
                    HEADER_LEN,
                );
                std::ptr::copy_nonoverlapping(
                    palette.as_ptr(),
                    game_addr(FILE_PALETTE_RVA) as *mut u8,
                    palette.len(),
                );
                game_write(WIDTH_RVA, buffer_width);
                game_write(HEIGHT_RVA, buffer_height);
                game_write(ROW_BYTES_RVA, row_setting);
                game_write(FLAGS_RVA, flags);
                game_write(WIDTH_LIMIT_RVA, width_limit);
                game_write(HEIGHT_LIMIT_RVA, height_limit);
            };
            check_state(
                &mut t,
                &format!(
                    "header left={left} top={top} {width}x{height} planes={planes} line={line_bytes}; buffer {buffer_width}x{buffer_height} row setting {row_setting:#x} flags {flags:#x} limits {width_limit} {height_limit}"
                ),
                call_original,
                Regs::SENTINEL,
                &prepare,
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
