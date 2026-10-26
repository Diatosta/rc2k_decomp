//! `race::flush_triangle_batch` (0x45FDCF): texture `dword_5D1D04`, the batch's render states, its triangles drawn by the device and the batch emptied (see `race/frame_flush.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! The device's draw (`sub_41ADF5`) and the log's writers are the game's (section 15).
//! Caller: `sub_45429B`. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version with the same: the batch
//! empty, drawn, and failing (logged).

use crate::framework::Regs;
use crate::race::frame_flush::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { flush_batch(r, &PLAIN, false, &mut batch_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
