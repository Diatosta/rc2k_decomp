//! `effects::stage_record`: `effects::run_frame`'s pick of the record of `FrameWork.lead_kept[0]`
//! among `ViewAndRecords.view_vars.view_records[0]`'s 0x34 (0x5C bytes each; by their count and
//! `race::draw_car`'s clamp to 0x33, the stages: a reading).
//!
//! Refuses a number past the table (unsigned).

use crate::errors::Refused;
use crate::race::owner_frame::Frame;
use crate::race::view_vars_store::{ViewRecord, ViewVars};

/// The record.
// The logic is in `rc2k_core::effects::race::stage_record`
pub fn stage_record(frame: &Frame) -> Result<&ViewRecord, Refused> {
    let n = rc2k_core::effects::race::stage_record(frame.frame_work.borrow().lead_kept[0])?;
    Ok(ViewVars::record_of(frame.view_and_records.get(), n)?)
}
