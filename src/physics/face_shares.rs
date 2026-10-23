//! `physics::face_shares` (0x442323): for the face corners named by `dword_639890` ..
//! `98` (points of `g_scenePoints`, x y z whole numbers) and the point (`dword_639824`,
//! `dword_63982C`): the corners copied to 0x639708 .. 0x63972B, the point to 0x6396FC /
//! 0x639704, the differences from the first corner (0x639738 .. 0x63975B), two cross
//! products over a third (0x63975C, 0x639760, 0x639764), their quotients (shares,
//! `flt_639810`, `flt_639814`) and those times `flt_5C5D98` (65536) as whole numbers
//! (0x639800, 0x639804). With `g_useSse` the quotient is by `rcpss` (0x639768).
//!
//! In:  nothing
//! Out: eax = 0x63974C's difference, ebx, ecx the third corner's y, z; the rest kept
//!      (flags not read: callers go on with `mov` and calls)
//!
//! See `physics/ground.rs`. Callers: `physics::face_under_point` (when inside), and
//! `sub_457B86`.
//! Self-test: a copy of the original on the test's ground (`TestGround`), every corner
//! triple of a grid's faces against points in and out, both ways (SSE switch off and
//! on), under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, Regs};
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { face_shares_at(&Ground::game(), &mut g) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
