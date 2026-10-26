//! `marks::draw_plane` (0x4C8F64): a quad laid on the hexagons' frame (a reading: the
//! ground glare): the rate, the place and two axes from the settings (+0x2720 .., +0x272C ..,
//! +0x2708 .., +0x2740), their lengths' shares, made one long and square
//! (`scene::normalise_vector3`, `scene::cross_product3`), scaled (`marks::scale_axes`);
//! nothing more unless the place's way faces the third axis away, or unless
//! `dword_641678`; `scene::depth_test_off`, render states 0x11 = 2, 0x12 = 2,
//! `scene::set_texture` (`dword_5D1E10`); the four corners; the texture places from corner
//! `dword_934C64` of `g_marksCorners`, then from `flt_934F3C` ..; colour `g_viewLight`;
//! unless a corner's depth is at or below `flt_5C5E18` (signed compares of the bits): the
//! corners projected, their depth (plus `g_marksKindFloat` and `flt_934F5C` .., times
//! `flt_8EB96D`) and rhw (1 over the depth times `flt_934F6C` ..), `scene::draw_fan`,
//! `scene::depth_test_on`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the path leaves them
//!
//! Caller: `sub_4C74DA` (0x4C752F). The x87 runs are the original's instructions over the
//! game's variables; the vector helpers and `scale_axes` their Rust versions; the device
//! calls through `part` (1 the depth test off, 2 a render state, 3 the texture, 4 the draw,
//! 5 the depth test on), in the game their Rust versions. Refuses null settings and a
//! corner past `g_marksCorners`, before anything. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose device calls go to stand-ins, against the Rust version,
//! on settings of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::x87;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::draw_plane", rva(0x4C8F64), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const AXES: [usize; 3] = [rva(0x89CF1A), rva(0x89CF26), rva(0x89CF32)];
const PLACE_RVA: usize = rva(0x8161A2); // flt_8161A2
const FACING_RVA: usize = rva(0x814718); // flt_814718
const SWITCH_RVA: usize = rva(0x641678); // dword_641678
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const CORNER_RVA: usize = rva(0x934C64); // dword_934C64
const MARK_CORNERS_RVA: usize = rva(0x8EB971); // g_marksCorners
const UV_K_RVA: usize = rva(0x8EB969); // flt_8EB969
const PLACES_RVA: usize = rva(0x934F3C); // flt_934F3C .. flt_934F58
const LIGHT_RVA: usize = rva(0x6415E4); // g_viewLight
const CORNERS_RVA: usize = rva(0x934C98); // flt_934C98 .. flt_934CC4: four of 12
const NEAR_RVA: usize = rva(0x5C5E18); // flt_5C5E18
const KIND_RVA: usize = rva(0x76529C); // g_marksKindFloat
const LIFTS_RVA: usize = rva(0x934F5C); // flt_934F5C ..
const RHW_K_RVA: usize = rva(0x934F6C); // flt_934F6C ..
const Z_K_RVA: usize = rva(0x8EB96D); // flt_8EB96D
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn addr(r: usize) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4C8F6A .. 0x4C9000: the rate (`flt_89CA34`), the place (`flt_89CEF6` ..) and the first axis (`flt_89CF1A` ..) from the settings, its length over `flt_934F34` into `flt_89CEEE`.
unsafe fn plane_axes(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2740]",
            "fmul dword ptr [{b} + 0x534F30]",
            "fstp dword ptr [{b} + 0x49CA34]",
            "fild dword ptr [{s} + 0x2720]",
            "fild dword ptr [{s} + 0x2724]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CEFE]",
            "fstp dword ptr [{b} + 0x49CEFA]",
            "fstp dword ptr [{b} + 0x49CEF6]",
            "fild dword ptr [{s} + 0x272C]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x49CF1A]",
            "fild dword ptr [{s} + 0x2730]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x49CF1E]",
            "fild dword ptr [{s} + 0x2734]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CF22]",
            "fld dword ptr [{b} + 0x49CF1A]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF1E]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF22]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{b} + 0x534F34]",
            "fstp dword ptr [{b} + 0x49CEEE]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C900D .. 0x4C906D: the second axis (`flt_89CF26` ..), its length over `flt_934F38` into `flt_89CEF2`.
unsafe fn plane_up(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2708]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x49CF26]",
            "fild dword ptr [{s} + 0x270C]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x49CF2A]",
            "fild dword ptr [{s} + 0x2710]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CF2E]",
            "fld dword ptr [{b} + 0x49CF26]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF2A]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF2E]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{b} + 0x534F38]",
            "fstp dword ptr [{b} + 0x49CEF2]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C90BE .. 0x4C90E2: the place as floats into `flt_8161A2` ...
unsafe fn plane_place(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x4161A2]",
            "fild dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x4161A6]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x4161AA]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C90ED .. 0x4C9113: the place's way dotted with the third axis into `flt_814718`.
unsafe fn plane_facing(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4161A2]",
            "fmul dword ptr [{b} + 0x49CF32]",
            "fld dword ptr [{b} + 0x4161A6]",
            "fmul dword ptr [{b} + 0x49CF36]",
            "fld dword ptr [{b} + 0x4161AA]",
            "fmul dword ptr [{b} + 0x49CF3A]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{b} + 0x414718]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C915A .. 0x4C9244: the plane's centre and corners (`flt_934C98` .. `flt_934CC4`) from the axes.
unsafe fn plane_corners(b: u32) {
    unsafe {
        core::arch::asm!(
            ".byte 0xD9, 0xEE",
            ".byte 0xD9, 0xEE",
            "fmul dword ptr [{b} + 0x49CEF2]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF1A]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF26]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEF6]",
            "fstp dword ptr [{b} + 0x534CBC]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF1E]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF2A]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEFA]",
            "fstp dword ptr [{b} + 0x534CC0]",
            "fmul dword ptr [{b} + 0x49CF2E]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CF22]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEFE]",
            "fstp dword ptr [{b} + 0x534CC4]",
            "fld dword ptr [{b} + 0x534CBC]",
            "fadd dword ptr [{b} + 0x49CF7A]",
            "fst dword ptr [{b} + 0x534C98]",
            "fadd dword ptr [{b} + 0x49CF6E]",
            "fst dword ptr [{b} + 0x534CA4]",
            "fsub dword ptr [{b} + 0x49CF7A]",
            "fstp dword ptr [{b} + 0x534CB0]",
            "fld dword ptr [{b} + 0x534CC0]",
            "fadd dword ptr [{b} + 0x49CF7E]",
            "fst dword ptr [{b} + 0x534C9C]",
            "fadd dword ptr [{b} + 0x49CF72]",
            "fst dword ptr [{b} + 0x534CA8]",
            "fsub dword ptr [{b} + 0x49CF7E]",
            "fstp dword ptr [{b} + 0x534CB4]",
            "fld dword ptr [{b} + 0x534CC4]",
            "fadd dword ptr [{b} + 0x49CF82]",
            "fst dword ptr [{b} + 0x534CA0]",
            "fadd dword ptr [{b} + 0x49CF76]",
            "fst dword ptr [{b} + 0x534CAC]",
            "fsub dword ptr [{b} + 0x49CF82]",
            "fstp dword ptr [{b} + 0x534CB8]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C93FA .. 0x4C94CA: the four corners projected.
unsafe fn plane_projected(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x534CA0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x534C98]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x534C98]",
            "fmul dword ptr [{b} + 0x534C9C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x534C9C]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x534CAC]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x534CA4]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x534CA4]",
            "fmul dword ptr [{b} + 0x534CA8]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x534CA8]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x534CB8]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x534CB0]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x534CB0]",
            "fmul dword ptr [{b} + 0x534CB4]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x534CB4]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x534CC4]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x534CBC]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x534CBC]",
            "fmul dword ptr [{b} + 0x534CC0]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x534CC0]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_plane(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let corner: u32 = game_read(CORNER_RVA);
        let b = addr(0);
        g.esi = s;
        plane_axes(b, s);
        let [a0, a1, a2] = AXES.map(addr);
        normalise_vector3(a0);
        plane_up(b, s);
        normalise_vector3(a1);
        [g.esi, g.edi, g.ebp] = [a1, a0, a2];
        cross_product3(a1, a0, a2)?;
        normalise_vector3(a2);
        [g.esi, g.edi, g.ebp] = [a0, a2, a1];
        cross_product3(a0, a2, a1)?;
        g = crate::marks::scale_axes::scale_axes(&g);
        g.esi = s;
        plane_place(b, s);
        normalise_vector3(addr(PLACE_RVA));
        [g.esi, g.edi] = [addr(PLACE_RVA), a2];
        plane_facing(b);
        if game_read::<u32>(FACING_RVA) & 0x8000_0000 == 0 {
            return Ok(g);
        }
        if game_read::<u32>(SWITCH_RVA) == 0 {
            return Ok(g);
        }
        if corner >= 0x100 {
            return Err(BadInput("a plane corner past g_marksCorners").into());
        }
        part(1, &mut g)?;
        for state in [0x11, 0x12] {
            [g.eax, g.ebx] = [2, state];
            part(2, &mut g)?;
        }
        g.eax = game_read(TEXTURE_RVA);
        part(3, &mut g)?;
        plane_corners(b);
        g.eax = addr(MARK_CORNERS_RVA) + corner * 0x80;
        let word = |off: u32| ((g.eax + off) as *const u16).read_unaligned() as u32;
        let uv_k: u32 = game_read(UV_K_RVA);
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex + 0x18, x87::int_times(word(u) as i32, uv_k));
            game_write(vertex + 0x1C, x87::int_times(word(v) as i32, uv_k));
        }
        [g.ebp, g.edi] = [word(8), word(0xA)];
        for n in 0..4 {
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex + 0x18, x87::loaded(game_read(PLACES_RVA + 8 * n)));
            game_write(
                vertex + 0x1C,
                x87::loaded(game_read(PLACES_RVA + 8 * n + 4)),
            );
        }
        g.ecx = game_read(LIGHT_RVA);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, g.ecx);
        }
        for n in 0..4 {
            g.ecx = game_read(CORNERS_RVA + 12 * n + 8);
            if (g.ecx as i32) <= game_read::<i32>(NEAR_RVA) {
                return Ok(g);
            }
        }
        plane_projected(b);
        let [kind, z_k]: [u32; 2] = [game_read(KIND_RVA), game_read(Z_K_RVA)];
        for n in 0..4 {
            let c = CORNERS_RVA + 12 * n;
            let vertex = VERTICES_RVA + 0x20 * n;
            [g.eax, g.ebx] = [game_read(c), game_read(c + 4)];
            game_write(vertex, g.eax);
            game_write(vertex + 4, g.ebx);
            let depth: u32 = game_read(c + 8);
            game_write(
                vertex + 8,
                x87::sum3_times(depth, kind, game_read(LIFTS_RVA + 4 * n), z_k),
            );
        }
        for n in 0..4 {
            let depth: u32 = game_read(CORNERS_RVA + 12 * n + 8);
            game_write(
                VERTICES_RVA + 0x20 * n + 0xC,
                x87::reciprocal_times(depth, game_read(RHW_K_RVA + 4 * n)),
            );
        }
        part(4, &mut g)?;
        part(5, &mut g)?;
    }
    Ok(g)
}

/// In the game: the device calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::depth_test_off::depth_test_off(r),
            2 => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, value, state);
            }
            3 => crate::scene::set_texture::set_texture_in_game(r),
            4 => crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part),
            _ => crate::scene::depth_test_on::depth_test_on(r),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_plane(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_plane(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov esi, g_marksSettings
    let entry: &[u8] = &[0x8B, 0x35, 0x28, 0x7A, 0x76, 0x00];
    let copy = match copy(
        0x4C8F64,
        0x691,
        entry,
        &[
            0xA3, 0x110, 0x127, 0x132, 0x149, 0x14F, 0x184, 0x1C9, 0x1D8, 0x1E7, 0x1F1, 0x686,
            0x68B,
        ],
        &[
            (0x44E3D1, 1),
            (0x44EB72, 2),
            (0x44E77E, 3),
            (0x44EA65, 4),
            (0x44E3E1, 5),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x2748 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        (addr(rva(0x89CA34)), 4),
        (addr(rva(0x89CEEE)), 0xA0),
        (addr(PLACE_RVA), 12),
        (addr(FACING_RVA), 4),
        (addr(CORNERS_RVA), 0x30),
        (addr(VERTICES_RVA), 0x80),
    ];
    let mut vars = regions.clone();
    vars.extend(
        [
            SETTINGS_RVA,
            SWITCH_RVA,
            CORNER_RVA,
            TEXTURE_RVA,
            NEAR_RVA,
            Z_K_RVA,
            UV_K_RVA,
        ]
        .map(at),
    );
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
                let p = |o: u32, v: u32| ((sa + o) as *mut u32).write_unaligned(v);
                for n in 0..3u32 {
                    p(0x2720 + 4 * n, [100, 50, 400][n as usize] + case * 9);
                    p(0x272C + 4 * n, [300, 60, 380][n as usize]);
                    p(0x2708 + 4 * n, [90, 250, 420][n as usize]);
                }
                p(0x2740, 3 + case);
                game_write(SWITCH_RVA, u32::from(case != 3));
                game_write(CORNER_RVA, 0x12 + case);
                game_write(NEAR_RVA, (-1e9f32).to_bits());
                game_write(Z_K_RVA, (1.0f32 / 3000.0).to_bits());
                game_write(UV_K_RVA, (1.0f32 / 256.0).to_bits());
                std::ptr::write_bytes(addr(VERTICES_RVA) as *mut u8, 0xAA, 0x80);
            },
            &|r| unsafe { draw_plane(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
