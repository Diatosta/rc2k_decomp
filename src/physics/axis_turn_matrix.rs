//! `physics::axis_turn_matrix` (0x421019): the matrix at edi of a turn by `flt_5C78F8`
//! about the axis `flt_5C78FC` .. `flt_5C7904` (Rodrigues' form, with |x| square-rooted
//! for the diagonal's x x, as the original has it), its work values left in
//! `flt_5C7098` .. `flt_5C70B8` (sine, cosine, 1 - cosine, three products of two axis
//! parts and three of one with the sine); the fourth row 0, 0, 0, 1; the fourth column's
//! three left as they were.
//!
//! In:  edi the matrix (0x40 bytes)
//! Out: every register kept; no flags read (callers go on with `push`, `lea`)
//!
//! Callers: `physics::move_slot` (0x4624D6) and `sub_43DD54` (0x43DD97).
//! Self-test: a copy of the original on a `physics::Mirror` of its work values and
//! inputs, angles and axes by lot, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::math::x87;
use crate::math::x87_run::{abs_sqrt_times_plus, one_less, sine_cosine_of};
use crate::physics::mirror::Mirror;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust core on the variables `m` reaches; the matrix at `out`.
pub unsafe fn axis_turn_at(m: &Mirror, out: u32) {
    unsafe {
        let g = |v: u32| m.get(v);
        let (s, c) = sine_cosine_of(g(0x5C78F8));
        m.put(0x5C709C, c);
        m.put(0x5C7098, s);
        let t = one_less(g(0x5C709C));
        m.put(0x5C70A0, t);
        let (x, y, z) = (g(0x5C78FC), g(0x5C7900), g(0x5C7904));
        m.put(0x5C70A4, x87::mul2(t, x, z));
        m.put(0x5C70A8, x87::mul2(t, x, y));
        m.put(0x5C70AC, x87::mul2(t, y, z));
        m.put(0x5C70B0, x87::mul(x, s));
        m.put(0x5C70B4, x87::mul(y, s));
        m.put(0x5C70B8, x87::mul(z, s));
        let (p1, p2, p3) = (g(0x5C70A4), g(0x5C70A8), g(0x5C70AC));
        let (q1, q2, q3) = (g(0x5C70B0), g(0x5C70B4), g(0x5C70B8));
        let put = |o: u32, v: u32| ((out + o) as *mut u32).write_unaligned(v);
        put(0x00, abs_sqrt_times_plus(x, t, c));
        put(0x04, x87::add(p2, q3));
        put(0x08, x87::sub(p1, q2));
        put(0x10, x87::sub(p2, q3));
        put(0x14, abs_sqrt_times_plus(y, t, c));
        put(0x18, x87::add(p3, q1));
        put(0x20, x87::add(p1, q2));
        put(0x24, x87::sub(p3, q1));
        put(0x28, abs_sqrt_times_plus(z, t, c));
        for o in [0x30, 0x34, 0x38] {
            put(o, 0);
        }
        put(0x3C, x87::loaded(1.0f32.to_bits()));
    }
}

/// The Rust version.
pub unsafe fn axis_turn_matrix(out: u32) {
    unsafe { axis_turn_at(&Mirror::game(), out) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { axis_turn_matrix(r.edi) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
