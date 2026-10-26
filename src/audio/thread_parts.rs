//! The audio thread's small set-up and tear-down parts (IDA, 2026-10-15): the work buffer
//! for reading samples, the primary buffer's play and stop, the 3D listener's release.

use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{game_addr, rva};

pub const SAMPLE_BUFFER_KB_RVA: usize = rva(0x5068EC); // g_sampleBufferKb (a setting)
pub const SAMPLE_BUFFER_MEMORY_RVA: usize = rva(0x5068F0); // g_sampleBufferMemory

/// Where the sample work buffer's things are.
#[derive(Clone, Copy)]
pub struct SampleBufferVars {
    pub kb: u32,
    pub size: u32,
    pub buffer: u32,
    pub list: u32,
}

impl SampleBufferVars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            kb: a(SAMPLE_BUFFER_KB_RVA),
            size: a(crate::audio::samples::BUFFER_SIZE_RVA),
            buffer: a(crate::audio::samples::BUFFER_RVA),
            list: a(SAMPLE_BUFFER_MEMORY_RVA),
        }
    }
}

/// `audio::take_sample_buffer`: the setting's KB (0: one) taken on its list.
pub unsafe fn take_sample_buffer(v: &SampleBufferVars) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::audio::levels::sample_buffer_size`.
        let size = rc2k_core::audio::levels::sample_buffer_size(read(v.kb));
        write(v.size, size);
        let b = crate::memory::allocate::allocate(v.list as *mut crate::memory::MemoryList, size)?;
        write(v.buffer, b as u32);
    }
    Ok(())
}

/// `audio::free_sample_buffer`: its list freed, size and buffer 0.
pub unsafe fn free_sample_buffer(v: &SampleBufferVars) -> Result<(), Refused> {
    unsafe {
        crate::memory::free_list::free_list(v.list as *mut crate::memory::MemoryList)?;
        write(v.size, 0);
        write(v.buffer, 0);
    }
    Ok(())
}

/// `audio::play_primary`: unless A3D, the primary buffer (at `primary`) played looping; a
/// failure sets the flag at `lost`.
pub unsafe fn play_primary(primary: u32, lost: u32) {
    unsafe {
        let p = read(primary);
        if crate::audio::buffer_settings::a3d() || p == 0 {
            return;
        }
        let play: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 = std::mem::transmute(
            crate::audio::com::method(p, crate::audio::buffer_play::PLAY),
        );
        if play(p, 0, 0, 1) != 0 {
            write(lost, 1);
        }
    }
}

/// `audio::stop_primary`: unless A3D, the primary buffer stopped.
pub unsafe fn stop_primary(primary: u32) {
    unsafe {
        let p = read(primary);
        if !crate::audio::buffer_settings::a3d() && p != 0 {
            crate::audio::com::call(p, crate::audio::buffer_play::STOP);
        }
    }
}

/// `audio::release_sound_listener`: the listener at `listener` released and forgotten.
pub unsafe fn release_sound_listener(listener: u32) {
    unsafe {
        let l = read(listener);
        if l != 0 {
            crate::audio::com::call(l, crate::audio::com::RELEASE);
            write(listener, 0);
        }
    }
}

// ---- for the self-tests ----

/// A copy of a part whose variables are the test's. (`test_copy::make` with its `Copy`
/// written out: each argument is one of the copy's facts, read off IDA.)
#[allow(clippy::too_many_arguments)]
pub fn copy_of(
    hook: &crate::framework::RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    jumps_out: &[usize],
    variables: &[(usize, u32)],
    references: usize,
) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls,
            jumps_out,
        },
        &|_| None,
        variables,
        references,
    )
}
