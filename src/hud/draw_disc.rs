//! `hud::draw_disc` (0x4B6C4E): the HUD's disc of 16 triangles: the ring's 16 entries (1 ..
//! 0x10 of `flt_75C188` / `flt_75C1CC`) from `g_sines` / `g_cosines` at k * 0x100, and of
//! the texture ring (`flt_75C210` / `flt_75C254`) at k * 0x100 less `dword_75C05C` times
//! `flt_75C028` times `flt_75C024` (`fistp`, & 0xFFF); then each triangle (the centre,
//! entry k + 1, entry k + 2 wrapping past 0x10) from `hud::disc_vertex` and drawn by
//! `hud::add_triangle`.
//!
//! In:  nothing
//! Out: eax the last texture angle, ebx 0x10, ecx 0x10, edx 0x10; the rest kept
//!
//! Callers: `sub_4B6B0A`, `sub_4B6B5B`, `sub_4B6BAC`, `sub_4B6BFD`. The vertices are
//! `disc_vertex`'s Rust version; the triangles through `part` (1), in the game
//! `add_triangle`'s Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose triangle call goes to a stand-in (the vertex calls to their
//! hook), against the Rust version, angles by lot.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::disc_vertex::{disc_vertex, tables, test_tables};
use crate::math::x87;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_disc", rva(0x4B6C4E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const RING_RVA: [usize; 2] = [rva(0x75C188), rva(0x75C1CC)]; // flt_75C188, flt_75C1CC
const TEXTURE_RING_RVA: [usize; 2] = [rva(0x75C210), rva(0x75C254)];
const SINES_RVA: usize = rva(0xA23AE4); // g_sines
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines
const TURN_RVA: [usize; 3] = [rva(0x75C05C), rva(0x75C028), rva(0x75C024)];
const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const SEGMENTS: u32 = 0x10;

/// A call by place: 1 `add_triangle`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `fld a; fchs; fmul b; fmul c; fistp`.
fn negated_times_times_to_int(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    unsafe {
        core::arch::asm!("fld dword ptr [{v}]", "fchs", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]", "fistp dword ptr [{o}]",
            v = in(reg) v.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
    };
    out
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_disc(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        for k in 0..SEGMENTS {
            let a = k << 8;
            let n = 4 * (k as usize + 1);
            game_write(
                RING_RVA[0] + n,
                x87::loaded(game_read(SINES_RVA + 4 * a as usize)),
            );
            game_write(
                RING_RVA[1] + n,
                x87::loaded(game_read(COSINES_RVA + 4 * a as usize)),
            );
            let t = negated_times_times_to_int(
                game_read(TURN_RVA[0]),
                game_read(TURN_RVA[1]),
                game_read(TURN_RVA[2]),
            );
            game_write(SCRATCH_RVA, t);
            g.eax = a.wrapping_add(t) & 0xFFF;
            game_write(
                TEXTURE_RING_RVA[0] + n,
                x87::loaded(game_read(SINES_RVA + 4 * g.eax as usize)),
            );
            game_write(
                TEXTURE_RING_RVA[1] + n,
                x87::loaded(game_read(COSINES_RVA + 4 * g.eax as usize)),
            );
        }
        // as the first loop leaves them (`div ebx`, then `mov edx, 0`)
        [g.ebx, g.ecx, g.edx] = [SEGMENTS, SEGMENTS, 0];
        let base = game_addr(VERTICES_RVA) as u32;
        for k in 0..SEGMENTS {
            let third = if k + 2 > SEGMENTS {
                k + 2 - SEGMENTS
            } else {
                k + 2
            };
            let mut r = g;
            r.edx = k;
            for (n, e) in [0, k + 1, third].into_iter().enumerate() {
                r.edi = base + 0x20 * n as u32;
                r.ecx = e;
                r = disc_vertex(&r)?;
            }
            part(1, &mut r)?;
        }
        [g.ebx, g.ecx, g.edx] = [SEGMENTS; 3];
    }
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_disc(r, &mut crate::hud::add_triangle::game_part_triangle) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov ecx, 0
    let entry: &[u8] = &[0xB9, 0x00, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x4B6C4E,
        0xC2,
        entry,
        &[0x8A, 0x98, 0xB0, 0xB5],
        &[(0x4B6DA7, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (game_addr(RING_RVA[0]) as u32, 0x44 * 4),
        (game_addr(VERTICES_RVA) as u32, 0x60),
        at(SCRATCH_RVA),
    ];
    let mut vars = regions.clone();
    vars.extend(tables());
    vars.extend(TURN_RVA.map(at));
    let saved = capture(&vars);
    for case in 0..4u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                test_tables(case);
                game_write(TURN_RVA[0], (case as f32 * 0.7 - 1.0).to_bits());
                game_write(TURN_RVA[1], 651.9f32.to_bits());
                game_write(TURN_RVA[2], 1.0f32.to_bits());
                std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0x60);
            },
            &|r| unsafe { draw_disc(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
