//! `threading::request_quit` (0x410BF9): asks the game to shut down: sets `dword_51837C` to 1
//! and, if it was not 1 already, starts the thread of slot 3 (`stru_518824`), whose routine
//! at 0x410C17 logs "SHUTDOWN:START", waits for the other threads and posts a close message
//! to the window.
//!
//! In:  nothing
//! Out: every register preserved
//! Two callers: the fatal-error reporter and `sub_415383`.
//! It cannot be run twice or undone, so shadow mode runs the original alone. The self-test
//! runs a copy of the original whose switch is a test variable and whose call to
//! `start_slot_thread` goes to a stand-in.

use crate::errors::FatalError;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_ptr, hook_asm, log, rva, shadow,
};
use crate::redirect::RedirectedCopy;
use crate::threading::start_slot_thread::start_slot_thread;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("threading::request_quit", rva(0x410BF9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const QUIT_RVA: usize = rva(0x51837C); // dword_51837C: 1 = the game is shutting down
const SHUTDOWN_SLOT_RVA: usize = rva(0x518824); // stru_518824: thread slot 3

// ---- Rust version ----

/// `requested` is the shutdown switch; `start` starts the thread of the slot at the address
/// it is given.
pub fn request_quit_with(
    requested: &AtomicU32,
    start: &mut dyn FnMut(u32) -> Result<(), FatalError>,
) -> Result<(), FatalError> {
    if requested.swap(1, Ordering::SeqCst) == 1 {
        return Ok(());
    }
    start(game_addr(SHUTDOWN_SLOT_RVA) as u32)
}

pub unsafe fn request_quit() -> Result<(), FatalError> {
    let requested = unsafe { &*game_ptr::<AtomicU32>(QUIT_RVA) };
    let before = requested.load(Ordering::SeqCst);
    let result = request_quit_with(requested, &mut |slot| unsafe {
        start_slot_thread(slot as *mut u8)
    });
    if result.is_err() {
        // Left as it was, so that the original, run in its place, tries the thread too.
        requested.store(before, Ordering::SeqCst);
    }
    result
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // It happens once, while the game is closing, when `stats` can no longer be asked: say
    // in the log which version ran.
    let was: u32 = unsafe { crate::framework::game_read(QUIT_RVA) };
    if shadow() {
        log(&format!(
            "threading::request_quit: the original runs (switch was {was})"
        ));
        unsafe { call_original(r) };
        return;
    }
    let result = unsafe { request_quit() };
    log(&format!(
        "threading::request_quit: the Rust version ran (switch was {was}): {result:?}"
    ));
    if let Err(e) = result {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test: a copy that sets a test variable and starts nothing ----

const FUNCTION_LEN: usize = 0x1E;
const ENTRY_BYTES: [u8; 6] = [0x60, 0xB8, 0x01, 0x00, 0x00, 0x00]; // pusha; mov eax, 1
/// The address operand of `xchg eax, dword_51837C`.
const SWITCH_OPERAND: usize = 8;
const START_CALL: usize = 0x17;

static TEST_SWITCH: AtomicU32 = AtomicU32::new(0);
static STARTS: AtomicU32 = AtomicU32::new(0);
static STARTED_SLOT: AtomicU32 = AtomicU32::new(0);

/// Stands in for `start_slot_thread`: notes the slot in esi.
#[unsafe(naked)]
unsafe extern "C" fn fake_start() {
    core::arch::naked_asm!(
        "mov [{slot}], esi",
        "inc dword ptr [{starts}]",
        "ret",
        slot = sym STARTED_SLOT,
        starts = sym STARTS,
    )
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let redirect =
        |target: usize| (target == rva(0x410C9C)).then_some(fake_start as *const () as usize);
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x410BF9)),
            FUNCTION_LEN,
            &ENTRY_BYTES,
            &[START_CALL],
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    unsafe { copy.patch_u32(SWITCH_OPERAND, &TEST_SWITCH as *const AtomicU32 as u32) };

    let observed = || [&TEST_SWITCH, &STARTS, &STARTED_SLOT].map(|v| v.load(Ordering::SeqCst));
    for switch in [0u32, 1, 2, 0xFFFF_FFFF] {
        let prepare = || {
            TEST_SWITCH.store(switch, Ordering::SeqCst);
            STARTS.store(0, Ordering::SeqCst);
            STARTED_SLOT.store(0x5EED, Ordering::SeqCst);
        };
        prepare();
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = observed();

        prepare();
        let mine = request_quit_with(&TEST_SWITCH, &mut |slot| {
            STARTED_SLOT.store(slot, Ordering::SeqCst);
            STARTS.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        let ours = observed();

        let preserved = ro.preserved_except(&before, 0);
        t.check(mine.is_ok() && theirs == ours && preserved, || {
            format!(
                "switch={switch:#x}: original switch/starts/slot {theirs:x?} preserved={preserved}; rust {mine:?} {ours:x?}"
            )
        });
    }
    t.finish()
}
