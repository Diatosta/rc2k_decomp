//! `audio::keep_sample` (0x40B192): a voice's prepared data copied to the samples' memory.
//!
//! In:  esi = the voice
//! Out: every register preserved
//!
//! Only for a voice with data, a length and a `data_key`: `length` bytes from the samples'
//! memory (`sub_409DFB`, inlined: `memory::allocate` on `g_sampleMemory`; its fatal
//! error 0x80E28021 cannot come), the data copied there and the voice's data pointed at it. One caller,
//! command 3 (`sub_40AEB9`).

use crate::audio::store::AUDIO;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::pointers::GamePtr;

/// `allocate`: `length` bytes of the samples' memory.
pub fn keep_sample(
    v: &mut Voice,
    allocate: &mut dyn FnMut(u32) -> Result<u32, Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::buffers::keeps_sample`.
    if !rc2k_core::audio::buffers::keeps_sample(v) {
        return Ok(());
    }
    let to = allocate(v.length)?;
    GamePtr::new(v.data).copy_to(GamePtr::new(to), v.length as usize, "a voice's sample")?;
    v.data = to;
    Ok(())
}

pub unsafe fn allocate_in_game(length: u32) -> Result<u32, Refused> {
    let list = AUDIO.sample_memory.as_ptr();
    // The allocator reports a failure itself (0x80108571), so `sub_409DFB`'s own fatal
    // error 0x80E28021 cannot come.
    Ok(unsafe { crate::memory::allocate::allocate(list, length) }? as u32)
}
