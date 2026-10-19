//! `system::set_exception_filter` (0x410D4C): the game's crash reporter made the process's
//! unhandled exception filter: `SetUnhandledExceptionFilter(TopLevelExceptionFilter)`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! The filter (0x410D59) is the game's and stays so (section 15): it skips an instruction
//! the processor test tried (`dword_51849C`, the place to go on at), else logs the
//! exception and raises fatal error 0x8000ED1E "Exception!". One caller, `start`
//! (0x401015); in the game this never gets a call (the hook DLL is loaded later). In the
//! standalone program the filter is not set: the game's code cannot run there, and the
//! program's own handler reports faults (`standalone.rs`). Shadow mode runs the original
//! alone. Self-test: the filter each version leaves, read back, the one there before put
//! back.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow, standalone};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Diagnostics::Debug::{
    LPTOP_LEVEL_EXCEPTION_FILTER, SetUnhandledExceptionFilter,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::set_exception_filter",
    rva(0x410D4C),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const FILTER_RVA: usize = rva(0x410D59); // TopLevelExceptionFilter

// ---- Rust version ----

/// The game's filter, as Win32 takes it.
fn game_filter() -> LPTOP_LEVEL_EXCEPTION_FILTER {
    unsafe { std::mem::transmute::<usize, LPTOP_LEVEL_EXCEPTION_FILTER>(game_addr(FILTER_RVA)) }
}

pub fn set_exception_filter() {
    if standalone() {
        return;
    }
    unsafe { SetUnhandledExceptionFilter(game_filter()) };
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    set_exception_filter();
}

// ---- self-test ----

/// The filter set now, left as it is.
fn current() -> usize {
    unsafe {
        let now = SetUnhandledExceptionFilter(None);
        SetUnhandledExceptionFilter(now);
        now.map_or(0, |f| f as usize)
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = unsafe { SetUnhandledExceptionFilter(None) };
    let before_regs = Regs::SENTINEL;
    let mut ro = before_regs;
    unsafe { call_original(&mut ro) };
    let theirs = current();
    unsafe { SetUnhandledExceptionFilter(None) };
    set_exception_filter();
    let mine = current();
    unsafe { SetUnhandledExceptionFilter(before) };
    let preserved = ro.preserved_except(&before_regs, 0);
    t.check(
        theirs == mine && mine == game_addr(FILTER_RVA) && preserved,
        || {
            format!(
                "original {theirs:#x} preserved={preserved}; rust {mine:#x}; the game's {:#x}",
                game_addr(FILTER_RVA)
            )
        },
    );
    t.finish()
}
