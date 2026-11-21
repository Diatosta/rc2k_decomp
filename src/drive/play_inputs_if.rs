//! `drive::play_inputs` while playing a replay (`replay_mode`
//! 2).

use crate::race::owner::Race;
pub(super) fn play_inputs_if(race: &Race) -> Result<(), crate::errors::BadInput> {
    if rc2k_core::drive::replay_frames::plays_inputs(
        race.course.number_vars.get().replay_mode.get(),
    ) {
        crate::drive::play_inputs::play_inputs(race)?;
    }
    Ok(())
}
