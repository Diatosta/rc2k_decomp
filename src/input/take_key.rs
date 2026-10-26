//! `input::take_key` (0x407F95): takes the pending key: returns `byte_4F6ADC` and sets it to 0,
//! in one `xchg`.
//!
//! In:  nothing
//! Out: al = the key, 0 if there was none (the rest of eax is preserved); ZF = (al == 0)
//! 65 callers. Shadow mode runs the original alone: taking the key twice would lose it. The
//! hook notes the key it hands out, so that callers can be shadow-compared (see
//! `noting_taken_key`). The
//! other flags of the original's `test al, al` (sign, parity) are not reproduced; carry and
//! zero are.

use crate::framework::{RawHook, Regs, SelfTest, game_ptr, hook_asm, rva, shadow};
use std::cell::Cell;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::take_key", rva(0x407F95), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const PENDING_KEY_RVA: usize = rva(0x4F6ADC); // byte_4F6ADC

pub fn pending_key() -> &'static AtomicU8 {
    unsafe { AtomicU8::from_ptr(game_ptr(PENDING_KEY_RVA)) }
}

// ---- Rust version ----

/// The pending key, or 0. There is none afterwards.
pub fn take_key() -> u8 {
    if let Some(key) = REPLAYED.get() {
        return key;
    }
    pending_key().swap(0, Ordering::SeqCst)
}

// ---- replaying a key, for shadow comparisons of functions that take one ----
//
// The key arrives from another thread at any moment, so a caller of `take_key` cannot be
// compared by running it twice on "the same" pending key. Instead the original runs first,
// the key it was given is noted here, and the Rust version is then run with that key
// replayed instead of the real one.

thread_local! {
    /// The key the hook last handed out on this thread.
    static LAST_TAKEN: Cell<Option<u8>> = const { Cell::new(None) };
    /// While set, `take_key` returns this and leaves the real pending key alone.
    static REPLAYED: Cell<Option<u8>> = const { Cell::new(None) };
}

/// Runs `f` and returns the key the hook handed out during it, if it was asked for one.
pub fn noting_taken_key<R>(f: impl FnOnce() -> R) -> (R, Option<u8>) {
    // An enclosing call keeps its own note if nothing is taken in here.
    let outer = LAST_TAKEN.replace(None);
    let result = f();
    let taken = LAST_TAKEN.get();
    if taken.is_none() {
        LAST_TAKEN.set(outer);
    }
    (result, taken)
}

/// Runs `f` with `take_key` returning `key` instead of the real pending key.
pub fn replaying_key<R>(key: u8, f: impl FnOnce() -> R) -> R {
    let outer = REPLAYED.replace(Some(key));
    let result = f();
    REPLAYED.set(outer);
    result
}

/// True if keys taken by original code are noted, that is, if this hook is on.
pub fn notes_taken_keys() -> bool {
    HOOK.is_enabled()
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // The console command `fatal` uses this hook to get onto the game's own thread.
    crate::fatal::raise::raise_test_if_armed(r);
    if shadow() {
        unsafe { call_original(r) };
        LAST_TAKEN.set(Some(r.eax as u8));
        return;
    }
    let key = take_key();
    LAST_TAKEN.set(Some(key));
    r.eax = (r.eax & !0xFF) | key as u32;
    r.set_zero(key == 0);
    r.set_carry(false);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::fatal::raise::raise_test_if_armed(r);
    let key = take_key();
    LAST_TAKEN.set(Some(key));
    r.eax = (r.eax & !0xFF) | key as u32;
    r.set_zero(key == 0);
    r.set_carry(false);
    Ok(())
}

// ---- self-test: a real key pressed during the test is lost or misread ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = pending_key().load(Ordering::SeqCst);
    for key in [0u8, 1, 0x1C, 0x68, 0x69, 0x80, 0xFF] {
        pending_key().store(key, Ordering::SeqCst);
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let left_o = pending_key().load(Ordering::SeqCst);

        pending_key().store(key, Ordering::SeqCst);
        let mine = take_key();
        let left_r = pending_key().load(Ordering::SeqCst);

        let mut expected = before;
        expected.eax = (before.eax & !0xFF) | mine as u32;
        let zero_o = ro.eflags & 0x40 != 0;
        let ok = ro.eax == expected.eax
            && ro.preserved_except(&before, crate::framework::EAX)
            && zero_o == (mine == 0)
            && !ro.carry()
            && left_o == 0
            && left_r == 0;
        t.check(ok, || {
            format!(
                "key={key:#x}: original eax={:#x} zero={zero_o} carry={} left={left_o:#x}; rust key={mine:#x} left={left_r:#x}",
                ro.eax,
                ro.carry()
            )
        });
    }
    pending_key().store(saved, Ordering::SeqCst);
    t.finish()
}
