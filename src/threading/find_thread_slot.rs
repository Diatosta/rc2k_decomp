use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, game_addr, hook_asm, log};
use crate::threading::thread_slot::ThreadSlot;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "threading::find_thread_slot",
    0x410CC5 - 0x400000,
    thunk,
    &TRAMPOLINE,
);

// The callers branch on ZF right after the call, so set it from esi like the original does.
hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = TRAMPOLINE,
    epilogue = "test esi, esi"
);

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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let mine = unsafe { reimplemented() };

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        r.esi = mine as u32;
        return;
    }

    let mut o = *r;

    unsafe { call_original(&mut o) };
    let original = o.esi as *const ThreadSlot;
    if original != mine {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH find_thread_slot: original={original:p} rust={mine:p}"
            ));
        }
    }
    r.esi = o.esi;
}
