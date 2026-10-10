//! `threading::start_slot_thread` (0x410C9C): starts the thread of a thread slot.
//!
//! In:  esi = the slot
//! Out: every register preserved
//! It clears the slot's flag byte at +0xC, creates a thread running the routine at +0 (no
//! parameter, default stack), which stores the thread's id at +4, and stores the handle at
//! +8. If no thread can be created the original reports fatal error 0x80108601.
//! Four callers: three in `sub_410B84` and `request_quit`.
//! It creates a real thread, so shadow mode runs the original alone. The self-test uses
//! slots of its own whose routine returns at once.

use crate::errors::FatalError;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, log, rva, shadow};
use std::ffi::c_void;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{CreateThread, WaitForSingleObject};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "threading::start_slot_thread",
    rva(0x410C9C),
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

// ---- slot layout ----

const ROUTINE: usize = 0;
const THREAD_ID: usize = 4;
const HANDLE: usize = 8;
const FLAG: usize = 0xC;
const SLOT_LEN: usize = 0x110;

// ---- Rust version ----

pub unsafe fn start_slot_thread(slot: *mut u8) -> Result<(), FatalError> {
    unsafe {
        slot.add(FLAG).write(0);
        let routine = (slot.add(ROUTINE) as *const usize).read_unaligned();
        let handle = CreateThread(
            std::ptr::null(),
            0,
            std::mem::transmute::<usize, Option<unsafe extern "system" fn(*mut c_void) -> u32>>(
                routine,
            ),
            std::ptr::null(),
            0,
            slot.add(THREAD_ID).cast(),
        );
        if handle.is_null() {
            return Err(FatalError {
                code: 0x8010_8601,
                message: "",
            });
        }
        (slot.add(HANDLE) as *mut u32).write_unaligned(handle as u32);
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // Rare, and one of the calls comes while the game is closing: say in the log what ran.
    let routine = unsafe { (r.esi as *const u32).read_unaligned() };
    if shadow() {
        log(&format!(
            "threading::start_slot_thread: the original runs (routine {routine:#x})"
        ));
        unsafe { call_original(r) };
        return;
    }
    let result = unsafe { start_slot_thread(r.esi as *mut u8) };
    log(&format!(
        "threading::start_slot_thread: the Rust version ran (routine {routine:#x}): {result:?}"
    ));
    // No thread could be created.
    if let Err(e) = result {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test: two slots of its own, whose threads end at once ----

unsafe extern "system" fn ends_at_once(_: *mut c_void) -> u32 {
    0
}

/// Waits for a test thread and closes its handle. True if it ended.
unsafe fn finish(slot: &[u8; SLOT_LEN]) -> bool {
    let handle = u32::from_le_bytes([slot[8], slot[9], slot[10], slot[11]]) as *mut c_void;
    if handle.is_null() {
        return false;
    }
    unsafe {
        let ended = WaitForSingleObject(handle, 2000) == 0;
        CloseHandle(handle);
        ended
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for round in 0..3 {
        let mut fresh = [0xAAu8; SLOT_LEN];
        fresh[..4].copy_from_slice(&(ends_at_once as *const () as u32).to_le_bytes());
        let (mut theirs, mut ours) = (fresh, fresh);

        let before = Regs {
            esi: theirs.as_mut_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = unsafe { start_slot_thread(ours.as_mut_ptr()) };

        // Thread id and handle differ between any two threads: both must be set, and
        // everything else must be the same.
        let set = |slot: &[u8; SLOT_LEN]| {
            slot[THREAD_ID..HANDLE] != fresh[THREAD_ID..HANDLE]
                && slot[HANDLE..FLAG] != fresh[HANDLE..FLAG]
        };
        let rest = |slot: &[u8; SLOT_LEN]| (slot[..THREAD_ID].to_vec(), slot[FLAG..].to_vec());
        let preserved = ro.preserved_except(&before, 0);
        let (set_o, set_r) = (set(&theirs), set(&ours));
        let same_rest = rest(&theirs) == rest(&ours) && theirs[FLAG] == 0;
        let (ended_o, ended_r) = unsafe { (finish(&theirs), finish(&ours)) };
        t.check(
            mine.is_ok() && set_o && set_r && same_rest && preserved && ended_o && ended_r,
            || {
                format!(
                    "round {round}: original set={set_o} ended={ended_o} preserved={preserved}; rust {mine:?} set={set_r} ended={ended_r}; same_rest={same_rest}"
                )
            },
        );
    }
    t.finish()
}
