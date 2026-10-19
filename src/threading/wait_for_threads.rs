//! `threading::wait_for_threads` (0x410BA8): the shutdown's wait: the input thread's,
//! the audio thread's and the network's receiving thread's slots (`g_inputThreadSlot`,
//! `g_audioThreadSlot`, `stru_518714`) each resumed if suspended (`sub_410D1D`), their
//! three handles waited for, all of them, three seconds at most
//! (`WaitForMultipleObjects`), and closed (`CloseHandle`, the last pushed first). The
//! handles are the three pushed as `sub_410D1D`'s arguments, which it leaves on the stack:
//! they are the array the wait gets.
//! `sub_410D1D` (`threading::resume_slot_thread`, not hooked: one Win32 call): with a slot
//! whose +0xC is 0xFF (`neg`: made 1), `ResumeThread` on its +8.
//!
//! In:  nothing
//! Out: eax the wait's result (into `popa`'s saved eax); the rest kept
//!
//! One caller, `threading::shutdown_thread`. The resumes through `part` (in the game
//! `resume_slot_thread` with `api`), the wait and the closes through `api` (the same
//! functions in the standalone). Shadow mode runs the original alone (handles are
//! closed). Self-test: a copy whose three slots are the test's and whose calls go to
//! noting stand-ins (the wait's array noted as its handles), against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::system::api_stand_ins::{Api, copy_with, set_array_place, set_gives};
use crate::system::exit_parts::{
    CLOSE_HANDLE, Part, RESUME, RESUME_THREAD, WAIT_MULTIPLE, compare_exit, dispatch_exit,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "threading::wait_for_threads",
    rva(0x410BA8),
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

/// g_inputThreadSlot, g_audioThreadSlot, stru_518714: three slots of 0x110 bytes in a row.
const SLOTS: usize = 0x5184F4;
const SLOT_LEN: u32 = 0x110;

/// `threading::resume_slot_thread` (0x410D1D): the slot's thread resumed if it was
/// suspended (+0xC 0xFF, made 1).
pub fn resume_slot_thread(slot: u32, api: Api) {
    if slot == 0 {
        return;
    }
    let flag = (slot + 0xC) as *mut u8;
    unsafe {
        if flag.read_volatile() == 0xFF {
            flag.write_volatile(1);
            api(
                RESUME_THREAD,
                &[((slot + 8) as *const u32).read_unaligned()],
            );
        }
    }
}

/// The Rust version on the slots from `slots`.
pub fn wait_for_threads_with(
    r: &mut Regs,
    slots: u32,
    part: Part,
    api: Api,
) -> Result<(), Refused> {
    let mut handles = [0u32; 3];
    for (k, handle) in handles.iter_mut().enumerate() {
        let slot = slots + SLOT_LEN * k as u32;
        *handle = unsafe { ((slot + 8) as *const u32).read_unaligned() };
        let mut g = Regs { esi: slot, ..*r };
        part(RESUME, &mut g)?;
    }
    // The stack: the last pushed first.
    let array = [handles[2], handles[1], handles[0]];
    let rc = api(WAIT_MULTIPLE, &[3, array.as_ptr() as u32, 1, 3000]);
    for handle in array {
        api(CLOSE_HANDLE, &[handle]);
    }
    r.eax = rc;
    Ok(())
}

/// The Rust version.
pub fn wait_for_threads(r: &mut Regs, api: Api) -> Result<(), Refused> {
    let mut resume = |_: u32, g: &mut Regs| -> Result<(), Refused> {
        resume_slot_thread(g.esi, &mut crate::system::exit_parts::game_api);
        Ok(())
    };
    wait_for_threads_with(r, game_addr(rva(SLOTS)) as u32, &mut resume, api)
}

fn in_game(r: &mut Regs, _part: Part, api: Api) -> Result<(), Refused> {
    wait_for_threads(r, api)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_exit(&HOOK, r, call_original, in_game);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = vec![0u8; 3 * SLOT_LEN as usize];
    let base = mem.as_mut_ptr() as u32;
    // pusha; lea esi, g_inputThreadSlot
    let entry = [0x60, 0x8D, 0x35, 0xF4, 0x84, 0x51, 0x00];
    let copy = match copy_with(
        0x410BA8,
        0x51,
        &entry,
        &[0xA, 0x18, 0x26, 0x37, 0x40, 0x45, 0x4A],
        &[RESUME],
        &[
            (0x37, WAIT_MULTIPLE, 4),
            (0x40, CLOSE_HANDLE, 1),
            (0x45, CLOSE_HANDLE, 1),
            (0x4A, CLOSE_HANDLE, 1),
        ],
        &[(SLOTS, 3 * SLOT_LEN as usize, base)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_array_place(WAIT_MULTIPLE);
    for rc in [0u32, 0x102, u32::MAX] {
        set_gives(WAIT_MULTIPLE, rc);
        compare_exit(
            &mut t,
            &format!("the wait gives {rc:#x}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe {
                for k in 0..3u32 {
                    let slot = base + SLOT_LEN * k;
                    ((slot + 8) as *mut u32).write_volatile(0x100 + k);
                    ((slot + 0xC) as *mut u8).write_volatile(if k == 1 { 0xFF } else { 0 });
                }
            },
            &[(base, 3 * SLOT_LEN as usize)],
            &|r, part, api| wait_for_threads_with(r, base, part, api),
            false,
        );
    }
    set_array_place(u32::MAX);
    // `resume_slot_thread` against a copy of 0x410D1D whose `ResumeThread` is noted.
    // push eax; push ecx; push edx; test esi, esi
    let resume_entry = [0x50, 0x51, 0x52, 0x85, 0xF6];
    let resume = match copy_with(
        0x410D1D,
        0x1C,
        &resume_entry,
        &[0x13],
        &[],
        &[(0x13, RESUME_THREAD, 1)],
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut slots = [[0u8; 0x10]; 2];
    for flag in [0xFFu8, 0, 1, 0xFE] {
        for null in [false, true] {
            for s in slots.iter_mut() {
                *s = [0x11; 0x10];
                s[8] = 0x42;
                s[0xC] = flag;
            }
            let (mine, theirs) = (slots[0].as_mut_ptr() as u32, slots[1].as_mut_ptr() as u32);
            crate::system::api_stand_ins::take_noted();
            resume_slot_thread(
                if null { 0 } else { mine },
                &mut crate::system::api_stand_ins::test_api,
            );
            let noted_mine = crate::system::api_stand_ins::take_noted();
            let mut o = Regs {
                esi: if null { 0 } else { theirs },
                ..Regs::SENTINEL
            };
            unsafe { crate::framework::call_game(&mut o, resume.addr()) };
            let noted_theirs = crate::system::api_stand_ins::take_noted();
            t.check(
                slots[0] == slots[1] && noted_mine == noted_theirs,
                || {
                    format!(
                        "resume, flag {flag:#x}, null {null}: slots {:x?}, calls {noted_theirs:x?} / {noted_mine:x?}",
                        slots
                    )
                },
            );
        }
    }
    let _ = &mut mem;
    t.finish()
}
