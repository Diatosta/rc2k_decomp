//! The HUD of a split screen of three or four players: each player's part of the screen gets a
//! compact panel in its lower corners, the gear on the left and the speed on the right, and its
//! place in the race in the upper left corner, by the player's HUD flags (0x20 the gear and
//! speed, 2 the place), and thin black dividers between the parts. The part three players leave
//! free in the quarter layout is black. Everything is drawn in screen pixels
//! (`Keep::Pixels`), its size following the part's.

use crate::errors::Refused;
use crate::hud::layout_common::{Keep, black_divider, gear_rect, keep, start_keep, white_opaque};
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::{RectAt, texture};
use crate::race::owner_frame::Frame;
use rc2k_core::race::view::{PlayerPart, Split};

/// The digit sprites start at this rectangle.
const DIGIT_BASE: u32 = rc2k_core::hud::panels::LAP_BASE;
/// A digit sprite's width and height at full size, in pixels (the sprite sheet's).
const DIGIT: (u32, u32) = (20, 28);
/// The thickness of the dividers between parts, in pixels.
const DIVIDER: u32 = 2;

/// The panel's scale for a part: its height over 400 (a quarter of a 800 x 600 screen is 300),
/// narrowed so that the stage time (about nine digits wide) fits the part's width.
pub fn scale(part: PlayerPart) -> f64 {
    let by_height = f64::from(part.height) / 400.0;
    let by_width = f64::from(part.width) / (f64::from(DIGIT.0) * 9.0);
    by_height.min(by_width).clamp(0.4, 2.0)
}

/// The size of a digit at scale `scale`.
fn digit_size(scale: f64) -> (u32, u32) {
    (
        (f64::from(DIGIT.0) * scale).round() as u32,
        (f64::from(DIGIT.1) * scale).round() as u32,
    )
}

/// The speed's sprites: its last three digits with the leading zeros left out, as the other
/// layouts show it.
fn speed_rects(value: u32) -> Vec<u32> {
    rc2k_core::hud::panels::leading_digits(value)
        .into_iter()
        .filter(|&(_, _, drawn)| drawn)
        .map(|(rect, _, _)| rect)
        .collect()
}

/// A sprite at (x, y) of `size_scale` times its own size (`add_scaled_sprite`'s scales are read
/// back whole, so the scales are the exact ratio).
fn scaled_sprite(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    rect: u32,
    at: (u32, u32),
    scale: f64,
) -> Result<(), Refused> {
    crate::hud::add_scaled_sprite::add_scaled_sprite_plain(
        frame,
        hud,
        pen,
        at.0,
        at.1,
        RectAt::number(rect),
        [scale, scale],
    )
}

/// A solid quad of `colour` over `rect` (left, top, right, bottom).
fn solid(frame: &Frame, hud: &Hud, pen: Pen, rect: [u32; 4], colour: u32) -> Result<(), Refused> {
    hud.block.texture.set(texture::SPRITES);
    hud.block.quad.alpha.set(0xFF);
    hud.block.quad.colour.set(colour);
    hud.block.quad.set_rect(rect);
    crate::hud::add_solid_quad::add_solid_quad(frame, hud, pen)
}

/// The share of the stage a car has run (0 to 1) from its distance shown and the stage's length.
fn progress(shown: u32, length: u32) -> f32 {
    let (shown, length) = (f32::from_bits(shown), f32::from_bits(length));
    if length > 0.0 && shown.is_finite() {
        (shown / length).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The sprites `rects` side by side at scale `scale`, the row's left end at `x` (or, with
/// `from_right`, its right end), its top at `y`; each sprite as wide as the sheet says.
fn row(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    rects: &[u32],
    (x, y): (u32, u32),
    from_right: bool,
    scale: f64,
) -> Result<(), Refused> {
    let width_of = |rect: u32| -> Result<u32, Refused> {
        let [_, _, w, _] = RectAt::number(rect).read(hud)?;
        Ok((f64::from(w) * scale).round() as u32)
    };
    let widths = rects
        .iter()
        .map(|&rect| width_of(rect))
        .collect::<Result<Vec<u32>, Refused>>()?;
    let total: u32 = widths.iter().sum();
    let mut at = if from_right {
        x.saturating_sub(total)
    } else {
        x
    };
    for (&rect, width) in rects.iter().zip(widths) {
        scaled_sprite(frame, hud, pen, rect, (at, y), scale)?;
        at += width;
    }
    Ok(())
}

/// The HUD of `views` players on their layout's parts.
pub fn layout_many(frame: &Frame, hud: &Hud, pen: Pen, views: u32) -> Result<(), Refused> {
    let panel = &frame.panel_and_overlay.get().panel;
    let screen = pen.system.session.screen.get();
    let size = (screen.width.get(), screen.height.get());
    let Some(split) = Split::many(views, crate::race::split_layout::index()) else {
        return Ok(());
    };
    start_keep(hud, Keep::Pixels);
    keep(hud, Keep::Pixels);
    // The part no player has: black.
    if let Some(free) = split.free_part(views, size) {
        black_divider(
            frame,
            hud,
            pen,
            [free.x, free.y, free.x + free.width, free.y + free.height],
        )?;
    }
    for player in 0..views {
        let Some(part) = split.part_of(views, player, size) else {
            continue;
        };
        // The dividers: the part's right and bottom edges where another part lies beyond them.
        if part.x + part.width + DIVIDER <= size.0 {
            black_divider(
                frame,
                hud,
                pen,
                [
                    part.x + part.width - DIVIDER / 2,
                    part.y,
                    part.x + part.width + DIVIDER - DIVIDER / 2,
                    part.y + part.height,
                ],
            )?;
        }
        if part.y + part.height + DIVIDER <= size.1 {
            black_divider(
                frame,
                hud,
                pen,
                [
                    part.x,
                    part.y + part.height - DIVIDER / 2,
                    part.x + part.width,
                    part.y + part.height + DIVIDER - DIVIDER / 2,
                ],
            )?;
        }
    }
    white_opaque(hud, texture::NEEDLES_AND_DIGITS);
    for player in 0..views {
        let Some(part) = split.part_of(views, player, size) else {
            continue;
        };
        let flags = panel.hud_flags[player as usize].get();
        let car = frame.cars.car(player as usize)?;
        let s = scale(part);
        let (_, dh) = digit_size(s);
        let margin = (dh / 3).max(2);
        let bottom = part.y + part.height - dh - margin;
        if flags & 0x20 != 0 {
            // The gear at the left, the speed at the right.
            scaled_sprite(
                frame,
                hud,
                pen,
                gear_rect(car.gear.get()),
                (part.x + margin, bottom),
                s,
            )?;
            let speed = speed_rects(car.engine_sound_speed_int.get());
            row(
                frame,
                hud,
                pen,
                &speed,
                (part.x + part.width - margin, bottom),
                true,
                s,
            )?;
        }
        if flags & 2 != 0 {
            // The stage time at the top right.
            let rects = rc2k_core::hud::panels::lap_rects(car.stage_time.get());
            row(
                frame,
                hud,
                pen,
                &rects,
                (part.x + part.width - margin, part.y + margin + dh / 4 + 2),
                true,
                s,
            )?;
        }
        if flags & 0x40 != 0 {
            // The progress bar along the part's top edge, a white fill on a black track.
            let height = (dh / 4).max(2);
            let (left, right) = (part.x + 2 * margin, part.x + part.width - 2 * margin);
            let top = part.y + 1;
            solid(frame, hud, pen, [left, top, right, top + height], 0)?;
            let done = progress(
                panel.car_distances[player as usize].get(),
                panel.overlay_miles.get(),
            );
            let fill = left + ((right - left) as f32 * done) as u32;
            solid(
                frame,
                hud,
                pen,
                [left, top, fill, top + height],
                0xFFFF_FFFF,
            )?;
            white_opaque(hud, texture::NEEDLES_AND_DIGITS);
        }
        if flags & 2 != 0 {
            let place = car.two_car_place.get();
            if (1..=9).contains(&place) {
                scaled_sprite(
                    frame,
                    hud,
                    pen,
                    DIGIT_BASE + place,
                    (part.x + margin, part.y + margin + dh / 4 + 2),
                    s,
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_speed_has_as_many_digits_as_it_needs() {
        let base = DIGIT_BASE;
        assert_eq!(speed_rects(0), vec![base]);
        assert_eq!(speed_rects(7), vec![base + 7]);
        assert_eq!(speed_rects(42), vec![base + 4, base + 2]);
        assert_eq!(speed_rects(100), vec![base + 1, base, base]);
        // Only the last three digits are shown.
        assert_eq!(speed_rects(1234), vec![base + 2, base + 3, base + 4]);
    }

    #[test]
    fn progress_is_a_share_held_to_the_bar() {
        let bits = f32::to_bits;
        assert_eq!(progress(bits(1.0), bits(4.0)), 0.25);
        assert_eq!(progress(bits(9.0), bits(4.0)), 1.0);
        assert_eq!(progress(bits(-1.0), bits(4.0)), 0.0);
        assert_eq!(progress(bits(1.0), bits(0.0)), 0.0);
        assert_eq!(progress(bits(f32::NAN), bits(4.0)), 0.0);
    }

    #[test]
    fn the_panel_follows_the_part_and_stays_in_bounds() {
        let part = |width, height| PlayerPart {
            x: 0,
            y: 0,
            width,
            height,
        };
        // A quarter of 640 x 480, a column of 160 x 480, and a 4K quarter.
        assert!((scale(part(320, 240)) - 0.6).abs() < 1e-9);
        assert!(scale(part(160, 480)) < scale(part(320, 240)) * 1.5);
        assert!(scale(part(1920, 1080)) <= 2.0);
        assert!(scale(part(10, 10)) >= 0.4);
    }
}
