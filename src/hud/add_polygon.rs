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
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;

pub fn add_polygon(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    let system = pen.system;
    use rc2k_core::hud::anchor;
    use rc2k_core::hud::batch::{self, MOST_VERTICES};
    use rc2k_core::hud::quads::screen_scaled;
    {
        let count: u32 = pen.polygons.polygon_vertex_count.get();
        let texture = hud.block.texture.get();
        let vertices = hud.block.vertex_count.get();
        let flushes = if count == 0 || count > MOST_VERTICES {
            false
        } else {
            batch::flushes(vertices, texture, hud.block.batch_texture.get())
        };
        batch::check_polygon(count, vertices, hud.block.index_count.get(), flushes)?;
        // A part of the screen is drawn into as a screen of its own (`hud::part_scope`).
        let part = crate::hud::part_scope::get();
        let [w, h]: [u32; 2] = match part {
            Some(part) => [part.width, part.height],
            None => [
                system.session.screen.get().width.get(),
                system.session.screen.get().height.get(),
            ],
        };
        let [kx, ky] = [
            hud.block.virtual_screen[0].get(),
            hud.block.virtual_screen[1].get(),
        ];
        let vertices = || pen.vertices.vertices.iter().take(count as usize);
        // On a screen wider than 4:3 the layout keeps its shape and the piece keeps to an edge
        // or the middle (`rc2k_core::hud::anchor`); the divider of two players one above the
        // other is stretched over the whole width.
        let rule = hud.block.anchor_rule.get();
        // The pieces keep to the HUD's box, centred on the screen (all of it by default); the
        // divider's own rule stretches over the whole screen.
        let (box_left, box_width) = if rule >= 3 {
            (0, w)
        } else {
            crate::hud::box_ratio::get().area(w, h)
        };
        let wide = anchor::is_wide(box_width, h) && (kx, ky) == (800.0, 600.0);
        // Two players side by side on a wide screen: each half is a part of its own.
        let keep = crate::hud::layout_common::Keep::Halves as u32;
        let part_middle = crate::hud::layout_common::Keep::PartMiddle as u32;
        let beside = (rule == keep || rule == part_middle)
            && anchor::is_wide(w, h)
            && (kx, ky) == (800.0, 600.0);
        let middle_of = || {
            let (low, high) = vertices().fold((f32::MAX, f32::MIN), |(lo, hi), v| {
                (lo.min(v.x.get()), hi.max(v.x.get()))
            });
            (low + high) / 2.0
        };
        let x_of: Box<dyn Fn(f32) -> f32> = if beside {
            let map = anchor::Beside::new(
                middle_of(),
                w,
                h,
                crate::hud::box_ratio::get(),
                rule == part_middle,
            );
            Box::new(move |x| map.x(x))
        } else if wide && rule == crate::hud::layout_common::Keep::Whole as u32 {
            // Stretched over the whole width, as the divider of two players one above the other.
            Box::new(move |x| x * w as f32 / 800.0)
        } else if wide && rule == 4 {
            Box::new(move |x| anchor::bar_x(x, box_width, h))
        } else if wide && rule == 5 {
            let (low, high) = vertices().fold((f32::MAX, f32::MIN), |(lo, hi), v| {
                (lo.min(v.x.get()), hi.max(v.x.get()))
            });
            let middle = (low + high) / 2.0;
            let (at, s) = (anchor::bar_x(middle, box_width, h), h as f32 / 600.0);
            Box::new(move |x| at + (x - middle) * s)
        } else if !wide {
            Box::new(move |x| box_left as f32 + screen_scaled(x, box_width, kx))
        } else {
            let (low, high) = vertices().fold((f32::MAX, f32::MIN), |(lo, hi), v| {
                (lo.min(v.x.get()), hi.max(v.x.get()))
            });
            let rule = match rule {
                1 => anchor::Rule::Halves,
                2 => anchor::Rule::Centre,
                _ => anchor::Rule::Thirds,
            };
            let at = anchor::anchor_of((low + high) / 2.0, rule);
            Box::new(move |x| box_left as f32 + anchor::screen_x(x, box_width, h, at))
        };
        // Pieces that are already in screen pixels are left where they are.
        if rule != crate::hud::layout_common::Keep::Pixels as u32 {
            for v in vertices() {
                v.x.set(x_of(v.x.get()));
                v.y.set(if wide || beside {
                    anchor::screen_y(v.y.get(), box_width, h)
                } else {
                    screen_scaled(v.y.get(), h, ky)
                });
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
            frame
                .panel_and_overlay
                .get()
                .panel
                .shown_texture
                .set(texture);
            hud.block.batch_texture.set(texture);
            crate::hud::set_batch_texture::set_batch_texture(pen.part, frame, pen.scene)?;
        }
        let vc = hud.block.vertex_count.get();
        // Room for them was checked above (the batch holds 0x400 vertices).
        for (from, to) in pen
            .vertices
            .vertices
            .iter()
            .zip(&hud.block.vertices[vc as usize..])
            .take(count as usize)
        {
            to.x.set(from.x.get());
            to.y.set(from.y.get());
            to.z.set(from.z.get());
            to.rhw.set(from.rhw.get());
            to.colour.set(from.colour.get());
            to.specular.set(from.specular.get());
            to.u.set(from.u.get());
            to.v.set(from.v.get());
        }
        let ic = hud.block.index_count.get();
        let indices = batch::polygon_indices(vc, count);
        for (k, &i) in indices.iter().enumerate() {
            hud.block.indices[ic as usize + k].set(i);
        }
        hud.block.index_count.set(ic + indices.len() as u32);
        hud.block.vertex_count.set(vc + count);
    }
    Ok(())
}
