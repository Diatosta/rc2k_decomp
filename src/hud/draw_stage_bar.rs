//! `hud::draw_stage_bar` (0x4B5A86): the stage's progress bar along the top: the quad record's
//! alpha 0xE0; a frame of five solid quads (0x212463, the last black); then for each of the
//! record's +0x74 parts: its colour (`HudBlock.stage_colours[0]` at the byte offset `overlay_part_kinds_a`[n]) into
//! `HudBlock.bar_colour`, its end (`HudBlock.ends[1]`, the sum of `g_overlayPartMiles` .., the last the
//! record's +0x68) and start (`HudBlock.ends[0]`) as places across 0x229 (`fistp` into `HudBlock.point[0]` /
//! `HudBlock.point[1]`); a black border of four quads, the part's colour inside under alpha 0x78, and
//! between parts a 0x212463 line.
//!
//! Takes: the HUD's record
//!
//! Caller: layout 0's drawer (`hud::layout_0`, 0x4B1DC7). Refuses more than 0x40 parts and a colour
//! offset past
//! `HudBlock.stage_colours[0]`'s four, before anything.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;
use crate::session::PlainDword;

pub fn set(hud: &Hud, n: usize, v: u32) {
    hud.block.quad.word(n).set(v);
}

/// The record's corners (left, top, right, bottom) and, with `Some`, its colour; a quad.
pub fn quad(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    colour: Option<u32>,
    c: [u32; 4],
) -> Result<(), Refused> {
    if let Some(colour) = colour {
        set(hud, 5, colour);
    }
    for (n, v) in c.into_iter().enumerate() {
        set(hud, 1 + n, v);
    }
    crate::hud::add_solid_quad::add_solid_quad(frame, hud, pen)
}

/// The bars of `kinds`' players (each player's colour into `colour_vars`), `dy` lower
/// (see the header and `hud::draw_stage_bars`'s).
// Converted: `rc2k_core::hud::stage_bars`
pub fn bars(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    kinds: &[&[PlainDword]],
    colour_vars: &[&PlainDword],
    dy: u32,
) -> Result<(), Refused> {
    use rc2k_core::hud::stage_bars as sb;
    let count = frame.panel_and_overlay.get().panel.overlay_parts.get();
    sb::check(count, kinds.len() as u32, |p, n| {
        kinds[p as usize][n as usize].get()
    })?;
    set(hud, 0, sb::SOLID);
    for (colour, c) in sb::frame(dy, kinds.len() == 2) {
        quad(frame, hud, pen, Some(colour), c)?;
    }
    if count == 0 {
        return Ok(());
    }
    hud.block.bar_width.set(sb::WIDTH);
    hud.block.ends[0].set(0.0);
    hud.block.ends[1].set(0.0);
    let total = f32::from_bits(frame.panel_and_overlay.get().panel.overlay_miles.get());
    for n in 0..count {
        let left = count - n;
        // `kind` is a byte offset into the colours, held to 0xC by `check` (so four bytes of the
        // four colours, though not always a whole one).
        let colours: Vec<u32> = kinds
            .iter()
            .map(|&k| {
                let kind: u32 = k[n as usize].get();
                let mut bytes = [0; 4];
                crate::session::read_run_bytes(
                    &[&hud.block.stage_colours],
                    kind as usize,
                    &mut bytes,
                    "a stage bar colour",
                )?;
                Ok(u32::from_le_bytes(bytes))
            })
            .collect::<Result<_, crate::errors::BadInput>>()?;
        for (to, &c) in colour_vars.iter().zip(&colours) {
            to.set(c);
        }
        let end = sb::part_end(
            frame.panel_and_overlay.get().overlay.overlay_part_miles[n as usize].get(),
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
            quad(frame, hud, pen, q.colour, q.corners)?;
        }
        hud.block.ends[0].set(end);
    }
    Ok(())
}

// Converted: `rc2k_core::hud::stage_bars` (through `bars`)
pub fn draw_stage_bar(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    bars(
        frame,
        hud,
        pen,
        &[&frame.panel_and_overlay.get().overlay.overlay_part_kinds_a],
        &[&hud.block.bar_colour],
        0,
    )
}

#[cfg(test)]
mod tests {
    /// Every colour offset the check lets through reads the four bytes the byte-offset read did.
    #[test]
    fn colour_bytes_as_the_old_read() {
        let hud = Box::new(crate::hud::store::HudBlock::new());
        for (k, cell) in hud.stage_colours.iter().enumerate() {
            cell.set((0x0101_0101 * (k as u32 + 1)) ^ 0x00A0_0C00);
        }
        for kind in 0..=rc2k_core::hud::stage_bars::LAST_KIND {
            let old: Result<u32, _> = crate::cells::load_value(&hud.stage_colours, kind as usize);
            let mut bytes = [0; 4];
            crate::session::read_run_bytes(&[&hud.stage_colours], kind as usize, &mut bytes, "t")
                .unwrap();
            assert_eq!(Ok(u32::from_le_bytes(bytes)), old, "{kind}");
        }
    }
}
