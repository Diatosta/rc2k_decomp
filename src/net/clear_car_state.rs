//! `net::clear_car_state` (0x463EB6): a car's block made ready for a network race: 51 of
//! its dwords (+0x64 .. +0x80, +0x2B8 .. +0x364) 0, +0x298 -1.
//!
//! In:  esi = the car's block
//! Out: every register and flag kept (only `mov`s; the flags are not compared: `call_game`
//!      does not load them)
//! Caller: `net::reset_cars`. Shadow mode: both run, the block's 0x370 bytes compared.
//! Self-test: a copy against the Rust version on a test block.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{Part, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::clear_car_state", rva(0x463EB6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the block at `block`.
pub fn clear_block(block: u32) {
    // Converted: `rc2k_core::net::cars::CLEARED`.
    for (offset, v) in rc2k_core::net::cars::CLEARED {
        unsafe { ((block + offset) as *mut u32).write_unaligned(v) };
    }
}

/// The Rust version: the registers it leaves (the entry's).
pub fn clear_car_state(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    clear_block(entry.esi);
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.esi, 0x370)];
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &regions, 0, &|r| {
        clear_car_state(r, &mut crate::net::lobby::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        clear_car_state(r, &mut crate::net::lobby::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov dword ptr [esi+2B8h], 0
    let copy = match crate::net::lobby::copy(
        0x463EB6,
        0x1F4,
        &[0xC7, 0x86, 0xB8, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut block = vec![0u8; 0x400];
    let at = block.as_mut_ptr() as u32;
    {
        run_both(
            &mut t,
            "a test block",
            copy.addr(),
            Regs {
                esi: at,
                ..Regs::SENTINEL
            },
            &[(at, 0x400)],
            0,
            &[],
            &|| unsafe {
                for n in 0..0x400u32 {
                    ((at + n) as *mut u8).write((n as u8) ^ 0x3C);
                }
            },
            &clear_car_state,
        );
    }
    let _ = &mut block;
    t.finish()
}
