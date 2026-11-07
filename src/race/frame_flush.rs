//! The race frame's device batches and the view's backdrop under `sub_45429B` (part R3, IDA
//! 2026-10-18): two batches of triangles flushed by the device (`sub_41ADF5`, the platform's
//! `DrawPrimitive`), and a quad over the view window.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::scene::owner::scene;
use crate::session::Dword;

/// The render states both batches set: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (2, 0x14), (1, 0x1B)];

/// A batch: its count, its first word (into eax for the draw), the vertices (ebx) and
/// indices (ecx), where the count is kept, and the second render state's value (+0x14).
pub struct Batch<'a> {
    pub count: &'a Dword,
    pub word: &'a Dword,
    /// The platform's draw on the batch's vertices and indices (`sub_41ADF5`, its result),
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

/// `race::flush_triangle_batch` (0x45FDCF) / `race::flush_texture_batch` (0x45FEFE): the
/// first sets texture `dword_5D1D04` (`scene::set_texture`); the render states
/// (eax the value, ebx the state); with triangles in the batch (its count also into its
/// kept count) the second looks up its texture (`scene::look_up_texture`) and sets it
/// inside `pusha`; the draw (`sub_41ADF5`, the platform's: edx the
/// count, eax the word, ebx the vertices, ecx the indices), a failure (eax not 0) logged as
/// "drawtri" and its code (level 1); the count and word 0. Its one caller,
/// `race::draw_race_scene`, reads no register it leaves.
pub fn flush_batch(frame: &Frame, b: &Batch, textured: bool) -> Result<(), Refused> {
    let scene = scene();
    use crate::scene::{set_render_state::set_render_state, set_texture::set_texture};
    if !textured {
        set_texture(scene.textures.kinds.kind_first_slots[135].get());
    }
    for (value, state) in STATES {
        set_render_state(if state == 0x14 { b.blend } else { value }, state);
    }
    let count = b.count.get();
    b.kept.set(count);
    if count != 0 {
        if textured {
            // Inside `pusha`: the look-up, then its texture set.
            crate::scene::look_up_texture::look_up_texture();
            set_texture(scene.drawing.scene.polygon_texture.get());
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
    use crate::math::x87_float::{int, ld, st};
    let one = ld(course.numbers.one.bits());
    let i = int(i as i32);
    st(if add { i + one } else { i - one })
}

/// `fild i; fdiv a; fdiv b; fstp`.
fn int_over_two(i: u32, a: u32, b: u32) -> u32 {
    use crate::math::x87_float::{int, ld, st};
    st(int(i as i32) / ld(a) / ld(b))
}

/// `race::draw_view_backdrop` (0x4651E6): a quad over the view window (`dword_623FB0` ..,
/// one pixel wider each way; its corners kept in `dword_623E10` .. `dword_623E24`) with
/// texture place (0xF80000, 0x80000) / `flt_5C5E2C` / `flt_5C5D98` at every corner (kept in
/// `dword_623DF0` / `dword_623E00`), colour 0xFF000000, `g_polygonVertexCount` 4; drawn with
/// the glow's render states (`sub_44E44D`), texture `dword_5D1DE8` (`scene::set_texture`),
/// the device's unclipped fan (`d3d::draw_fan_unclipped`), then the depth buffer on. Its one
/// caller, `race::draw_mirror_view`, reads no register it leaves.
pub fn draw_view_backdrop(race: &Race) {
    let course = &race.course;
    let shared = &race.shared;
    let scene = scene();
    shared.vars.quad_uv[0].set(0xF8_0000);
    shared.vars.quad_uv[4].set(0x8_0000);
    let left = scene.drawing.scene.view_offset[0].get();
    shared.vars.quad_corners[0].set(left);
    shared.vars.quad_corners[1].set(left.wrapping_add(scene.drawing.scene.view_size[0].get()));
    let top = scene.drawing.scene.view_offset[1].get();
    shared.vars.quad_corners[4].set(top);
    shared.vars.quad_corners[5].set(top.wrapping_add(scene.drawing.scene.view_size[1].get()));
    let v = &crate::d3d::vertices_store::VERTICES.vertices;
    let x0 = int_and_one(course, shared.vars.quad_corners[0].get(), false);
    v[0].x.set_bits(x0);
    v[3].x.set_bits(x0);
    let x1 = int_and_one(course, shared.vars.quad_corners[1].get(), true);
    v[1].x.set_bits(x1);
    v[2].x.set_bits(x1);
    let y0 = int_and_one(course, shared.vars.quad_corners[4].get(), false);
    v[0].y.set_bits(y0);
    v[1].y.set_bits(y0);
    let y1 = int_and_one(course, shared.vars.quad_corners[5].get(), true);
    v[2].y.set_bits(y1);
    v[3].y.set_bits(y1);
    let (a, b) = (course.numbers.k256.bits(), course.numbers.k65536.bits());
    for vertex in &v[..4] {
        vertex
            .u
            .set_bits(int_over_two(shared.vars.quad_uv[0].get(), a, b));
    }
    for vertex in &v[..4] {
        vertex
            .v
            .set_bits(int_over_two(shared.vars.quad_uv[4].get(), a, b));
    }
    for vertex in &v[..4] {
        vertex.colour.set(0xFF00_0000);
    }
    scene.drawing.scene.polygon_vertex_count.set(4);
    crate::scene::glow_render_states::glow_render_states();
    crate::scene::set_texture::set_texture(scene.textures.kinds.kind_first_slots[192].get());
    crate::d3d::draw_fan_unclipped::draw_fan_unclipped();
    crate::scene::depth_buffer_on::depth_buffer(true);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock, same};
    use crate::race::owner::race;

    // The original blocks, verbatim.
    pub(super) fn int_and_one_asm(i: u32, add: bool) -> u32 {
        let mut out = 0u32;
        let v = [i, race().course.numbers.one.bits()];
        unsafe {
            if add {
                core::arch::asm!(
                    "fild dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]",
                    v = in(reg) v.as_ptr(), o = in(reg) &raw mut out,
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            } else {
                core::arch::asm!(
                    "fild dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fstp dword ptr [{o}]",
                    v = in(reg) v.as_ptr(), o = in(reg) &raw mut out,
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            }
        }
        out
    }

    pub(super) fn int_over_two_asm(i: u32, a: u32, b: u32) -> u32 {
        let (v, mut out) = ([i, a, b], 0u32);
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
                "fstp dword ptr [{o}]",
                v = in(reg) v.as_ptr(), o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        out
    }

    #[test]
    fn int_blocks_match() {
        let course = &race().course;
        let _l = lock();
        let _saved = Saved::new(&[0x5C5D38]);
        let mut r = Rng(5);
        at_each_control_word(|| {
            for _ in 0..20000 {
                race().course.numbers.one.set_bits(r.float());
                let (i, a, b) = (r.int() as u32, r.float(), r.float());
                for add in [false, true] {
                    let (want, got) = (int_and_one_asm(i, add), int_and_one(course, i, add));
                    assert!(same(want, got), "{i} {add}: {want:#x} {got:#x}");
                }
                let (want, got) = (int_over_two_asm(i, a, b), int_over_two(i, a, b));
                assert!(same(want, got), "{i} {a:#x} {b:#x}: {want:#x} {got:#x}");
            }
        });
    }
}
