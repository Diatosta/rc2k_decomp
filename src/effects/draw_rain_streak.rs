//! `effects::draw_rain_streak` (0x4C1DCD): a streak (a point and where it was) drawn as a quad
//! facing the viewer. The point (st(0) .. st(2), scaled by `flt_8BDD65` .. into
//! `flt_8BDE49` ..) turned by the matrix at `dword_8BDD89` .. plus the whole numbers at
//! `dword_8BDD71` .. into `flt_8BDE3D` .. (its depth also `flt_8BDD5D`); the point less
//! `dword_8BC47D` .. turned the same into `flt_8BDE49` ... Unless the first depth is at or
//! below 0x20000 or the second at or below 0x48000000 (signed compares of the bits), or
//! `dword_8BC405` is 0: both projected (`flt_624088` over the depth; scales `flt_8C5FE5`,
//! `flt_8C5FE9`, centre `flt_62409C`, `flt_6240A0`) into `flt_8BDD41` .., the half width
//! `flt_8C5FED` over the screen length across the line, the four vertices' x, y and depth
//! (times `flt_8BDD61`), `effects::clip_quad`, and unless all is outside `scene::draw_fan`
//! and `g_polygonVertexCount` 4. Rain, by the look of it (a reading).
//!
//! In:  st(0) .. st(2) the point (popped)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C16F9` (0x4C1AD0). The arithmetic is plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables. Refuses null marks settings (`clip_quad`'s
//! refusal), before anything.

use crate::d3d::vertices_store::VERTICES;
use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::rain::{RainRules, RainState, Weather};
use crate::errors::{BadInput, Refused};

/// The original's 0x4C1DCD .. 0x4C1F11: the point scaled and turned, and the last place.
fn turn(weather: &Weather, p: &[[u8; 10]; 3]) {
    turn_x87(p, &weather.rain);
}

/// The original's 0x4C1F4A .. 0x4C20B3: both projected and the quad's x, y and depth.
fn project(weather: &Weather) {
    project_x87(&weather.rain, &weather.rain_rules);
}

/// The Rust version (see the header).
pub fn draw_rain_streak(weather: &Weather, place: [[u8; 10]; 3]) -> Result<(), Refused> {
    let rain = &weather.rain;
    {
        if crate::effects::owner::effects().setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        turn(weather, &place);
        if rain.depth.bits() as i32 <= 0x2_0000
            || rain.drawn.get() == 0
            || rain.second[2].bits() as i32 <= 0x4800_0000
        {
            return Ok(());
        }
        project(weather);
        if clip_quad()? {
            return Ok(());
        }
        draw_fan();
        crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_vertex_count
            .set_bits(4u32);
    }
    Ok(())
}

use crate::effects::plain::st_cell;
use crate::math::x87_float as bh;

/// `turn`'s x87 block as plain Rust on the x87 model (79 instructions), same order and memory places.
fn turn_x87(p: &[[u8; 10]; 3], r: &RainState) {
    let t0 = bh::from_f80(p[2]);
    let t1 = bh::from_f80(p[1]);
    let t2 = bh::from_f80(p[0]);
    let t3 = t0 * bh::ld(r.point_scale[0].bits());
    r.second[0].set_bits(bh::st(t3));
    let t4 = t1 * bh::ld(r.point_scale[1].bits());
    r.second[1].set_bits(bh::st(t4));
    let t5 = t2 * bh::ld(r.point_scale[2].bits());
    r.second[2].set_bits(bh::st(t5));
    let t6 = t3 * bh::ld(r.matrix_copy[0].bits());
    let t7 = t4 * bh::ld(r.matrix_copy[3].bits());
    let t8 = t5 * bh::ld(r.matrix_copy[6].bits());
    let t9 = t7 + t8;
    let t10 = t6 + t9;
    let t11 = t10 + bh::int(r.point[0].get() as i32);
    r.turned[0].set_bits(bh::st(t11));
    let t12 = t3 * bh::ld(r.matrix_copy[1].bits());
    let t13 = t4 * bh::ld(r.matrix_copy[4].bits());
    let t14 = t5 * bh::ld(r.matrix_copy[7].bits());
    let t15 = t13 + t14;
    let t16 = t12 + t15;
    let t17 = t16 + bh::int(r.point[1].get() as i32);
    r.turned[1].set_bits(bh::st(t17));
    let t18 = t3 * bh::ld(r.matrix_copy[2].bits());
    let t19 = t4 * bh::ld(r.matrix_copy[5].bits());
    let t20 = t5 * bh::ld(r.matrix_copy[8].bits());
    let t21 = t19 + t20;
    let t22 = t18 + t21;
    let t23 = t22 + bh::int(r.point[2].get() as i32);
    r.depth.set_bits(bh::st(t23));
    r.turned[2].set_bits(bh::st(t23));
    let t24 = bh::ld(r.second[1].bits());
    let t25 = t24 - bh::ld(r.motion[1].bits());
    let t26 = bh::ld(r.second[0].bits());
    let t27 = t26 - bh::ld(r.motion[0].bits());
    let t28 = bh::ld(r.second[2].bits());
    let t29 = t28 - bh::ld(r.motion[2].bits());
    let t30 = t27 * bh::ld(r.matrix_copy[0].bits());
    let t31 = t25 * bh::ld(r.matrix_copy[3].bits());
    let t32 = t29 * bh::ld(r.matrix_copy[6].bits());
    let t33 = t31 + t32;
    let t34 = t30 + t33;
    let t35 = t34 + bh::int(r.point[0].get() as i32);
    r.second[0].set_bits(bh::st(t35));
    let t36 = t27 * bh::ld(r.matrix_copy[1].bits());
    let t37 = t25 * bh::ld(r.matrix_copy[4].bits());
    let t38 = t29 * bh::ld(r.matrix_copy[7].bits());
    let t39 = t37 + t38;
    let t40 = t36 + t39;
    let t41 = t40 + bh::int(r.point[1].get() as i32);
    r.second[1].set_bits(bh::st(t41));
    let t42 = t27 * bh::ld(r.matrix_copy[2].bits());
    let t43 = t25 * bh::ld(r.matrix_copy[5].bits());
    let t44 = t29 * bh::ld(r.matrix_copy[8].bits());
    let t45 = t43 + t44;
    let t46 = t42 + t45;
    let t47 = t46 + bh::int(r.point[2].get() as i32);
    r.second[2].set_bits(bh::st(t47));
}

/// `project`'s x87 block as plain Rust on the x87 model (85 instructions), same order and memory places.
fn project_x87(r: &RainState, k: &RainRules) {
    let t0 = bh::ld(r.second[2].bits());
    let t1 = bh::ld(crate::scene::owner::scene().drawing.scene.view_scale.bits()) / t0;
    let t2 = bh::ld(r.turned[2].bits());
    let t3 = bh::ld(crate::scene::owner::scene().drawing.scene.view_scale.bits()) / t2;
    let t4 = bh::ld(r.second[0].bits());
    let t5 = bh::ld(r.second[1].bits());
    let t6 = bh::ld(r.turned[0].bits());
    let t7 = bh::ld(r.turned[1].bits());
    let t8 = t4 * t1;
    let t9 = t5 * t1;
    let t10 = t6 * t3;
    let t11 = t7 * t3;
    let t12 = t1 * bh::ld(k.view_scale[0].to_bits());
    let t13 = t3 * bh::ld(k.view_scale[1].to_bits());
    let t14 = t8 + bh::ld(crate::scene::owner::scene().drawing.scene.view_centre[0].bits());
    r.ends[0][0].set_bits(bh::st(t14));
    let t15 = -t9;
    let t16 = t15 + bh::ld(crate::scene::owner::scene().drawing.scene.view_centre[1].bits());
    r.ends[0][1].set_bits(bh::st(t16));
    let t17 = t10 + bh::ld(crate::scene::owner::scene().drawing.scene.view_centre[0].bits());
    r.ends[1][0].set_bits(bh::st(t17));
    let t18 = -t11;
    let t19 = t18 + bh::ld(crate::scene::owner::scene().drawing.scene.view_centre[1].bits());
    r.ends[1][1].set_bits(bh::st(t19));
    r.ends[0][2].set_bits(bh::st(t12));
    r.ends[1][2].set_bits(bh::st(t13));
    let t20 = bh::ONE;
    let t21 = t20 / bh::ld(crate::scene::owner::scene().drawing.scene.farthest.bits());
    let t22 = bh::ld(r.ends[1][0].bits());
    let t23 = t22 - bh::ld(r.ends[0][0].bits());
    r.edge_a[1].set_bits(bh::st(t23));
    let t24 = t23 * t23;
    let t25 = bh::ld(r.ends[1][1].bits());
    let t26 = t25 - bh::ld(r.ends[0][1].bits());
    let t27 = -t26;
    r.edge_a[0].set_bits(bh::st(t27));
    let t28 = t27 * t27;
    let t29 = t24 + t28;
    let t30 = t29.sqrt();
    let t31 = bh::ld(k.half_width.to_bits()) / t30;
    let t32 = t31 * bh::ld(r.edge_a[0].bits());
    let t33 = t32 * bh::ld(r.ends[0][2].bits());
    r.edge_a[0].set_bits(bh::st(t33));
    let t34 = t32 * bh::ld(r.ends[1][2].bits());
    r.edge_b[0].set_bits(bh::st(t34));
    let t35 = t31 * bh::ld(r.edge_a[1].bits());
    let t36 = t35 * bh::ld(r.ends[0][2].bits());
    r.edge_a[1].set_bits(bh::st(t36));
    let t37 = t35 * bh::ld(r.ends[1][2].bits());
    r.edge_b[1].set_bits(bh::st(t37));
    r.depth_scale.set_bits(bh::st(t21));
    let t38 = bh::ld(r.second[2].bits());
    let t39 = t38 * bh::ld(r.depth_scale.bits());
    st_cell(&VERTICES.vertices[0].z, t39);
    st_cell(&VERTICES.vertices[1].z, t39);
    st_cell(&VERTICES.vertices[2].z, t39);
    st_cell(&VERTICES.vertices[3].z, t39);
    let t40 = bh::ld(r.ends[0][0].bits());
    st_cell(&VERTICES.vertices[0].x, t40);
    let t41 = t40 - bh::ld(r.edge_a[0].bits());
    st_cell(&VERTICES.vertices[1].x, t41);
    let t42 = bh::ld(r.ends[1][0].bits());
    st_cell(&VERTICES.vertices[3].x, t42);
    let t43 = t42 - bh::ld(r.edge_b[0].bits());
    st_cell(&VERTICES.vertices[2].x, t43);
    let t44 = bh::ld(r.ends[0][1].bits());
    st_cell(&VERTICES.vertices[0].y, t44);
    let t45 = t44 - bh::ld(r.edge_a[1].bits());
    st_cell(&VERTICES.vertices[1].y, t45);
    let t46 = bh::ld(r.ends[1][1].bits());
    st_cell(&VERTICES.vertices[3].y, t46);
    let t47 = t46 - bh::ld(r.edge_b[1].bits());
    st_cell(&VERTICES.vertices[2].y, t47);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn turn_x87(mk: usize, p: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{p} + 20]",
            "fld tbyte ptr [{p} + 10]",
            "fld tbyte ptr [{p}]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E25]",
            "fst dword ptr [{mk} + 0x158F09]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E29]",
            "fst dword ptr [{mk} + 0x158F0D]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E2D]",
            "fst dword ptr [{mk} + 0x158F11]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E49]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E55]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E61]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E31]",
            "fstp dword ptr [{mk} + 0x158EFD]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E4D]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E59]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E65]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E35]",
            "fstp dword ptr [{mk} + 0x158F01]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E51]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E5D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E69]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E39]",
            "fst dword ptr [{mk} + 0x158E1D]",
            "fstp dword ptr [{mk} + 0x158F05]",
            "fld dword ptr [{mk} + 0x158F0D]",
            "fsub dword ptr [{mk} + 0x157541]",
            "fld dword ptr [{mk} + 0x158F09]",
            "fsub dword ptr [{mk} + 0x15753D]",
            "fld dword ptr [{mk} + 0x158F11]",
            "fsub dword ptr [{mk} + 0x157545]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E49]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E55]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E61]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E31]",
            "fstp dword ptr [{mk} + 0x158F09]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x158E4D]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x158E59]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x158E65]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E35]",
            "fstp dword ptr [{mk} + 0x158F0D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E51]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x158E5D]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x158E69]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x158E39]",
            "fstp dword ptr [{mk} + 0x158F11]",
            mk = in(reg) mk,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn project_x87(rc: usize, bf: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x158F11]",
            "fdivr dword ptr [{rc} + 0x224088]",
            "fld dword ptr [{mk} + 0x158F05]",
            "fdivr dword ptr [{rc} + 0x224088]",
            "fld dword ptr [{mk} + 0x158F09]",
            "fld dword ptr [{mk} + 0x158F0D]",
            "fld dword ptr [{mk} + 0x158EFD]",
            "fld dword ptr [{mk} + 0x158F01]",
            ".byte 0xD9, 0xCB",
            ".byte 0xD8, 0xCD",
            ".byte 0xD9, 0xCA",
            ".byte 0xD8, 0xCD",
            ".byte 0xD9, 0xC9",
            ".byte 0xD8, 0xCC",
            ".byte 0xD9, 0xCB",
            ".byte 0xD8, 0xCC",
            ".byte 0xD9, 0xCD",
            "fmul dword ptr [{mk} + 0x1610A5]",
            ".byte 0xD9, 0xCC",
            "fmul dword ptr [{mk} + 0x1610A9]",
            ".byte 0xD9, 0xCA",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x158E01]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x158E05]",
            ".byte 0xD9, 0xC9",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x158E0D]",
            ".byte 0xD9, 0xCA",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x158E11]",
            "fstp dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158E15]",
            ".byte 0xD9, 0xE8",
            "fdiv dword ptr [{rc} + 0x2241C8]",
            "fld dword ptr [{mk} + 0x158E0D]",
            "fsub dword ptr [{mk} + 0x158E01]",
            "fst dword ptr [{mk} + 0x158DED]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x158E11]",
            "fsub dword ptr [{mk} + 0x158E05]",
            ".byte 0xD9, 0xE0",
            "fst dword ptr [{mk} + 0x158DE9]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x1610AD]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158DE9]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158DE9]",
            "fmul dword ptr [{mk} + 0x158E15]",
            "fstp dword ptr [{mk} + 0x158DF5]",
            "fmul dword ptr [{mk} + 0x158DED]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x158E09]",
            "fstp dword ptr [{mk} + 0x158DED]",
            "fmul dword ptr [{mk} + 0x158E15]",
            "fstp dword ptr [{mk} + 0x158DF9]",
            "fstp dword ptr [{mk} + 0x158E21]",
            "fld dword ptr [{mk} + 0x158F11]",
            "fmul dword ptr [{mk} + 0x158E21]",
            "fst dword ptr [{bf} + 0x5605A8]",
            "fst dword ptr [{bf} + 0x5605C8]",
            "fst dword ptr [{bf} + 0x5605E8]",
            "fstp dword ptr [{bf} + 0x560608]",
            "fld dword ptr [{mk} + 0x158E01]",
            "fst dword ptr [{bf} + 0x5605A0]",
            "fsub dword ptr [{mk} + 0x158DE9]",
            "fstp dword ptr [{bf} + 0x5605C0]",
            "fld dword ptr [{mk} + 0x158E0D]",
            "fst dword ptr [{bf} + 0x560600]",
            "fsub dword ptr [{mk} + 0x158DF5]",
            "fstp dword ptr [{bf} + 0x5605E0]",
            "fld dword ptr [{mk} + 0x158E05]",
            "fst dword ptr [{bf} + 0x5605A4]",
            "fsub dword ptr [{mk} + 0x158DED]",
            "fstp dword ptr [{bf} + 0x5605C4]",
            "fld dword ptr [{mk} + 0x158E11]",
            "fst dword ptr [{bf} + 0x560604]",
            "fsub dword ptr [{mk} + 0x158DF9]",
            "fstp dword ptr [{bf} + 0x5605E4]",
            rc = in(reg) rc,
            bf = in(reg) bf,
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::d3d::vertices_store::VERTICES;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::rain::tests::with_state_at;
    use crate::session::Float;

    /// The vertex float at byte `off` of the vertices (x, y or z of a vertex).
    fn vertex_cell(off: usize) -> &'static Float {
        let v = &VERTICES.vertices[off / 32];
        match off % 32 {
            0x0 => &v.x,
            0x4 => &v.y,
            0x8 => &v.z,
            _ => panic!("not a tested vertex field"),
        }
    }

    /// Runs `f` with the vertex floats at `offsets` holding the test buffer's (at `at`)
    /// bytes; what `f` leaves goes back to the buffer, the vertices as they were.
    fn staged<R>(at: usize, offsets: &[usize], f: impl FnOnce() -> R) -> R {
        let host = |off: usize| crate::pointers::host_at(at + off).cast::<u32>();
        let saved: Vec<u32> = offsets.iter().map(|&o| vertex_cell(o).bits()).collect();
        for &o in offsets {
            vertex_cell(o).set_bits(unsafe { host(o).read_unaligned() });
        }
        let out = f();
        for (&o, &old) in offsets.iter().zip(&saved) {
            unsafe { host(o).write_unaligned(vertex_cell(o).bits()) };
            vertex_cell(o).set_bits(old);
        }
        out
    }

    /// The three ten-byte values at `at` of the test buffer.
    fn values(at: usize) -> [[u8; 10]; 3] {
        std::array::from_fn(|n| {
            std::array::from_fn(|i| unsafe { crate::pointers::host_at(at + 10 * n + i).read() })
        })
    }

    #[test]
    fn turn_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1574C5,
                    hi: 0x1610BD,
                    reads: &[
                        (0x15753D, F32),
                        (0x157541, F32),
                        (0x157545, F32),
                        (0x158E25, F32),
                        (0x158E29, F32),
                        (0x158E2D, F32),
                        (0x158E31, I32),
                        (0x158E35, I32),
                        (0x158E39, I32),
                        (0x158E49, F32),
                        (0x158E4D, F32),
                        (0x158E51, F32),
                        (0x158E55, F32),
                        (0x158E59, F32),
                        (0x158E5D, F32),
                        (0x158E61, F32),
                        (0x158E65, F32),
                        (0x158E69, F32),
                        (0x158F09, F32),
                        (0x158F0D, F32),
                        (0x158F11, F32),
                    ],
                    writes: &[
                        (0x158E1D, F32),
                        (0x158EFD, F32),
                        (0x158F01, F32),
                        (0x158F05, F32),
                        (0x158F09, F32),
                        (0x158F0D, F32),
                        (0x158F11, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x1E,
                    reads: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::turn_x87(b[0], b[1]),
            |b| with_state_at(b[0], |r, _| super::turn_x87(&values(b[1]), r)),
        );
    }

    #[test]
    fn project_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[
                        (0x224088, F32),
                        (0x22409C, F32),
                        (0x2240A0, F32),
                        (0x2241C8, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x5605A0,
                    hi: 0x56060C,
                    reads: &[],
                    writes: &[
                        (0x5605A0, F32),
                        (0x5605A4, F32),
                        (0x5605A8, F32),
                        (0x5605C0, F32),
                        (0x5605C4, F32),
                        (0x5605C8, F32),
                        (0x5605E0, F32),
                        (0x5605E4, F32),
                        (0x5605E8, F32),
                        (0x560600, F32),
                        (0x560604, F32),
                        (0x560608, F32),
                    ],
                },
                Op::Base {
                    lo: 0x1574C5,
                    hi: 0x1610BD,
                    reads: &[
                        (0x158DE9, F32),
                        (0x158DED, F32),
                        (0x158DF5, F32),
                        (0x158DF9, F32),
                        (0x158E01, F32),
                        (0x158E05, F32),
                        (0x158E09, F32),
                        (0x158E0D, F32),
                        (0x158E11, F32),
                        (0x158E15, F32),
                        (0x158E21, F32),
                        (0x158EFD, F32),
                        (0x158F01, F32),
                        (0x158F05, F32),
                        (0x158F09, F32),
                        (0x158F0D, F32),
                        (0x158F11, F32),
                        (0x1610A5, F32),
                        (0x1610A9, F32),
                        (0x1610AD, F32),
                    ],
                    writes: &[
                        (0x158DE9, F32),
                        (0x158DED, F32),
                        (0x158DF5, F32),
                        (0x158DF9, F32),
                        (0x158E01, F32),
                        (0x158E05, F32),
                        (0x158E09, F32),
                        (0x158E0D, F32),
                        (0x158E11, F32),
                        (0x158E15, F32),
                        (0x158E21, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::project_x87(b[0], b[1], b[2]),
            |b| {
                with_state_at(b[2], |r, k| {
                    staged(
                        b[1].wrapping_add(0x5605A0),
                        &[
                            0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                        ],
                        || super::project_x87(r, k),
                    );
                });
            },
        );
    }
}
