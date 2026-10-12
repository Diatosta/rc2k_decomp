//! `menu::load_background` (0x47EFFF): loads a menu screen's background picture and draws
//! the frame sprite onto it.
//!
//! In:  eax = the picture's path (may be written to, see `files::swap_missing_extension`);
//!      ebx as it comes: it is the palette the sprite is drawn with
//! Out: every register preserved. The carry flag is whatever the blitter leaves: the
//!      callers test it (and call a set carry fatal error 0x80712345), but it is no result
//!      of the loading. The Rust version returns with it clear, as the hooked blitter does.
//! `gfx::load_picture`, whose result is not looked at; `gfx::convert_picture`;
//! `g_pictureFlags` = the screen mode's flags (`dword_4F4934`) and 0xFFF;
//! `gfx::picture_as_surface`; sprite 0xA0 of `g_spriteTable` at (0, 0).
//! 76 callers, nearly all of them screen loops.
//! Shadow mode runs the original alone: everything it calls is compared in its own hook.
//! Manual patch with a private copy (0x35 bytes, four calls relocated): the first call
//! starts at the second byte, inside what the patch overwrites.
//! Refuses what the functions it calls refuse.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::blit_glyph::{BlitCall, record_calls, recording_works};
use crate::gfx::blit_marker_pair::sprite_table;
use crate::gfx::convert_picture::convert_picture_now;
use crate::gfx::load_picture::{LoaderTest, TEST_PATHS, load_picture};
use crate::gfx::picture::FLAGS_RVA;
use crate::gfx::picture_as_surface::picture_as_surface;
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::load_background", rva(0x47EFFF), thunk, &ORIGINAL)
    .private_copy(0x35)
    .relocate_calls(&[0x01, 0x06, 0x1A, 0x2E])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const MODE_FLAGS_RVA: usize = rva(0x4F4934); // dword_4F4934: the screen mode's flags
const FRAME_SPRITE: u32 = 0xA0;

// ---- Rust version ----

/// `palette` is ebx at entry.
pub unsafe fn load_background(
    path: *mut u8,
    palette: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        // Whether the file was read is not looked at.
        load_picture(path)?;
        convert_picture_now()?;
        game_write(FLAGS_RVA, game_read::<u32>(MODE_FLAGS_RVA) & 0xFFF);
        picture_as_surface()?;
        canvas.blit_glyph(BlitCall::new(FRAME_SPRITE, 0, 0, palette, sprite_table()))?;
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        match unsafe { load_background(r.eax as *mut u8, r.ebx, &mut GameCanvas) } {
            Ok(()) => r.set_carry(false),
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let flags: u32 = unsafe { game_read(FLAGS_RVA) };
    let expected = unsafe { game_read::<u32>(MODE_FLAGS_RVA) } & 0xFFF;
    if flags != expected || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!("the original left the picture's flags {flags:#x}, the screen mode says {expected:#x}")
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test: the loader's test files, with the sprite's drawing recorded and not done ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let test = match LoaderTest::begin() {
        Ok(test) => test,
        Err(why) => return t.skip(why),
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    for path in TEST_PATHS {
        for palette in [0u32, 0x1234_5678] {
            let run = |rust: bool| {
                test.prepare();
                let mut buffer = path.to_vec();
                buffer.push(0);
                let before = Regs {
                    eax: buffer.as_mut_ptr() as u32,
                    ebx: palette,
                    ..Regs::SENTINEL
                };
                let (ok, calls) = if rust {
                    let mut canvas = RecordingCanvas::new(false);
                    let ok = unsafe { load_background(buffer.as_mut_ptr(), palette, &mut canvas) }
                        .is_ok();
                    (ok, canvas.calls)
                } else {
                    let mut ro = before;
                    let calls = record_calls(true, || unsafe { call_game(&mut ro, original) });
                    (ro.preserved_except(&before, 0), calls)
                };
                (ok, calls, buffer, test.observed())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0, || {
                let first = theirs.3.iter().zip(&mine.3).position(|(a, b)| a != b);
                format!(
                    "path={:?} palette={palette:#x}: original ok={} calls {:?} left the path {:?}; rust ok={} calls {:?} left {:?}; first different byte of what they left {first:?}",
                    String::from_utf8_lossy(path),
                    theirs.0,
                    theirs.1,
                    String::from_utf8_lossy(&theirs.2),
                    mine.0,
                    mine.1,
                    String::from_utf8_lossy(&mine.2)
                )
            });
        }
    }
    test.end();
    t.finish()
}
