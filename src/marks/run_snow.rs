//! `marks::run_snow` (0x4C25F6): the snow's frame. `dword_8DB4C4` the settings' +0x58; the
//! light's place (`flt_8DB50C` ..) turned by the view's place (`flt_5EA3CC`) and the view
//! (`g_viewMatrix`) into `flt_8DB430` .., its way (`flt_8DB518` ..) turned without the
//! places into `flt_8DB43C` ..; `marks::make_fade_colours`. `dword_8D21A4` the settings'
//! +0x2760 (the flakes; none: `dword_8D215C` 0 and back), held to 0x800; the settings'
//! +0x28 .. +0x48 into `dword_8D216C` .., +0x1C .. +0x24 into `dword_8D2160` ..;
//! `marks::place_view_2`, `scene::set_texture` (`dword_5D1E10`), `scene::depth_buffer_on`,
//! `scene::depth_test_off`, render states 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1;
//! the four vertices' rhw `dword_8DB524`, colour `g_viewLight` | 0xFF000000, specular 0,
//! texture places from `dword_8DB528` ..; `g_polygonVertexCount` 4; 1 over `flt_6241C8`
//! into `flt_8D2110` .. and `flt_8D21F8`; when the settings' +0x10 is set the fall
//! `dword_8D2194` gains the clock's step (since `dword_8D2150`) times `flt_8DB538`
//! (`fistp`), kept to 28 bits, `dword_8D2150` the clock; the turn's nine floats and the
//! focal length each four times into `xmmword_8D2030` ..; `marks::draw_snow`;
//! `scene::depth_test_on`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_frame` (0x4D166B). Its calls through `part` (1 `make_fade_colours`,
//! 2 `place_view_2`, 3 the texture, 4 the depth buffer, 5 the depth test off, 6 a render
//! state, 7 `draw_snow`, 8 the depth test on); in the game their Rust versions. The two
//! turns are the original's instructions over the game's variables. Refuses null
//! settings, before anything. Shadow mode runs the original alone (it draws). Self-test:
//! a copy whose calls go to stand-ins, against the Rust version, on settings of the
//! test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::x87;

const KEPT_RVA: usize = rva(0x8DB4C4); // dword_8DB4C4
const FLAKES_RVA: usize = rva(0x8D21A4); // dword_8D21A4
const NONE_RVA: usize = rva(0x8D215C); // dword_8D215C
const MOST: i32 = 0x800;
const COPIES_A: usize = rva(0x8D216C); // dword_8D216C .. : the settings' +0x28 ..
const COPIES_B: usize = rva(0x8D2160); // dword_8D2160 .. : the settings' +0x1C ..
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const RHW_RVA: usize = rva(0x8DB524); // dword_8DB524
const LIGHT_RVA: usize = rva(0x6415E4); // g_viewLight
const UV_RVA: usize = rva(0x8DB528); // dword_8DB528 .. dword_8DB534
const DEPTH_OF_RVA: usize = rva(0x6241C8); // flt_6241C8
const INVERSES: [usize; 5] = [
    rva(0x8D2110),
    rva(0x8D2114),
    rva(0x8D2118),
    rva(0x8D211C),
    rva(0x8D21F8),
];
const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
const LAST_CLOCK_RVA: usize = rva(0x8D2150); // dword_8D2150
const FALL_RVA: usize = rva(0x8D2194); // dword_8D2194
const FALL_K_RVA: usize = rva(0x8DB538); // flt_8DB538
const TURN_RVA: usize = rva(0x8D2220); // dword_8D2220 .. dword_8D2240
const WIDE_RVA: usize = rva(0x8D2030); // xmmword_8D2030 .. xmmword_8D20B0
const FOCAL_RVA: usize = rva(0x624088); // flt_624088
const WIDE_FOCAL_RVA: usize = rva(0x8D20F0); // xmmword_8D20F0
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The original's 0x4C2605 .. 0x4C276E: the light's place and way turned.
unsafe fn turn_light(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4DB50C]",
            "fld dword ptr [{b} + 0x4DB510]",
            "fld dword ptr [{b} + 0x4DB514]",
            "fld dword ptr [{b} + 0x1EA3CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{b} + 0x1EA3DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{b} + 0x1EA3EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA3FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA3D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{b} + 0x1EA3E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{b} + 0x1EA3F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA400]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA3D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{b} + 0x1EA3E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{b} + 0x1EA3F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA404]",
            ".byte 0xD9, 0xCA",
            "fld dword ptr [{b} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{b} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{b} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{b} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{b} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{b} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{b} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{b} + 0x4DB430]",
            "fstp dword ptr [{b} + 0x4DB434]",
            "fstp dword ptr [{b} + 0x4DB438]",
            "fld dword ptr [{b} + 0x4DB518]",
            "fld dword ptr [{b} + 0x4DB51C]",
            "fld dword ptr [{b} + 0x4DB520]",
            "fld dword ptr [{b} + 0x1EA3CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{b} + 0x1EA3DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{b} + 0x1EA3EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA3D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{b} + 0x1EA3E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{b} + 0x1EA3F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA3D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{b} + 0x1EA3E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{b} + 0x1EA3F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            "fld dword ptr [{b} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{b} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{b} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{b} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{b} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{b} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{b} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{b} + 0x4DB43C]",
            "fstp dword ptr [{b} + 0x4DB440]",
            "fstp dword ptr [{b} + 0x4DB444]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn run_snow() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        game_write(KEPT_RVA, get(s + 0x58));
        turn_light(game_addr(0) as u32);
        crate::marks::make_fade_colours::make_fade_colours();
        let s: u32 = game_read(SETTINGS_RVA);
        let mut flakes = get(s + 0x2760) as i32;
        game_write(FLAKES_RVA, flakes);
        if flakes == 0 {
            game_write(NONE_RVA, 0u32);
            return Ok(());
        }
        if flakes > MOST {
            flakes = MOST;
            game_write(FLAKES_RVA, flakes);
        }
        for n in 0..9 {
            game_write(COPIES_A + 4 * n, get(s + 0x28 + 4 * n as u32));
        }
        for n in 0..3 {
            game_write(COPIES_B + 4 * n, get(s + 0x1C + 4 * n as u32));
        }
        crate::marks::place_view_2::place_view_2()?;
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_off::depth_test_off();
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        let rhw: u32 = game_read(RHW_RVA);
        let light = game_read::<u32>(LIGHT_RVA) | 0xFF00_0000;
        let [u0, v0, u1, v1]: [u32; 4] = game_read(UV_RVA);
        let uv = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)];
        for (n, (u, v)) in uv.into_iter().enumerate() {
            let at = VERTICES_RVA + 0x20 * n;
            game_write(at + 0xC, rhw);
            game_write(at + 0x10, light);
            game_write(at + 0x14, 0u32);
            game_write(at + 0x18, u);
            game_write(at + 0x1C, v);
        }
        game_write(VERTEX_COUNT_RVA, 4u32);
        let inverse = x87::reciprocal(game_read(DEPTH_OF_RVA));
        for r in INVERSES {
            game_write(r, inverse);
        }
        let s: u32 = game_read(SETTINGS_RVA);
        if get(s + 0x10) != 0 {
            let ticks = game_read::<u32>(CLOCK_RVA).wrapping_sub(game_read(LAST_CLOCK_RVA));
            let fall = x87::int_times_plus_int_to_int(
                ticks as i32,
                game_read(FALL_K_RVA),
                game_read(FALL_RVA),
            );
            game_write(FALL_RVA, fall);
        }
        game_write(FALL_RVA, game_read::<u32>(FALL_RVA) & 0x0FFF_FFFF);
        game_write(LAST_CLOCK_RVA, game_read::<u32>(CLOCK_RVA));
        let turn: [u32; 9] = game_read(TURN_RVA);
        for (n, v) in turn.into_iter().enumerate() {
            for m in 0..4 {
                game_write(WIDE_RVA + 0x10 * n + 4 * m, v);
            }
        }
        let focal: u32 = game_read(FOCAL_RVA);
        for m in 0..4 {
            game_write(WIDE_FOCAL_RVA + 4 * m, focal);
        }
        crate::marks::draw_snow::draw_snow()?;
        crate::scene::depth_test_on::depth_test_on();
    }
    Ok(())
}
