//! `hud::add_polygon` (0x4B7413): `g_polygonVertices` (`g_polygonVertexCount` of them)
//! scaled from the virtual screen to the real one (x times `g_screenWidth` over
//! `flt_75C02C`, y times `g_screenHeight` over `flt_75C030`) and added to the batch: when
//! the batch holds more than 0x64 vertices or the texture asked for (`dword_75C034`) is not
//! the batch's (`dword_75C038`), `hud::flush_batch`, the texture made the batch's and the
//! shown one (`dword_6D14FE`), `hud::set_batch_texture`; the vertices copied after the
//! batch's; the indices of a triangle (and with other than three vertices a second one
//! making a quad).
//!
//! In:  nothing (`g_polygonVertices`, `g_polygonVertexCount`)
//!
//! Callers: `hud::draw_stage_bar`, `hud::add_triangle`, `hud::turned_vertex`, `hud::add_quad`.
//! Refuses a count of 0 (the original's loop would run on) or past 0x10, and a batch it would write
//! past, before anything.

use crate::d3d::vertices_store::VERTICES;
use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::race::owner_frame::Frame;
use crate::scene::owner::scene;
use crate::system::owner::system;

/// The Rust version (see the header). What the original leaves in eax, ecx, esi and edi
/// is read by none of its callers (register liveness, 2026-10-23).
// Converted: `rc2k_core::hud::batch::{flushes, check_polygon, polygon_indices}`,
// `rc2k_core::hud::quads::screen_scaled`
pub fn add_polygon(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    let system = system();
    use rc2k_core::hud::batch::{self, MOST_VERTICES};
    use rc2k_core::hud::quads::screen_scaled;
    {
        let count: u32 = scene().drawing.scene.polygon_vertex_count.bits();
        let texture = hud.block.texture.get();
        let vertices = hud.block.vertex_count.get();
        let flushes = if count == 0 || count > MOST_VERTICES {
            false
        } else {
            batch::flushes(vertices, texture, hud.block.batch_texture.get())
        };
        batch::check_polygon(count, vertices, hud.block.index_count.get(), flushes)?;
        let [w, h]: [u32; 2] = [
            system.session.screen.width.get(),
            system.session.screen.height.get(),
        ];
        let [kx, ky] = [
            hud.block.virtual_screen[0].get(),
            hud.block.virtual_screen[1].get(),
        ];
        for n in 0..count as usize {
            let v = &VERTICES.vertices[n];
            v.x.set_bits(screen_scaled(v.x.get(), w, kx).to_bits());
            v.y.set_bits(screen_scaled(v.y.get(), h, ky).to_bits());
        }
        if flushes {
            crate::hud::flush_batch::flush_batch(hud);
            frame.panel_and_overlay.panel.shown_texture.set(texture);
            hud.block.batch_texture.set(texture);
            crate::hud::set_batch_texture::set_batch_texture(frame)?;
        }
        let vc = hud.block.vertex_count.get();
        crate::pointers::copy_between(
            &VERTICES.vertices,
            0,
            &hud.block.vertices,
            (vc * 0x20) as usize,
            (count * 0x20) as usize,
            "the hud's vertices",
        )?;
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
