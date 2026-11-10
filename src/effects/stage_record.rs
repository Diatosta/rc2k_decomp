//! `effects::stage_record` (0x4D2DE0): `effects::run_frame`'s pick of the record of
//! `dword_64165C` among `off_606320`'s 0x34 (0x5C bytes each; by their count and
//! `race::draw_car`'s clamp to 0x33, the stages: a reading).
//!
//! One caller, `effects::run_frame`. Refuses a number past the table (unsigned).

use crate::errors::Refused;
use crate::race::owner_frame::Frame;
use crate::race::view_vars_store::{ViewRecord, ViewVars};

/// The record (the original's esi; read by no
/// caller).
// Converted: `rc2k_core::effects::race::stage_record`
pub fn stage_record(frame: &Frame) -> Result<&'static ViewRecord, Refused> {
    let n = rc2k_core::effects::race::stage_record(frame.frame_work.lead_kept[0].bits())?;
    Ok(ViewVars::record_of(n)?)
}
