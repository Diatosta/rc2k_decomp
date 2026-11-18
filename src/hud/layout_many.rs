//! The HUD of a split screen of three or four players: each player's part of the screen gets the
//! one-player HUD (`hud::layout_0`), drawn as if the part were a screen of its own
//! (`hud::part_scope`): the dials, stage bar, clocks and signs keep to the part's corners and
//! middle by its shape. Each player's values (`Panel::parts`) stand in the first view's slots
//! while that player's part is drawn. The part three players leave free in the quarter layout
//! shows the whole course's map with each car on it (`race::overview_map`; black without one).
//! Thin black dividers lie between the parts.

use crate::errors::Refused;
use crate::hud::layout_common::{Keep, black_divider, keep, start_keep};
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::texture;
use crate::race::owner_frame::Frame;
use rc2k_core::race::view::{PlayerPart, Split};

/// The thickness of the dividers between parts, in pixels.
const DIVIDER: u32 = 2;
/// The overview map picture's side, in texels.
const OVERVIEW_SIDE: u32 = rc2k_core::race::overview::SIDE;

/// A solid quad of `colour` over `rect` (left, top, right, bottom).
fn solid(frame: &Frame, hud: &Hud, pen: Pen, rect: [u32; 4], colour: u32) -> Result<(), Refused> {
    hud.block.texture.set(texture::SPRITES);
    hud.block.quad.alpha.set(0xFF);
    hud.block.quad.colour.set(colour);
    hud.block.quad.set_rect(rect);
    crate::hud::add_solid_quad::add_solid_quad(frame, hud, pen)
}

/// The overview map in the part no player has: the course's picture as large as fits (a margin
/// all round), each car a square in its player's colour on a black one, at its place on the map
/// (held to the picture's edge where it is off it).
fn overview(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    free: PlayerPart,
    views: u32,
) -> Result<(), Refused> {
    let panel = &frame.panel_and_overlay.get().panel;
    let side = free.width.min(free.height) * 9 / 10;
    let (left, top) = (
        free.x + (free.width - side) / 2,
        free.y + (free.height - side) / 2,
    );
    // The picture as runs of one colour along its rows (colours cut to 4 bits each, so that runs
    // are long): the edges on the part's pixels by the same rule, so that the runs meet.
    let edge = |n: u32| left + n * side / OVERVIEW_SIDE;
    let mut runs = Vec::new();
    crate::race::overview_map::with_picture(|picture| {
        for (y, row) in picture.chunks(OVERVIEW_SIDE as usize).enumerate() {
            let y = y as u32;
            let cut = |c: Option<u32>| c.map(|c| c & 0xF0_F0F0);
            let mut x = 0;
            while x < row.len() {
                let colour = cut(row[x]);
                let end = row[x..]
                    .iter()
                    .position(|&c| cut(c) != colour)
                    .map_or(row.len(), |p| x + p);
                if let Some(colour) = colour {
                    runs.push((x as u32, end as u32, y, colour));
                }
                x = end;
            }
        }
    });
    for (from, to, y, colour) in runs {
        solid(
            frame,
            hud,
            pen,
            [
                edge(from),
                top + y * side / OVERVIEW_SIDE,
                edge(to),
                top + (y + 1) * side / OVERVIEW_SIDE,
            ],
            colour,
        )?;
    }
    let fit = rc2k_core::race::overview::Fit {
        origin: (panel.overview_fit[0].get(), panel.overview_fit[1].get()),
        ratio: panel.overview_fit[2].get(),
    };
    let mark = (side / 32).max(4);
    let side = side as f32;
    for player in 0..views.min(4) as usize {
        let cell = &panel.overview_cars[player];
        let (x, y) = fit.in_picture((cell[0].get(), cell[1].get()));
        let to_screen = |v: f32, from: u32| {
            from + (v.clamp(0.0, OVERVIEW_SIDE as f32) / OVERVIEW_SIDE as f32 * side) as u32
        };
        let (cx, cy) = (to_screen(x, left), to_screen(y, top));
        let half = mark / 2;
        solid(
            frame,
            hud,
            pen,
            [
                cx.saturating_sub(half + 1),
                cy.saturating_sub(half + 1),
                cx + half + 1,
                cy + half + 1,
            ],
            0,
        )?;
        solid(
            frame,
            hud,
            pen,
            [
                cx.saturating_sub(half),
                cy.saturating_sub(half),
                cx + half,
                cy + half,
            ],
            hud.block.marker_colours[player].get(),
        )?;
    }
    Ok(())
}

/// The panel's first-view slots (what the one-player HUD reads), kept while a part's player
/// takes them.
struct Slots {
    view: crate::race::panel_store::ViewA,
    flags: u32,
    start: u32,
    distance: u32,
    lights: [u32; 4],
    marks: [u32; 5],
}

impl Slots {
    fn keep(panel: &crate::race::panel_store::Panel) -> Self {
        let view = <crate::race::panel_store::ViewA as crate::memory::cells::ZeroCells>::ZERO;
        crate::race::frame_draw::copy_view(&panel.view_a, &view);
        Slots {
            view,
            flags: panel.hud_flags[0].get(),
            start: panel.start_values[0].get(),
            distance: panel.distance[0].get(),
            lights: lights_of(&panel.lights[0]),
            marks: marks_of(&panel.marks[0]),
        }
    }

    fn put_back(&self, panel: &crate::race::panel_store::Panel) {
        crate::race::frame_draw::copy_view(&self.view, &panel.view_a);
        panel.hud_flags[0].set(self.flags);
        panel.start_values[0].set(self.start);
        panel.distance[0].set(self.distance);
        set_lights(&panel.lights[0], self.lights);
        set_marks(&panel.marks[0], self.marks);
    }
}

fn marks_of(m: &crate::race::panel_store::Marks) -> [u32; 5] {
    [
        m.marks[0].get(),
        m.marks[1].get(),
        m.marks[2].get(),
        m.marks[3].get(),
        m.base.get(),
    ]
}

fn set_marks(m: &crate::race::panel_store::Marks, v: [u32; 5]) {
    for (cell, value) in m.marks.iter().zip(v) {
        cell.set(value);
    }
    m.base.set(v[4]);
}

fn lights_of(l: &crate::race::panel_store::Lights) -> [u32; 4] {
    [l.lit.get(), l.width.get(), l.alpha.get(), l.drop.get()]
}

fn set_lights(l: &crate::race::panel_store::Lights, v: [u32; 4]) {
    l.lit.set(v[0]);
    l.width.set(v[1]);
    l.alpha.set(v[2]);
    l.drop.set(v[3]);
}

/// The HUD of `views` players on their layout's parts: each part gets the one-player HUD, laid out
/// by the part's own shape. The players after the second have no map picture of their own, and
/// the map of the first goes when the overview map has its picture.
pub fn layout_many(frame: &Frame, hud: &Hud, pen: Pen, views: u32) -> Result<(), Refused> {
    let panel = &frame.panel_and_overlay.get().panel;
    let screen = pen.system.session.screen.get();
    let size = (screen.width.get(), screen.height.get());
    let Some(split) = Split::many(views, crate::race::split_layout::index()) else {
        return Ok(());
    };
    // The part no player has: black, or the overview map.
    if let Some(free) = split.free_part(views, size) {
        start_keep(hud, Keep::Pixels);
        keep(hud, Keep::Pixels);
        black_divider(
            frame,
            hud,
            pen,
            [free.x, free.y, free.x + free.width, free.y + free.height],
        )?;
        if panel.overview_on.get() != 0 {
            overview(frame, hud, pen, free, views)?;
        }
    }
    let kept = Slots::keep(panel);
    let live_motion = [
        panel
            .view_a
            .motion
            .each_ref()
            .map(crate::memory::session::PlainDword::get),
        panel
            .view_b
            .motion
            .each_ref()
            .map(crate::memory::session::PlainDword::get),
    ];
    let mut result = Ok(());
    for player in 0..views.min(4) {
        let Some(part) = split.part_of(views, player, size) else {
            continue;
        };
        let n = player as usize;
        crate::race::frame_draw::copy_view(&panel.parts[n], &panel.view_a);
        // The first two players' maps move with their cars, the others have none.
        let (map, motion) = match n {
            0 => (texture::DIAL_FIRST, Some(live_motion[0])),
            1 => (texture::DIAL_SECOND, Some(live_motion[1])),
            _ => (texture::DIAL_FIRST, None),
        };
        for (cell, value) in panel.view_a.motion.iter().zip(motion.unwrap_or_default()) {
            cell.set(value);
        }
        let mut flags = panel.hud_flags[n].get();
        if motion.is_none() {
            flags &= !0x10;
        }
        if n > 1 {
            // The signs come from the first two views' marks.
            flags &= !0x80;
        }
        panel.hud_flags[0].set(flags);
        panel.start_values[0].set(kept_start(panel, n, kept.start));
        panel.distance[0].set(panel.car_distances[n].get());
        set_lights(&panel.lights[0], lights_of(&panel.lights[n.min(1)]));
        set_marks(&panel.marks[0], marks_of(&panel.marks[n.min(1)]));
        crate::hud::tenths::tenths(frame, hud);
        crate::hud::part_scope::set(Some(part));
        crate::hud::part_scope::set_icon(Some(n));
        start_keep(hud, Keep::Thirds);
        let drawn = crate::hud::layout_0::layout_0_on(frame, hud, pen, map);
        crate::hud::part_scope::set(None);
        crate::hud::part_scope::set_icon(None);
        if drawn.is_err() {
            result = drawn;
            break;
        }
    }
    kept.put_back(panel);
    crate::hud::tenths::tenths(frame, hud);
    result?;
    // The dividers between the parts, over the HUDs, in screen pixels.
    start_keep(hud, Keep::Pixels);
    keep(hud, Keep::Pixels);
    for player in 0..views.min(4) {
        let Some(part) = split.part_of(views, player, size) else {
            continue;
        };
        // The part's right and bottom edges where another part lies beyond them.
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
    Ok(())
}

/// The start value of player `n`'s needle sign: the first two players' own, the others' the first's.
fn kept_start(panel: &crate::race::panel_store::Panel, n: usize, first: u32) -> u32 {
    match n {
        0 => first,
        1 => panel.start_values[1].get(),
        _ => first,
    }
}
