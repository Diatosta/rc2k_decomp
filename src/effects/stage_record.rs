//! `effects::stage_record` (0x4D2DE0): `effects::run_frame`'s pick of the record of
//! `dword_64165C` among `off_606320`'s 0x34 (0x5C bytes each; by their count and
//! `sub_4616AA`'s clamp to 0x33, the stages: a reading).
//!
//! In:  nothing
//! Out: the record (the original's esi; returned, read by no caller)
//!
//! One caller, `effects::run_frame`. Refuses a number past the table (unsigned).

use crate::errors::Refused;
use crate::race::frame_store::FRAME_WORK;
use crate::race::view_vars_store::ViewVars;

/// The Rust version (see the header): the record's pointer (the original's esi; read by
/// no caller).
// Converted: `rc2k_core::effects::race::stage_record`
pub fn stage_record() -> Result<u32, Refused> {
    let n = rc2k_core::effects::race::stage_record(FRAME_WORK.lead_kept[0].bits())?;
    Ok(ViewVars::record_of(n)?)
}
