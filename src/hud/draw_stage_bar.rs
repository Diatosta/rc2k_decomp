//! `hud::draw_stage_bar` (0x4B5A86): the stage's progress bar along the top: the quad
//! record's alpha 0xE0; a frame of five solid quads (0x212463, the last black); then for
//! each of the record's +0x74 parts: its colour (`dword_75BF70` at the byte offset
//! `dword_6D160C`[n]) into `dword_75BFA0`, its end (`dword_75BFFC`, the sum of
//! `g_overlayPartMiles` .., the last the record's +0x68) and start (`dword_75BFF8`) as
//! places across 0x229 (`fistp` into `dword_75BFF0` / `dword_75BFF4`); a black border of
//! four quads, the part's colour inside under alpha 0x78, and between parts a 0x212463
//! line.
//!
//! In:  ebp the HUD's record
//! Out: ebx the parts drawn (kept with none: the original zeroes it after the test), ecx 0;
//!      the rest kept
//!
//! Caller: layout 0's drawer (0x4B1DC7). Refuses more than 0x40 parts and a colour offset past
//! `dword_75BF70`'s four, before anything.

use crate::errors::Refused;
use crate::framework::rva;
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::store::Slot;
use crate::hud::store::read_f32 as f;
use crate::hud::store::{game_addr, game_read, game_write};

pub const MILES_RVA: usize = rva(0x6D150C); // g_overlayPartMiles: 0x40
pub const KINDS_RVA: usize = rva(0x6D160C); // dword_6D160C: 0x40
pub const COLOURS_RVA: Slot = Slot::at(0x75BF70); // dword_75BF70: 4
const COLOUR_RVA: Slot = Slot::at(0x75BFA0); // dword_75BFA0
pub const PLACES_RVA: [Slot; 2] = [Slot::at(0x75BFF0), Slot::at(0x75BFF4)];
pub const ENDS_RVA: [Slot; 2] = [Slot::at(0x75BFF8), Slot::at(0x75BFFC)]; // start, end
pub const WIDTH_RVA: Slot = Slot::at(0x75C000); // dword_75C000
pub fn set(n: usize, v: u32) {
    unsafe { game_write(QUAD_RVA + 4 * n, v) };
}

/// The record's corners (left, top, right, bottom) and, with `Some`, its colour; a quad.
pub fn quad(colour: Option<u32>, c: [u32; 4]) -> Result<(), Refused> {
    if let Some(colour) = colour {
        set(5, colour);
    }
    for (n, v) in c.into_iter().enumerate() {
        set(1 + n, v);
    }
    unsafe { crate::hud::add_solid_quad::add_solid_quad() }
}

/// The bars of `kinds`' players (each player's colour into `colour_vars`), `dy` lower
/// (see the header and `hud::draw_stage_bars_1`'s).
// Converted: `rc2k_core::hud::stage_bars`
pub unsafe fn bars(
    record: u32,
    kinds: &[usize],
    colour_vars: &[Slot],
    dy: u32,
) -> Result<(), Refused> {
    use rc2k_core::hud::stage_bars as sb;
    unsafe {
        let get = |o: u32| ((record + o) as *const u32).read_unaligned();
        let count = get(0x74);
        sb::check(count, kinds.len() as u32, |p, n| {
            game_read(kinds[p as usize] + 4 * n as usize)
        })?;
        set(0, sb::SOLID);
        for (colour, c) in sb::frame(dy, kinds.len() == 2) {
            quad(Some(colour), c)?;
        }
        if count == 0 {
            return Ok(());
        }
        game_write(WIDTH_RVA, sb::WIDTH);
        game_write(ENDS_RVA[0], 0u32);
        game_write(ENDS_RVA[1], 0u32);
        let total = f32::from_bits(get(0x68));
        for n in 0..count {
            let left = count - n;
            let colours: Vec<u32> = kinds
                .iter()
                .map(|&k| {
                    let kind: u32 = game_read(k + 4 * n as usize);
                    ((game_addr(COLOURS_RVA) as u32 + kind) as *const u32).read_unaligned()
                })
                .collect();
            for (&to, &c) in colour_vars.iter().zip(&colours) {
                game_write(to, c);
            }
            let end = sb::part_end(
                f(MILES_RVA + 4 * n as usize),
                f(ENDS_RVA[1]),
                left == 1,
                total,
            );
            game_write(ENDS_RVA[1], end.to_bits());
            let a = sb::place(sb::WIDTH, f(ENDS_RVA[0]), total);
            game_write(PLACES_RVA[0], a);
            let b = sb::place(sb::WIDTH, end, total);
            game_write(PLACES_RVA[1], b);
            for q in sb::part_quads(a, b, &colours, left == 1, dy) {
                if let Some(alpha) = q.alpha {
                    set(0, alpha);
                }
                quad(q.colour, q.corners)?;
            }
            game_write(ENDS_RVA[0], end.to_bits());
        }
    }
    Ok(())
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::stage_bars` (through `bars`)
pub unsafe fn draw_stage_bar(record: u32) -> Result<(), Refused> {
    unsafe { bars(record, &[KINDS_RVA], &[COLOUR_RVA], 0) }
}
