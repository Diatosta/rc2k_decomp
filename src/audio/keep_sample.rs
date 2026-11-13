//! Copies a voice's prepared data to the samples' memory (only for a voice with data, a length and
//! a `data_key`) and points the voice's data at the copy. The copy is Rust's
//! (`audio::sample_data::keep`) and the voice's data is its id.

use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::errors::Refused;

pub fn keep_sample(state: &AudioOwn, v: &mut Voice) -> Result<(), Refused> {
    if !rc2k_core::audio::buffers::keeps_sample(v) {
        return Ok(());
    }
    v.data = crate::audio::sample_data::keep(v.data, v.length, &state.sample_memory)?;
    Ok(())
}
