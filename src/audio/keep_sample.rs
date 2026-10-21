//! `audio::keep_sample` (0x40B192): a voice's prepared data copied to the samples' memory.
//!
//! In:  esi = the voice
//! Out: every register preserved
//!
//! Only for a voice with data, a length and a `data_key`: `length` bytes from the samples'
//! memory (`sub_409DFB`, inlined: `memory::allocate` on `g_sampleMemory`; its fatal
//! error 0x80E28021 cannot come), the data copied there and the voice's data pointed at it. One caller,
//! command 3 (`sub_40AEB9`).
//! Shadow mode runs the original alone (memory would be taken twice). Self-test: a
//! redirected copy whose allocation is the test's, 4 cases.

use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::keep_sample", rva(0x40B192), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `allocate`: `length` bytes of the samples' memory.
pub unsafe fn keep_sample(
    v: &mut Voice,
    allocate: &mut dyn FnMut(u32) -> Result<u32, Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::buffers::keeps_sample`.
    if !rc2k_core::audio::buffers::keeps_sample(v) {
        return Ok(());
    }
    let to = allocate(v.length)?;
    unsafe { std::ptr::copy(v.data as *const u8, to as *mut u8, v.length as usize) };
    v.data = to;
    Ok(())
}

pub unsafe fn allocate_in_game(length: u32) -> Result<u32, Refused> {
    let list = game_addr(SAMPLE_MEMORY_RVA) as *mut _;
    // The allocator reports a failure itself (0x80108571), so `sub_409DFB`'s own fatal
    // error 0x80E28021 cannot come.
    Ok(unsafe { crate::memory::allocate::allocate(list, length) }? as u32)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { keep_sample(voice(r.esi), &mut |n| allocate_in_game(n)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

static GIVEN: AtomicU32 = AtomicU32::new(0);
static ASKED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    ASKED.lock().unwrap().push(r.eax);
    r.eax = GIVEN.load(Ordering::SeqCst);
    r.set_carry(false);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B192,
            len: 0x2C,
            entry: &[0x60, 0x83, 0x7E, 0x1C, 0x00],
            calls: &[0x16],
            jumps_out: &[],
        },
        &|callee| (callee == rva(0x409DFB)).then(|| stand_in(0)),
        &[],
        0,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let source: Vec<u8> = (0..0x100u32).map(|i| (i * 3) as u8).collect();
    let mut memory = vec![0u8; 0x100];
    for (data, length, key) in [(1u32, 0x40u32, 1u32), (0, 0x40, 1), (1, 0, 1), (1, 0x80, 0)] {
        let make = || Voice {
            data: if data == 1 { source.as_ptr() as u32 } else { 0 },
            length,
            data_key: key,
            ..Voice::default()
        };
        GIVEN.store(memory.as_mut_ptr() as u32, Ordering::SeqCst);
        memory.fill(0xAA);
        ASKED.lock().unwrap().clear();
        let mut v = make();
        let before = Regs {
            esi: &mut v as *mut Voice as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            v,
            memory.clone(),
            std::mem::take(&mut *ASKED.lock().unwrap()),
        );
        memory.fill(0xAA);
        let mut v = make();
        let mine = unsafe {
            keep_sample(&mut v, &mut |n| {
                ASKED.lock().unwrap().push(n);
                Ok(memory.as_mut_ptr() as u32)
            })
        };
        let mine2 = (
            v,
            memory.clone(),
            std::mem::take(&mut *ASKED.lock().unwrap()),
        );
        t.check(
            mine.is_ok() && theirs == mine2 && ro.preserved_except(&before, 0),
            || {
                format!(
                    "data {data}, length {length:#x}, key {key}: original {:x?}; rust {:x?}",
                    theirs.0, mine2.0
                )
            },
        );
    }
    t.finish()
}
