//! `race::split_colours` (0x45B16A): a colour per split for the panel: `dword_601830`
//! dwords at edi set to 0xC, then for each of the block esi's +0x90 values at +0x140, 8 when
//! it is not above the one before (0x63 before the first), else 4.
//!
//! In:  esi = a view block, edi = the colours (`dword_6D160C` / `dword_6D170C`, 0x40 dwords)
//! Out: ecx = +0x90; eax the last colour and edx the last value when there was one (flags
//!      not read: `mov` / `retn` follow)
//!
//! See `race/frame_draw.rs`. Callers: `sub_45B147` (0x45B153, 0x45B164), once per view.
//! The values are `CarBlock` +0x140, a dword per checkpoint (written by `sub_436833`),
//! `check_count` (+0x90) of them; what they hold (places at each split) is a reading. Refuses
//! (before anything) a count past the buffers' 0x40 dwords. Shadow mode: by state (the 0x40
//! dwords at edi). Self-test: a test block and buffer (0xAA), `dword_601830` set and put
//! back, counts 0, negative, 1, 5, 0x40 with rising, falling, equal and far values.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { split_colours(r.esi, r.edi, game_read(MARK_COUNT_RVA)) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
