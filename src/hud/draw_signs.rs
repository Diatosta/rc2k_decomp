//! `hud::draw_signs` (0x4B4F2B): up to four signs of the HUD's record (+0xA4 .. +0xB0, 0
//! none): sign v as rectangle 0x4C + (v - 1) & 0xF of `dword_75B940`, centred in its 0x40
//! wide place by its width (`dword_75BF80`[(v - 1) & 7]), the places from +0xB4 on 0x40
//! apart, at 0x36 (0x16 with `dword_6D1502` 0).
//!
//! In:  ebp the HUD's record
//! Out: ebx the last place, ecx the last sign's row (0 when the last is none), eax / esi
//!      the last drawn sign's rectangle (as `add_sprite` keeps them)
//!
//! Caller: layout 0's drawer (0x4B1F27). The sprites through `part` (1), in the game
//! `add_sprite`'s Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, signs by
//! lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::hud::draw_marker::RECTS_RVA;

const WIDTHS_RVA: usize = rva(0x75BF80); // dword_75BF80: 8
pub const FLAT_RVA: usize = rva(0x6D1502); // dword_6D1502

/// A call by place: 1 `add_sprite`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The row: `y`, or 0x20 higher with `dword_6D1502` 0.
pub fn row(y: u32) -> u32 {
    row_by(y, 0x20)
}

/// The row: `y`, or `lift` higher with `dword_6D1502` 0.
pub fn row_by(y: u32, lift: u32) -> u32 {
    rc2k_core::hud::signs::row(y, lift, unsafe { game_read(FLAT_RVA) })
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::signs` (through `signs_with`)
pub unsafe fn draw_signs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { signs_with(entry, 0xA4, 0xB4, 0, 0x36, 0x20, part) }
}

/// Four signs from the record's `first` .. on, the places from its `start` field plus
/// `shift`, at row `y` (shared with `hud::draw_signs_1a` / `_1b`).
// Converted: `rc2k_core::hud::signs::{sign, place, row}`
pub unsafe fn signs_with(
    entry: &Regs,
    first: u32,
    start: u32,
    shift: u32,
    y: u32,
    lift: u32,
    part: Part,
) -> Result<Regs, Refused> {
    use rc2k_core::hud::signs;
    let mut g = *entry;
    unsafe {
        let get = |o: u32| ((g.ebp + o) as *const u32).read_unaligned();
        let places = get(start).wrapping_add(shift);
        for n in 0..signs::SIGNS as u32 {
            g.ebx = signs::place(places, n);
            let v = get(first + 4 * n);
            g.ecx = v;
            let Some(sign) = signs::sign(places, n, v, |k| game_read(WIDTHS_RVA + 4 * k as usize))
            else {
                continue;
            };
            g.eax = sign.rectangle;
            let mut r = Regs {
                ebx: sign.x,
                ecx: row_by(y, lift),
                esi: game_addr(RECTS_RVA) as u32 + g.eax,
                ..g
            };
            part(1, &mut r)?;
            g.ecx = r.ecx;
            g.esi = r.esi;
        }
    }
    Ok(g)
}

/// In the game: `add_sprite`'s Rust version.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::hud::draw_marker::game_part(place, r)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_signs(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
