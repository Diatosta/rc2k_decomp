use crate::framework::{game_addr, log, RawHook, MAX_LOGGED, SHADOW_MODE};
use crate::threading::thread_slot::ThreadSlot;
use std::arch::naked_asm;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("threading::find_thread_slot", 0x410CC5 - 0x400000, thunk, &TRAMPOLINE);

const _: () = assert!(size_of::<ThreadSlot>() == 0x110);
const _: () = assert!(std::mem::offset_of!(ThreadSlot, thread_id) == 4);

const SLOTS_RVA: usize = 0x5184F4 - 0x400000;
const SLOT_COUNT: usize = 4;

unsafe fn reimplemented() -> *const ThreadSlot {
    let slots = game_addr(SLOTS_RVA) as *const ThreadSlot;
    let tid = unsafe { GetCurrentThreadId() };
    for i in 0..SLOT_COUNT {
        let slot = unsafe { slots.add(i) };

        if unsafe { (&raw const (*slot).thread_id).read_volatile() } == tid {
            log(&format!("find_thread_slot: tid={tid:#x}"));
            unsafe {
                log(&format!(
                    "Got thread slot at {slot:p} unknown={:#x} thread_id={:#x}",
                    (&raw const (*slot).unknown0).read_volatile(),
                    (&raw const (*slot).thread_id).read_volatile()
                ));
            }
            return slot;
        }
    }
    std::ptr::null()
}

extern "C" fn dispatch() -> *const ThreadSlot {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let mine = unsafe { reimplemented() };

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        return mine;
    }

    let original = unsafe { call_original() };
    if original != mine {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH find_thread_slot: original={original:p} rust={mine:p}"
            ));
        }
    }
    original
}

#[unsafe(naked)]
unsafe extern "C" fn thunk() {
    naked_asm!(
        "push eax",
        "push ecx",
        "push edx",
        "call {dispatch}",
        "mov esi, eax",
        "pop edx",
        "pop ecx",
        "pop eax",
        "test esi, esi",
        "ret",
        dispatch = sym dispatch
    );
}

#[unsafe(naked)]
unsafe extern "C" fn call_original() -> *const ThreadSlot {
    naked_asm!(
        "push esi",
        "call dword ptr [{t}]",
        "mov eax, esi",
        "pop esi",
        "ret",
        t = sym TRAMPOLINE,
    )
}
