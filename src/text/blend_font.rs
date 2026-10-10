//! `text::blend_font` (0x405E92): makes the font a highlighted menu item is drawn with, whose
//! colours pulse between those of two fonts.
//!
//! In:  ecx = font A, edx = font B, eax = the time, ebx = the length of a half pulse (0x100)
//! Out: every register preserved
//! In the half pulses where `eax & ebx` is not 0 the blend goes from A towards B, in the
//! others from B towards A. The font it starts from is copied (0x20 bytes) to the blended
//! font `stru_51E384`. Unless `dword_4F3F74` is 0, a 15-entry palette is then made in
//! `dword_4F3B74` with `gfx::blend_palette` from the two fonts' palettes, `eax & (ebx - 1)`
//! of the way (with `dword_4F3F74` = 2 that amount t is eased: t * t >> 8, then the same on
//! 255 - t), and the blended font is given that palette.
//! Two callers: the menu items and `sub_4695C6`.
//! Refuses null fonts, and null palettes when a palette is made.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
};
use crate::gfx::blend_palette::blend_palette;
use crate::gfx::pixel_format::{SCRATCH_RVA, surface_format};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::text::font::Font;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("text::blend_font", rva(0x405E92), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const BLENDED_FONT_RVA: usize = rva(0x51E384); // stru_51E384
const BLENDED_PALETTE_RVA: usize = rva(0x4F3B74); // dword_4F3B74
const BLEND_MODE_RVA: usize = rva(0x4F3F74); // dword_4F3F74: 0 = no palette is made, 2 = eased
const PALETTE_ENTRIES: u32 = 15;
const FONT_LEN: usize = 0x20;

/// Everything `blend_font` may write.
pub fn blend_regions() -> [Region; 3] {
    [
        (game_addr(BLENDED_FONT_RVA) as u32, FONT_LEN),
        (
            game_addr(BLENDED_PALETTE_RVA) as u32,
            PALETTE_ENTRIES as usize * 4,
        ),
        at(SCRATCH_RVA),
    ]
}

// ---- Rust version ----

pub unsafe fn blend_font(
    time: u32,
    half_pulse: u32,
    a: *const Font,
    b: *const Font,
) -> Result<(), BadInput> {
    if a.is_null() || b.is_null() {
        return Err(BadInput("a null font"));
    }
    let (from, to) = if half_pulse & time != 0 {
        (a, b)
    } else {
        (b, a)
    };
    unsafe {
        let mode: u32 = game_read(BLEND_MODE_RVA);
        let blended: *mut Font = game_ptr(BLENDED_FONT_RVA);
        // The original reads the palettes after the copy, so if the second font is the
        // blended font itself, its palette is by then the first one's.
        let from_palette = from.read_unaligned().palette;
        let to_palette = if std::ptr::eq(to, blended) {
            from_palette
        } else {
            to.read_unaligned().palette
        };
        if mode != 0 && (from_palette == 0 || to_palette == 0) {
            return Err(BadInput("a font without a palette"));
        }
        std::ptr::copy(from as *const u8, blended as *mut u8, FONT_LEN);
        if mode == 0 {
            return Ok(());
        }
        let mut t = time & half_pulse.wrapping_sub(1);
        if mode == 2 {
            t = t.wrapping_mul(t) >> 8;
            t ^= 0xFF;
            t = t.wrapping_mul(t) >> 8;
            t ^= 0xFF;
        }
        let palette: *mut u32 = game_ptr(BLENDED_PALETTE_RVA);
        blend_palette(
            from_palette as *const u32,
            to_palette as *const u32,
            palette,
            PALETTE_ENTRIES,
            t,
            &surface_format(),
        )?;
        let mut font = blended.read_unaligned();
        font.palette = palette as u32;
        blended.write_unaligned(font);
    }
    Ok(())
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { blend_font(r.eax, r.ebx, r.ecx as *const Font, r.edx as *const Font) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &blend_regions(), &rust);
}

// ---- self-test: two fonts of its own, with each blend mode ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = blend_regions();
    let saved = capture(&regions);
    let saved_mode: u32 = unsafe { game_read(BLEND_MODE_RVA) };

    let palette_a: Vec<u32> = (0..15u32)
        .map(|n| n.wrapping_mul(0x0123_4567) ^ 0x00F0_3C1E)
        .collect();
    let palette_b: Vec<u32> = (0..15u32)
        .map(|n| n.wrapping_mul(0x89AB_CDEF) ^ 0xFF12_8040)
        .collect();
    // A font is eight dwords, the palette being the one at +0x14.
    let font = |seed: u32, palette: &Vec<u32>| {
        let mut words = [0u32; 8].map(|_| seed);
        for (n, word) in words.iter_mut().enumerate() {
            *word = seed.wrapping_add(n as u32 * 0x0101_0101);
        }
        words[5] = palette.as_ptr() as u32;
        words
    };
    let (font_a, font_b) = (font(0x1111_0000, &palette_a), font(0x2222_0000, &palette_b));

    for mode in [0u32, 1, 2] {
        for time in [0u32, 0x40, 0xFF, 0x100, 0x180, 0x1FF, 0x1_2345, 0xFFFF_FFFF] {
            let before = Regs {
                eax: time,
                ebx: 0x100,
                ecx: font_a.as_ptr() as u32,
                edx: font_b.as_ptr() as u32,
                ..Regs::SENTINEL
            };
            check_state(
                &mut t,
                &format!("mode={mode} time={time:#x}"),
                call_original,
                before,
                &|| {
                    restore(&regions, &[0xAA; FONT_LEN + 15 * 4 + 4]);
                    unsafe { game_write(BLEND_MODE_RVA, mode) };
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe { game_write(BLEND_MODE_RVA, saved_mode) };
    restore(&regions, &saved);
    t.finish()
}
