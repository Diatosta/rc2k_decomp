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
        let [w, h]: [u32; 2] = [
            system.session.screen.get().width.get(),
            system.session.screen.get().height.get(),
        ];
        let [kx, ky] = [
            hud.block.virtual_screen[0].get(),
            hud.block.virtual_screen[1].get(),
        ];
        for v in pen.vertices.vertices.iter().take(count as usize) {
            v.x.set(screen_scaled(v.x.get(), w, kx));
            v.y.set(screen_scaled(v.y.get(), h, ky));
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
