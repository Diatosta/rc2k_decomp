//! `net::keep_hud_options` (0x463E39): this machine's car's HUD bits (+0x220 of
//! `g_carBlocks[g_netPlayer]`) kept in `dword_75B00C` for the next race.
//!
//! In:  nothing
//! Out: esi = the car's block, eax = the bits; the rest and the flags kept
//! Caller: `net::lets_race`. Shadow mode: both run, the dword compared. Self-test: a copy
//! against the Rust version on a test block.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{PLAYER, Part, Saved, dword, p, region, run_both};
use crate::net::reset_cars::CAR_BLOCKS;
use crate::net::set_hud_options::{HUD_BITS, own_block};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::keep_hud_options", rva(0x463E39), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves.
pub fn keep_hud_options(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = own_block();
    r.eax = unsafe { ((r.esi + 0x220) as *const u32).read_unaligned() };
    p(HUD_BITS, r.eax);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &[dword(HUD_BITS)], 0, &|r| {
        keep_hud_options(r, &mut crate::net::lobby::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov esi, g_netPlayer
    let copy = match crate::net::lobby::copy(
        0x463E39,
        0x19,
        &[0x8B, 0x35, 0x20, 0xA9, 0x52, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let car = vec![0x1234_5678u32; 0x100];
    let at = car.as_ptr() as u32;
    let all = [region(CAR_BLOCKS, 0x20), dword(PLAYER), dword(HUD_BITS)];
    let saved = Saved::new(&all);
    run_both(
        &mut t,
        "a test block",
        copy.addr(),
        Regs::SENTINEL,
        &all,
        0,
        &[],
        &|| {
            p(PLAYER, 7);
            p(CAR_BLOCKS + 28, at);
            p(HUD_BITS, 0);
        },
        &keep_hud_options,
    );
    drop(saved);
    t.finish()
}
