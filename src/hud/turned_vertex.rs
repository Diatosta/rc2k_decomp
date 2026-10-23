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
//! test's, points and angles by lot, at the test precisions (`service::floats`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::hud::add_quad::QUAD_RVA;
use crate::math::turn_float_point_xz::turn_float_point_xz_plain;
use rc2k_core::hud::disc;

const POINT_RVA: [usize; 2] = [rva(0x75BFF0), rva(0x75BFF4)]; // dword_75BFF0, dword_75BFF4
const UV_SHIFT_RVA: [usize; 2] = [rva(0x75C298), rva(0x75C29C)];
const UV_SCALE_RVA: usize = rva(0x75C00C); // flt_75C00C
const SIZE_RVA: usize = rva(0x75C2B8); // dword_75C2B8
const CENTRE_RVA: [usize; 2] = [rva(0x75C2A0), rva(0x75C2A4)];

/// The Rust version (see the header), the point as `f64`s (what the x87 holds at the
/// game's double precision): the registers it leaves.
// Converted: `rc2k_core::hud::disc::{turned_texture, turned_place, white_under}`
pub unsafe fn turned_vertex(entry: &Regs, point: [f64; 2]) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let [y, x] = point.map(|v| v as f32);
        game_write(POINT_RVA[1], y.to_bits());
        game_write(POINT_RVA[0], x.to_bits());
        let put = |o: u32, v: f32| ((g.edi + o) as *mut u32).write_unaligned(v.to_bits());
        let f = |r: usize| f32::from_bits(game_read(r));
        let scale = f(UV_SCALE_RVA);
        put(0x18, disc::turned_texture(x, f(UV_SHIFT_RVA[0]), scale));
        put(0x1C, disc::turned_texture(y, f(UV_SHIFT_RVA[1]), scale));
        let [tx, ty, tz, _] = turn_float_point_xz_plain(x.to_bits(), g.ebx, y.to_bits(), g.edx);
        [g.eax, g.ebx, g.ecx] = [tx, ty, tz];
        game_write(POINT_RVA[1], tz);
        game_write(POINT_RVA[0], tx);
        let size = f(SIZE_RVA);
        let [tx, tz] = [tx, tz].map(f32::from_bits);
        put(0, disc::turned_place(tx, size, f(CENTRE_RVA[0])));
        put(4, disc::turned_place(tz, size, f(CENTRE_RVA[1])));
        g.eax = disc::white_under(game_read::<u32>(QUAD_RVA));
        ((g.edi + 0x10) as *mut u32).write_unaligned(g.eax);
        ((g.edi + 0x14) as *mut u32).write_unaligned(0);
    }
    Ok(g)
}
