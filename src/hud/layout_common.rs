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

/// How the pieces drawn next keep to the screen where it is wider than 4:3 (`hud::add_polygon`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Keep {
    /// Each piece to the third of the layout it lies in.
    Thirds = 0,
    /// Each piece to the half it lies in (two players side by side).
    Halves = 1,
    /// All to the middle.
    Middle = 2,
    /// Stretched over the whole width (the divider of two players one above the other).
    Whole = 3,
    /// The top bar over the whole width of the screen, whatever the HUD's box: its end caps and
    /// what lies on them keep to the edges, its middle stretches (`rc2k_core::hud::anchor::bar_x`).
    Bar = 4,
    /// A small piece on the top bar's middle part: its middle goes where the bar puts it and it
    /// keeps its shape.
    BarMarker = 5,
}

/// The layout's own rule set at the start of a HUD.
pub fn start_keep(hud: &Hud, keep: Keep) {
    hud.block.base_rule.set(keep as u32);
    hud.block.anchor_rule.set(keep as u32);
}

/// The pieces drawn next keep as `keep`.
pub fn keep(hud: &Hud, keep: Keep) {
    hud.block.anchor_rule.set(keep as u32);
}

/// The pieces drawn next keep as the layout's own rule again.
pub fn keep_layout(hud: &Hud) {
    hud.block.anchor_rule.set(hud.block.base_rule.get());
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

/// The needle's sign by a start value (`pick`, `needle_kind` and the texture set): its
/// rectangle.
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

/// The gears physics allows (reverse to seventh): only a damaged replay file brings one outside.
const GEARS: std::ops::RangeInclusive<i32> = -1..=7;

/// Whether the clamp of the gear has been logged in this race (`start_gear_log` clears it).
static GEAR_CLAMP_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A new race: the next out-of-range gear is logged again.
pub fn start_gear_log() {
    GEAR_CLAMP_LOGGED.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// The gear to show for the gear value `gear` (a signed dword), and the log line when it had to
/// be clamped to `GEARS`.
fn clamped_gear(gear: u32) -> (i32, Option<String>) {
    let read = gear as i32;
    let shown = read.clamp(*GEARS.start(), *GEARS.end());
    let line = (read != shown).then(|| {
        format!(
            "hud: gear {read} outside {} ..= {} (a damaged replay?), shown as {shown}",
            GEARS.start(),
            GEARS.end()
        )
    });
    (shown, line)
}

/// The rectangle of the gear sprite for the gear value `gear`: 0x41 over the gear, which is
/// clamped to what physics allows (the first clamp of a race is logged).
pub fn gear_rect(gear: u32) -> u32 {
    let (shown, line) = clamped_gear(gear);
    if let Some(line) = line
        && !GEAR_CLAMP_LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        crate::logfile::program_log::log(&line);
    }
    (shown + 0x41) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gears_in_range_are_untouched() {
        for g in -1..=7i32 {
            assert_eq!(clamped_gear(g as u32), (g, None));
            assert_eq!(gear_rect(g as u32), (g + 0x41) as u32);
        }
    }

    #[test]
    fn gears_outside_are_clamped_and_named() {
        let (shown, line) = clamped_gear(40);
        assert_eq!(shown, 7);
        assert_eq!(
            line.as_deref(),
            Some("hud: gear 40 outside -1 ..= 7 (a damaged replay?), shown as 7")
        );
        let (shown, line) = clamped_gear(-9i32 as u32);
        assert_eq!(shown, -1);
        assert!(line.unwrap().contains("gear -9 outside"));
        assert_eq!(gear_rect(0x8000_0000), 0x40);
        assert_eq!(gear_rect(1000), 0x48);
    }

    #[test]
    fn the_log_flag_is_set_by_a_clamp_and_cleared_by_a_new_race() {
        gear_rect(99);
        assert!(GEAR_CLAMP_LOGGED.load(std::sync::atomic::Ordering::Relaxed));
        start_gear_log();
        assert!(!GEAR_CLAMP_LOGGED.load(std::sync::atomic::Ordering::Relaxed));
    }
}
