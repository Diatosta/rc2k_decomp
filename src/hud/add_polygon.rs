//! The polygon vertices scaled from the virtual screen to the real one (x times the screen's
//! width over `HudBlock.virtual_screen[0]`, y times the height over `virtual_screen[1]`) and
//! added to the batch: when the batch holds more than 0x64 vertices or the texture asked for
//! (`HudBlock.texture`) is not the batch's (`HudBlock.batch_texture`), `hud::flush_batch`, the
//! texture made the batch's and the shown one (`shown_texture`), `hud::set_batch_texture`; the
//! vertices copied after the batch's; the indices of a triangle (and with other than three
//! vertices a second one making a quad).
//!
//! Refuses a count of 0 or past 0x10, and a batch it would write past, before anything.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;

pub fn add_polygon(frame: &Frame, hud: &mut Hud, pen: Pen) -> Result<(), Refused> {
    let system = pen.system;
    use rc2k_core::hud::batch::{self, MOST_VERTICES};
    use rc2k_core::hud::place::{Place, Placement};
    {
        let count: u32 = pen.polygons.scalars.borrow().polygon_vertex_count;
        let texture = hud.block.texture;
        let vertices = hud.block.vertex_count;
        let flushes = if count == 0 || count > MOST_VERTICES {
            false
        } else {
            batch::flushes(vertices, texture, hud.block.batch_texture)
        };
        batch::check_polygon(count, vertices, hud.block.index_count, flushes)?;
        // A part of the screen is drawn into as a screen of its own (`PartScope`).
        let part = pen.scope.part;
        let [w, h]: [u32; 2] = match part {
            Some(part) => [part.width, part.height],
            None => [
                system.session.screen.get().width.get(),
                system.session.screen.get().height.get(),
            ],
        };
        let rule = hud.block.anchor_rule;
        // Where a phone's notch and home indicator leave the HUD room (the whole screen or part
        // elsewhere): the layout is made for that, and put where it is.
        let inset = if rule >= 3 {
            [0, 0, w, h]
        } else {
            crate::hud::safe_inset::get(
                (
                    system.session.screen.get().width.get(),
                    system.session.screen.get().height.get(),
                ),
                part,
            )
        };
        let vertices = || pen.vertices.vertices.iter().take(count as usize);
        let span = |of: fn(&crate::d3d::vertices_store::Vertex) -> f32| {
            vertices().fold((f32::MAX, f32::MIN), |(low, high), v| {
                (low.min(of(v)), high.max(of(v)))
            })
        };
        // On a screen wider than 4:3 the layout keeps its shape and the piece keeps to an edge
        // or the middle; the divider of two players one above the other is stretched over the
        // whole width (`rc2k_core::hud::place`).
        let placement = Placement::of(&Place {
            rule,
            width: w,
            inset,
            hud_box: crate::hud::box_ratio::get(),
            virtual_screen: (hud.block.virtual_screen[0], hud.block.virtual_screen[1]),
            marker_k: [hud.block.marker_k[0], hud.block.marker_k[1]],
            x_span: span(|v| v.x.get()),
            y_span: span(|v| v.y.get()),
            in_part: part.is_some(),
            one_player: part.is_none()
                && batch::Layout::of(frame.panel.borrow().players) == batch::Layout::Single,
        });
        // Pieces that are already in screen pixels are left where they are.
        if !placement.keeps_pixels() {
            for v in vertices() {
                v.x.set(placement.x(v.x.get()));
                v.y.set(placement.y(v.y.get()));
            }
        }
        if let Some(part) = part {
            for v in vertices() {
                v.x.set(v.x.get() + part.x as f32);
                v.y.set(v.y.get() + part.y as f32);
            }
        }
        if flushes {
            crate::hud::flush_batch::flush_batch(pen.part, hud);
            frame.panel.borrow_mut().shown_texture = texture;
            hud.block.batch_texture = texture;
            crate::hud::set_batch_texture::set_batch_texture(pen.part, frame, pen.scene)?;
        }
        let vc = hud.block.vertex_count;
        // Room for them was checked above (the batch holds 0x400 vertices).
        for (from, to) in pen
            .vertices
            .vertices
            .iter()
            .zip(&mut hud.block.vertices[vc as usize..])
            .take(count as usize)
        {
            to.x = from.x.get();
            to.y = from.y.get();
            to.z = from.z.get();
            to.rhw = from.rhw.get();
            to.colour = from.colour.get();
            to.specular = from.specular.get();
            to.u = from.u.get();
            to.v = from.v.get();
        }
        let ic = hud.block.index_count;
        let indices = batch::polygon_indices(vc, count);
        for (k, &i) in indices.iter().enumerate() {
            hud.block.indices[ic as usize + k] = i;
        }
        hud.block.index_count = ic + indices.len() as u32;
        hud.block.vertex_count = vc + count;
    }
    Ok(())
}
