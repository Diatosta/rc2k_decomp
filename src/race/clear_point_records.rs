//! `race::clear_point_records` (0x4500FF): `dword_5C6FC4` 0, then each of the 0x1000
//! point records (0x20 bytes) at `g_pointRecords`: its first dword -1, or with `g_useSse`
//! set its first 16 bytes `xmmword_62DAE0` (`movntps`; the Rust version writes the same
//! bytes with an ordinary store).
//!
//! In:  nothing
//! Out: ecx 0, edi past the records (callers: a `retn` at 0x4500F9, a `call` in
//!      `sub_4557AB`; no flags read)
//!
//! See `race/frame_draw.rs`. Shadow mode: by state (the records' 0x20000 bytes and the
//! dword). Self-test: a copy of the original whose three variables are the test's (4
//! references), on a 16-aligned test area filled with 0xAA, with and without SSE, against
//! the Rust core on the same.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { clear_point_records() })
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
