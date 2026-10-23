//! `race::checkpoint_colours` (0x4548E3): from the current checkpoint's (`dword_71A070`)
//! face (`g_checkpointFaces`, 0x40-byte records at `g_courseFaces`): its +0x10 byte's low
//! and high nibbles (0 read as 0x10) to `dword_614D08` / `dword_614D0C`, `dword_614D10` =
//! `dword_9ED808[checkpoint]` - `dword_71A998`, and `dword_63F17C` the low nibble (twice
//! the distance below 0x40000000, signed), the high (from 0xC0000000), else the larger.
//!
//! In:  nothing
//! Out: eax that, ebx the checkpoint, esi the face (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4548A8` (0x4548D8), each view drawn. A reading:
//! colours (fog levels?) blended by the distance to the checkpoint. The tables are read
//! unchecked, as the original. Shadow mode: by state. Self-test: the variables and table
//! entries set (the face list pointed at a test record), distances around the bounds; all
//! put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// dword_71A070
// g_checkpointFaces
// g_courseFaces (a pointer)
// dword_9ED808
// dword_71A998

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { checkpoint_colours() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
