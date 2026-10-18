//! `scene::kind_hidden_at` (0x4522F7): `scene::kind_is_hidden` for the object whose
//! number is the byte at esi+1, eax kept (`push eax` / `pop eax`).
//!
//! In:  esi the record (its byte +1 the object)
//! Out: CF as `kind_is_hidden` leaves it; every register kept
//!
//! Caller: `sub_45E3BF` (0x45E3D0, `jb`).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): both settings, a test record with objects 0 .. 0xFF and test kinds.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::scene::kind_is_hidden::kind_is_hidden;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::kind_hidden_at", rva(0x4522F7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: whether the record's object is hidden by its kind.
pub unsafe fn kind_hidden_at(record: u32) -> bool {
    let object = unsafe { ((record + 1) as *const u8).read() } as u32;
    // A byte: never past the table.
    unsafe { kind_is_hidden(object) }.is_ok_and(|(hidden, _)| hidden)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { kind_hidden_at(r.esi) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all: Vec<Region> = vec![(a(0x71C3E0), 4), (a(0x62389C), 4), (a(0x5CD27C), 0x400)];
    let saved = capture(&all);
    for k in 0..0x100usize {
        let kind = [0x4Eu32, 0x30, 7, 0x2F, 0x31, 0x4D, 0x4F, 0x130][k % 8];
        unsafe { game_write(rva(0x5CD27C + 4 * k), kind) };
    }
    let mut record = [0xAAu8; 4];
    for (switch, mode) in [(0u32, 0u32), (0, 2), (1, 0), (1, 2)] {
        for object in (0..0x100u32).step_by(5) {
            record[1] = object as u8;
            let at = record.as_ptr() as u32;
            let set = || unsafe {
                game_write(rva(0x71C3E0), switch);
                game_write(rva(0x62389C), mode);
            };
            check_state(
                &mut t,
                &format!("switch {switch}, mode {mode}, object {object:#x}"),
                call_original,
                Regs {
                    esi: at,
                    ..Regs::SENTINEL
                },
                &set,
                &[],
                &rust,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
