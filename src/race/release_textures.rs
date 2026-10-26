//! `race::release_textures` (0x443396): `race::release_stage_textures` and
//! `scene::reset_texture_tables` (two calls and a `retn`).
//!
//! In:  nothing
//! Out: the registers as the two leave them
//!
//! See `race/race_end.rs`. Ten callers (`race::race_from_menu`, menus, films; none reads
//! flags after it). It begins with its call: private copy, both calls relocated, manual
//! patch. Shadow mode runs the original alone (it releases the textures). Self-test: a
//! copy whose calls go to stand-ins, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::race_end::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::release_textures", rva(0x443396), thunk, &ORIGINAL)
    .private_copy(0xB)
    .relocate_calls(&[0x0, 0x5])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { release_textures(r, &mut release_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { release_textures(r, &mut release_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x443396,
        0xB,
        &[0xE8, 0x04, 0xB4, 0x00, 0x00],
        &[0x0, 0x5],
        &[(0x44E79F, 1), (0x42247B, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [crate::service::harness::at(rva(0x621870))];
    let saved = crate::service::harness::capture(&regions);
    run_both(
        &mut t,
        "the calls",
        &copy,
        &regions,
        &|| unsafe { crate::framework::game_write(rva(0x621870), 0x5A5A_5A5Au32) },
        &|r| unsafe { release_textures(r, &mut test_part) },
    );
    crate::service::harness::restore(&regions, &saved);
    t.finish()
}
