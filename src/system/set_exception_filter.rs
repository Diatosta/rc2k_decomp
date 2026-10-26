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

use crate::framework::{Regs, game_addr, rva, standalone};
use windows_sys::Win32::System::Diagnostics::Debug::{
    LPTOP_LEVEL_EXCEPTION_FILTER, SetUnhandledExceptionFilter,
};

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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    set_exception_filter();
    Ok(())
}
