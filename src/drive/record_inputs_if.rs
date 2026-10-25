//! `drive::record_inputs_if` (0x46008D): `drive::record_inputs` while recording a replay
//! (`dword_5C5CE0` 1) for the cars below `dword_63DB94`.
//!
//! In:  nothing
//! Out: while recording eax `dword_63DB94`, then as `record_inputs` leaves the registers
//!      when called; no flags read
//!
//! Caller: `drive::replay_step` (`sub_460B2F`).

use crate::race::numbers_store::NUMBERS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::records_inputs`.
pub unsafe fn record_inputs_if() {
    let records = rc2k_core::drive::replay_frames::records_inputs(
        NUMBERS.replay_mode.get(),
        RACE_WORK.current_car.get(),
        || RACE_VIEW.players.get(),
    );
    if records == Some(true) {
        unsafe { crate::drive::record_inputs::record_inputs() };
    }
}
