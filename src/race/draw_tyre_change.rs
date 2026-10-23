//! `race::draw_tyre_change` (0x41C38C): the tyre change's texture (`dword_5BEBE0`, then
//! copied to the device's `dword_5BEBE4`), for the device at esi. ebp = esi; the clip's
//! left and right kept round it. Its lock (`d3d::lock_texture`); unless that failed or the
//! texture was drawn before (`dword_5BEBE8` 1), strings 0x129 (font `stru_51E304`, its
//! +0x1C 0xF8, at 4, 0x80), then with the clip's left 0x80 0x12A (`unk_51E2E4`, at 0xFC, 0)
//! and 0x12B (`unk_51E2C4`, at 0x84, a line down: `dword_51E2CC`), each by
//! `text::draw_string_by_id` inside `pusha` / `popa`. Then (also after a failed lock:
//! section 16) the clip's left 0x80; the rectangle (0x80, 0x32, 0x80, 0x4B) filled with
//! colour 0 (`gfx::fill_rect`, `dword_5C2A1C`, palette `unk_5BEC2C`); string 0x12E and the
//! record's time (`g_tyreChangeRecord`) in the text buffer, drawn three times (font +0x1C
//! 0x168, at 0x84, 0x32; each 0x78 left and a line down: `text::draw_text_keep_regs`);
//! the clip's left 0; (0, 0x99, 0x100, 0x19) filled; the record's name
//! (`g_tyreChangeRecordName`, +0x1C 0xF8, at 4, 0x99); for player 0 (`dword_5BEBEC`) the
//! clip's right 0x7F and x 0, else its left 0x80 and x 0x80; (x, 0xB2, 0x80, 0x4B) filled;
//! with the device's +0x4F0 1 string 0x12D and the time (+0x4F4), else 0x12C, drawn three
//! times (+0x1C 0x168, at x + 4, 0xB2, as before); the unlock (`d3d::unlock_texture`), the
//! copy to the device (`d3d::copy_to_device_texture` 0x41A8F9: the platform's, called in
//! the game), `dword_5BEBE8` 1.
//!
//! In:  esi the device
//! Out: ebp = esi; the rest kept; the flags of the copy (no caller reads them)
//!
//! Callers: `race::tyre_change_gauge` (twice), which calls the Rust version directly. Its
//! calls by place: 1 lock, 2 unlock, 3 `draw_text_keep_regs`, 4 `draw_string_by_id` (keep
//! regs, 0x402368), 5 `fill_rect`, 6 the copy; the text is built with the Rust versions
//! of `strings::append_string_by_id` and `append_time`.
//! Refuses a text longer than the text buffer, and what its calls refuse.
//! Shadow mode runs the original alone (it writes a surface). Self-test: a copy whose
//! lock, unlock, draws, fills and copy go to stand-ins that note them (the lock failing or
//! not), against the Rust version with the same: drawn before or not, both players, done
//! or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::race::frame::Part;

pub const LOCK: u32 = 1;
pub const UNLOCK: u32 = 2;
pub const TEXT: u32 = 3;
pub const BY_ID: u32 = 4;
pub const FILL: u32 = 5;
pub const COPY: u32 = 6;

const TEXTURE: usize = 0x5BEBE0; // dword_5BEBE0
const DEVICE_TEXTURE: usize = 0x5BEBE4; // dword_5BEBE4
const DRAWN: usize = 0x5BEBE8; // dword_5BEBE8
const PLAYER: usize = 0x5BEBEC; // dword_5BEBEC
const RECORD: usize = 0x5BEC28; // g_tyreChangeRecord
const RECORD_NAME: usize = 0x5BEC08; // g_tyreChangeRecordName
const FONT: usize = 0x51E304; // stru_51E304
const FONT_B: usize = 0x51E2E4; // unk_51E2E4
const FONT_C: usize = 0x51E2C4; // unk_51E2C4
const LINE: usize = 0x51E2CC; // dword_51E2CC
const RECT: usize = 0x5C2A1C; // dword_5C2A1C
const PALETTE: usize = 0x5BEC2C; // unk_5BEC2C
const TEXT_BUFFER: usize = 0x93E430; // text
const CLIP_LEFT: usize = 0x4F503C; // g_clipLeft
const CLIP_RIGHT: usize = 0x4F5040; // g_clipRight
const COPY_RVA: usize = rva(0x41A8F9); // d3d::copy_to_device_texture

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

/// The text buffer: string `id`, then (with `time`) a time, then a NUL.
unsafe fn build_text(id: u32, time: Option<u32>) -> Result<(), Refused> {
    use crate::strings::append_time::{append_time, show_hours};
    let mut t = Vec::new();
    unsafe { crate::strings::append_string_by_id::push_string(&mut t, id) };
    if let Some(h) = time {
        t.extend(append_time(h, show_hours()));
    }
    t.push(0);
    if t.len() > crate::strings::text_from_ids::TEXT_LEN {
        return Err(BadInput("a tyre change text longer than the text buffer").into());
    }
    unsafe { std::ptr::copy_nonoverlapping(t.as_ptr(), a(TEXT_BUFFER) as *mut u8, t.len()) };
    Ok(())
}

/// The rectangle at `dword_5C2A1C` and its fill (eax 0, ebx it, esi the palette).
fn fill(
    w: &mut Regs,
    part: &mut Part,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), Refused> {
    w.eax = 0;
    w.ebx = a(RECT);
    w.esi = a(PALETTE);
    for (k, v) in [x, y, width, height].into_iter().enumerate() {
        p(RECT + 4 * k, v);
    }
    part(FILL, w)
}

/// A text drawn three times, each 0x78 to the left and a line down.
fn three_times(w: &mut Regs, part: &mut Part, x: u32, y: u32) -> Result<(), Refused> {
    w.eax = a(FONT);
    p(FONT + 0x1C, 0x168);
    w.ebx = a(TEXT_BUFFER);
    w.ecx = x;
    w.edx = y;
    part(TEXT, w)?;
    for _ in 0..2 {
        w.ecx = w.ecx.wrapping_sub(0x78);
        w.edx = w.edx.wrapping_add(g(LINE));
        part(TEXT, w)?;
    }
    Ok(())
}

/// The steps on `r` (it leaves ebp = esi).
pub unsafe fn draw_tyre_change_in(r: &mut Regs, mut part: Part) -> Result<(), Refused> {
    let device = r.esi;
    let clips = (g(CLIP_LEFT), g(CLIP_RIGHT));
    r.ebp = device;
    let mut w = *r;
    let done = (|| -> Result<(), Refused> {
        w.eax = g(TEXTURE);
        part(LOCK, &mut w)?;
        if !w.carry() && g(DRAWN) != 1 {
            for (n, (id, font, x, y)) in [
                (0x129u32, FONT, 4u32, 0x80u32),
                (0x12A, FONT_B, 0xFC, 0),
                (0x12B, FONT_C, 0x84, 0),
            ]
            .into_iter()
            .enumerate()
            {
                w.eax = id;
                w.ebx = a(font);
                w.ecx = x;
                match n {
                    0 => {
                        p(FONT + 0x1C, 0xF8);
                        w.edx = y;
                    }
                    1 => w.edx = 0,
                    _ => w.edx = w.edx.wrapping_add(g(LINE)),
                }
                part(BY_ID, &mut w)?;
                if n == 0 {
                    p(CLIP_LEFT, 0x80);
                }
            }
        }
        p(CLIP_LEFT, 0x80);
        fill(&mut w, &mut part, 0x80, 0x32, 0x80, 0x4B)?;
        unsafe { build_text(0x12E, Some(g(RECORD))) }?;
        three_times(&mut w, &mut part, 0x84, 0x32)?;
        p(CLIP_LEFT, 0);
        fill(&mut w, &mut part, 0, 0x99, 0x100, 0x19)?;
        w.eax = a(FONT);
        p(FONT + 0x1C, 0xF8);
        w.ebx = a(RECORD_NAME);
        w.ecx = 4;
        w.edx = 0x99;
        part(TEXT, &mut w)?;
        if g(PLAYER) == 0 {
            w.ecx = 0;
            p(CLIP_RIGHT, 0x7F);
        } else {
            w.ecx = 0x80;
            p(CLIP_LEFT, 0x80);
        }
        let x = w.ecx;
        fill(&mut w, &mut part, x, 0xB2, 0x80, 0x4B)?;
        let state = unsafe { ((device + 0x4F0) as *const u32).read_unaligned() };
        if state == 1 {
            let time = unsafe { ((device + 0x4F4) as *const u32).read_unaligned() };
            unsafe { build_text(0x12D, Some(time)) }?;
        } else {
            unsafe { build_text(0x12C, None) }?;
        }
        three_times(&mut w, &mut part, x.wrapping_add(4), 0xB2)?;
        w.eax = g(TEXTURE);
        part(UNLOCK, &mut w)?;
        w.eax = g(TEXTURE);
        w.ebx = g(DEVICE_TEXTURE);
        part(COPY, &mut w)?;
        p(DRAWN, 1);
        Ok(())
    })();
    p(CLIP_LEFT, clips.0);
    p(CLIP_RIGHT, clips.1);
    done?;
    r.eflags = w.eflags;
    Ok(())
}

/// The calls in the game: the Rust versions, the copy the platform's.
pub fn draw_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            LOCK => {
                let g = crate::d3d::lock_texture::lock_texture(r)?;
                r.eflags = g.eflags;
            }
            UNLOCK => {
                let g = crate::d3d::unlock_texture::unlock_texture(r)?;
                r.eflags = g.eflags;
            }
            TEXT => crate::race::name_plates::text_in_game(r)?,
            BY_ID => {
                crate::text::draw_string_by_id::draw_string_by_id_at(
                    r.eax,
                    r.ebx as *mut crate::text::font::Font,
                    r.ecx as i32,
                    r.edx as i32,
                    &mut crate::text::draw_text::GameCanvas,
                )?;
            }
            FILL => crate::gfx::rect::fill_rect(r.ebx, r.eax, r.esi)?,
            _ => {
                // The platform's (Direct3D): called in the game.
                let mut c = *r;
                crate::calls::call(COPY_RVA, &mut c);
                r.eflags = c.eflags;
            }
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut w = *r;
    match unsafe { draw_tyre_change_in(&mut w, &mut draw_part) } {
        Ok(()) => {
            r.ebp = w.ebp;
            r.eflags = w.eflags;
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
