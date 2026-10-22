//! `scene::clear_fog` (0x44FC19): no fog.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! `dword_636610` = 0 and the 0x1000 bytes of `g_fogTable` = 0. (The original
//! clears the table first; the result is the same.)
//! One caller.
//! Self-test: the tables filled with another byte first, and put back afterwards.

use crate::errors::Refused;
use crate::framework::{
    EAX, ECX, EDI, RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::clear_fog", rva(0x44FC19), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 1] = [
    (rva(0x63661C), 0x1000, 0x00), // g_fogTable
];

/// Cleared first.
const CLEARED_RVA: usize = rva(0x636610); // dword_636610

// ---- Rust version ----

pub unsafe fn clear_fog() -> Left {
    unsafe {
        game_write(CLEARED_RVA, 0u32);
        for (table, len, byte) in FILLS {
            std::ptr::write_bytes(game_addr(table) as *mut u8, byte, len);
        }
    }
    let (last, len, _) = FILLS[FILLS.len() - 1];
    Left::nothing()
        .with(EAX, 0x0)
        .with(ECX, 0)
        .with(EDI, (game_addr(last) + len) as u32)
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions: Vec<Region> = FILLS
        .iter()
        .map(|&(table, len, _)| (game_addr(table) as u32, len))
        .collect();
    regions.push(at(CLEARED_RVA));
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { clear_fog() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    for byte in [0xAAu8, 0x00, 0xFF, 0x5C] {
        check_state(
            &mut t,
            &format!("filled with {byte:#x} before"),
            call_original,
            Regs::SENTINEL,
            &|| restore(&regions, &vec![byte; saved.len()]),
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
