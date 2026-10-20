//! `files::is_cd_drive` (0x403310): whether a drive is a CD-ROM, in the flags.
//!
//! In:  eax low byte = the drive's letter, 0 for the current drive
//! Out: every register preserved (ecx, edx pushed, `pusha` .. `popa`); the flags of
//!      `cmp eax, 5` on `files::drive_type`'s answer: zero set for a CD-ROM, carry set
//!      below it
//! Two callers: the installer (0x413182) and `system::main_loop` (0x415443, the base
//! directory's drive; nine `nop`s after the call: the answer is not looked at). Manual
//! patch with a private copy (0xF bytes, the call relocated): the call starts at +3.
//! Shadow mode: both (the Win32 call only reads), compared. Self-test: the original and the
//! Rust version on every letter and some others, the flags compared.

use crate::files::drive_type::drive_type;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("files::is_cd_drive", rva(0x403310), thunk, &ORIGINAL)
    .private_copy(0xF)
    .relocate_calls(&[0x03])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// (carry, zero): `cmp` of the drive's kind with a CD-ROM's.
pub unsafe fn is_cd_drive(letter: u8) -> (bool, bool) {
    // Converted: `rc2k_core::files::io::is_cd`.
    rc2k_core::files::io::is_cd(unsafe { drive_type(letter) })
}

// ---- hook ----

fn flags(r: &Regs) -> (bool, bool) {
    (r.carry(), r.eflags & ZERO_FLAG != 0)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = unsafe { is_cd_drive(r.eax as u8) };
    if !shadow() {
        r.set_carry(mine.0);
        r.set_zero(mine.1);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if flags(&o) != mine || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "letter={:#x}: original {:?}, rust {mine:?}",
                r.eax as u8,
                flags(&o)
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for letter in (b'a'..=b'z').chain(*b"\0CZ1:") {
        let before = Regs {
            eax: 0xAABB_CC00 | letter as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = unsafe { is_cd_drive(letter) };
        let preserved = ro.preserved_except(&before, 0);
        t.check(preserved && flags(&ro) == mine, || {
            format!(
                "letter={letter:#x}: original {:?} preserved={preserved}; rust {mine:?}",
                flags(&ro)
            )
        });
    }
    t.finish()
}
