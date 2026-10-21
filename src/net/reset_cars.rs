//! `net::reset_cars` (0x463E52): the eight cars' blocks (`g_carBlocks`) made ready for a
//! network race: +0xAA8 the car's number + 1, +0x684 4, +0x28 2, +0x34, +0x30, +0x20,
//! +0x3C 0x80, then `net::clear_car_state`; `dword_6238CC` and `dword_6238D0` 0.
//!
//! In:  nothing
//! Out: ecx = 8; the rest kept; the flags of `cmp ecx, 8` (no caller reads them)
//! Caller: `net::set_up_race`. Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy whose call goes to a stand-in, against the Rust version, on
//! test blocks put in `g_carBlocks`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{Part, Saved, dispatch_lobby, dword, g, p, region, run_both};
use rc2k_core::net::cars;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::reset_cars", rva(0x463E52), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CAR_BLOCKS: usize = 0x71B3B8; // g_carBlocks: eight pointers
const CLEAR_CAR_STATE: u32 = 0x463EB6; // net::clear_car_state
const AFTER: [usize; 2] = [0x6238CC, 0x6238D0]; // dword_6238CC, dword_6238D0

/// The Rust version: the registers it leaves.
pub fn reset_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    for car in 0..cars::CARS {
        let mut c = r;
        c.ecx = car;
        c.esi = g(CAR_BLOCKS + 4 * car as usize);
        c.eax = car + 1;
        // Converted: `rc2k_core::net::cars::reset_fields`.
        for (offset, v) in cars::reset_fields(car) {
            unsafe { ((c.esi + offset) as *mut u32).write_unaligned(v) };
        }
        part(CLEAR_CAR_STATE, &mut c)?;
        r.eflags = c.eflags;
    }
    r.ecx = 8;
    for x in AFTER {
        p(x, 0);
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, reset_cars);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov ecx, 0
    let copy = match crate::net::lobby::copy(
        0x463E52,
        0x64,
        &[0xB9, 0x00, 0x00, 0x00, 0x00],
        &[0x43],
        &[CLEAR_CAR_STATE],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut blocks = vec![0u8; 8 * 0xB00];
    let at = blocks.as_mut_ptr() as u32;
    let all = [
        region(CAR_BLOCKS, 0x20),
        dword(AFTER[0]),
        dword(AFTER[1]),
        (at, 8 * 0xB00),
    ];
    let saved = Saved::new(&all);
    run_both(
        &mut t,
        "eight test blocks",
        copy.addr(),
        Regs::SENTINEL,
        &all,
        0,
        &[],
        &|| unsafe {
            for n in 0..8 * 0xB00u32 {
                ((at + n) as *mut u8).write((n >> 2) as u8);
            }
            for car in 0..8u32 {
                p(CAR_BLOCKS + 4 * car as usize, at + 0xB00 * car);
            }
            p(AFTER[0], 5);
            p(AFTER[1], 6);
        },
        &reset_cars,
    );
    drop(saved);
    let _ = &mut blocks;
    t.finish()
}
