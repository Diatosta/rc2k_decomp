//! `race::map_segment` (0x446036): a stretch of the best line, from one map anchor to the
//! next, placed on the map.
//!
//! In:  nothing (the anchors in `dword_6236C4`, `dword_6236C0`)
//! Out: see `race::best_line::map_segment`
//!
//! See `race/best_line.rs`. Callers: `race::map_checkpoints` (inside `pusha`) and the code
//! at 0x44601B (the whole line as one stretch). Refuses anchors past the lists. Self-test:
//! faces and checkpoints of the test's, the game's put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        map_segment(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

pub fn anchors(n: usize, f: &mut [u8]) {
    f.fill(0x66);
    let place = if n.is_multiple_of(4) {
        [(n * 3) as u16, (n * 5 + 1) as u16]
    } else {
        [0, 0]
    };
    f[0x2C..0x2E].copy_from_slice(&place[0].to_le_bytes());
    f[0x2E..0x30].copy_from_slice(&place[1].to_le_bytes());
}
