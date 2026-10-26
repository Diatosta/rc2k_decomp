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
//! Caller: layout 0's drawer (0x4B1DC7). The quads through `part` (1 `add_solid_quad`),
//! in the game its Rust version. Refuses more than 0x40 parts and a colour offset past
//! `dword_75BF70`'s four, before anything. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose quads go to a stand-in noting the quad record at each,
//! against the Rust version, parts by lot, at
//! both test control words (the test's values are exact at both).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::hud::add_quad::QUAD_RVA;

pub const MILES_RVA: usize = rva(0x6D150C); // g_overlayPartMiles: 0x40
pub const KINDS_RVA: usize = rva(0x6D160C); // dword_6D160C: 0x40
pub const COLOURS_RVA: usize = rva(0x75BF70); // dword_75BF70: 4
const COLOUR_RVA: usize = rva(0x75BFA0); // dword_75BFA0
pub const PLACES_RVA: [usize; 2] = [rva(0x75BFF0), rva(0x75BFF4)];
pub const ENDS_RVA: [usize; 2] = [rva(0x75BFF8), rva(0x75BFFC)]; // start, end
pub const WIDTH_RVA: usize = rva(0x75C000); // dword_75C000
pub const PARTS: u32 = rc2k_core::hud::stage_bars::PARTS;

/// A call by place: 1 `add_solid_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

pub fn set(n: usize, v: u32) {
    unsafe { game_write(QUAD_RVA + 4 * n, v) };
}

/// The record's corners (left, top, right, bottom) and, with `Some`, its colour; a quad.
pub fn quad(g: &Regs, colour: Option<u32>, c: [u32; 4], part: &mut Part) -> Result<(), Refused> {
    if let Some(colour) = colour {
        set(5, colour);
    }
    for (n, v) in c.into_iter().enumerate() {
        set(1 + n, v);
    }
    let mut r = *g;
    part(1, &mut r)
}

/// The bars of `kinds`' players (each player's colour into `colour_vars`), `dy` lower
/// (see the header and `hud::draw_stage_bars_1`'s): the registers they leave.
// Converted: `rc2k_core::hud::stage_bars`
pub unsafe fn bars(
    entry: &Regs,
    kinds: &[usize],
    colour_vars: &[usize],
    dy: u32,
    part: Part,
) -> Result<Regs, Refused> {
    use rc2k_core::hud::stage_bars as sb;
    let mut part = part;
    let mut g = *entry;
    unsafe {
        let get = |o: u32| ((entry.ebp + o) as *const u32).read_unaligned();
        let f = |r: usize| f32::from_bits(game_read(r));
        let count = get(0x74);
        sb::check(count, kinds.len() as u32, |p, n| {
            game_read(kinds[p as usize] + 4 * n as usize)
        })?;
        set(0, sb::SOLID);
        for (colour, c) in sb::frame(dy, kinds.len() == 2) {
            quad(&g, Some(colour), c, &mut part)?;
        }
        g.ecx = count;
        if count == 0 {
            return Ok(g);
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
            let mut r = Regs { ecx: left, ..g };
            for q in sb::part_quads(a, b, &colours, left == 1, dy) {
                if let Some(alpha) = q.alpha {
                    set(0, alpha);
                }
                [r.eax, r.ebx] = [q.eax, q.ebx];
                if let Some(edx) = q.edx {
                    r.edx = edx;
                }
                quad(&r, q.colour, q.corners, &mut part)?;
            }
            game_write(ENDS_RVA[0], end.to_bits());
        }
        g.ebx = count;
        g.ecx = 0;
    }
    Ok(g)
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::stage_bars` (through `bars`)
pub unsafe fn draw_stage_bar(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { bars(entry, &[KINDS_RVA], &[COLOUR_RVA], 0, part) }
}

pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::add_solid_quad as sq;
    *r = unsafe { sq::add_solid_quad(r, &mut crate::hud::add_triangle::game_part_quad) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_stage_bar(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
