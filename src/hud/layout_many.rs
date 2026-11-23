//! The HUD of a split screen of three or four players: each player's part of the screen gets the
//! one-player HUD (`hud::layout_0`), drawn as if the part were a screen of its own
//! (`hud::pen::PartScope`): the dials, stage bar, clocks and signs keep to the part's corners and
//! middle by its shape. Each player's values (`Panel::parts`) stand in the first view's slots
//! while that player's part is drawn. The part three players leave free in the quarter layout
//! shows the whole course's map with each car on it (`race::overview_map`; black without one).
//! Thin black dividers lie between the parts.

use crate::errors::Refused;
use crate::hud::block_store::{RectAt, texture};
use crate::hud::layout_common::{Keep, black_divider, keep, start_keep};
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;
use crate::race::panel_store::{Lights, Marks, Panel, ViewA};
use rc2k_core::race::view::PlayerPart;

/// The HUD flag of the stage bar, which `Panel::flat` holds of the HUD being drawn.
const FLAT_BIT: u32 = 0x40;
/// The thickness of the dividers between parts, in pixels.
const DIVIDER: u32 = 2;
/// The overview map picture's side, in texels.
const OVERVIEW_SIDE: u32 = rc2k_core::race::overview::SIDE;

/// A solid quad of `colour` over `rect` (left, top, right, bottom).
fn solid(
    frame: &Frame,
    hud: &mut Hud,
    pen: Pen,
    rect: [u32; 4],
    colour: u32,
) -> Result<(), Refused> {
    hud.block.texture = texture::SPRITES;
    hud.block.quad.alpha = 0xFF;
    hud.block.quad.colour = colour;
    hud.block.quad.set_rect(rect);
    crate::hud::add_solid_quad::add_solid_quad(frame, hud, pen)
}

/// The overview map in the part no player has: the course's picture as large as fits (a margin
/// all round), each car a square in its player's colour on a black one, at its place on the map
/// (held to the picture's edge where it is off it).
fn overview(
    frame: &Frame,
    hud: &mut Hud,
    pen: Pen,
    free: PlayerPart,
    views: u32,
) -> Result<(), Refused> {
    let (overview_fit, overview_cars) = {
        let panel = frame.panel.borrow();
        (panel.overview_fit, panel.overview_cars)
    };
    let side = free.width.min(free.height) * 9 / 10;
    let (left, top) = (
        free.x + (free.width - side) / 2,
        free.y + (free.height - side) / 2,
    );
    // The picture as runs of one colour along its rows (colours cut to 4 bits each, so that runs
    // are long): the edges on the part's pixels by the same rule, so that the runs meet.
    let edge = |n: u32| left + n * side / OVERVIEW_SIDE;
    let mut runs = Vec::new();
    crate::race::overview_map::with_picture(frame, |picture| {
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
        origin: (overview_fit[0], overview_fit[1]),
        ratio: overview_fit[2],
    };
    let mark = (side / 32).max(4);
    let side = side as f32;
    for player in 0..views.min(4) as usize {
        let cell = overview_cars[player];
        let (x, y) = fit.in_picture((cell[0], cell[1]));
        let to_screen = |v: f32, from: u32| {
            from + (v.clamp(0.0, OVERVIEW_SIDE as f32) / OVERVIEW_SIDE as f32 * side) as u32
        };
        let (cx, cy) = (to_screen(x, left), to_screen(y, top));
        // The map disc's own round marker, in the player's colour, scaled to the map's size.
        let [_, _, w, h] = RectAt::number(crate::hud::draw_map_cars::RECT).read(hud)?;
        let scale = f64::from(mark + 2) / f64::from(w.max(h).max(1));
        let (w, h) = (
            rc2k_core::hud::quads::scaled_size(w, scale),
            rc2k_core::hud::quads::scaled_size(h, scale),
        );
        hud.block.texture = texture::SPRITES;
        hud.block.quad.alpha = 0xFF;
        hud.block.quad.colour = hud.block.marker_colours[player];
        crate::hud::add_scaled_sprite::add_scaled_sprite_plain(
            frame,
            hud,
            pen,
            cx.saturating_sub(w / 2),
            cy.saturating_sub(h / 2),
            RectAt::number(crate::hud::draw_map_cars::RECT),
            [scale, scale],
        )?;
    }
    Ok(())
}

/// The panel's first-view slots (what the one-player HUD reads), kept while a part's player
/// takes them.
struct Slots {
    view: ViewA,
    flags: u32,
    flat: u32,
    start: u32,
    distance: u32,
    lights: Lights,
    marks: Marks,
}

impl Slots {
    fn keep(panel: &Panel) -> Self {
        Slots {
            view: panel.view_a,
            flags: panel.hud_flags[0],
            flat: panel.flat,
            start: panel.start_values[0],
            distance: panel.distance[0],
            lights: panel.lights[0],
            marks: panel.marks[0],
        }
    }

    fn put_back(&self, panel: &mut Panel) {
        panel.view_a = self.view;
        panel.hud_flags[0] = self.flags;
        panel.flat = self.flat;
        panel.start_values[0] = self.start;
        panel.distance[0] = self.distance;
        panel.lights[0] = self.lights;
        panel.marks[0] = self.marks;
    }
}

/// The HUD of `views` players on their layout's parts: each part gets the one-player HUD, laid out
/// by the part's own shape. Each player's map disc has a texture of its own, and the map of the
/// first goes when the overview map has its picture.
pub fn layout_many(frame: &Frame, hud: &mut Hud, pen: Pen, views: u32) -> Result<(), Refused> {
    let screen = pen.system.session.screen.get();
    let size = (screen.width.get(), screen.height.get());
    let Some(split) = frame.panel.borrow().many_split else {
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
        if frame.panel.borrow().overview_on != 0 {
            overview(frame, hud, pen, free, views)?;
        }
    }
    let (kept, live_motion) = {
        let panel = frame.panel.borrow();
        (
            Slots::keep(&panel),
            [panel.view_a.motion, panel.view_b.motion],
        )
    };
    let mut result = Ok(());
    for player in 0..views.min(4) {
        let Some(part) = split.part_of(views, player, size) else {
            continue;
        };
        let n = player as usize;
        // Every player's map moves with the player's car and has a texture of its own.
        let map = [
            texture::DIAL_FIRST,
            texture::DIAL_SECOND,
            texture::DIAL_THIRD,
            texture::DIAL_FOURTH,
        ][n];
        {
            let mut panel = frame.panel.borrow_mut();
            panel.view_a = panel.parts[n];
            if n < 2 {
                panel.view_a.motion = live_motion[n];
            }
            let flags = panel.hud_flags[n];
            panel.hud_flags[0] = flags;
            // The pieces placed by whether the HUD has its stage bar go by the part's own.
            panel.flat = flags & FLAT_BIT;
            panel.start_values[0] = kept_start(&panel, n, kept.start);
            panel.distance[0] = panel.car_distances[n];
            panel.lights[0] = panel.lights[n.min(1)];
            panel.marks[0] = *panel.marks_of_view(n);
        }
        crate::hud::tenths::tenths(frame, hud);
        start_keep(hud, Keep::Thirds);
        let car = frame.panel.borrow().part_cars[n] as usize;
        let part_pen = Pen {
            scope: crate::hud::pen::PartScope {
                part: Some(part),
                player: Some(n),
                icon: Some(car),
            },
            ..pen
        };
        let drawn = crate::hud::layout_0::layout_0_on(frame, hud, part_pen, map);
        if drawn.is_err() {
            result = drawn;
            break;
        }
    }
    kept.put_back(&mut frame.panel.borrow_mut());
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
fn kept_start(panel: &Panel, n: usize, first: u32) -> u32 {
    match n {
        0 => first,
        1 => panel.start_values[1],
        _ => first,
    }
}
