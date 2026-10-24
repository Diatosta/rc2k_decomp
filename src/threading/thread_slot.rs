//! The game's four thread slots, out of the image since 2026-10-24 (phase 5; they were
//! `g_inputThreadSlot`, `g_audioThreadSlot`, `stru_518714` and `stru_518824`, 0x5184F4 ..
//! 0x518934, four of 0x110 bytes in a row). A slot: +0 the thread's routine, +4 its thread id,
//! +8 its handle, +0xC its flag (1 running, 0xFF suspended), +0x10 .. +0x110 the path
//! buffer `files::resolve_path` uses for the thread.

use crate::moved::Place;

#[repr(C)]
pub struct ThreadSlot {
    /// The routine: its IDA address, as the image held it before relocation
    /// (`start_slot_thread` makes the running address of it).
    pub routine: u32,
    pub thread_id: u32,
    _rest: [u8; 0x108],
}

impl ThreadSlot {
    const fn new(routine: u32) -> Self {
        Self {
            routine,
            thread_id: 0,
            _rest: [0; 0x108],
        }
    }
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

pub static SLOTS: Place<[ThreadSlot; SLOT_COUNT]> = Place::new([
    ThreadSlot::new(0x4078DA),
    ThreadSlot::new(0x409CCE),
    ThreadSlot::new(0x4183C3),
    ThreadSlot::new(0x410C17),
]);

/// The address of slot `n`.
pub fn slot(n: usize) -> usize {
    assert!(n < SLOT_COUNT);
    SLOTS.addr() + n * SLOT_LEN
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
                super::SLOTS.ptr() as *const u8,
                mine.as_mut_ptr(),
                d.len(),
            )
        };
        // The image's routines are relocated addresses; compared here as IDA's.
        assert_eq!(mine, d);
    }
}
