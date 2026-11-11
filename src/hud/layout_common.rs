//! The helpers the three HUD layouts share (`hud::layout_0`, `_1`, `_2`): time digits and signs,
//! the clock and lap panels, the sprite drawers, the needle's sign and the state the layouts set
//! between draws (texture, alpha, colour of the quad record).

use crate::errors::Refused;
use crate::hud::draw_markers::{int_over_80, taken_off};
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::{RectAt, texture};
use crate::race::owner_frame::Frame;
use rc2k_core::hud::panels;

/// A time made digits (`digits`: `rc2k_core::hud::panels::time_digits`), each
/// plus `base`.
pub fn time_digits(hud: &Hud, value: u32, base: u32) -> [u32; 6] {
    let d = panels::time_digits(value, base);
    for (n, v) in d.into_iter().enumerate() {
        hud.block.digits[n].set(v);
    }
    d
}

/// A sprite: rectangle `rect` of the sprite sheet at (x, y).
pub fn sprite(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    rect: u32,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    crate::hud::add_sprite::add_sprite(frame, hud, pen, x, y, RectAt::number(rect))
}

/// The sign of a time into `sign`: `plus` or `minus`.
pub fn sign(hud: &Hud, value: u32, plus: u32, minus: u32) -> u32 {
    let s = panels::sign(value, plus, minus);
    hud.block.sign.set(s);
    s
}

/// A sprite drawer for the shared panels: (rectangle, x, y).
pub type Draw<'a> = &'a mut dyn FnMut(u32, u32, u32) -> Result<(), Refused>;

/// A clock panel: the time `t` (digits from 0x18), its sign 0x22 / 0x23, then its
/// rectangles (`panels::clock_rects`) at `xs`, row `y`.
pub fn clock_panel(hud: &Hud, t: u32, xs: [u32; 8], y: u32, draw: Draw) -> Result<(), Refused> {
    time_digits(hud, t, panels::CLOCK_BASE);
    sign(hud, t, 0x22, 0x23);
    for (rect, x) in panels::clock_rects(t).into_iter().zip(xs) {
        draw(rect, x, y)?;
    }
    Ok(())
}

/// A split-time panel: the time `t` (digits from 0xC, sign 0 / 1), then its rectangles
/// (`panels::lap_rects`) at `xs`, row `y`.
pub fn lap_panel(hud: &Hud, t: u32, xs: [u32; 8], y: u32, draw: Draw) -> Result<(), Refused> {
    time_digits(hud, t, panels::LAP_BASE);
    sign(hud, t, 0, 1);
    for (rect, x) in panels::lap_rects(t).into_iter().zip(xs) {
        draw(rect, x, y)?;
    }
    Ok(())
}

/// The time `v` (digits from 0xC, sign 0 / 1) as its last three digits at `xs`, row `y`,
/// the leading ones left out while they are 0 (`leading`;
/// `panels::leading_digits`).
pub fn leading_time(hud: &Hud, v: u32, xs: [u32; 3], y: u32, draw: Draw) -> Result<(), Refused> {
    time_digits(hud, v, panels::LAP_BASE);
    sign(hud, v, 0, 1);
    hud.block.leading.set(1);
    for (digit, slot, drawn) in panels::leading_digits(v) {
        if !drawn {
            continue;
        }
        draw(digit, xs[slot], y)?;
        if slot < 2 {
            hud.block.leading.set(0);
        }
    }
    Ok(())
}

/// The needle's sign by a start value (`pick`, `needle_kind`, the texture, as the original
/// writes them): its rectangle.
pub fn needle_sign(hud: &Hud, field: u32) -> u32 {
    hud.block.pick.set(0x5E);
    hud.block.texture.set(0xC0);
    hud.block.needle_kind.set(field);
    let (pick, texture) = panels::needle_sign(field);
    if field != 0 {
        hud.block.pick.set(pick);
        hud.block.texture.set(texture);
    }
    pick
}

/// A sprite scaled 0xC0 over `uv_scale_x` both ways; the scales then taken off into
/// `fistp_scratch`.
pub fn scaled(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    rect: u32,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    let at = RectAt::number(rect);
    let k = hud.block.uv_scale_x.get();
    hud.block.fistp_scratch.set(0xC0);
    let v = int_over_80(0xC0, k);
    crate::hud::add_scaled_sprite::add_scaled_sprite(frame, hud, pen, x, y, at, [v, v])?;
    hud.block.fistp_scratch.set(taken_off(v));
    Ok(())
}

/// The state of the dials: alpha 0xE0 and the first dial's texture.
pub fn begin_dials(hud: &Hud) {
    hud.block.quad.alpha.set(0xE0);
    hud.block.texture.set(texture::DIAL_FIRST);
}

/// The state of a white, opaque draw on `texture`.
pub fn white_opaque(hud: &Hud, texture: u32) {
    hud.block.quad.colour.set(0xFFFF_FFFF);
    hud.block.quad.alpha.set(0xFF);
    hud.block.texture.set(texture);
}

/// A black, opaque solid quad over `rect` (left, top, right, bottom): the divider of two players.
pub fn black_divider(frame: &Frame, hud: &Hud, pen: Pen, rect: [u32; 4]) -> Result<(), Refused> {
    hud.block.texture.set(texture::SPRITES);
    hud.block.quad.alpha.set(0xFF);
    hud.block.quad.colour.set(0);
    hud.block.quad.set_rect(rect);
    crate::hud::add_solid_quad::add_solid_quad(frame, hud, pen)
}

/// The rectangle of the gear sprite for the gear value `gear`.
pub fn gear_rect(gear: u32) -> u32 {
    gear.wrapping_add(0x41)
}
