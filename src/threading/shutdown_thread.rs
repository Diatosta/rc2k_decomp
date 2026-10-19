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
//! Shadow mode runs the original alone. Self-test: a copy whose slot and window are the
//! test's and whose calls go to noting stand-ins (its `ExitThread` one returns to the
//! copy's caller), against the Rust version, with a window and without.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::system::api_stand_ins::{Api, copy_with, note, set_gives};
use crate::system::exit_parts::{
    EXIT_THREAD, HEX8_2_NL, LOG_2, LOG_2_NL, POST_MESSAGE, Part, WAIT_FOR_THREADS, compare_exit,
    dispatch_exit,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "threading::shutdown_thread",
    rva(0x410C17),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_exit(&HOOK, r, call_original, in_game);
}

// ---- self-test ----

extern "C" fn note_exit(code: u32) {
    note(EXIT_THREAD, &[code]);
}

/// The copy's `ExitThread`: notes its argument and returns to the copy's caller (the
/// stack there is the function's entry one less its `push 0` and the call).
#[unsafe(naked)]
unsafe extern "C" fn exit_stand_in() {
    core::arch::naked_asm!(
        "pushad",
        "push dword ptr [esp + 36]",
        "call {note}",
        "add esp, 4",
        "popad",
        "add esp, 8",
        "ret",
        note = sym note_exit,
    )
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut slot = [0u8; 0x110];
    let mut window = 0u32;
    let (slot_at, window_at) = (slot.as_mut_ptr() as u32, &raw mut window as u32);
    // cld; lea esi, stru_518824
    let entry = [0xFC, 0x8D, 0x35, 0x24, 0x88, 0x51, 0x00];
    let copy = match copy_with(
        0x410C17,
        0x63,
        &entry,
        &[0xE, 0x18, 0x24, 0x2B, 0x46, 0x56, 0x5E],
        &[LOG_2_NL, WAIT_FOR_THREADS, LOG_2, HEX8_2_NL],
        &[(0x46, POST_MESSAGE, 4)],
        &[(SLOT, 0x110, slot_at), (WINDOW, 4, window_at)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    unsafe { copy.retarget(0x5F, exit_stand_in as *const () as usize) };
    set_gives(POST_MESSAGE, 1);
    for hwnd in [0u32, 0x0001_0042] {
        compare_exit(
            &mut t,
            &format!("window {hwnd:#x}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe {
                for k in 0..0x110 {
                    ((slot_at + k) as *mut u8).write_volatile(0x5A);
                }
                (window_at as *mut u32).write_volatile(hwnd);
            },
            &[(slot_at, 0x110), (window_at, 4)],
            &|r, part, api| shutdown_thread_with(r, slot_at, window_at, part, api),
            false,
        );
    }
    let _ = (&mut slot, &mut window);
    t.finish()
}
