//! `net::lets_race` (0x4640AA): the network race driven: `dword_63DB84` and the eight stored
//! cars' +0x74 (`g_carBlockStore.out`) 0, the HUD set (`net::set_hud_options`), the race
//! (`drive::play_race`), the HUD kept (`net::keep_hud_options`); then this machine's car
//! (`g_carBlocks[dword_63DBA0]`): eax its +0x88 (the race's time, it seems), ebx 0, or -1
//! when its +0xB0 is -1.
//!
//! In:  nothing
//! Out: eax, ebx as above, esi the car's block; the rest as the race leaves them; the flags
//!      of the last compare (no caller reads them)
//! Caller: `net::lets_rally`. Its calls through `part` (`net::lobby_calls`: the Rust
//! versions). Long-running (the race): ungated. Shadow mode runs the original alone.
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, +0xB0 -1 or
//! not, on a test block.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    PLAY_RACE, Part, Saved, dispatch_lobby, dword, fill, g, p, region, run_both,
};
use crate::net::reset_cars::CAR_BLOCKS;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::lets_race", rva(0x4640AA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SET_HUD: u32 = 0x463D3F; // net::set_hud_options
const KEEP_HUD: u32 = 0x463E39; // net::keep_hud_options
const CLEARED: usize = 0x63DB84; // dword_63DB84
const LOCAL: usize = 0x63DBA0; // dword_63DBA0
/// `g_carBlockStore.out` of the eight stored cars.
const OUTS: [usize; 8] = [
    0x70F49C, 0x71047C, 0x71145C, 0x71243C, 0x71341C, 0x7143FC, 0x7153DC, 0x7163BC,
];

/// The Rust version: the registers it leaves.
pub fn lets_race(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(CLEARED, 0);
    for x in OUTS {
        p(x, 0);
    }
    part(SET_HUD, &mut r)?;
    part(PLAY_RACE, &mut r)?;
    part(KEEP_HUD, &mut r)?;
    r.eax = g(LOCAL);
    r.esi = g(CAR_BLOCKS + 4 * r.eax as usize);
    let get = |offset: u32| unsafe { ((r.esi + offset) as *const u32).read_unaligned() };
    r.eax = get(0x88);
    // Converted: `rc2k_core::net::record::not_finished`.
    r.ebx = rc2k_core::net::record::not_finished(get(0xB0));
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, lets_race);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov dword_63DB84, 0
    let copy = match crate::net::lobby::copy(
        0x4640AA,
        0x89,
        &[0xC7, 0x05, 0x84, 0xDB, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0x5A, 0x5F, 0x64],
        &[SET_HUD, PLAY_RACE, KEEP_HUD],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut car = vec![0u32; 0x40];
    let at = car.as_mut_ptr() as u32;
    let mut all = vec![
        region(CAR_BLOCKS, 0x20),
        dword(CLEARED),
        dword(LOCAL),
        (at, 0x100),
    ];
    for x in OUTS {
        all.push(dword(x));
    }
    let saved = Saved::new(&all);
    for finished in [0u32, u32::MAX, 5] {
        run_both(
            &mut t,
            &format!("+0xB0 {finished:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                fill(CLEARED, 4, 62);
                p(LOCAL, 2);
                p(CAR_BLOCKS + 8, at);
                unsafe {
                    ((at + 0x88) as *mut u32).write(0x4321);
                    ((at + 0xB0) as *mut u32).write(finished);
                }
            },
            &lets_race,
        );
    }
    drop(saved);
    let _ = &mut car;
    t.finish()
}
