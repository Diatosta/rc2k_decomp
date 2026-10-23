//! `physics::camera_frame` (0x44FF1B): the camera's shake, then its step. The shake
//! (`flt_637644`) and the offset `dword_637638` .. 0; in modes 1 .. 3 (`unk_71A65C`) with
//! `unk_71A22C` 0: `unk_71A8E4` x `flt_614098` held within ± `flt_614094` (the limit
//! negated and back, `fchs`) into `dword_63763C`, and the shake a random number
//! (`math::next_random`) / 65536 / `flt_5C5E28` x `flt_5C5D40` x it; both 0 again with
//! `dword_601024` set. Then `physics::update_camera`.
//!
//! In:  nothing
//! Out: the registers as `update_camera` leaves them; no flags read
//!
//! Callers: `sub_43B635`, `sub_459360`, `sub_46062E`.
//! Refuses what `update_camera` refuses (everything written put back). Shadow mode runs
//! the original alone (as its ground checks).
//! Self-test (`update_camera::camera_test`, in a race; skipped otherwise; the carry not
//! compared, as no caller reads it).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::camera_ground::carry_entry;
use crate::physics::flow::{base, g, hold_high, hold_low, negate, p, x87_run};
use crate::race::car_starts::Part;

// Converted: `rc2k_core::physics::camera_view::shakes` (the x87 steps stay).
fn steps(r: &mut Regs, part: Part) -> Result<(), Refused> {
    for x in [0x637644, 0x637638, 0x63763C, 0x637640] {
        p(x, 0);
    }
    if rc2k_core::physics::camera_view::shakes(g(0x71A65C), g(0x71A22C)) {
        x87_run!(
            "fld dword ptr [{b} + 0x31A8E4]",
            "fmul dword ptr [{b} + 0x214098]",
            "fstp dword ptr [{b} + 0x23763C]";
            b = base()
        );
        hold_high(0x63763C, 0x614094);
        negate(0x614094);
        hold_low(0x63763C, 0x614094);
        negate(0x614094);
        r.eax = unsafe { crate::math::next_random::next_random() };
        p(0x62400C, r.eax);
        x87_run!(
            "fild dword ptr [{b} + 0x22400C]",
            "fdiv dword ptr [{b} + 0x1C5D98]",
            "fdiv dword ptr [{b} + 0x1C5E28]",
            "fmul dword ptr [{b} + 0x1C5D40]",
            "fmul dword ptr [{b} + 0x23763C]",
            "fstp dword ptr [{b} + 0x237644]";
            b = base()
        );
        if g(0x601024) != 0 {
            p(0x63763C, 0);
            p(0x637644, 0);
        }
    }
    crate::physics::update_camera::steps(r, part)
}

/// The Rust version, the probes through `part`.
pub unsafe fn camera_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    {
        let mut r = *entry;
        steps(&mut r, part)?;
        Ok(r)
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    carry_entry(r, camera_frame)
}
