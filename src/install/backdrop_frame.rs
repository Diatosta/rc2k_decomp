//! `install::backdrop_frame` (0x415273, `loc_415273`, inside `sub_41525D`): the frame of
//! `install::show_backdrop`: the surface locked (`gfx::lock_surface`; failing: carry set,
//! nothing more), the backdrop picture copied to it, unlocked and shown.
//!
//! In:  nothing
//! Out: as `gfx::show_frame` leaves them, or the lock's carry
//! Caller `screens::run_frame` (eax this address). Its calls through `part` (the
//! platform's and `gfx::copy_picture_to_surface`). A call at +0: private copy, manual
//! patch. Shadow mode runs the original alone. Self-test: a copy whose four calls go to
//! stand-ins (the lock failing or not), against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CF, COPY_PICTURE, LOCK, Part, SHOW, UNLOCK, ZF, answer, at, both, copy, quiet,
};
use crate::race::viewer::dispatch_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::backdrop_frame", rva(0x415273), thunk, &ORIGINAL)
    .private_copy(0x17)
    .relocate_calls(&[0x0, 0x7, 0xC, 0x11])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The surface locked, the backdrop drawn, the frame shown; the registers as the calls
/// leave them.
pub fn backdrop_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(at(LOCK), &mut r)?;
    if r.carry() {
        return Ok(r);
    }
    for callee in [COPY_PICTURE, UNLOCK, SHOW] {
        part(at(callee), &mut r)?;
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        backdrop_frame(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry = [0xE8, 0x35, 0x06, 0xFF, 0xFF];
    let copy = match copy(
        0x415273,
        0x17,
        &entry,
        &[
            (0x0, 0x4058AD),
            (0x7, 0x4060E3),
            (0xC, 0x405949),
            (0x11, 0x4057D1),
        ],
        &[0x4058AD, 0x4060E3, 0x405949, 0x4057D1],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for fails in [false, true] {
        let prepare = || {
            answer(Box::new(move |place, r| {
                r.set_carry(place == at(LOCK) && fails);
                // Each call's own zero flag (the stand-ins otherwise pass on the flags
                // they were called with, which differ between the sides).
                r.set_zero(place == at(SHOW));
                r.eax = place;
            }))
        };
        both(
            &mut t,
            &format!("the lock fails {fails}"),
            &copy,
            Regs::SENTINEL,
            &[],
            CF | ZF,
            &[],
            &prepare,
            &|r, part, _| backdrop_frame(r, part),
        );
    }
    quiet();
    t.finish()
}
