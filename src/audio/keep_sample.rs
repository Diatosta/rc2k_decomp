//! `audio::keep_sample` (0x40B192): a voice's prepared data copied to the samples' memory.
//!
//! In:  esi = the voice
//! Out: every register preserved
//!
//! Only for a voice with data, a length and a `data_key`: `length` bytes from the samples'
//! memory (`sub_409DFB`, inlined: `memory::allocate` on `g_sampleMemory`; its fatal
//! error 0x80E28021 cannot come), the data copied there and the voice's data pointed at it. One caller,
//! command 3 (`sub_40AEB9`). Since phase 12 the copy is Rust's (`audio::sample_data::keep`,
//! counted on `g_sampleMemory` as the block was) and the voice's data its id.

use crate::audio::owner::audio;
use crate::audio::voices::*;
use crate::errors::Refused;

pub fn keep_sample(v: &mut Voice) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::buffers::keeps_sample`.
    if !rc2k_core::audio::buffers::keeps_sample(v) {
        return Ok(());
    }
    v.data = crate::audio::sample_data::keep(v.data, v.length, &audio().state.sample_memory)?;
    Ok(())
}
