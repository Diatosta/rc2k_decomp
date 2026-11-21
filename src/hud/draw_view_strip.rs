//! Three flat quads side by side for a view (a reading: a strip of the screen's overlay): the
//! view's top (a table entry) and height 0x19 scaled from 600 to the screen's height, left (a
//! table entry) and width 0x78 scaled from 800 to its width (into the strip's work rectangle),
//! texture rows 0x320000 apart; then the shared tail (`overlay_tail`, which
//! `hud::draw_view_strips` uses as well): with `hud_flags` bit 0x40 and `split` 0, two more quads
//! at the top corners (texture `polygon_textures` 0xC4). Refuses a view past 5 (the tables' end),
//! before anything.

use crate::errors::{BadInput, Refused};
use crate::gfx::strips::Strips;
use crate::race::owner::Race;
use crate::race::view_scratch::ViewScratch;
use crate::scene::owner::Scene;
use crate::system::owner::System;
use rc2k_core::hud::anchor::{BoxRatio, Rule};
use rc2k_core::hud::strips;

/// Sets the quad's input at byte offset `o` from the colour: its colour and see-through,
/// texture coordinates, then corners (three consecutive fields).
fn set_quad_input(scratch: &ViewScratch, o: usize, value: u32) {
    let mut quad = scratch.quad.borrow_mut();
    match o {
        0..0x8 => quad.colour[o / 4] = value,
        0x8..0x28 => quad.uv[(o - 8) / 4] = value,
        _ => quad.corners[(o - 0x28) / 4] = value,
    }
}

/// What the overlay's strips are drawn with.
pub struct Overlay<'a> {
    pub race: &'a Race,
    pub scratch: &'a ViewScratch,
    /// The drawing part's scratch (the game's thread's: the overlays draw after the parts).
    pub part: &'a crate::scene::scratch_store::PartScratch,
    pub system: &'a System,
    pub scene: &'a Scene,
    pub strips: &'a Strips,
    pub net: &'a crate::net::owner::Net,
}

/// The strip's sizes into `strips.sizes` (`strips::sizes`).
fn sizes(system: &System, strips: &Strips, v: [u32; 4], rule: Rule) {
    sizes_in(system, strips, v, rule, crate::hud::box_ratio::get());
}

/// `sizes` in the box `hud_box`.
fn sizes_in(system: &System, strips: &Strips, v: [u32; 4], rule: Rule, hud_box: BoxRatio) {
    // The strip keeps to the part of the screen clear of a phone's notch and home indicator.
    let [ox, oy, width, height] = crate::hud::safe_inset::get(
        (
            system.session.screen.get().width.get(),
            system.session.screen.get().height.get(),
        ),
        None,
    );
    let (mut sizes, _) = strips::sizes(v, width, height, rule, hud_box);
    sizes[0] += oy;
    sizes[2] += ox;
    for (n, x) in sizes.into_iter().enumerate() {
        strips.sizes[n].set(x);
    }
}

/// One flat quad: its inputs from `left`, `width`, `rows` (the texture's rows) and the
/// sizes (`strips::quad_inputs`), the texture when given, then the device's settings.
fn quad(
    overlay: &Overlay,
    left: u32,
    width: u32,
    rows: u32,
    texture: Option<u32>,
) -> Result<(), Refused> {
    let Overlay {
        race,
        scratch,
        part,
        system,
        scene,
        strips: strips_store,
        net,
    } = *overlay;
    let screen = system.session.screen.get();
    let polygon = &scratch.polygon;
    let course = &race.course;
    let sizes: [u32; 4] = std::array::from_fn(|n| strips_store.sizes[n].get());
    for (o, v) in strips::quad_inputs(left, width, &sizes, rows) {
        set_quad_input(scratch, o, v);
    }
    if let Some(t) = texture {
        polygon.scalars.borrow_mut().polygon_textures[0] = t;
    }
    scene.drawing.scene.borrow_mut().polygon_alpha = 0xFF;
    set_quad_input(scratch, 0, 0xFFFF_FFFF);
    polygon.scalars.borrow_mut().polygon_sides[0] = 1;
    polygon.scalars.borrow_mut().polygon_sides[1] = 0;
    crate::scene::draw_flat_quad::draw_flat_quad(
        polygon,
        part,
        &scratch.face,
        course.reads(),
        crate::scene::workers::DrawReads::on_game_thread(crate::scene::workers::LentOwners {
            race,
            scene,
            system,
            net,
        }),
        screen,
        scratch,
    )
}

/// The shared tail (see the header).
pub(super) fn overlay_tail(overlay: &Overlay) -> Result<(), Refused> {
    let Overlay {
        race,
        system,
        strips: strips_store,
        ..
    } = *overlay;
    let frame = &race.frame;
    let (shown, players) = (
        frame.panel.borrow().hud_flags[0],
        crate::hud::overlay_pass::shown_layout(&race.shared),
    );
    if !strips::tail_shown(shown, players) {
        return Ok(());
    }
    for (v, rows) in strips::TAIL {
        // The labels at the top corners lie on the bar's end caps: at the screen's own edges,
        // whatever the HUD's box (`hud::layout_0` keeps the bar there too).
        sizes_in(system, strips_store, v, Rule::Thirds, BoxRatio::Full);
        let get = |n: usize| strips_store.sizes[n].get();
        quad(overlay, get(2), get(3), rows, Some(strips::TAIL_TEXTURE))?;
    }
    Ok(())
}

/// The layout the strips are drawn for (0 one player, 1 side by side, 2 one above the other).
fn view_layout(race: &Race) -> u32 {
    crate::hud::overlay_pass::shown_layout(&race.shared)
}

/// View `view`'s strip, then the shared tail.
pub fn draw_view_strip(overlay: &Overlay, view: u32) -> Result<(), Refused> {
    let Overlay {
        race,
        system,
        strips,
        ..
    } = *overlay;
    let frame = &race.frame;
    if view >= strips::VIEWS {
        return Err(BadInput("a view past the overlay's tables").into());
    }
    {
        let top = strips::top_entry(view, frame.panel.borrow().flat);
        let top: u32 = strips.tops[top as usize];
        let left: u32 = strips.lefts[view as usize];
        // Two players side by side each keep to their own half.
        let rule = if view_layout(race) == 1 {
            Rule::Halves
        } else {
            Rule::Thirds
        };
        sizes(
            system,
            strips,
            [top, strips::HEIGHT, left, strips::WIDTH],
            rule,
        );
        let get = |n: usize| strips.sizes[n].get();
        let (mut left, width, mut rows) = (get(2), get(3), 0);
        for _ in 0..3 {
            quad(overlay, left, width, rows, None)?;
            left = left.wrapping_add(width);
            rows = rows.wrapping_add(strips::ROWS_STEP);
        }
        overlay_tail(overlay)
    }
}
