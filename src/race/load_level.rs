//! `race::load_level` (0x4455FC): the course data read into the course memory and set up
//! (locacurrcourse's "loadlevel").
//!
//! In:  nothing
//! Out: eax and edx the cdat digits, ecx 0, ebx the face count, esi the last face, edi past
//!      the header; ebp preserved
//!
//! See `race/course_data.rs`. One caller, `sub_4394CB` (locacurrcourse). A failed read is
//! fatal error 0x80650210 (no message), raised from Rust. Shadow mode runs the original
//! alone (it reads and logs). Self-test: a redirected copy whose eleven calls note
//! themselves (the read given or failed; its jump to the fatal stub landed), against the
//! Rust version noting the same.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::course_data::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::load_level", rva(0x4455FC), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { load_level(r, &mut load_part) } {
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
        let g = unsafe { load_level(r, &mut load_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

/// Whether the test's read fails.
static FAILS: AtomicBool = AtomicBool::new(false);
const GIVES: u32 = 0x1_2340;

fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    if place == 1 {
        r.eax = GIVES;
        r.set_carry(FAILS.load(Ordering::SeqCst));
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    note(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::{WENT_FATAL, fatal_landing, set_handler, stand_in};
    use crate::race::level_init::take_called;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x4455FC)),
            0x9C,
            &[0xB8, 0x00, 0x00, 0x00, 0x00],
            &[
                0x05, 0x21, 0x3A, 0x52, 0x5F, 0x65, 0x71, 0x77, 0x83, 0x89, 0x95,
            ],
            &|c| {
                LOAD_CALLEES
                    .iter()
                    .find(|&&(k, _)| rva(k) == c)
                    .map(|&(_, p)| stand_in(p))
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    unsafe { copy.retarget(0x46, fatal_landing as *const () as usize) };
    let kept = [at(READ_BYTES_RVA)];
    let saved = capture(&kept);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for fails in [false, true] {
        FAILS.store(fails, Ordering::SeqCst);
        set_handler(note);
        restore(&kept, &saved);
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let fatal = WENT_FATAL.load(Ordering::SeqCst) == 1;
        let theirs = ((!fatal).then(|| regs(&ro)), take_called(), capture(&kept));
        restore(&kept, &saved);
        let mine = unsafe { load_level(&Regs::SENTINEL, &mut test_part) };
        let mine_fatal =
            matches!(&mine, Err(crate::errors::Refused::Fatal(f)) if f.code == 0x8065_0210);
        let mine = (mine.as_ref().ok().map(regs), take_called(), capture(&kept));
        t.check(theirs == mine && fatal == mine_fatal, || {
            format!(
                "read fails {fails}: original fatal {fatal} {:x?} {:x?}; rust fatal {mine_fatal} {:x?} {:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    restore(&kept, &saved);
    t.finish()
}
