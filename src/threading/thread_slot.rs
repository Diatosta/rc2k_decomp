//! The game's four thread slots (`g_inputThreadSlot`, `g_audioThreadSlot`, the net slot and
//! the shutdown slot, 0x5184F4 .. 0x518934, four of 0x110 bytes in a row). A slot: +0 the thread's routine, +4 its thread id,
//! +8 its handle, +0xC its flag (1 running, 0xFF suspended), +0x10 .. +0x110 the path
//! buffer `files::resolve_path` uses for the thread.

use crate::errors::{BadInput, Refused};
use crate::session::{Byte, Bytes, Dword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ThreadSlot {
    /// The routine: its IDA address, as the image held it before relocation
    /// (`start_slot_thread` makes the running address of it).
    pub routine: Dword,
    pub thread_id: Dword,
    /// The thread's handle (+8).
    pub handle: Dword,
    /// +0xC: 1 running, 0xFF suspended.
    pub flag: Byte,
    _pad: Bytes<3>,
    /// +0x10: the buffer `files::resolve_path` uses for the thread.
    pub path: Bytes<0x100>,
}

impl ThreadSlot {
    const fn new(routine: u32) -> Self {
        Self {
            routine: Dword::new(routine),
            thread_id: Dword::new(0),
            handle: Dword::new(0),
            flag: Byte::new(0),
            _pad: Bytes::new([0; 3]),
            path: Bytes::new([0; 0x100]),
        }
    }
}

/// Slot `n`.
pub fn slot_ref(n: usize) -> Result<&'static ThreadSlot, BadInput> {
    SLOTS.get(n).ok_or(BadInput("a thread slot past the four"))
}

/// The slot of the input thread, the audio thread, the network receive thread and the
/// shutdown thread.
pub const INPUT: usize = 0;
pub const AUDIO: usize = 1;
pub const NET: usize = 2;
pub const SHUTDOWN: usize = 3;
/// The slots, in the order they are started (`start_threads` starts the first three).
pub const SLOT_COUNT: usize = 4;
pub const SLOT_LEN: usize = size_of::<ThreadSlot>();

pub static SLOTS: [ThreadSlot; SLOT_COUNT] = [
    ThreadSlot::new(0x4078DA),
    ThreadSlot::new(0x409CCE),
    ThreadSlot::new(0x4183C3),
    ThreadSlot::new(0x410C17),
];

/// A slot's routine (the IDA address the slot holds) run by its Rust version; None for
/// an address that is none of the four.
pub fn run_routine(routine: u32) -> Option<Result<(), Refused>> {
    let audio = crate::audio::owner::audio();
    let input = crate::input::owner::input();
    match routine {
        0x4078DA => Some(crate::input::input_thread::thread_routine(input)),
        0x409CCE => Some(crate::audio::thread::audio_thread(audio)),
        0x4183C3 => Some(crate::net::receive_thread::receive_thread()),
        0x410C17 => Some(crate::threading::shutdown_thread::shutdown_thread()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn slots_start_as_the_games() {
        assert_eq!(super::SLOT_LEN, 0x110);
        let d = crate::moved::image_bytes(0x5184F4, super::SLOT_COUNT * super::SLOT_LEN);
        let mut mine = vec![0u8; d.len()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                super::SLOTS.as_ptr().cast::<u8>(),
                mine.as_mut_ptr(),
                d.len(),
            );
        };
        // The image's routines are relocated addresses; compared here as IDA's.
        assert_eq!(mine, d);
    }
}
