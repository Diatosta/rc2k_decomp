//! `system::read_keyboard_type` (0x410AA3): `g_keyboardType` (0x518404) 2 when the
//! keyboard's subtype (`GetKeyboardType(1)`) is 0xD01 .. 0xD06 (a Japanese keyboard's);
//! otherwise it is left as it is.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! One caller, `start` (0x40101F); in the game this never gets a call (the hook DLL is
//! loaded later). The Win32 call is made from Rust (the same in both programs). Manual
//! patch with a private copy (0x22 bytes, the call relocated): the call starts at +3.
//! Shadow mode: both (the call only reads), compared. Self-test: the original and the Rust
//! version from two fillings of the variable.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardType;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::read_keyboard_type",
    rva(0x410AA3),
    thunk,
    &ORIGINAL,
)
.private_copy(0x22)
.relocate_calls(&[0x03])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const KEYBOARD_TYPE_RVA: usize = rva(0x518404); // g_keyboardType

// ---- Rust version ----

/// The value the variable is given for a subtype, if any.
pub fn keyboard_type(subtype: u32) -> Option<u32> {
    // Converted: `rc2k_core::system::machine::keyboard_type`.
    rc2k_core::system::machine::keyboard_type(subtype)
}

pub unsafe fn read_keyboard_type() {
    let subtype = unsafe { GetKeyboardType(1) } as u32;
    if let Some(kind) = keyboard_type(subtype) {
        unsafe { game_write(KEYBOARD_TYPE_RVA, kind) };
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        unsafe { read_keyboard_type() };
        return;
    }
    unsafe {
        let before = game_read::<u32>(KEYBOARD_TYPE_RVA);
        read_keyboard_type();
        let mine = game_read::<u32>(KEYBOARD_TYPE_RVA);
        game_write(KEYBOARD_TYPE_RVA, before);
        call_original(r);
        let theirs = game_read::<u32>(KEYBOARD_TYPE_RVA);
        if mine != theirs {
            HOOK.mismatch(|| format!("from {before:#x}: original {theirs:#x}, rust {mine:#x}"));
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { read_keyboard_type() };
    Ok(())
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    unsafe {
        let saved = game_read::<u32>(KEYBOARD_TYPE_RVA);
        for fill in [0u32, 0xAAAA_AAAA] {
            game_write(KEYBOARD_TYPE_RVA, fill);
            let before = Regs::SENTINEL;
            let mut ro = before;
            call_original(&mut ro);
            let theirs = game_read::<u32>(KEYBOARD_TYPE_RVA);
            game_write(KEYBOARD_TYPE_RVA, fill);
            read_keyboard_type();
            let mine = game_read::<u32>(KEYBOARD_TYPE_RVA);
            let preserved = ro.preserved_except(&before, 0);
            t.check(preserved && theirs == mine, || {
                format!(
                    "fill {fill:#x}: original {theirs:#x} preserved={preserved}; rust {mine:#x}"
                )
            });
        }
        // The subtypes around the range.
        for (subtype, kind) in [
            (0xD00, None),
            (0xD01, Some(2)),
            (0xD06, Some(2)),
            (0xD07, None),
        ] {
            t.check(keyboard_type(subtype) == kind, || {
                format!("subtype {subtype:#x}: {:?}", keyboard_type(subtype))
            });
        }
        game_write(KEYBOARD_TYPE_RVA, saved);
    }
    t.finish()
}
