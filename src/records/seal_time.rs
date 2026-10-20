//! `records::seal_time` (0x40264D): a time sealed for the records. The time eax, at most
//! 0x22550FF; negated when a cheat of 0x16000 is on, or negated and bit 30 cleared when
//! cheat 0x40000 is on or `g_patchSwitch` is set; into the block's first dword
//! (`dword_4E0534`), bl / cl / dl its next three bytes, its eighth byte 0x5A plus the
//! seven before it each inverted (a byte sum); then `race::seal_block`.
//!
//! In:  eax the time, bl, cl, dl three bytes (the car, the stage, ...)
//! Out: eax `value`, ebx `dword_4E0540` (the sealed block); the rest kept
//!
//! Caller: `sub_4A3D38`. The cipher is its Rust version. Self-test: the original and the
//! Rust version on times and cheats by lot (the key as the game has it); the block, the
//! subkeys and the result compared; put back.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::save_seal::{BLOCK_RVA, SEALED_RVA, seal_block, seal_regions};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("records::seal_time", rva(0x40264D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const PATCH_SWITCH_RVA: usize = rva(0x71C6D0); // g_patchSwitch
/// The time as sealed (converted: `rc2k_core::records::sealed_value`).
pub fn sealed_value(time: u32, cheats: u32, patch_switch: u32) -> u32 {
    rc2k_core::records::sealed_value(time, cheats, patch_switch != 0)
}

/// The block's eight bytes (converted: `rc2k_core::records::seal_block`).
pub use rc2k_core::records::seal_block as block;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn seal_time(entry: &Regs) -> Regs {
    unsafe {
        let value = sealed_value(
            entry.eax,
            game_read(CHEATS_RVA),
            game_read(PATCH_SWITCH_RVA),
        );
        let b = block(value, [entry.ebx as u8, entry.ecx as u8, entry.edx as u8]);
        game_write(BLOCK_RVA, b);
        seal_block();
        Regs {
            eax: game_read(SEALED_RVA[0]),
            ebx: game_read(SEALED_RVA[1]),
            ..*entry
        }
    }
}

fn regions() -> Vec<Region> {
    let mut v = seal_regions();
    v.push((crate::framework::game_addr(BLOCK_RVA) as u32, 8));
    v
}

fn rust(r: &Regs) -> Result<Left, crate::errors::Refused> {
    Ok(left_of(&unsafe { seal_time(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([CHEATS_RVA, PATCH_SWITCH_RVA].map(at));
    let saved = capture(&vars);
    let times = [
        0u32,
        1,
        0x1234_5678,
        rc2k_core::records::LONGEST_TIME,
        0x22551000,
        0x7FFF_FFFF,
        0xFFFF_FFFF,
    ];
    for (n, &time) in times.iter().enumerate() {
        let cheats = [0u32, 0x2000, 0x40000, 0, 0x10000, 0x40000 | 0x4000, 0][n];
        let switch = u32::from(n == 3 || n == 6);
        check_state(
            &mut t,
            &format!("time {time:#x}, cheats {cheats:#x}, switch {switch}"),
            call_original,
            Regs {
                eax: time,
                ebx: 0x1100 + n as u32 * 0x3B,
                ecx: 0x2200 + n as u32 * 0x29,
                edx: 0xFF - n as u32,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&vars, &saved);
                game_write(CHEATS_RVA, cheats);
                game_write(PATCH_SWITCH_RVA, switch);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
