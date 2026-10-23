//! `race::add_sort_entry` (0x454D57): appends an entry to `dword_9DB730` (0x1000 of 8
//! bytes, the count `dword_63DC50`): +4 ebx; +0 (eax << 16) | bx when (ebx & 0x7FFFFFFF)
//! is below 0x1000000, else 0.
//!
//! In:  eax (a kind byte), ebx (a key)
//! Out: edx the entry, ebx & 0x7FFFFFFF, eax as written (when written), CF clear (`clc`;
//!      its callers in `sub_454C99` jump on and pop ebx: not read)
//!
//! See `race/frame_draw.rs`. A reading: the faces to draw, sorted later by key. Refuses
//! (before anything) an entry past the 0x1000. Shadow mode: by state (the entry and the
//! count). Self-test: counts 0, 5, 0xFFF, keys below and above 0x1000000 and with the top
//! bit; all put back; 0x1000 refused.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_sort_entry(r.eax, r.ebx) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
