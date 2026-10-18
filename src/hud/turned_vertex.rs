//! `hud::turned_vertex` (0x4B7058): one vertex of a turned HUD shape at edi: the point
//! (st(1) x, st(0) y, rounded into `dword_75BFF0` / `dword_75BFF4`); its texture place
//! (x plus `flt_75C298`, y plus `flt_75C29C`, over `flt_75C00C`); the point turned by edx
//! (`math::turn_float_point_xz`, ebx the middle value), times `dword_75C2B8` plus
//! `dword_75C2A0` / `dword_75C2A4`; colour white under the quad record's alpha, specular 0.
//!
//! In:  st(0) y, st(1) x (popped), edi the vertex, edx the angle, ebx passed on
//! Out: eax the colour, ecx the turned y, ebx as the turn leaves it; edx, ebp kept
//!
//! Callers: `sub_4B6ECD`'s four (code at 0x4B6FFE ..). The turn is its Rust version. Thunk
//! by `two_in_hook!`. Self-test: the original and the Rust version on a vertex of the
//! test's, points and angles by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::hud::add_quad::QUAD_RVA;
use crate::math::turn_float_point_xz::turn_float_point_xz;
use crate::math::turn_scratch_xz::{X_RVA, Y_RVA, Z_RVA};
use crate::scene::matrix3::{extended, rounded};
use crate::scene::vector3::{Pair80, call_with_two, two_in_hook};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PAIR: Pair80 = Pair80::new();

pub static HOOK: RawHook =
    RawHook::new("hud::turned_vertex", rva(0x4B7058), thunk, &ORIGINAL).self_test(self_test);

two_in_hook!(
    thunk,
    call_original,
    original_with_two,
    dispatch = dispatch,
    original = ORIGINAL,
    pair = PAIR
);

const POINT_RVA: [usize; 2] = [rva(0x75BFF0), rva(0x75BFF4)]; // dword_75BFF0, dword_75BFF4
const UV_SHIFT_RVA: [usize; 2] = [rva(0x75C298), rva(0x75C29C)];
const UV_SCALE_RVA: usize = rva(0x75C00C); // flt_75C00C
const SIZE_RVA: usize = rva(0x75C2B8); // dword_75C2B8
const CENTRE_RVA: [usize; 2] = [rva(0x75C2A0), rva(0x75C2A4)];

/// `fld a; fadd b; fdiv c; fstp`.
fn plus_over(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    unsafe {
        core::arch::asm!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]", "fstp dword ptr [{o}]",
            v = in(reg) v.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
    };
    out
}

/// `fld a; fmul b; fadd c; fstp`.
fn times_plus(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    unsafe {
        core::arch::asm!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]", "fstp dword ptr [{o}]",
            v = in(reg) v.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
    };
    out
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn turned_vertex(entry: &Regs, point: [[u8; 10]; 2]) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let [y, x] = [rounded(point[0]), rounded(point[1])];
        game_write(POINT_RVA[1], y);
        game_write(POINT_RVA[0], x);
        let put = |o: u32, v: u32| ((g.edi + o) as *mut u32).write_unaligned(v);
        let scale: u32 = game_read(UV_SCALE_RVA);
        put(0x18, plus_over(x, game_read(UV_SHIFT_RVA[0]), scale));
        put(0x1C, plus_over(y, game_read(UV_SHIFT_RVA[1]), scale));
        let [tx, ty, tz, _] = turn_float_point_xz(x, g.ebx, y, g.edx);
        [g.eax, g.ebx, g.ecx] = [tx, ty, tz];
        game_write(POINT_RVA[1], tz);
        game_write(POINT_RVA[0], tx);
        let size: u32 = game_read(SIZE_RVA);
        put(0, times_plus(tx, size, game_read(CENTRE_RVA[0])));
        put(4, times_plus(tz, size, game_read(CENTRE_RVA[1])));
        g.eax = (game_read::<u32>(QUAD_RVA) << 24) | 0xFF_FFFF;
        put(0x10, g.eax);
        put(0x14, 0);
    }
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::framework::shadow() {
        unsafe { original_with_two(r) };
        return;
    }
    match unsafe { turned_vertex(r, PAIR.inputs()) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, original_with_two) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // fstp dword_75BFF4
    let entry: &[u8] = &[0xD9, 0x1D, 0xF4, 0xBF, 0x75, 0x00];
    let copy = match copy(0x4B7058, 0x96, entry, &[0x43], &[]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut vertex = [0xAAAA_AAAAu32; 8];
    let va = vertex.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        (va, 0x20),
        (game_addr(POINT_RVA[0]) as u32, 8),
        at(X_RVA),
        at(Y_RVA),
        at(Z_RVA),
    ];
    let mut vars = regions.clone();
    vars.extend(
        [
            UV_SHIFT_RVA[0],
            UV_SHIFT_RVA[1],
            UV_SCALE_RVA,
            SIZE_RVA,
            CENTRE_RVA[0],
            CENTRE_RVA[1],
            QUAD_RVA,
        ]
        .map(at),
    );
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let k = case as f32;
        let point = [
            extended((12.5f32 - k * 7.0).to_bits()),
            extended((-30.0f32 + k * 11.0).to_bits()),
        ];
        let before = Regs {
            edi: va,
            edx: (case * 0x3A1) & 0xFFF,
            ebx: 0x3F80_0000,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            std::ptr::write_bytes(va as *mut u8, 0xAA, 0x20);
            game_write(UV_SHIFT_RVA[0], 64.0f32.to_bits());
            game_write(UV_SHIFT_RVA[1], 32.0f32.to_bits());
            game_write(UV_SCALE_RVA, 256.0f32.to_bits());
            game_write(SIZE_RVA, (1.5f32 + k).to_bits());
            game_write(CENTRE_RVA[0], 320.0f32.to_bits());
            game_write(CENTRE_RVA[1], 400.0f32.to_bits());
            game_write(QUAD_RVA, 0x80 + case);
        };
        prepare();
        let mut ro = before;
        unsafe { call_with_two(&mut ro, copy.addr(), &point) };
        let theirs = (Some(regs(&ro)), capture(&regions));
        prepare();
        let mine = unsafe { turned_vertex(&before, point) };
        let mine = (mine.as_ref().ok().map(regs), capture(&regions));
        t.check(theirs == mine, || {
            format!(
                "case {case}: original {:x?}; rust {:x?}; state same {}",
                theirs.0,
                mine.0,
                theirs.1 == mine.1
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
