//! `marks::load_kind_data` (0x4C1235): `init_marks`' step 1: the marks' kind from the
//! settings, then the edges and the splines.
//!
//! In:  nothing
//! Out: as `marks::load_splines` leaves them
//!
//! See `marks/edges.rs`. One caller, `marks::init_marks`. Shadow mode runs the original
//! alone (its steps read files and take memory). Refuses as its steps do. Self-test: a
//! redirected copy whose four calls go to stand-ins noting every register, against the
//! Rust version on the same stand-ins (settings of the test's); put back afterwards.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::marks::edges::*;
use crate::marks::masks::{NOTED, note_step, test_step};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::load_kind_data", rva(0x4C1235), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { load_kind_data(r, &mut kind_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { load_kind_data(r, &mut kind_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov esi, g_marksSettings
    let entry: &[u8] = &[0x8B, 0x35, 0x28, 0x7A, 0x76, 0x00];
    let copy = match crate::race::mud::copy(
        0x4C1235,
        0x30,
        entry,
        &[0x16, 0x1B, 0x25, 0x2A],
        &[(0x429B18, 0), (0x4C0F2E, 1), (0x4B96CD, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note_step);
    let scene = crate::marks::lines::TestScene::new();
    let settings = scene.settings.as_ptr() as u32;
    let regions = [at(crate::marks::kind_values::KIND_RVA)];
    let saved = capture(&regions);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for kind in [0u32, 7, 0x21] {
        let prepare = || unsafe {
            restore(&regions, &saved);
            ((settings + 0x26A8) as *mut u32).write_unaligned(kind);
            NOTED.lock().unwrap().clear();
        };
        prepare();
        let mut theirs = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut theirs, copy.addr()) };
        let theirs = (
            regs(&theirs),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        prepare();
        let mine = unsafe { load_kind_data(&Regs::SENTINEL, &mut test_step) };
        let mine = (
            mine.as_ref().map(regs).unwrap_or_default(),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!("kind {kind:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    drop(scene);
    t.finish()
}
