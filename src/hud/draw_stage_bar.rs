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
use crate::hud::store::HUD;
use crate::session::Dword;
use crate::store::game_read;

pub const MILES_RVA: usize = rva(0x6D150C); // g_overlayPartMiles: 0x40
pub const KINDS_RVA: usize = rva(0x6D160C); // dword_6D160C: 0x40
pub fn set(n: usize, v: u32) {
    HUD.quad[n].set(v);
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
    colour_vars: &[&Dword],
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
        HUD.bar_width.set(sb::WIDTH);
        HUD.ends[0].set_bits(0);
        HUD.ends[1].set_bits(0);
        let total = f32::from_bits(get(0x68));
        for n in 0..count {
            let left = count - n;
            let colours: Vec<u32> = kinds
                .iter()
                .map(|&k| {
                    let kind: u32 = game_read(k + 4 * n as usize);
                    ((HUD.stage_colours.as_ptr() as u32 + kind) as *const u32).read_unaligned()
                })
                .collect();
            for (to, &c) in colour_vars.iter().zip(&colours) {
                to.set(c);
            }
            let end = sb::part_end(
                f32::from_bits(game_read(MILES_RVA + 4 * n as usize)),
                HUD.ends[1].get(),
                left == 1,
                total,
            );
            HUD.ends[1].set(end);
            let a = sb::place(sb::WIDTH, HUD.ends[0].get(), total);
            HUD.point[0].set(a);
            let b = sb::place(sb::WIDTH, end, total);
            HUD.point[1].set(b);
            for q in sb::part_quads(a, b, &colours, left == 1, dy) {
                if let Some(alpha) = q.alpha {
                    set(0, alpha);
                }
                quad(q.colour, q.corners)?;
            }
            HUD.ends[0].set(end);
        }
    }
    Ok(())
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::stage_bars` (through `bars`)
pub unsafe fn draw_stage_bar(record: u32) -> Result<(), Refused> {
    unsafe { bars(record, &[KINDS_RVA], &[&HUD.bar_colour], 0) }
}
