//! `screens::draw_message_box` (0x415EA6): string 0x1FE ("not in this version" and the
//! like) in a box in the middle of the surface: the string measured (font 0x51E824, x as
//! the caller's ecx), the box 0x32 wider and 0x19 higher (`dword_52983C` .. `dword_529848`:
//! x, y, width, height) filled with colour 1 (`gfx::fill_rect`) and framed with colour 0xA
//! (`gfx::frame_rect`), both through `g_fontPalettes`, then the string drawn centred (font
//! 0x51E904).
//!
//! In:  ecx = the x the measuring starts at
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `screens::cheat_locked_note` .. (0x4AAAD5, 0x48ADF2 and others). Refuses what
//! `gfx::fill_rect` refuses (a surface of neither 16 nor 32 bits), before anything. Shadow
//! mode runs the original alone (it draws on the surface). Self-test: a copy (its calls to
//! the game's, hooked) against the Rust version, both drawing on a test surface; skipped
//! without the fonts.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::screens::font_at;
use crate::text::draw_text::{Canvas, GameCanvas};
use rc2k_core::screens::message_box::{FILL, FRAME, STRING, layout};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_message_box", rva(0x415EA6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BOX: usize = 0x52983C; // dword_52983C .. dword_529848

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The Rust version (`x` the caller's ecx).
pub unsafe fn draw_message_box(x: i32, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::gfx::rect::{fill_rect, frame_rect};
    use crate::text::draw_string_by_id::draw_string_by_id_at;
    use crate::text::measure_string_by_id::measure_string_by_id_at;
    unsafe {
        let bits = crate::gfx::surface::surface().bits_per_pixel;
        if bits != 16 && bits != 32 {
            return Err(BadInput("a message box on a surface of neither 16 nor 32 bits").into());
        }
        // Converted: `rc2k_core::screens::message_box::layout`.
        let m = measure_string_by_id_at(STRING, font_at(0x51E824), x, canvas)?;
        let (sw, sh) = (k(0x4F49A0), k(0x4F49A4));
        let font = font_at(0x51E904);
        let b = layout(
            m.width,
            m.outcome.y,
            sw,
            sh,
            (font as *const u8).add(8).cast::<u32>().read_unaligned(),
        );
        game_write(rva(BOX + 8), b.rect[2]);
        game_write(rva(BOX + 0xC), b.rect[3]);
        game_write(rva(BOX), b.rect[0]);
        game_write(rva(BOX + 4), b.rect[1]);
        let rect = game_addr(rva(BOX)) as u32;
        let palettes = k(0x51DEB0);
        fill_rect(rect, FILL, palettes)?;
        frame_rect(rect, 0, 0, FRAME, palettes);
        draw_string_by_id_at(STRING, font, b.text_x, b.text_y, canvas)?;
        Ok(())
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::draw_message_box") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { draw_message_box(r.ecx as i32, &mut GameCanvas) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::draw_message_box") {
        return Ok(());
    }
    unsafe { draw_message_box(r.ecx as i32, &mut GameCanvas) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::gfx::rect::RectStage;
    use crate::service::harness::capture;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = crate::screens::draw_menu_item_text::text_unavailable() {
        return t.skip(why);
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x415EA6,
            len: 0x95,
            entry: &[0x60, 0xB8, 0xFE, 0x01, 0x00, 0x00, 0x8D, 0x1D],
            calls: &[0x0C, 0x50, 0x6A, 0x8E],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let stage = RectStage::new();
    let mut regions = stage.regions();
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    regions.extend([
        g(BOX, 0x10),
        g(0x4F49A0, 8),
        (crate::strings::text_from_ids::text() as u32, 0x100),
    ]);
    let fonts = [g(0x51E824, 0x20), g(0x51E904, 0x20)];
    let kept_fonts = capture(&fonts);
    let saved = capture(&regions);
    for (bits, size) in [(16u32, (64u32, 32u32)), (32, (40, 30))] {
        let prepare = || unsafe {
            crate::service::harness::restore(&fonts, &kept_fonts);
            stage.prepare(bits, [0, 63, 0, 31]);
            game_write(rva(0x4F49A0), [size.0, size.1]);
        };
        prepare();
        let mut ro = Regs {
            ecx: 0,
            ..Regs::SENTINEL
        };
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&regions);
        prepare();
        let done = unsafe { draw_message_box(0, &mut GameCanvas) };
        let mine = capture(&regions);
        t.check(done.is_ok() && theirs == mine, || {
            let at = theirs.iter().zip(&mine).position(|(a, b)| a != b);
            format!(
                "bits {bits} size {size:?}: {} first difference at {at:x?}",
                if done.is_ok() { "done" } else { "refused" }
            )
        });
    }
    crate::service::harness::restore(&regions, &saved);
    crate::service::harness::restore(&fonts, &kept_fonts);
    stage.end();
    t.finish()
}
