//! `scene::clear_slot_textures` (0x4256A8): no texture slot holds or wants a texture.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `g_slotLoadedTextures` and `g_slotWantedTextures`, 0x400 dwords
//! each, so that every slot is loaded anew.
//! One caller, `scene::reload_slot_textures`.
//! Self-test: the tables filled with another byte first, and put back afterwards.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::clear_slot_textures",
    rva(0x4256A8),
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
    (rva(0x5DB968), 0x1000, 0xFF), // g_slotLoadedTextures
    (rva(0x5DA968), 0x1000, 0xFF), // g_slotWantedTextures
];

// ---- Rust version ----

pub unsafe fn clear_slot_textures() -> Left {
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
    Ok(unsafe { clear_slot_textures() })
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
