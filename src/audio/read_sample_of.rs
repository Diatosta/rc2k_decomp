//! `audio::read_sample_of` (0x40B1BE): a sample record's file read.
//!
//! In:  ebx = the sample record
//! Out: carry = not read (no path, or `audio::read_sample`'s carry); every register preserved
//!
//! The record's `data_key` is its file's path: none sets the carry, else
//! `audio::read_sample` on it. One caller, command 3 (`sub_40AEB9`).
//! Compared in shadow mode through `read_sample`'s own comparison (the original alone
//! here). Self-test: a redirected copy whose call notes eax and gives a carry, 4 cases.

use crate::audio::samples::read_sample_in_game;
use crate::audio::voices::SampleRecord;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::read_sample_of", rva(0x40B1BE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `read`: the reading of a path, true for found. Returns true for the carry clear.
pub unsafe fn read_sample_of(
    record: &SampleRecord,
    read: &mut dyn FnMut(u32) -> Result<bool, Refused>,
) -> Result<bool, Refused> {
    if record.data_key == 0 {
        return Ok(false);
    }
    read(record.data_key)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let record = unsafe { *(r.ebx as *const SampleRecord) };
    match unsafe { read_sample_of(&record, &mut |p| read_sample_in_game(p as *const u8)) } {
        Ok(found) => r.set_carry(!found),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

static NOTED: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static CARRY: Mutex<bool> = Mutex::new(false);

fn handler(_: u32, r: &mut Regs) {
    NOTED.lock().unwrap().push(r.eax);
    r.set_carry(*CARRY.lock().unwrap());
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B1BE,
            len: 0x12,
            entry: &[0x50, 0x8B, 0x43, 0x0C, 0x85, 0xC0],
            calls: &[0x8],
            jumps_out: &[],
        },
        &|callee| (callee == rva(0x40B21B)).then(|| stand_in(0)),
        &[],
        0,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    for (key, carry) in [(0u32, false), (0x1234, false), (0x1234, true), (0, true)] {
        let record = SampleRecord {
            data_key: key,
            ..SampleRecord::default()
        };
        *CARRY.lock().unwrap() = carry;
        NOTED.lock().unwrap().clear();
        let before = Regs {
            ebx: &record as *const SampleRecord as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (ro.carry(), std::mem::take(&mut *NOTED.lock().unwrap()));
        let mine = unsafe {
            read_sample_of(&record, &mut |p| {
                NOTED.lock().unwrap().push(p);
                Ok(!carry)
            })
        };
        let mine = (
            !mine.unwrap_or(false),
            std::mem::take(&mut *NOTED.lock().unwrap()),
        );
        t.check(theirs == mine && ro.preserved_except(&before, 0), || {
            format!("key {key:#x}, carry {carry}: original {theirs:?}; rust {mine:?}")
        });
    }
    t.finish()
}
