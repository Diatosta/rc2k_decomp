//! The two frames of `install::install_files` (code inside it, given to `screens::run_frame`
//! in eax; ported 2026-10-20 for the standalone, which has no game code to run them). Each
//! starts with `call $+5`: its body (`pusha` .. `popa; retn`) runs twice, the second time
//! from the pushed return, so both buffers of the flip get the frame:
//! - `install::progress_frame` (0x413295): the surface locked (`gfx::lock_surface`;
//!   failing: carry, nothing more this time round); the bar's place (`dword_51D4A3`: x, y,
//!   width, height) filled with colour 0 (`gfx::fill_rect`, esi `g_blendedPalette`); the
//!   bar's width `g_filesCopied * 0x190 / g_filesToCopy`; the bar's glyphs
//!   (`install::blit_glyph_list`, two at `unk_51D483`, esi `g_installBarGlyphs`) clipped
//!   at x + that + 1 (`g_clipRight`, put back); a quarter of it as a percentage (`text`)
//!   drawn in `font` (0x51E6A4) at (x + 0xC8, y + 2); unlocked, shown;
//! - `install::copying_frame` (0x413340): locked; the backdrop (`gfx::copy_picture_to_surface`);
//!   two texts (`install::draw_string_list`, `unk_51D433`); a frame (`gfx::frame_rect`,
//!   colour 0x2A, `unk_51D4B3`); unlocked, shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of the last call (the show, or
//!      the lock that failed)
//!
//! Every call by `at(..)`: the image is elsewhere in the standalone (a bare IDA address
//! there faulted, 2026-10-20). Callers: `screens::run_frame` from `install_files` (0x41318D, 0x413248, 0x413255). Their
//! calls through `part` (the installer's `game_part`); the percentage text written here.
//! Refuses (before anything) no files to copy, and a bar width past 32 bits (the original's
//! `div` faults). `call $+5` at +0: private copies, manual patches. Shadow mode runs the
//! original alone. Self-test: copies whose calls go to stand-ins (the percentage's run for
//! real), against the Rust version: the lock failing or not, three counts.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::install::installer::{
    COPY_PICTURE, FILL_RECT, FRAME_RECT, LOCK, Part, SHOW, UNLOCK, at,
};

const BLIT_GLYPH_LIST: usize = 0x402628; // install::blit_glyph_list
const DRAW_STRING_LIST: usize = 0x402370; // install::draw_string_list
const DRAW_TEXT: usize = 0x4023D5; // text::draw_text
// strings::append_percent
// strings::append_nul

const BAR_RVA: usize = rva(0x51D4A3); // dword_51D4A3: the bar's x, y, width, height
const BAR_GLYPHS_LIST_RVA: usize = rva(0x51D483); // unk_51D483
const BAR_GLYPHS_RVA: usize = rva(0x51BB3C); // g_installBarGlyphs
const COPIED_RVA: usize = rva(0x51BAE8); // g_filesCopied
const TO_COPY_RVA: usize = rva(0x51BAEC); // g_filesToCopy
const CLIP_RIGHT_RVA: usize = rva(0x4F5040); // g_clipRight
const PALETTE_RVA: usize = rva(0x4F3B74); // g_blendedPalette
const TEXT_RVA: usize = rva(0x93E430); // text
const FONT_RVA: usize = rva(0x51E6A4); // font
const TEXTS_RVA: usize = rva(0x51D433); // unk_51D433: the copying page's two texts
const FRAME_RVA: usize = rva(0x51D4B3); // unk_51D4B3: its frame

fn a(x: usize) -> u32 {
    game_addr(x) as u32
}
fn g(x: usize) -> u32 {
    unsafe { game_read(x) }
}

/// The bar's width, or the refusal of a count the original's `div` faults on.
fn bar_width() -> Result<u32, Refused> {
    let to_copy = g(TO_COPY_RVA);
    if to_copy == 0 {
        return Err(BadInput("no files to copy: the original divides by 0").into());
    }
    let width = g(COPIED_RVA) as u64 * 0x190 / to_copy as u64;
    u32::try_from(width)
        .map_err(|_| BadInput("a bar width past 32 bits: the original's div faults").into())
}

/// `install::progress_frame`'s Rust version.
pub fn progress_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let width = bar_width()?;
    let mut r = *entry;
    for _ in 0..2 {
        r = Regs {
            eflags: r.eflags,
            ..*entry
        };
        part(at(LOCK), &mut r)?;
        if r.carry() {
            continue;
        }
        r.eax = 0;
        r.ebx = a(BAR_RVA);
        r.esi = a(PALETTE_RVA);
        part(at(FILL_RECT), &mut r)?;
        let x = g(BAR_RVA);
        let kept_clip = g(CLIP_RIGHT_RVA);
        unsafe { game_write(CLIP_RIGHT_RVA, width.wrapping_add(x).wrapping_add(1)) };
        r.eax = 2;
        r.ebx = a(BAR_GLYPHS_LIST_RVA);
        r.ecx = 0;
        r.edx = 0;
        r.esi = g(BAR_GLYPHS_RVA);
        part(at(BLIT_GLYPH_LIST), &mut r)?;
        unsafe { game_write(CLIP_RIGHT_RVA, kept_clip) };
        let mut text = crate::strings::append_percent::append_percent(width >> 2);
        text.push(0);
        unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), a(TEXT_RVA) as *mut u8, text.len()) };
        r.eax = a(FONT_RVA);
        r.ebx = a(TEXT_RVA);
        r.ecx = x.wrapping_add(0xC8);
        r.edx = g(BAR_RVA + 4).wrapping_add(2);
        part(at(DRAW_TEXT), &mut r)?;
        part(at(UNLOCK), &mut r)?;
        part(at(SHOW), &mut r)?;
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// `install::copying_frame`'s Rust version.
pub fn copying_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    for _ in 0..2 {
        r = Regs {
            eflags: r.eflags,
            ..*entry
        };
        part(at(LOCK), &mut r)?;
        if r.carry() {
            continue;
        }
        part(at(COPY_PICTURE), &mut r)?;
        r.eax = 2;
        r.ebx = a(TEXTS_RVA);
        r.ecx = 0;
        r.edx = 0;
        part(at(DRAW_STRING_LIST), &mut r)?;
        r.eax = 0x2A;
        r.ebx = a(FRAME_RVA);
        r.ecx = 0;
        r.edx = 0;
        r.esi = a(PALETTE_RVA);
        part(at(FRAME_RECT), &mut r)?;
        part(at(UNLOCK), &mut r)?;
        part(at(SHOW), &mut r)?;
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

pub mod progress {
    use super::*;
    /// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
    pub fn entry(r: &mut Regs) -> Result<(), Refused> {
        crate::race::viewer::regs_entry(r, &|r| {
            progress_frame(r, &mut crate::install::installer::game_part)
        })
    }
}

pub mod copying {
    use super::*;
    /// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
    pub fn entry(r: &mut Regs) -> Result<(), Refused> {
        crate::race::viewer::regs_entry(r, &|r| {
            copying_frame(r, &mut crate::install::installer::game_part)
        })
    }
}
