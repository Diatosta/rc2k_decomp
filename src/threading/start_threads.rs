//! `threading::start_threads` (0x410B84): the game's three worker threads started.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `threading::start_slot_thread` for the slot at `slot` (0x5184F4), the audio thread's
//! (`g_audioThreadSlot`, 0x518604) and `stru_518714`. One caller, `start` (0x4010B1). In
//! the standalone program the threads whose routines are not ported end at once; the audio
//! thread's is (`audio::thread`). Shadow mode runs the original alone (threads cannot be
//! started twice). Self-test: a redirected copy whose three calls go to a stand-in noting
//! esi, 1 case.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("threading::start_threads", rva(0x410B84), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The three slots, in the order they are started.
pub const SLOTS: [usize; 3] = [0x5184F4, 0x518604, 0x518714];

/// Each slot's start, by its address.
pub fn start_threads(start: &mut dyn FnMut(u32) -> Result<(), Refused>) -> Result<(), Refused> {
    for slot in SLOTS {
        start(game_addr(rva(slot)) as u32)?;
    }
    Ok(())
}

fn game_start(slot: u32) -> Result<(), Refused> {
    unsafe { crate::threading::start_slot_thread::start_slot_thread(slot as *mut u8) }?;
    Ok(())
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    start_threads(&mut game_start)?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

static NOTED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    NOTED.lock().unwrap().push(r.esi);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x410B84,
            len: 0x24,
            entry: &[0x60, 0x8D, 0x35, 0xF4, 0x84, 0x51, 0x00],
            calls: &[0x07, 0x12, 0x1D],
            jumps_out: &[],
        },
        &|callee| (callee == rva(0x410C9C)).then(|| stand_in(0)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    NOTED.lock().unwrap().clear();
    let before = Regs::SENTINEL;
    let mut ro = before;
    unsafe { call_game(&mut ro, c.addr()) };
    let theirs = std::mem::take(&mut *NOTED.lock().unwrap());
    let done = start_threads(&mut |slot| {
        NOTED.lock().unwrap().push(slot);
        Ok(())
    });
    let mine = std::mem::take(&mut *NOTED.lock().unwrap());
    t.check(
        done.is_ok() && ro.preserved_except(&before, 0) && theirs == mine,
        || format!("original {theirs:x?}; rust {mine:x?}"),
    );
    t.finish()
}
