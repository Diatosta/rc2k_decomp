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
//! Out: the registers as `depth_test_on` leaves them
//!
//! Caller: `marks::run_frame` (0x4D166B). Its calls through `part` (1 `make_fade_colours`,
//! 2 `place_view_2`, 3 the texture, 4 the depth buffer, 5 the depth test off, 6 a render
//! state, 7 `draw_snow`, 8 the depth test on); in the game their Rust versions. The two
//! turns are the original's instructions over the game's variables. Refuses null
//! settings, before anything. Shadow mode runs the original alone (it draws). Self-test:
//! a copy whose calls go to stand-ins, against the Rust version, on settings of the
//! test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::x87;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::run_snow", rva(0x4C25F6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

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

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_snow(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        g.ebp = s;
        g.eax = get(s + 0x58);
        game_write(KEPT_RVA, g.eax);
        g.esi = game_addr(rva(0x5EA2CC)) as u32;
        turn_light(game_addr(0) as u32);
        part(1, &mut g)?;
        let s: u32 = game_read(SETTINGS_RVA);
        g.esi = s;
        let mut flakes = get(s + 0x2760) as i32;
        game_write(FLAKES_RVA, flakes);
        if flakes == 0 {
            game_write(NONE_RVA, 0u32);
            return Ok(g);
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
        part(2, &mut g)?;
        g.eax = game_read(TEXTURE_RVA);
        part(3, &mut g)?;
        part(4, &mut g)?;
        part(5, &mut g)?;
        for (value, state) in STATES {
            g.eax = value;
            g.ebx = state;
            part(6, &mut g)?;
        }
        g.edi = game_addr(VERTICES_RVA) as u32;
        g.eax = game_read(RHW_RVA);
        let light = game_read::<u32>(LIGHT_RVA) | 0xFF00_0000;
        let [u0, v0, u1, v1]: [u32; 4] = game_read(UV_RVA);
        let uv = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)];
        for (n, (u, v)) in uv.into_iter().enumerate() {
            let at = VERTICES_RVA + 0x20 * n;
            game_write(at + 0xC, g.eax);
            game_write(at + 0x10, light);
            game_write(at + 0x14, 0u32);
            game_write(at + 0x18, u);
            game_write(at + 0x1C, v);
        }
        [g.eax, g.ebx, g.ecx, g.edx] = [u0, v0, u1, v1];
        game_write(VERTEX_COUNT_RVA, 4u32);
        let inverse = x87::reciprocal(game_read(DEPTH_OF_RVA));
        for r in INVERSES {
            game_write(r, inverse);
        }
        let s: u32 = game_read(SETTINGS_RVA);
        g.esi = s;
        if get(s + 0x10) != 0 {
            g.eax = game_read::<u32>(CLOCK_RVA).wrapping_sub(game_read(LAST_CLOCK_RVA));
            let fall = x87::int_times_plus_int_to_int(
                g.eax as i32,
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
        [g.eax, g.ebx, g.ecx] = [turn[6], turn[7], turn[8]];
        g.eax = game_read(FOCAL_RVA);
        for m in 0..4 {
            game_write(WIDE_FOCAL_RVA + 4 * m, g.eax);
        }
        part(7, &mut g)?;
        part(8, &mut g)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let left = match place {
            1 => crate::marks::make_fade_colours::make_fade_colours(r),
            2 => crate::marks::place_view_2::place_view_2(r)?,
            7 => crate::marks::draw_snow::draw_snow(r, &mut crate::marks::draw_snow::game_part)?,
            _ => {
                match place {
                    3 => crate::scene::set_texture::set_texture_in_game(r),
                    4 => crate::scene::depth_buffer_on::depth_buffer(r, true),
                    5 => crate::scene::depth_test_off::depth_test_off(r),
                    6 => {
                        let (value, state) = (r.eax, r.ebx);
                        crate::scene::set_render_state::set_render_state(r, value, state);
                    }
                    _ => crate::scene::depth_test_on::depth_test_on(r),
                }
                return Ok(());
            }
        };
        *r = Regs { esp: r.esp, ..left };
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_snow(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { run_snow(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov ebp, g_marksSettings
    let entry: &[u8] = &[0x8B, 0x2D, 0x28, 0x7A, 0x76, 0x00];
    let copy = match copy(
        0x4C25F6,
        0x4A4,
        entry,
        &[
            0x178, 0x225, 0x22F, 0x234, 0x239, 0x248, 0x257, 0x266, 0x275, 0x284, 0x499, 0x49E,
        ],
        &[
            (0x4C37EF, 1),
            (0x4C2AA5, 2),
            (0x44E77E, 3),
            (0x44E41F, 4),
            (0x44E3D1, 5),
            (0x44EB72, 6),
            (0x4C2E36, 7),
            (0x44E3E1, 8),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x2764 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        (game_addr(rva(0x8D2030)) as u32, 0xD0),
        (game_addr(rva(0x8D2110)) as u32, 0x10),
        (game_addr(rva(0x8D2150)) as u32, 0x58),
        (game_addr(rva(0x8D21F8)) as u32, 4),
        (game_addr(rva(0x8DB430)) as u32, 0x18),
        at(KEPT_RVA),
        (game_addr(VERTICES_RVA) as u32, 0x80),
        at(VERTEX_COUNT_RVA),
    ];
    let mut vars = regions.clone();
    vars.extend(
        [
            SETTINGS_RVA,
            CLOCK_RVA,
            FALL_K_RVA,
            RHW_RVA,
            LIGHT_RVA,
            DEPTH_OF_RVA,
        ]
        .map(at),
    );
    vars.push((game_addr(UV_RVA) as u32, 16));
    vars.push((game_addr(rva(0x8DB50C)) as u32, 0x18));
    vars.push((game_addr(TURN_RVA) as u32, 0x24));
    let saved = capture(&vars);
    for case in 0..4u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SETTINGS_RVA, sa);
                let s = sa as *mut u32;
                for n in 0..0x40 {
                    *s.add(n) = (n as u32).wrapping_mul(0x0301_0507) ^ case;
                }
                *s.add(4) = u32::from(case != 1);
                *s.add(0x2760 / 4) = [0, 0x40, 0x900, 0xFFFF_FFF0][case as usize];
                game_write(CLOCK_RVA, 1000 + case * 77);
                game_write(LAST_CLOCK_RVA, 990u32);
                game_write(FALL_RVA, 0x0FFF_FF00u32 + case);
                game_write(FALL_K_RVA, (2.5f32 + case as f32).to_bits());
                game_write(DEPTH_OF_RVA, 3000.0f32.to_bits());
                for n in 0..6 {
                    game_write(rva(0x8DB50C) + 4 * n, (n as f32 * 7.5 - 3.0).to_bits());
                }
                for n in 0..9 {
                    game_write(TURN_RVA + 4 * n, (n as f32 * 0.1 + case as f32).to_bits());
                }
            },
            &|r| unsafe { run_snow(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
