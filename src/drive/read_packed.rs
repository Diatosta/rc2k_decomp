//! `drive::read_packed` (0x41EEB5): ecx bytes from the file eax into ebx: a count of 0 or
//! more is a plain read (its `jns` goes straight to `files::read`); below 0, -ecx packed
//! bytes read into `g_pngInfo` and unpacked into ebx (`drive::unpack_rle`), eax the bytes
//! unpacked.
//!
//! In:  eax the file, ebx the buffer, ecx the count (below 0: packed)
//! Out: eax the bytes read or unpacked, CF set when the read failed (ZF from `test eax`
//!      after an unpacking); a plain read keeps ebx, ecx, edx, a packed one every register
//!      but eax
//!
//! Caller: `drive::read_replay_frame`. It begins with a jump out (`jns files::read`):
//! hooked by a private copy (0x2F bytes, the jump and the two calls relocated) and a
//! hand-made patch.
//! Refuses what `unpack_rle` refuses (nothing written then but the packed bytes).
//! Shadow mode: the file is put back where it was after the Rust version's read.
//! Self-test: the original and the Rust version on a test file (written to the temp
//! folder, read from its start each time): plain and packed reads.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::files::handle::read;
use crate::framework::{EAX, Regs};
use crate::physics::flow::a;
use crate::screens::harness::Left;

/// `g_pngInfo`: the packed bytes' buffer.
pub const PACKED: usize = 0x93E8B0;

/// The Rust version: (eax, CF, ZF when it unpacked).
// Converted: `rc2k_core::drive::race_steps::packed_bytes` (the unpacking:
// `rc2k_core::drive::rle`).
pub unsafe fn read_packed(
    file: u32,
    buffer: u32,
    count: u32,
) -> Result<(u32, bool, Option<bool>), Refused> {
    let Some(packed) = rc2k_core::drive::race_steps::packed_bytes(count) else {
        let (n, ok) = unsafe { read(file, buffer as *mut u8, count) };
        return Ok((n, !ok, None));
    };
    let (n, ok) = unsafe { read(file, a(PACKED) as *mut u8, packed) };
    if !ok {
        return Ok((n, true, None));
    }
    let out = unsafe { crate::drive::unpack_rle::unpack_rle(a(PACKED), buffer) }?;
    Ok((out, false, Some(out == 0)))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (eax, carry, _) = unsafe { read_packed(r.eax, r.ebx, r.ecx) }?;
    Ok(Left::carry(carry).with(EAX, eax))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
