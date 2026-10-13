//! `scene::reset_texture_places` (0x423FE1): no texture has a place.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `dword_5D9968` and `dword_5D8968` (each texture's place), 0x400
//! dwords each.
//! Two callers.
//! Self-test: the tables filled with another byte first, and put back afterwards.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::reset_texture_places",
    rva(0x423FE1),
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

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 2] = [
    (rva(0x5D9968), 0x1000, 0xFF), // dword_5D9968
    (rva(0x5D8968), 0x1000, 0xFF), // dword_5D8968
];

// ---- Rust version ----

pub unsafe fn reset_texture_places() -> Left {
    unsafe {
        for (table, len, byte) in FILLS {
            std::ptr::write_bytes(game_addr(table) as *mut u8, byte, len);
        }
    }
    let (last, len, _) = FILLS[FILLS.len() - 1];
    Left::nothing()
        .with(EAX, 0xffffffff)
        .with(ECX, 0)
        .with(EDI, (game_addr(last) + len) as u32)
}

// ---- hook ----

fn regions() -> Vec<Region> {
    FILLS
        .iter()
        .map(|&(table, len, _)| (game_addr(table) as u32, len))
        .collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { reset_texture_places() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
