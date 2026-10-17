//! `race::permute_bits` (0x402848): edx:eax's bits picked by a table at ebx (a count
//! n, then n + 1 picks: bit 5 set eax's bit (b & 0x1F), else edx's) gathered into
//! esi:edi from the top down.
//!
//! In:  eax, edx = the bits, ebx = the table
//! Out: esi:edi = the result, ebx = past the table; the rest preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_402735` five times (the save's sealing;
//! none reads its flags). Shadow mode: the Rust version and the original, registers
//! compared (nothing written). Self-test: the game's five tables and tables of the
//! test's (counts 0, 0x3F, high bits set in the picks), values by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::race_shell::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::permute_bits", rva(0x402848), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    Vec::new()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { permute_bits(r) };
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

#[allow(unused_variables)] // not every test uses every helper
fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let _ = (
        game_addr(0),
        at(0),
        game_write::<u32> as unsafe fn(usize, u32),
    );
    let regions = regions();
    let mut tables: Vec<Vec<u8>> = vec![
        vec![0],
        vec![3, 0x20, 0x1F, 0x3F, 0xC1],
        (0..0x41u32)
            .map(|n| {
                if n == 0 {
                    0x3F
                } else {
                    ((n * 37) & 0xFF) as u8
                }
            })
            .collect(),
    ];
    for at in [0x4E015Cusize, 0x4E0179, 0x4E0196, 0x4E01D7, 0x4E0218] {
        let p = game_addr(rva(at)) as *const u8;
        let n = unsafe { *p } as usize + 2;
        tables.push(unsafe { std::slice::from_raw_parts(p, n) }.to_vec());
    }
    let mut seed = 0x2026_1017u32;
    for (k, table) in tables.iter().enumerate() {
        for _ in 0..4 {
            seed = seed.wrapping_mul(0x0101_0101).wrapping_add(0x3C6E_F35F);
            let (eax, edx) = (seed, seed.rotate_left(13) ^ 0xA5A5_5A5A);
            check_state(
                &mut t,
                &format!("table {k}, eax {eax:#x}, edx {edx:#x}"),
                call_original,
                Regs {
                    eax,
                    edx,
                    ebx: table.as_ptr() as u32,
                    ..Regs::SENTINEL
                },
                &|| {},
                &regions,
                &rust,
            );
        }
    }
    t.finish()
}
