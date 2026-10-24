//! `hud::draw_needle` (0x4B6ECD): a HUD needle: its value (`dword_75C2E0`, as a float
//! through `dword_75C060`) held to `flt_75C018` at least; with `dword_75BFAC` held to
//! `flt_75C2C8` at most and made an angle over `flt_75C2CC` plus `flt_75C2BC`; without,
//! held to `flt_75C2D4`, and below `flt_75C2C4` over `flt_75C2D0` plus `flt_75C2BC`, else
//! less `flt_75C2C4` over `flt_75C2CC` plus `flt_75C2C0`; times `flt_75C010` times
//! `flt_75C014` (`fistp`), times -16 the turn; the needle's four corners (x `flt_75C2A8` /
//! `flt_75C2B0`, y `flt_75C2AC` / `flt_75C2B4`) by `hud::turned_vertex`, then
//! `hud::add_polygon` on four. (The holds by the compares' C0, an unordered compare
//! counting as below.)
//!
//! In:  nothing
//! Out: edx the turn, then as `add_polygon` leaves them
//!
//! Callers: the needles' set-ups (`hud::needle_*`). The corners and the polygon through
//! `part` (1 a corner with its two values, 2 the polygon), in the game their Rust
//! versions. Shadow mode runs the original alone (it draws). Self-test: a copy whose calls
//! go to stand-ins (the corners' taking their values off), against the Rust version,
//! values by lot on both ways.
//!
//! Converted 2026-10-21: the holds, the choice of the angle's way, the turn's register and
//! the corners through `rc2k_core::hud::disc` (compares of floats only: exact); the value
//! made a float and the turn's arithmetic and rounding stay the x87's (they follow its
//! control word). `dword_75C060` is written once, with the turn (the original also holds
//! the value there between, read by nothing).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::rva;
use crate::hud::batch::POLYGON_COUNT_RVA;
use crate::hud::store::Slot;
use crate::hud::store::read_f32 as f;
use crate::hud::store::{game_addr, game_read, game_write};
use crate::math::x87;
use crate::scene::matrix3::extended;

const KIND_RVA: Slot = Slot::at(0x75BFAC); // dword_75BFAC
const VALUE_RVA: Slot = Slot::at(0x75C2E0); // dword_75C2E0
const SCRATCH_RVA: Slot = Slot::at(0x75C060); // dword_75C060
const LEAST_RVA: Slot = Slot::at(0x75C018); // flt_75C018
/// flt_75C2BC .. flt_75C2D4: base, base 2, split, scale, scale 2, most, most 2
const K_RVA: Slot = Slot::at(0x75C2BC);
const TURN_K_RVA: [Slot; 2] = [Slot::at(0x75C010), Slot::at(0x75C014)];
const CORNERS_RVA: [Slot; 4] = [
    Slot::at(0x75C2A8),
    Slot::at(0x75C2AC),
    Slot::at(0x75C2B0),
    Slot::at(0x75C2B4),
];
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

/// `fld v [; fsub less]; fdiv over; fadd plus; fmul k0; fmul k1; fistp`.
fn turn_of(v: u32, less: Option<u32>, over: u32, plus: u32, k: [u32; 2]) -> u32 {
    let (a, mut out) = ([v, less.unwrap_or(0), over, plus, k[0], k[1]], 0u32);
    unsafe {
        if less.is_some() {
            core::arch::asm!("fld dword ptr [{a}]", "fsub dword ptr [{a} + 4]", "fdiv dword ptr [{a} + 8]",
                "fadd dword ptr [{a} + 12]", "fmul dword ptr [{a} + 16]", "fmul dword ptr [{a} + 20]", "fistp dword ptr [{o}]",
                a = in(reg) a.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
        } else {
            core::arch::asm!("fld dword ptr [{a}]", "fdiv dword ptr [{a} + 8]",
                "fadd dword ptr [{a} + 12]", "fmul dword ptr [{a} + 16]", "fmul dword ptr [{a} + 20]", "fistp dword ptr [{o}]",
                a = in(reg) a.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
        }
    };
    out
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::disc::{needle_turn, needle_edx, needle_corners}`
pub unsafe fn draw_needle() -> Result<(), Refused> {
    unsafe {
        use rc2k_core::hud::disc::{needle_corners, needle_edx, needle_turn};
        use rc2k_core::math::extended::to_f64;
        let kind = game_read::<u32>(KIND_RVA) != 0;
        let value = f32::from_bits(x87::from_int(game_read(VALUE_RVA)));
        let t = needle_turn(value, kind, f(LEAST_RVA), |n| f(K_RVA + 4 * n));
        let tk = TURN_K_RVA.map(|r| game_read::<u32>(r));
        // The rounding follows the x87's control word: the x87's.
        let turn = turn_of(
            t.value.to_bits(),
            t.less.map(f32::to_bits),
            t.over.to_bits(),
            t.plus.to_bits(),
            tk,
        );
        game_write(SCRATCH_RVA, turn);
        let angle = needle_edx(turn);
        for (n, (x, y)) in needle_corners(CORNERS_RVA.map(|r| game_read::<u32>(r)))
            .into_iter()
            .enumerate()
        {
            let vertex = game_addr(VERTICES_RVA) as u32 + 0x20 * n as u32;
            let point = [extended(y), extended(x)].map(to_f64);
            crate::hud::turned_vertex::turned_vertex(vertex, angle, point);
        }
        game_write(POLYGON_COUNT_RVA, 4u32);
        crate::hud::add_polygon::add_polygon()
    }
}
