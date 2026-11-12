//! The race frame's device batches and the view's backdrop under `race::draw_race_scene`: two batches of triangles flushed by the device (0x41ADF5, the platform's
//! `DrawPrimitive`), and a quad over the view window.

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::scene::owner::scene;
use crate::session::Dword;

/// The render states both batches set: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (2, 0x14), (1, 0x1B)];

/// A batch: its count, its first word (the draw's first argument), the vertices and
/// indices, where the count is kept, and the second render state's value (+0x14).
pub struct Batch<'a> {
    pub count: &'a Dword,
    pub word: &'a Dword,
    /// The platform's draw on the batch's vertices and indices (0x41ADF5, its result),
    /// given their counts.
    pub draw: fn(&Frame, u32, u32) -> u32,
    pub kept: &'a Dword,
    pub blend: u32,
    pub text: &'static std::ffi::CStr,
}

/// `race::flush_triangle_batch`'s (0x45FDCF).
pub fn plain(frame: &Frame) -> Batch<'_> {
    Batch {
        count: &frame.draw_batches.plain_count,
        word: &frame.draw_batches.plain_word,
        draw: |frame, vertices, indices| {
            crate::platform_3d::draw_indexed(
                &frame.draw_batches.plain_vertices,
                vertices,
                &frame.draw_batches.plain_indices,
                indices,
            )
        },
        kept: &frame.draw_batches.plain_kept,
        blend: 2,
        text: c"drawtri",
    }
}

/// `race::flush_texture_batch`'s (0x45FEFE).
pub fn textured(frame: &Frame) -> Batch<'_> {
    Batch {
        count: &frame.draw_batches.textured_count,
        word: &frame.draw_batches.textured_word,
        draw: |frame, vertices, indices| {
            crate::platform_3d::draw_indexed(
                &frame.draw_batches.textured_vertices,
                vertices,
                &frame.draw_batches.textured_indices,
                indices,
            )
        },
        kept: &frame.draw_batches.textured_kept,
        blend: 6,
        text: c"drawtri",
    }
}

/// `race::flush_triangle_batch` (0x45FDCF) / `race::flush_texture_batch`: the first sets texture
/// `KindTextureTables.kind_first_slots[135]` (`scene::set_texture`); the render states; with triangles in the batch (its count
/// also into its kept count) the second looks up its texture (`scene::look_up_texture`) and sets it
/// inside `pusha`; the draw, a failure logged as "drawtri" and its code (level 1); the count and
/// word 0.
pub fn flush_batch(frame: &Frame, b: &Batch, textured: bool) -> Result<(), Refused> {
    let scene = scene();
    use crate::scene::{set_render_state::set_render_state, set_texture::set_texture};
    if !textured {
        set_texture(scene, scene.textures.kinds.kind_first_slots[135].get());
    }
    for (value, state) in STATES {
        set_render_state(if state == 0x14 { b.blend } else { value }, state);
    }
    let count = b.count.get();
    b.kept.set(count);
    if count != 0 {
        if textured {
            // Inside `pusha`: the look-up, then its texture set.
            crate::scene::look_up_texture::look_up_texture(
                crate::scene::polygon_work::front_end(),
                scene,
            );
            set_texture(
                scene,
                crate::scene::polygon_work::front_end()
                    .polygon_texture
                    .get(),
            );
        }
        let result = (b.draw)(frame, b.word.get(), count);
        if result != 0 {
            crate::logfile::log_write::log_nul(&b.text, false, 1);
            crate::logfile::log_hex8::log_hex8(result, false, 1);
        }
    }
    b.count.set(0);
    b.word.set(0);
    Ok(())
}

/// `fild i; fadd/fsub one; fstp`.
fn int_and_one(course: &Course, i: u32, add: bool) -> u32 {
    use crate::math::x87_float::{int, st};
    let one = course.numbers.one.ld();
    let i = int(i as i32);
    st(if add { i + one } else { i - one })
}

/// `fild i; fdiv a; fdiv b; fstp`.
fn int_over_two(
    i: u32,
    a: impl crate::math::x87_float::Loadable,
    b: impl crate::math::x87_float::Loadable,
) -> u32 {
    use crate::math::x87_float::{int, ld, st};
    st(int(i as i32) / ld(a) / ld(b))
}

/// `race::draw_view_backdrop` (0x4651E6): a quad over the view window (`view_offset` .., one pixel
/// wider each way; its corners kept in `quad_corners` .. `RaceVars.quad_corners[5]`) with texture place
/// (0xF80000, 0x80000) / `k256` / `k65536` at every corner (kept in `quad_uv` /
/// `RaceVars.quad_uv[4]`), colour 0xFF000000, `g_polygonVertexCount` 4; drawn with the glow's render
/// states (0x44E44D), texture `KindTextureTables.kind_first_slots[192]` (`scene::set_texture`), the device's unclipped fan
/// (`d3d::draw_fan_unclipped`), then the depth buffer on.
pub fn draw_view_backdrop(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    view: &crate::race::view_scratch::ViewScratch,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
) {
    let course = &race.course;
    view.quad.uv[0].set(0xF8_0000);
    view.quad.uv[4].set(0x8_0000);
    let left = view.polygon.window.view_offset[0].get();
    view.quad.corners[0].set(left);
    view.quad.corners[1].set(left.wrapping_add(view.polygon.window.view_size[0].get()));
    let top = view.polygon.window.view_offset[1].get();
    view.quad.corners[4].set(top);
    view.quad.corners[5].set(top.wrapping_add(view.polygon.window.view_size[1].get()));
    let v = &buffer.vertices;
    let x0 = int_and_one(course, view.quad.corners[0].get(), false);
    v[0].x.set_bits(x0);
    v[3].x.set_bits(x0);
    let x1 = int_and_one(course, view.quad.corners[1].get(), true);
    v[1].x.set_bits(x1);
    v[2].x.set_bits(x1);
    let y0 = int_and_one(course, view.quad.corners[4].get(), false);
    v[0].y.set_bits(y0);
    v[1].y.set_bits(y0);
    let y1 = int_and_one(course, view.quad.corners[5].get(), true);
    v[2].y.set_bits(y1);
    v[3].y.set_bits(y1);
    let (a, b) = (course.numbers.k256.get(), course.numbers.k65536.get());
    for vertex in &v[..4] {
        vertex.u.set_bits(int_over_two(view.quad.uv[0].get(), a, b));
    }
    for vertex in &v[..4] {
        vertex.v.set_bits(int_over_two(view.quad.uv[4].get(), a, b));
    }
    for vertex in &v[..4] {
        vertex.colour.set(0xFF00_0000);
    }
    // The original left the depth, 1/w and specular as the buffer held them (the drawing
    // before's: 0 on the first three vertices, a leftover on the fourth, depth tested
    // against the view's); 0 here, so that the backdrop does not hang on what was drawn
    // before it (frame plan F7: the mirror drawn on a thread of its own).
    for vertex in &v[..4] {
        vertex.z.set_bits(0);
        vertex.rhw.set_bits(0);
        vertex.specular.set(0);
    }
    // The count in the view's own polygon work (the original's is the one shared; the
    // mirror's backdrop is drawn on the mirror's part, frame plan F7).
    view.polygon.polygon_vertex_count.set(4);
    crate::scene::glow_render_states::glow_render_states();
    crate::scene::set_texture::set_texture(scene, scene.textures.kinds.kind_first_slots[192].get());
    crate::d3d::draw_fan_unclipped::draw_fan_unclipped(&view.polygon, buffer);
    crate::scene::depth_buffer_on::depth_buffer(true);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, lock, same};
    use crate::race::owner::race;
    use crate::test_support::vectors::{Recorder, word};

    /// `numbers.one` as it was, put back when dropped.
    struct KeptOne(u32);

    impl Drop for KeptOne {
        fn drop(&mut self) {
            race().course.numbers.one.set_bits(self.0);
        }
    }

    // The original blocks, verbatim.

    #[test]
    fn int_blocks_match() {
        let course = &race().course;
        let _l = lock();
        let _kept = KeptOne(race().course.numbers.one.bits());
        let mut r = Rng(5);
        let mut file = Recorder::replay("race.frame_flush.int_blocks");
        at_each_control_word(|| {
            for _ in 0..20000 {
                race().course.numbers.one.set_bits(r.float());
                let (i, a, b) = (r.int() as u32, r.float(), r.float());
                for add in [false, true] {
                    file.input(
                        &[
                            i.to_le_bytes(),
                            race().course.numbers.one.bits().to_le_bytes(),
                        ]
                        .concat(),
                    );
                    let want = word(&mut file);
                    let got = int_and_one(course, i, add);
                    assert!(same(want, got), "{i} {add}: {want:#x} {got:#x}");
                }
                for w in [i, a, b] {
                    file.input(&w.to_le_bytes());
                }
                let want = word(&mut file);
                let got = int_over_two(i, a, b);
                assert!(same(want, got), "{i} {a:#x} {b:#x}: {want:#x} {got:#x}");
            }
        });
        file.finish();
    }
}
