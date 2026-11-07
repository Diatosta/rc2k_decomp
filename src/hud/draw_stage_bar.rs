//! `hud::draw_stage_bar` (0x4B5A86): the stage's progress bar along the top: the quad
//! record's alpha 0xE0; a frame of five solid quads (0x212463, the last black); then for
//! each of the record's +0x74 parts: its colour (`dword_75BF70` at the byte offset
//! `dword_6D160C`[n]) into `dword_75BFA0`, its end (`dword_75BFFC`, the sum of
//! `g_overlayPartMiles` .., the last the record's +0x68) and start (`dword_75BFF8`) as
//! places across 0x229 (`fistp` into `dword_75BFF0` / `dword_75BFF4`); a black border of
//! four quads, the part's colour inside under alpha 0x78, and between parts a 0x212463
//! line.
//!
//! Takes: the HUD's record
//!
//! Caller: layout 0's drawer (`hud::layout_0`, 0x4B1DC7). Refuses more than 0x40 parts and a colour
//! offset past
//! `dword_75BF70`'s four, before anything.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::race::owner_frame::Frame;
use crate::session::Dword;

pub fn set(hud: &Hud, n: usize, v: u32) {
    hud.block.quad[n].set(v);
}

/// The record's corners (left, top, right, bottom) and, with `Some`, its colour; a quad.
pub fn quad(frame: &Frame, hud: &Hud, colour: Option<u32>, c: [u32; 4]) -> Result<(), Refused> {
    if let Some(colour) = colour {
        set(hud, 5, colour);
    }
    for (n, v) in c.into_iter().enumerate() {
        set(hud, 1 + n, v);
    }
    crate::hud::add_solid_quad::add_solid_quad(frame, hud)
}

/// The bars of `kinds`' players (each player's colour into `colour_vars`), `dy` lower
/// (see the header and `hud::draw_stage_bars`'s).
// Converted: `rc2k_core::hud::stage_bars`
pub fn bars(
    frame: &Frame,
    hud: &Hud,
    kinds: &[&[Dword]],
    colour_vars: &[&Dword],
    dy: u32,
) -> Result<(), Refused> {
    use rc2k_core::hud::stage_bars as sb;
    let count = frame.panel_and_overlay.panel.overlay_parts.get();
    sb::check(count, kinds.len() as u32, |p, n| {
        kinds[p as usize][n as usize].get()
    })?;
    set(hud, 0, sb::SOLID);
    for (colour, c) in sb::frame(dy, kinds.len() == 2) {
        quad(frame, hud, Some(colour), c)?;
    }
    if count == 0 {
        return Ok(());
    }
    hud.block.bar_width.set(sb::WIDTH);
    hud.block.ends[0].set_bits(0);
    hud.block.ends[1].set_bits(0);
    let total = f32::from_bits(frame.panel_and_overlay.panel.overlay_miles.get());
    for n in 0..count {
        let left = count - n;
        // `kind` is a byte offset into the colours (the original's, unchecked).
        let colours: Vec<u32> = kinds
            .iter()
            .map(|&k| {
                let kind: u32 = k[n as usize].get();
                crate::pointers::load_value(&hud.block.stage_colours, kind as usize)
            })
            .collect::<Result<_, _>>()?;
        for (to, &c) in colour_vars.iter().zip(&colours) {
            to.set(c);
        }
        let end = sb::part_end(
            frame.panel_and_overlay.overlay.overlay_part_miles[n as usize].get(),
            hud.block.ends[1].get(),
            left == 1,
            total,
        );
        hud.block.ends[1].set(end);
        let a = sb::place(sb::WIDTH, hud.block.ends[0].get(), total);
        hud.block.point[0].set(a);
        let b = sb::place(sb::WIDTH, end, total);
        hud.block.point[1].set(b);
        for q in sb::part_quads(a, b, &colours, left == 1, dy) {
            if let Some(alpha) = q.alpha {
                set(hud, 0, alpha);
            }
            quad(frame, hud, q.colour, q.corners)?;
        }
        hud.block.ends[0].set(end);
    }
    Ok(())
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::stage_bars` (through `bars`)
pub fn draw_stage_bar(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    bars(
        frame,
        hud,
        &[&frame.panel_and_overlay.overlay.overlay_part_kinds_a],
        &[&hud.block.bar_colour],
        0,
    )
}
