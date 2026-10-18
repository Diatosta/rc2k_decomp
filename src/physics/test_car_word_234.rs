//! `physics::test_car_word_234` (0x43FDD5): `cmp` of `g_car` +0x234 with 0, then
//! `jz $+2` and `retn` either way: only the flags change (ZF when 0, CF clear). Its one
//! caller reads none of them (the next flag reader follows a `cmp`): by the look of it a
//! test whose work was cut out.
//!
//! In:  nothing
//! Out: every register kept; ZF = the word is 0, CF clear
//!
//! Caller: `sub_43B635` (0x43B96B), every physics step.
//! Self-test: a copy of the original (the word the test's) on 0, 1, -1, 0x80000000.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_read, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::test_car_word_234",
    rva(0x43FDD5),
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

const WORD_RVA: usize = rva(0x71A20C); // g_car + 0x234

/// The core: the word's value, eax at entry.
pub fn test_word(word: u32, eax: u32) -> FlagResult {
    FlagResult {
        eax,
        ecx: None,
        ebx: None,
        carry: false,
        zero: Some(word == 0),
    }
}

/// The Rust version.
pub unsafe fn test_car_word_234(eax: u32) -> FlagResult {
    test_word(unsafe { game_read(WORD_RVA) }, eax)
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { test_car_word_234(r.eax) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cell = vec![0u32; 1];
    let w = cell.as_mut_ptr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43FDD5,
            len: 0xA,
            entry: &[0x83, 0x3D, 0x0C, 0xA2, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x71A20C, w as u32)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for v in [0u32, 1, u32::MAX, 0x8000_0000] {
        unsafe { w.write_volatile(v) };
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let mine = test_word(unsafe { w.read_volatile() }, Regs::SENTINEL.eax);
        let ok =
            mine.matches(&Regs::SENTINEL, &ro) && crate::audio::buffer_play::zero(&ro) == (v == 0);
        t.check(ok, || format!("word {v:#x}: the registers or flags differ"));
    }
    drop((copy, cell));
    t.finish()
}
