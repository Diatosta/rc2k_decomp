//! `files::drive_type` (0x4031D8): what kind of drive a letter is.
//!
//! In:  eax = the drive's letter in its low byte, or 0 there for the current drive
//! Out: eax = `GetDriveTypeA`'s answer (5 = a CD-ROM); everything else preserved
//! The letter is lowered through `g_lowerCase` and asked about as `x:\` (`sub_4031EE`
//! makes that text in a dword). Three callers: the search for the game's CD, the
//! installer, and `sub_403310`, the main loop's "is the base directory on a CD" check
//! (whose answer this build no longer looks at: nine `nop`s follow its call at 0x415448).
//! Shadow mode asks twice. Manual patch with a private copy (0x16 bytes, both calls
//! relocated): a call starts in the first bytes.

use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, hook_asm, rva, shadow};
use crate::settings::find_key::LOWER_RVA;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Storage::FileSystem::GetDriveTypeA;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("files::drive_type", rva(0x4031D8), thunk, &ORIGINAL)
    .private_copy(0x16)
    .relocate_calls(&[0x02, 0x0D])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CD_ROM: u32 = 5;

// ---- Rust version ----

/// The root that is asked about: `x:\`, or none for the current drive.
pub unsafe fn drive_root(letter: u8) -> Option<[u8; 4]> {
    (letter != 0).then(|| unsafe { [game_read::<u8>(LOWER_RVA + letter as usize), b':', b'\\', 0] })
}

pub unsafe fn drive_type(letter: u8) -> u32 {
    unsafe {
        match drive_root(letter) {
            Some(root) => GetDriveTypeA(root.as_ptr()),
            None => GetDriveTypeA(std::ptr::null()),
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = unsafe { drive_type(r.eax as u8) };
    if !shadow() {
        r.eax = mine;
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax != mine || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| format!("letter={:#x}: original {} rust {mine}", r.eax as u8, o.eax));
    }
    r.eax = o.eax;
    r.eflags = o.eflags;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for letter in (b'a'..=b'z').chain(*b"\0CZ1:\xC9") {
        for high in [0u32, 0xAABB_CC00] {
            let before = Regs {
                eax: high | letter as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let mine = unsafe { drive_type(letter) };
            let preserved = ro.preserved_except(&before, EAX);
            t.check(mine == ro.eax && preserved, || {
                format!(
                    "letter={letter:#x}: original {} preserved={preserved}; rust {mine}",
                    ro.eax
                )
            });
        }
    }
    t.finish()
}
