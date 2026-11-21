//! The stage's progress bar along the top: a frame of five solid quads, then for each of the
//! record's parts its colour, its start and end as places across 0x229, a black border of four
//! quads, the part's colour inside under alpha 0x78 and between parts a line. Refuses more than
//! 0x40 parts and a colour offset past `HudBlock.stage_colours`' four, before anything.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::memory::session::PlainDword;
use crate::race::owner_frame::Frame;

/// The record's corners (left, top, right, bottom) and, with `Some`, its colour; a quad.
pub fn quad(
    frame: &Frame,
    hud: &mut Hud,
    pen: Pen,
    colour: Option<u32>,
    c: [u32; 4],
) -> Result<(), Refused> {
    if let Some(colour) = colour {
        hud.block.quad.colour = colour;
    }
    hud.block.quad.set_rect(c);
    crate::hud::add_solid_quad::add_solid_quad(frame, hud, pen)
}

/// The four bytes of the four stage colours (little endian, one after the other) from byte
/// `kind` on: a kind is a byte offset, held to 0xC by the check, so some read a whole colour
/// and some the end of one and the start of the next.
fn stage_colour_at(colours: &[u32; 4], kind: usize) -> Result<u32, crate::errors::BadInput> {
    let all: [u8; 16] = std::array::from_fn(|i| colours[i / 4].to_le_bytes()[i % 4]);
    let bytes = kind
        .checked_add(4)
        .and_then(|end| all.get(kind..end))
        .ok_or(crate::errors::BadInput(
            "a stage bar colour past the four colours",
        ))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// The bars of `kinds`' players, `dy` lower (see the header and `hud::draw_stage_bars`'s).
pub fn bars(
    frame: &Frame,
    hud: &mut Hud,
    pen: Pen,
    kinds: &[&[PlainDword]],
    dy: u32,
) -> Result<(), Refused> {
    use rc2k_core::hud::stage_bars as sb;
    let count = frame.panel.borrow().overlay_parts;
    sb::check(count, kinds.len() as u32, |p, n| {
        kinds[p as usize][n as usize].get()
    })?;
    hud.block.quad.alpha = sb::SOLID;
    for (colour, c) in sb::frame(dy, kinds.len() == 2) {
        quad(frame, hud, pen, Some(colour), c)?;
    }
    if count == 0 {
        return Ok(());
    }
    hud.block.bar_width = sb::WIDTH;
    hud.block.ends[0] = 0.0;
    hud.block.ends[1] = 0.0;
    let total = f32::from_bits(frame.panel.borrow().overlay_miles);
    for n in 0..count {
        let left = count - n;
        // `kind` is a byte offset into the colours, held to 0xC by `check` (so four bytes of the
        // four colours, though not always a whole one).
        let colours: Vec<u32> = kinds
            .iter()
            .map(|&k| stage_colour_at(&hud.block.stage_colours, k[n as usize].get() as usize))
            .collect::<Result<_, crate::errors::BadInput>>()?;
        let end = sb::part_end(
            frame.overlay.get().overlay_part_miles[n as usize].get(),
            hud.block.ends[1],
            left == 1,
            total,
        );
        hud.block.ends[1] = end;
        let a = sb::place(sb::WIDTH, hud.block.ends[0], total);
        hud.block.point[0] = a;
        let b = sb::place(sb::WIDTH, end, total);
        hud.block.point[1] = b;
        for q in sb::part_quads(a, b, &colours, left == 1, dy) {
            if let Some(alpha) = q.alpha {
                hud.block.quad.alpha = alpha;
            }
            quad(frame, hud, pen, q.colour, q.corners)?;
        }
        hud.block.ends[0] = end;
    }
    Ok(())
}

pub fn draw_stage_bar(frame: &Frame, hud: &mut Hud, pen: Pen) -> Result<(), Refused> {
    bars(
        frame,
        hud,
        pen,
        &[&frame.overlay.get().overlay_part_kinds_a],
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every colour offset the check lets through reads the four bytes from that offset on.
    #[test]
    fn colour_bytes_from_the_offset() {
        let colours = [0x0403_0201, 0x0807_0605, 0x0C0B_0A09, 0x100F_0E0D];
        for kind in 0..=rc2k_core::hud::stage_bars::LAST_KIND {
            let k = kind as usize;
            let wanted = u32::from_le_bytes([k as u8 + 1, k as u8 + 2, k as u8 + 3, k as u8 + 4]);
            assert_eq!(stage_colour_at(&colours, k), Ok(wanted), "{kind}");
        }
        assert!(stage_colour_at(&colours, 13).is_err());
    }
}
