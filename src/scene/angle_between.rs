//! `scene::angle_between` (0x450223): the direction from one place to another as an
//! angle of sixteen bits a turn.
//!
//! In:  eax, ebx = the first place, ecx, edx = the second
//! Out: eax = the angle; ecx, edx = the two differences; everything else preserved
//!
//! `fpatan` of (ecx - eax) over (edx - ebx), over pi, times `flt_5C5E40` (0.5) and
//! `flt_5C5D98` (65536), rounded (`fistp`), its low sixteen bits; the result is also
//! left in `dword_62400C`. The instructions themselves, as `fpatan`'s result is the
//! processor's.
//! Four callers.
//! Self-test: 300 pairs of places.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDX, Regs, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const HALF_RVA: usize = rva(0x5C5E40); // flt_5C5E40, 0.5
const TURN_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0

// ---- Rust version ----

/// The angle of the two differences.
pub unsafe fn angle_of(first: u32, second: u32) -> u32 {
    unsafe {
        let angle = x87::angle_scaled_to_int(
            first as i32,
            second as i32,
            game_read(HALF_RVA),
            game_read(TURN_RVA),
        ) as u32
            & 0xFFFF;
        game_write(CHECKED_RVA, angle);
        angle
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (first, second) = (r.ecx.wrapping_sub(r.eax), r.edx.wrapping_sub(r.ebx));
    Ok(Left::nothing()
        .with(EAX, unsafe { angle_of(first, second) })
        .with(ECX, first)
        .with(EDX, second))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
