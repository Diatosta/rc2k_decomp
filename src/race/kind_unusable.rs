//! `race::kind_unusable` (0x449D34): carry for a car kind (eax) past the table
//! (`race::kind_past_table`) or 6.
//!
//! In:  eax = the kind
//! Out: carry set for 6 or 0x17 or more; zero as the last `cmp` left it; every
//!      register preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_449B79` three times (`jb` after each). It
//! begins with its call: private copy, the call relocated, manual patch. Its flags are
//! its result, so dispatch sets carry and zero. Shadow mode: the original, its flags
//! compared with the Rust version's. Self-test: kinds round 6, 0x17 and the sign, with the
//! flags clear and set before.

use crate::audio::buffer_play::zero;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::race_shell::kind_unusable;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::kind_unusable", rva(0x449D34), thunk, &ORIGINAL)
    .private_copy(0x10)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn set(r: &mut Regs, (carry, z): (bool, bool)) {
    r.set_carry(carry);
    r.set_zero(z);
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = kind_unusable(r.eax);
    if shadow() {
        let before = *r;
        unsafe { call_original(r) };
        let theirs = (r.carry(), zero(r));
        if theirs != mine || !r.preserved_except(&before, 0) {
            HOOK.mismatch(|| format!("eax {:#x}: original {theirs:?}; rust {mine:?}", before.eax));
        }
        return;
    }
    set(r, mine);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for eax in [
        0u32,
        1,
        5,
        6,
        7,
        0x16,
        0x17,
        0x18,
        0x7FFF_FFFF,
        0x8000_0000,
        0xFFFF_FFFF,
    ] {
        for flags in [0u32, 0x8D5] {
            let before = Regs {
                eax,
                eflags: flags,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let theirs = (ro.carry(), zero(&ro));
            let mine = kind_unusable(eax);
            t.check(theirs == mine && ro.preserved_except(&before, 0), || {
                format!("eax {eax:#x}, flags {flags:#x}: original {theirs:?}; rust {mine:?}")
            });
        }
    }
    t.finish()
}
