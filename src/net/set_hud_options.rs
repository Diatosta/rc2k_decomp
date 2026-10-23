//! `net::set_hud_options` (0x463D3F): this machine's car (`g_carBlocks[g_netPlayer]`) given
//! the player's HUD: its bits (+0x220) from `dword_75B00C`, or, when that is -1, made from
//! the switches (`dword_75ADF8` 0x10, `75ADFC` 8, `75AE00` 1, `75AE08` 4, `75AE14` 2,
//! `75AE04` 0x100, `75AE10` 0x40, `75AE0C` 0x20, `75AE18` 0x80, each when not 0); its view
//! (+0x684) from `dword_75AE24`; `dword_623898` from `dword_4F50D0`; five more settings
//! (`dword_75AD00`, `75AD0C` .. `75AD18`) to `dword_6238A8` ...
//!
//! In:  nothing
//! Out: esi = the car's block, eax = its HUD bits; with the switches read, ebx = the last
//!      (`dword_75AE18`); the flags of the last compare (no caller reads them)
//! Caller: `net::lets_race`. Shadow mode: both run, the block's two dwords and the
//! settings compared. Self-test: a copy against the Rust version, on bits given and on
//! the switches in two patterns, on a test block.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{PLAYER, Part, Saved, dword, fill, g, p, region, run_both};
use crate::net::reset_cars::CAR_BLOCKS;
use rc2k_core::net::cars;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::set_hud_options", rva(0x463D3F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const HUD_BITS: usize = 0x75B00C; // dword_75B00C
/// The switches, in the original's order (their bits: `rc2k_core::net::cars::HUD_SWITCH_BITS`).
const SWITCHES: [usize; 9] = [
    0x75ADF8, 0x75ADFC, 0x75AE00, 0x75AE08, 0x75AE14, 0x75AE04, 0x75AE10, 0x75AE0C, 0x75AE18,
];
const VIEW: usize = 0x75AE24;
/// The settings copied: (to, from).
const COPIED: [(usize, usize); 6] = [
    (0x623898, 0x4F50D0),
    (0x6238A8, 0x75AD00),
    (0x6238AC, 0x75AD0C),
    (0x6238B0, 0x75AD10),
    (0x6238B4, 0x75AD14),
    (0x6238B8, 0x75AD18),
];

/// This machine's car's block.
pub fn own_block() -> u32 {
    g(CAR_BLOCKS + 4 * g(PLAYER) as usize)
}

/// The Rust version: the registers it leaves.
pub fn set_hud_options(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = own_block();
    r.eax = g(HUD_BITS);
    // Converted: `rc2k_core::net::cars::hud_bits`.
    if r.eax == cars::NOT_KEPT {
        let switches = SWITCHES.map(g);
        r.eax = cars::hud_bits(switches);
        r.ebx = switches[8];
    }
    unsafe {
        ((r.esi + 0x220) as *mut u32).write_unaligned(r.eax);
        ((r.esi + 0x684) as *mut u32).write_unaligned(g(VIEW));
    }
    for (to, from) in COPIED {
        p(to, g(from));
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    let block = own_block();
    let regions = [
        (block + 0x220, 4),
        (block + 0x684, 4),
        region(0x623898, 4),
        region(0x6238A8, 0x14),
    ];
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &regions, 0, &|r| {
        set_hud_options(r, &mut crate::net::lobby::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        set_hud_options(r, &mut crate::net::lobby::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov esi, g_netPlayer
    let copy = match crate::net::lobby::copy(
        0x463D3F,
        0xFA,
        &[0x8B, 0x35, 0x20, 0xA9, 0x52, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut car = vec![0u8; 0x700];
    let at = car.as_mut_ptr() as u32;
    let all = [
        region(CAR_BLOCKS, 0x20),
        dword(PLAYER),
        dword(HUD_BITS),
        region(0x75ADF8, 0x30),
        region(0x623898, 4),
        region(0x6238A8, 0x14),
        (at, 0x700),
    ];
    let saved = Saved::new(&all);
    for (bits, pattern) in [(0x1234u32, 0u32), (u32::MAX, 0x155), (u32::MAX, 0xAA)] {
        run_both(
            &mut t,
            &format!("bits {bits:#x}, switches {pattern:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                fill(0x75ADF8, 0x30, 61);
                for (n, switch) in SWITCHES.iter().enumerate() {
                    p(
                        *switch,
                        if pattern & (1 << n) != 0 {
                            0x10 + n as u32
                        } else {
                            0
                        },
                    );
                }
                p(HUD_BITS, bits);
                p(PLAYER, 3);
                p(CAR_BLOCKS + 12, at);
                unsafe { std::ptr::write_bytes(at as *mut u8, 0x77, 0x700) };
            },
            &set_hud_options,
        );
    }
    drop(saved);
    let _ = &mut car;
    t.finish()
}
