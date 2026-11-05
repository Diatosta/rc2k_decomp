//! The tyre-change screen's drawing (stream B, 2026-10-18): `drive::tyre_change_screen`
//! (0x445047), `drive::tyre_screen` (0x41C5FA), `drive::tyre_screen_parts` (0x41C790),
//! `drive::tyre_turned_quad` (0x41C969), `drive::tyre_quad` (0x41DD3B). Their calls by
//! place: 0 `d3d::set_texture` (0x41ACF5), 1 `d3d::set_render_state` (0x41AB5B), 2
//! `tyre_quad`, 3 `tyre_turned_quad`, 4 `tyre_screen_parts`, 5 the device's DrawPrimitive
//! (`sub_41ADE1`: platform, a call into the game), 6 `tyre_screen`. In the game the Rust
//! versions and the game's three.
//!
//! This file: the places, the quads' shared steps and the tests' frame.

use crate::d3d::vertices_store::{VERTICES, Vertex};
use crate::errors::Refused;

use crate::math::x87_float::{int, ld, st};
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::session::{Dword, MACHINE, SCREEN};

/// `d3d::set_texture` (0x41ACF5): eax the texture.
pub fn set_texture(texture: u32) {
    crate::d3d::set_texture::set_texture(texture);
}

/// `d3d::set_render_state` (0x41AB5B): eax the value, ebx the state.
pub fn set_render_state(value: u32, state: u32) {
    crate::d3d::set_render_state::set_render_state(value, state);
}

/// The quads' fixed words (0x41DD68 .. / 0x41CA49 ..): z 0, rhw 1.0, the colour, the
/// specular 0.
pub fn quad_words(colour: u32) {
    for k in 0..4 {
        let v = &VERTICES.vertices[k];
        v.z.set_bits(0);
        v.rhw.set_bits(0x3F80_0000);
        v.colour.set(colour);
        v.specular.set(0);
    }
}

/// The quads' shared tail (0x41DDA1 ..): the texture places from the rectangle `uv`
/// (x, y, w, h, whole numbers) x `flt_5C2C64`, and the places scaled to the screen
/// (`g_screenWidth` x `flt_5C2C68`, `g_screenHeight` x `flt_5C2C6C`, through
/// `dword_5183CC`).
pub fn quad_tail(uv: [u32; 4]) {
    quad_tail_math(
        uv,
        &VERTICES.vertices[..4],
        [SCREEN.width.get(), SCREEN.height.get()],
        &MACHINE.scratch,
    );
}

/// The tail's arithmetic: `s` the texture rectangle (whole numbers), `v` the four vertices,
/// `sc` the screen size (whole numbers), `t` a scratch float (the scale
/// `VIEWER_PANEL.texel_scale`).
fn quad_tail_math(s: [u32; 4], v: &[Vertex], sc: [u32; 2], t: &Dword) {
    let k = ld(VIEWER_PANEL.texel_scale.bits());
    let whole = |n: usize| int(s[n] as i32);
    let left = whole(0) * k;
    v[0].u.set_bits(st(left));
    v[2].u.set_bits(st(left));
    let right = (whole(0) + whole(2)) * ld(VIEWER_PANEL.texel_scale.bits());
    v[1].u.set_bits(st(right));
    v[3].u.set_bits(st(right));
    let top = whole(1) * ld(VIEWER_PANEL.texel_scale.bits());
    v[0].v.set_bits(st(top));
    v[1].v.set_bits(st(top));
    let bottom = (whole(1) + whole(3)) * ld(VIEWER_PANEL.texel_scale.bits());
    v[2].v.set_bits(st(bottom));
    v[3].v.set_bits(st(bottom));
    t.set_bits(st(
        int(sc[0] as i32) * ld(VIEWER_PANEL.texel_scale_small.bits())
    ));
    for vertex in &v[..4] {
        vertex.x.set_bits(st(ld(vertex.x.bits()) * ld(t.bits())));
    }
    t.set_bits(st(
        int(sc[1] as i32) * ld(VIEWER_PANEL.texel_scale_smaller.bits())
    ));
    for vertex in &v[..4] {
        vertex.y.set_bits(st(ld(vertex.y.bits()) * ld(t.bits())));
    }
}

/// The draw (`d3d::draw_vertex_strip`, 0x41ADE1) of 4 vertices at `g_polygonVertices`;
/// its result (Direct3D's, always 0) is read by no one: the quads keep every register
/// round it (`pusha` / `popa`).
pub fn draw() -> Result<(), Refused> {
    crate::d3d::draw_vertex_strip::draw_vertex_strip(
        crate::pointers::GamePtr::to(&VERTICES.vertices),
        4,
    )?;
    Ok(())
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions.
    reference!(ref_tail(s, v, sc, t, rc)
        "fild dword ptr [{s}]",
        "fmul dword ptr [{rc} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x18]",
        "fstp dword ptr [{v} + 0x58]",
        "fild dword ptr [{s}]",
        "fiadd dword ptr [{s} + 0x8]",
        "fmul dword ptr [{rc} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x38]",
        "fstp dword ptr [{v} + 0x78]",
        "fild dword ptr [{s} + 0x4]",
        "fmul dword ptr [{rc} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x1C]",
        "fstp dword ptr [{v} + 0x3C]",
        "fild dword ptr [{s} + 0x4]",
        "fiadd dword ptr [{s} + 0xC]",
        "fmul dword ptr [{rc} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x5C]",
        "fstp dword ptr [{v} + 0x7C]",
        "fild dword ptr [{sc}]",
        "fmul dword ptr [{rc} + 0x1C2C68]",
        "fstp dword ptr [{t}]",
        "fld dword ptr [{v}]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v}]",
        "fld dword ptr [{v} + 0x20]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x20]",
        "fld dword ptr [{v} + 0x40]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x40]",
        "fld dword ptr [{v} + 0x60]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x60]",
        "fild dword ptr [{sc} + 0x4]",
        "fmul dword ptr [{rc} + 0x1C2C6C]",
        "fstp dword ptr [{t}]",
        "fld dword ptr [{v} + 0x4]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x4]",
        "fld dword ptr [{v} + 0x24]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x24]",
        "fld dword ptr [{v} + 0x44]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x44]",
        "fld dword ptr [{v} + 0x64]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x64]");

    #[test]
    fn quad_tail_matches_the_assembly() {
        let blocks = vec![Block {
            name: "quad_tail",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I), (8, In::I), (0xC, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[
                        (0, In::F),
                        (0x20, In::F),
                        (0x40, In::F),
                        (0x60, In::F),
                        (4, In::F),
                        (0x24, In::F),
                        (0x44, In::F),
                        (0x64, In::F),
                    ],
                    writes: &[
                        (0, Out::F),
                        (0x20, Out::F),
                        (0x40, Out::F),
                        (0x60, Out::F),
                        (4, Out::F),
                        (0x24, Out::F),
                        (0x44, Out::F),
                        (0x64, Out::F),
                        (0x18, Out::F),
                        (0x58, Out::F),
                        (0x38, Out::F),
                        (0x78, Out::F),
                        (0x1C, Out::F),
                        (0x3C, Out::F),
                        (0x5C, Out::F),
                        (0x7C, Out::F),
                    ],
                },
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C2C64, In::F), (0x1C2C68, In::F), (0x1C2C6C, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_tail(a[0], a[1], a[2], a[3], a[4]) },
            plain: |a| {
                // SAFETY: the buffers are the operands' (a rectangle's four dwords, four
                // vertices, the screen's two, the scratch dword), lent and aligned.
                let (rect, verts, screen, scratch) = unsafe {
                    (
                        *(crate::pointers::host_at(a[0]) as *const [u32; 4]),
                        &*(crate::pointers::host_at(a[1]) as *const [Vertex; 4]),
                        *(crate::pointers::host_at(a[2]) as *const [u32; 2]),
                        &*(crate::pointers::host_at(a[3]) as *const Dword),
                    )
                };
                quad_tail_math(rect, verts, screen, scratch);
            },
        }];
        sparse(&blocks, 4000, 205);
    }
}
