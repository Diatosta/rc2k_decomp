//! `race::object_bytes` (0x455263): four bytes for each of the 0x100 objects, read by
//! `sub_454C99`: `byte_63DC5C` record +0xE bit 0, `byte_63DD5C` 1, `byte_63DE5C` the kinds'
//! byte (`dword_5D3AE8`) of the record's kind (`dword_979F50`, 0x40 bytes each),
//! `byte_63DF5C` 0xFF, or with faces (`dword_632F70`) the kinds' byte of the first face's
//! (`g_faces[dword_632B70]`) +4 word.
//!
//! In:  nothing
//! Out: ebx 0x100, ecx 0x3FC0, eax the last kind value read (a `retn` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45487A` (0x4548A2). What the bytes mean is a
//! reading. The original reads where the tables point, unchecked; so does the Rust
//! version. Shadow mode: by state (the four 0x100-byte tables). Self-test: a copy of the
//! original whose tables are the test's (13 references), on patterns, against the Rust
//! core on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { object_bytes() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
