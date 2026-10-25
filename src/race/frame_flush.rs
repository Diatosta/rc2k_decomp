//! The race frame's device batches and the view's backdrop under `sub_45429B` (part R3, IDA
//! 2026-10-18): two batches of triangles flushed by the device (`sub_41ADF5`, the platform's
//! `DrawPrimitive`), and a quad over the view window.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

fn ga(a: u32) -> u32 {
    game_addr(rva(a as usize)) as u32
}

/// The render states both batches set: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (2, 0x14), (1, 0x1B)];

/// A batch: its count, its first word (into eax for the draw), the vertices (ebx) and
/// indices (ecx), where the count is kept, and the second render state's value (+0x14).
pub struct Batch {
    pub count: u32,
    pub word: u32,
    pub vertices: u32,
    pub indices: u32,
    pub kept: u32,
    pub blend: u32,
    pub text: &'static std::ffi::CStr,
}

/// `race::flush_triangle_batch`'s (0x45FDCF).
pub const PLAIN: Batch = Batch {
    count: 0x6E44F4,
    word: 0x6E44F8,
    vertices: 0x6D94F4,
    indices: 0x6E14F4,
    kept: 0x6EF7C4,
    blend: 2,
    text: c"drawtri",
};

/// `race::flush_texture_batch`'s (0x45FEFE).
pub const TEXTURED: Batch = Batch {
    count: 0x6EF7BC,
    word: 0x6EF7C0,
    vertices: 0x6E44FC,
    indices: 0x6EC6FC,
    kept: 0x6EF7C8,
    blend: 6,
    text: c"drawtri",
};

/// `race::flush_triangle_batch` (0x45FDCF) / `race::flush_texture_batch` (0x45FEFE): the
/// first sets texture `dword_5D1D04` (`scene::set_texture`); the render states
/// (eax the value, ebx the state); with triangles in the batch (its count also into its
/// kept count) the second looks up its texture (`scene::look_up_texture`) and sets it
/// inside `pusha`; the draw (`sub_41ADF5`, the platform's: edx the
/// count, eax the word, ebx the vertices, ecx the indices), a failure (eax not 0) logged as
/// "drawtri" and its code (level 1); the count and word 0. Its one caller,
/// `race::draw_race_scene`, reads no register it leaves.
pub unsafe fn flush_batch(b: &Batch, textured: bool) -> Result<(), Refused> {
    use crate::scene::{set_render_state::set_render_state, set_texture::set_texture};
    unsafe {
        if !textured {
            set_texture(k(0x5D1D04));
        }
        for (value, state) in STATES {
            set_render_state(if state == 0x14 { b.blend } else { value }, state);
        }
        let count = k(b.count);
        w(b.kept, count);
        if count != 0 {
            if textured {
                // Inside `pusha`: the look-up, then its texture set.
                crate::scene::look_up_texture::look_up_texture();
                set_texture(DRAW.polygon_texture.get());
            }
            let result =
                crate::platform_3d::draw_indexed(ga(b.vertices), k(b.word), ga(b.indices), count);
            if result != 0 {
                crate::logfile::log_write::log_write(b.text.as_ptr().cast(), false, 1);
                crate::logfile::log_hex8::log_hex8(result, false, 1);
            }
        }
        w(b.count, 0);
        w(b.word, 0);
        Ok(())
    }
}

/// `fild i; fadd/fsub one; fstp`.
fn int_and_one(i: u32, add: bool) -> u32 {
    use crate::math::x87_float::{int, ld, st};
    let one = ld(NUMBERS.one.bits());
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
pub unsafe fn draw_view_backdrop() {
    RACE_VARS.quad_uv[0].set(0xF8_0000);
    RACE_VARS.quad_uv[4].set(0x8_0000);
    let left = DRAW.view_offset[0].get();
    RACE_VARS.quad_corners[0].set(left);
    RACE_VARS.quad_corners[1].set(left.wrapping_add(DRAW.view_size[0].get()));
    let top = DRAW.view_offset[1].get();
    RACE_VARS.quad_corners[4].set(top);
    RACE_VARS.quad_corners[5].set(top.wrapping_add(DRAW.view_size[1].get()));
    let v = |o: u32| 0x9605A0 + o;
    let x0 = int_and_one(RACE_VARS.quad_corners[0].get(), false);
    w(v(0), x0);
    w(v(0x60), x0);
    let x1 = int_and_one(RACE_VARS.quad_corners[1].get(), true);
    w(v(0x20), x1);
    w(v(0x40), x1);
    let y0 = int_and_one(RACE_VARS.quad_corners[4].get(), false);
    w(v(4), y0);
    w(v(0x24), y0);
    let y1 = int_and_one(RACE_VARS.quad_corners[5].get(), true);
    w(v(0x44), y1);
    w(v(0x64), y1);
    let (a, b) = (NUMBERS.k256.bits(), NUMBERS.k65536.bits());
    for n in 0..4 {
        w(
            v(0x18 + 0x20 * n),
            int_over_two(RACE_VARS.quad_uv[0].get(), a, b),
        );
    }
    for n in 0..4 {
        w(
            v(0x1C + 0x20 * n),
            int_over_two(RACE_VARS.quad_uv[4].get(), a, b),
        );
    }
    for n in 0..4 {
        w(v(0x10 + 0x20 * n), 0xFF00_0000);
    }
    DRAW.polygon_vertex_count.set(4);
    crate::scene::glow_render_states::glow_render_states();
    crate::scene::set_texture::set_texture(k(0x5D1DE8));
    crate::d3d::draw_fan_unclipped::draw_fan_unclipped();
    crate::scene::depth_buffer_on::depth_buffer(true);
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock, same};

    // The original blocks, verbatim.
    pub(super) fn int_and_one_asm(i: u32, add: bool) -> u32 {
        let mut out = 0u32;
        let v = [i, NUMBERS.one.bits()];
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
        let _l = lock();
        let _saved = Saved::new(&[0x5C5D38]);
        let mut r = Rng(5);
        at_each_control_word(|| {
            for _ in 0..20000 {
                NUMBERS.one.set_bits(r.float());
                let (i, a, b) = (r.int() as u32, r.float(), r.float());
                for add in [false, true] {
                    let (want, got) = (int_and_one_asm(i, add), int_and_one(i, add));
                    assert!(same(want, got), "{i} {add}: {want:#x} {got:#x}");
                }
                let (want, got) = (int_over_two_asm(i, a, b), int_over_two(i, a, b));
                assert!(same(want, got), "{i} {a:#x} {b:#x}: {want:#x} {got:#x}");
            }
        });
    }
}
