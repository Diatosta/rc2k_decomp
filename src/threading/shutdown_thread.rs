//! `threading::shutdown_thread` (0x410C17): the routine of thread slot 3 (`stru_518824`),
//! started by `threading::request_quit`: "SHUTDOWN:START" logged, the slot's +0xC 1 (in
//! its work), the other threads waited for (`threading::wait_for_threads`), "SHUTDOWN:wfmo
//! rc=" and the wait's result (8 hex digits) logged, the window (`hWnd`, if any) asked to
//! close (`PostMessageA` WM_CLOSE), +0xC 0, "SHUTDOWN:DONE" logged and the thread ended
//! (`ExitThread(0)`).
//!
//! In:  nothing (a thread's start; `esi` is set to the slot)
//! Out: does not return
//!
//! No callers: `CreateThread`'s (`start_slot_thread`), and a fatal error on this thread
//! enters it again (`fatal::report`'s `jmp [esi]`). Its calls through `part` (the log's,
//! `wait_for_threads`) and `api` (`PostMessageA`, `ExitThread`: the same functions in the
//! standalone, where the window's handle is 0 and nothing is posted). The thread ends in
//! `dispatch`: the thunk does not wait for self-tests and is not counted as inside a hook.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::system::api_stand_ins::Api;
use crate::system::exit_parts::{
    EXIT_THREAD, HEX8_2_NL, LOG_2, LOG_2_NL, POST_MESSAGE, Part, WAIT_FOR_THREADS, exit_entry,
};

const SLOT: usize = 0x518824; // stru_518824: thread slot 3
const WINDOW: usize = 0x5184A4; // hWnd
const START: usize = 0x4DA8D7; // "SHUTDOWN:START"
const RC: usize = 0x4DA8E6; // "SHUTDOWN:wfmo rc="
const DONE: usize = 0x4DA8F8; // "SHUTDOWN:DONE"
const WM_CLOSE: u32 = 0x10;

/// The Rust version on the slot at `slot` and the window's handle at `window`. Returns
/// only where `api`'s `ExitThread` does (a test's).
pub fn shutdown_thread_with(
    r: &mut Regs,
    slot: u32,
    window: u32,
    part: Part,
    api: Api,
) -> Result<(), Refused> {
    let text = |x: usize| game_addr(rva(x)) as u32;
    r.esi = slot;
    let mut l = Regs {
        eax: text(START),
        ..*r
    };
    part(LOG_2_NL, &mut l)?;
    let flag = (slot + 0xC) as *mut u8;
    unsafe { flag.write_volatile(1) };
    part(WAIT_FOR_THREADS, r)?;
    let mut l = Regs {
        eax: text(RC),
        ..*r
    };
    part(LOG_2, &mut l)?;
    let mut l = *r;
    part(HEX8_2_NL, &mut l)?;
    let hwnd = unsafe { (window as *const u32).read_volatile() };
    if hwnd != 0 {
        r.eax = api(POST_MESSAGE, &[hwnd, WM_CLOSE, 0, 0]);
    }
    unsafe { flag.write_volatile(0) };
    let mut l = Regs {
        eax: text(DONE),
        ..*r
    };
    part(LOG_2_NL, &mut l)?;
    api(EXIT_THREAD, &[0]);
    Ok(())
}

fn in_game(r: &mut Regs, part: Part, api: Api) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    shutdown_thread_with(r, a(SLOT), a(WINDOW), part, api)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    exit_entry(r, in_game)
}
