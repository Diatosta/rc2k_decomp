//! `gfx::read_texture_picture` (0x405356): a texture's picture read into the picture
//! buffer.
//!
//! In:  eax = the path (may be written to, as `gfx::read_picture_file` may), or 0
//! Out: read: ebx = the decoded pixels (`g_decodedPixels`), ecx, edx = their width
//!      and height (`g_decodedWidth`, `g_decodedHeight`), carry clear; not read: ebx =
//!      the picture buffer, ecx, edx = the picture's size, carry set, or clear for no
//!      path; everything else preserved
//!
//! The picture's size 256 x 256 with rows that follow one another (row length
//! 0xFFFFFFFD), its first 0x10000 dwords filled with -1, then `read_picture_file` if
//! there is a path. Five call sites: `gfx::load_picture_to_texture` (twice) and
//! `scene::upload_plate` (twice), the second of each pair for the picture kept for
//! the alpha.
//! Refuses no picture buffer, and what `read_picture_file` refuses.
//! Shadow mode runs the original alone: the reading is compared in its own hook.
//! Self-test: `gfx::load_picture`'s files, and no path.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EBX, ECX, EDX, RawHook, Regs, SelfTest, call_game, game_read, hook_asm, rva, shadow,
};
use crate::gfx::load_picture::{LoaderTest, TEST_PATHS};
use crate::gfx::pcx::{DECODED_HEIGHT_RVA, DECODED_WIDTH_RVA, PIXELS_RVA};
use crate::gfx::picture::{HEIGHT_RVA, WIDTH_RVA, picture};
use crate::gfx::read_picture_file::read_picture_file;
use crate::screens::harness::Left;
use rc2k_core::gfx::picture_size as sizes;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::read_texture_picture", rva(0x405356), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers it leaves: the pixels, their width and height, and whether the
/// reading failed.
pub unsafe fn read_texture_picture(path: *mut u8) -> Result<(u32, u32, u32, bool), Refused> {
    unsafe {
        let buffer = picture();
        if buffer.is_null() {
            return Err(BadInput("no picture buffer").into());
        }
        // Converted: `rc2k_core::gfx::picture_size` (`TEXTURE`, `TEXTURE_FILLED`).
        crate::gfx::picture::set_size(sizes::TEXTURE);
        std::ptr::write_bytes(buffer, 0xFF, sizes::TEXTURE_FILLED * 4);
        let failed = if path.is_null() {
            false
        } else if read_picture_file(path)? {
            return Ok((
                game_read(PIXELS_RVA),
                game_read(DECODED_WIDTH_RVA),
                game_read(DECODED_HEIGHT_RVA),
                false,
            ));
        } else {
            true
        };
        Ok((
            buffer as u32,
            game_read(WIDTH_RVA),
            game_read(HEIGHT_RVA),
            failed,
        ))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { read_texture_picture(r.eax as *mut u8) } {
        Ok((pixels, width, height, failed)) => Left::carry(failed)
            .with(EBX, pixels)
            .with(ECX, width)
            .with(EDX, height)
            .apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    match unsafe { read_texture_picture(r.eax as *mut u8) } {
        Ok((pixels, width, height, failed)) => Left::carry(failed)
            .with(EBX, pixels)
            .with(ECX, width)
            .with(EDX, height)
            .apply(r),
        Err(e) => return Err(e),
    }
    Ok(())
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = match LoaderTest::begin() {
        Ok(test) => test,
        Err(why) => return t.skip(why),
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    let paths: Vec<Option<&[u8]>> = TEST_PATHS
        .iter()
        .map(|&path| Some(path))
        .chain([None])
        .collect();
    for path in paths {
        let run = |rust: bool| {
            test.prepare();
            let mut buffer = path.map(|p| [p, &[0]].concat());
            let at = buffer
                .as_mut()
                .map_or(std::ptr::null_mut(), |b| b.as_mut_ptr());
            let before = Regs {
                eax: at as u32,
                ..Regs::SENTINEL
            };
            let left = if rust {
                unsafe { read_texture_picture(at) }
                    .ok()
                    .map(|(b, c, d, f)| {
                        let mut r = before;
                        Left::carry(f)
                            .with(EBX, b)
                            .with(ECX, c)
                            .with(EDX, d)
                            .apply(&mut r);
                        (r, f)
                    })
            } else {
                let mut ro = before;
                unsafe { call_game(&mut ro, original) };
                Some((ro, ro.carry()))
            };
            let regs = left.map(|(r, carry)| {
                (
                    [
                        // eax: the path, in a buffer of each side's own
                        (r.eax == before.eax) as u32,
                        r.ebx,
                        r.ecx,
                        r.edx,
                        r.esi,
                        r.edi,
                        r.ebp,
                    ],
                    carry,
                )
            });
            (regs, buffer, test.observed())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.is_some(), || {
            let first = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "path={:?}: original {:x?}, rust {:x?}; first different byte of what they left {first:?}",
                path.map(String::from_utf8_lossy),
                theirs.0,
                mine.0
            )
        });
    }
    test.end();
    t.finish()
}
