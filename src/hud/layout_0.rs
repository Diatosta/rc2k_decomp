//! `hud::layout_0` (0x4B1D8C): the HUD of layout 0 (the record's +0xF4 neither 1 nor 2),
//! by the record's flags (+0xF8): 0x10 the dial (`dials::DISC_0`) and a sprite; 0x40 the
//! stage bar (`hud::draw_stage_bar`), the marker (`hud::draw_marker`) and the two time
//! panels (sprite 0x29 + the record's +0x28 / +0x30 held to 0 .. 0x16, the times
//! `dword_75C074` / `dword_75C078` as digits); 8 the cut call (`sub_4B6A6D`: ecx 0 and an
//! empty function); always the icons (`hud::icons`); 0x20 two sprites (the second by the
//! record's +0x18) and the time +0x14 with its leading digits left out while they are 0;
//! 1 the sign by +0xCC (also into `dword_75BFAC`) and the needle (`dials::NEEDLE_0`); 0x80
//! the signs (`draw_signs::LAYOUT_0`); 2 the time +0x1C's last two digits, two sprites and the
//! time `dword_75C07C`. Textures (`dword_75C034`), the quad record's alpha and colour set
//! between, as the original.

use crate::errors::Refused;
use crate::hud::draw_signs::row;
use crate::hud::owner::Hud;
use crate::hud::store::rect_place;
use crate::pointers::GamePtr;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::panels;

/// A time made digits (`dword_75C08C` ..: `rc2k_core::hud::panels::time_digits`), each
/// plus `base`.
pub fn time_digits(hud: &Hud, value: u32, base: u32) -> [u32; 6] {
    let d = panels::time_digits(value, base);
    for (n, v) in d.into_iter().enumerate() {
        hud.block.digits[n].set(v);
    }
    d
}

/// A sprite: rectangle `rect` of `dword_75B940` at (x, y).
pub fn sprite(frame: &Frame, hud: &Hud, rect: u32, x: u32, y: u32) -> Result<(), Refused> {
    crate::hud::add_sprite::add_sprite(frame, hud, x, y, rect_place(hud, rect))
}

/// The sign of a time into `dword_75C0A4`: `plus` or `minus`.
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
/// the leading ones left out while they are 0 (`dword_75C03C`;
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

/// The needle's sign by the record's `field` (`dword_75C040`, `dword_75BFAC`, the
/// texture, as the original writes them): its rectangle.
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

/// The Rust version (see the header): the HUD of one player, `record` the HUD's record.
// Converted: `rc2k_core::hud::panels` (the digits, signs, panels, leading digits, the
// stage panel and the needle's sign)
pub fn layout_0(frame: &Frame, hud: &Hud, record: GamePtr) -> Result<(), Refused> {
    use crate::hud::*;
    // A field of the record by an offset from a table or the caller, as the original walks it.
    let get = |o: u32| record.offset(o).read::<u32>("a hud record's field");
    let flags = frame.panel_and_overlay.panel.hud_flags[0].get();
    hud.block.quad[0].set(0xE0);
    hud.block.texture.set(0xC6);
    if flags & 0x10 != 0 {
        dials::disc(frame, hud, record, &dials::DISC_0)?;
    }
    hud.block.texture.set(0xC0);
    if flags & 0x40 != 0 {
        draw_stage_bar::draw_stage_bar(frame, hud)?;
    }
    hud.block.quad[5].set(0xFFFF_FFFF);
    hud.block.quad[0].set(0xFF);
    hud.block.texture.set(0xDC);
    icons::icons(frame, hud, record, &icons::LAYOUT_0)?;
    hud.block.quad[5].set(0xFFFF_FFFF);
    hud.block.quad[0].set(0xFF);
    hud.block.texture.set(0xC1);
    if flags & 0x20 != 0 {
        sprite(frame, hud, 0x49, 9, 0x215)?;
        sprite(
            frame,
            hud,
            frame
                .panel_and_overlay
                .panel
                .view_a
                .gear
                .get()
                .wrapping_add(0x41),
            0x11,
            0x21D,
        )?;
    }
    let pick = needle_sign(hud, frame.panel_and_overlay.panel.start_values[0].get());
    if flags & 1 != 0 {
        sprite(frame, hud, pick, 0x21, 0x1B8)?;
    }
    hud.block.texture.set(0xC0);
    if flags & 0x10 != 0 {
        sprite(frame, hud, 0x5F, 0x282, 0x1B8)?;
    }
    if flags & 0x40 != 0 {
        draw_marker::draw_marker(frame, hud)?;
    }
    hud.block.quad[5].set(0xFFFF_FFFF);
    if flags & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, record, &draw_signs::LAYOUT_0)?;
    }
    hud.block.texture.set(0xC1);
    if flags & 1 != 0 {
        dials::needle(frame, hud, record, &dials::NEEDLE_0)?;
    }
    if flags & 0x40 != 0 {
        let panels = [
            (
                0x28u32,
                0x4Cu32,
                &hud.block.times[0],
                [0x14u32, 0x22, 0x2F, 0x36, 0x44, 0x51, 0x58, 0x66],
            ),
            (
                0x30,
                0x2F8,
                &hud.block.times[1],
                [0x2BD, 0x2CB, 0x2D8, 0x2DF, 0x2ED, 0x2FA, 0x301, 0x30F],
            ),
        ];
        for (field, x, time, xs) in panels {
            let Some(rect) = panels::stage_rect(get(field)?) else {
                continue;
            };
            sprite(frame, hud, rect, x, 1)?;
            let t = time.get();
            clock_panel(hud, t, xs, 0x13, &mut |r, x, y| sprite(frame, hud, r, x, y))?;
        }
    }
    if flags & 2 != 0 {
        let v = frame.panel_and_overlay.panel.view_a.place.get();
        if v != 0xFFFF_FFFF {
            let d = time_digits(hud, v, 2);
            sign(hud, v, 0, 1);
            sprite(frame, hud, d[4], 0x60, row(frame, 0x36))?;
            sprite(frame, hud, d[5], 0x8C, row(frame, 0x36))?;
        }
        sprite(frame, hud, 0, 0xE, row(frame, 0x33))?;
        sprite(frame, hud, 0x4A, 0xF, row(frame, 0x58))?;
        let t = hud.block.times[2].get();
        let xs = [0x2E, 0x42, 0x56, 0x5E, 0x72, 0x86, 0x8E, 0xA2];
        lap_panel(hud, t, xs, row(frame, 0x63), &mut |r, x, y| {
            sprite(frame, hud, r, x, y)
        })?;
    }
    if flags & 0x20 != 0 {
        let v = frame.panel_and_overlay.panel.view_a.speed.get();
        leading_time(hud, v, [0x43, 0x57, 0x6B], 0x23F, &mut |r, x, y| {
            sprite(frame, hud, r, x, y)
        })?;
    }
    Ok(())
}
