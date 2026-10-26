//! `drive::any_key_down` (0x444F55): whether any of the 0x80 key bytes (`g_keysPressed`)
//! is set: their or into eax, CF set when not 0.
//!
//! In:  nothing
//! Out: CF; eax the bytes' or, ecx 0x80, esi `g_keysPressed`; the rest kept
//!
//! Caller: `drive::race_frame` (`sub_444D46`, `jb`).
//! Self-test: the original and the Rust version on the key bytes (saved and put back):
//! none, one, several.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, Regs};
use crate::physics::flow::a;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::inputs::{KEY_COUNT, keys_or};

const KEYS: usize = 0x4F6CDC; // g_keysPressed

/// The Rust version: (the bytes' or, esi).
// Converted: `rc2k_core::drive::inputs::keys_or`.
pub unsafe fn any_key_down() -> (u32, u32) {
    let at = a(KEYS);
    let keys = unsafe { (at as *const [u8; KEY_COUNT]).read_unaligned() };
    (keys_or(&keys) as u32, at)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let (or, esi) = unsafe { any_key_down() };
    Ok(Left::carry(or != 0)
        .with(EAX, or)
        .with(ECX, 0x80)
        .with(ESI, esi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
