//! The race frame's two device batches' render states and the view's backdrop under
//! `race::draw_race_scene`: a quad over the view window.

use crate::race::owner_course::CourseReads;

/// The render states both batches set: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (2, 0x14), (1, 0x1B)];

/// Sets the render states of the plain batch (`textured` false: it sets the plain texture first) or
/// the textured batch (second render state's value 6) in drawing part `part`. The race adds no
/// triangles to either batch, so there is nothing to draw.
pub fn flush_batch(
    scene: &crate::scene::owner::Scene,
    part: &crate::scene::scratch_store::PartScratch,
    textured: bool,
) {
    use crate::scene::{set_render_state::set_render_state, set_texture::set_texture};
    if !textured {
        set_texture(
            part,
            scene,
            scene.textures.kinds.borrow().kind_first_slots[135],
        );
    }
    let blend = if textured { 6 } else { 2 };
    for (value, state) in STATES {
        set_render_state(part, if state == 0x14 { blend } else { value }, state);
    }
}

/// (i + one) or (i - one) as a float.
fn int_and_one(course: CourseReads<'_>, i: u32, add: bool) -> f32 {
    use crate::math::x87_float::{int, stf};
    let one = course.numbers.one.ld();
    let i = int(i as i32);
    stf(if add { i + one } else { i - one })
}

/// i / a / b as a float.
fn int_over_two(
    i: u32,
    a: impl crate::math::x87_float::Loadable,
    b: impl crate::math::x87_float::Loadable,
) -> f32 {
    use crate::math::x87_float::{int, ld, stf};
    stf(int(i as i32) / ld(a) / ld(b))
}

/// Draws a quad over the view window (`view_offset` .., one pixel wider each way; its corners kept
/// in `quad_corners` .. `RaceVars.quad_corners[5]`) with texture place / `k256` / `k65536` at every
/// corner (kept in `quad_uv` / `RaceVars.quad_uv[4]`), colour 0xFF000000, `polygon_vertex_count` 4;
/// drawn with the glow's render states, texture `KindSlots.kind_first_slots[192]`
/// (`scene::set_texture`), the device's unclipped fan (`d3d::draw_fan_unclipped`), then the depth
/// buffer on.
pub(super) fn draw_view_backdrop(
    reads: crate::scene::workers::DrawReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
    part: &crate::scene::scratch_store::PartScratch,
) {
    let buffer = &part.vertices;
    view.quad.borrow_mut().uv[0] = 0xF8_0000;
    view.quad.borrow_mut().uv[4] = 0x8_0000;
    let left = view.polygon.window.view_offset[0].get();
    view.quad.borrow_mut().corners[0] = left;
    view.quad.borrow_mut().corners[1] = left.wrapping_add(view.polygon.window.view_size[0].get());
    let top = view.polygon.window.view_offset[1].get();
    view.quad.borrow_mut().corners[4] = top;
    view.quad.borrow_mut().corners[5] = top.wrapping_add(view.polygon.window.view_size[1].get());
    let v = &buffer.vertices;
    let x0 = int_and_one(reads.course(), view.quad.borrow().corners[0], false);
    v[0].x.set(x0);
    v[3].x.set(x0);
    let x1 = int_and_one(reads.course(), view.quad.borrow().corners[1], true);
    v[1].x.set(x1);
    v[2].x.set(x1);
    let y0 = int_and_one(reads.course(), view.quad.borrow().corners[4], false);
    v[0].y.set(y0);
    v[1].y.set(y0);
    let y1 = int_and_one(reads.course(), view.quad.borrow().corners[5], true);
    v[2].y.set(y1);
    v[3].y.set(y1);
    let (a, b) = (reads.numbers.k256.get(), reads.numbers.k65536.get());
    for vertex in &v[..4] {
        vertex.u.set(int_over_two(view.quad.borrow().uv[0], a, b));
    }
    for vertex in &v[..4] {
        vertex.v.set(int_over_two(view.quad.borrow().uv[4], a, b));
    }
    for vertex in &v[..4] {
        vertex.colour.set(0xFF00_0000);
    }
    // The depth, 1/w and specular are set to 0 here, so that the backdrop does not hang on what
    // the buffer held from the drawing before it (the mirror is drawn on a thread of its own).
    for vertex in &v[..4] {
        vertex.z.set(0.0);
        vertex.rhw.set(0.0);
        vertex.specular.set(0);
    }
    // The count in the view's own polygon work.
    view.polygon.polygon_vertex_count.set(4);
    crate::scene::glow_render_states::glow_render_states(part);
    crate::scene::set_texture::set_texture_with(
        part,
        reads.object_tables,
        reads.kind_textures.first_slot(192),
    );
    crate::d3d::draw_fan_unclipped::draw_fan_unclipped(part, &view.polygon, buffer);
    crate::scene::depth_buffer_on::depth_buffer(part, true);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, same};
    use crate::test_support::own::race;
    use crate::test_support::vectors::{Recorder, word};

    // The recorded blocks.

    #[test]
    fn int_blocks_match() {
        let course = &race().course;
        let mut r = Rng(5);
        let mut file = Recorder::replay("race.frame_flush.int_blocks");
        at_each_control_word(|| {
            for _ in 0..20000 {
                let one = f32::from_bits(r.float());
                let numbers = crate::race::numbers_store::Numbers {
                    one: crate::memory::session::ConstFloat::new(one),
                    ..race().course.numbers
                };
                let (i, a, b) = (r.int() as u32, r.float(), r.float());
                for add in [false, true] {
                    file.input(&[i.to_le_bytes(), one.to_le_bytes()].concat());
                    let want = word(&mut file);
                    let got = int_and_one(
                        CourseReads {
                            numbers: &numbers,
                            ..course.reads()
                        },
                        i,
                        add,
                    )
                    .to_bits();
                    assert!(same(want, got), "{i} {add}: {want:#x} {got:#x}");
                }
                for w in [i, a, b] {
                    file.input(&w.to_le_bytes());
                }
                let want = word(&mut file);
                let got = int_over_two(i, a, b).to_bits();
                assert!(same(want, got), "{i} {a:#x} {b:#x}: {want:#x} {got:#x}");
            }
        });
        file.finish();
    }
}
