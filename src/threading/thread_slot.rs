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
#[cfg(test)]
pub const SLOT_LEN: usize = size_of::<ThreadSlot>();

pub static SLOTS: [ThreadSlot; SLOT_COUNT] = [
    ThreadSlot::new(0x4078DA),
    ThreadSlot::new(0x409CCE),
    ThreadSlot::new(0x4183C3),
    ThreadSlot::new(0x410C17),
];

/// The owners the slot threads work with. Every one is a plain static whose state the
/// other threads may read (atomic cells); none holds a `GameThread` state, which only the
/// game's thread may reach.
#[derive(Clone, Copy)]
pub struct SlotOwners<'a> {
    pub audio: &'static crate::audio::owner::Audio,
    pub input: &'a crate::input::owner::Input,
    pub sound: &'a crate::sound::owner::Sound,
    pub system: &'a crate::system::owner::System,
    pub race: &'a crate::race::owner::Race,
    pub net: &'a crate::net::owner::Net,
    /// Only for the receiving thread's chat lines, which measure text with its fonts.
    pub gfx: &'static crate::gfx::owner::Gfx,
}

impl SlotOwners<'static> {
    /// The owners as the accessors give them: the root of a slot's thread fetches them (the
    /// graphics owner by `on_any_thread`, as the receiving thread has always reached it).
    pub fn fetch() -> Self {
        SlotOwners {
            audio: crate::audio::owner::audio(),         // m6-root
            input: crate::input::owner::input(),         // m6-root
            sound: crate::sound::owner::sound(),         // m6-root
            system: crate::system::owner::system(),      // m6-root
            race: crate::race::owner::race(),            // m6-root
            net: crate::net::owner::net(),               // m6-root
            gfx: crate::gfx::owner::GFX.on_any_thread(), // m6-root
        }
    }
}

/// A slot's routine (the IDA address the slot holds) run by its Rust version; None for
/// an address that is none of the four.
pub fn run_routine(routine: u32, o: SlotOwners<'_>) -> Option<Result<(), Refused>> {
    match routine {
        0x4078DA => Some(crate::input::input_thread::input_thread(
            o.input, o.sound, o.system, o.race,
        )),
        0x409CCE => Some(crate::audio::thread::audio_thread(
            o.audio, o.sound, o.system,
        )),
        0x4183C3 => Some(crate::net::receive_thread::receive_thread(
            o.net, o.gfx, o.system,
        )),
        0x410C17 => Some(crate::threading::shutdown_thread::shutdown_thread()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn slot_layout() {
        assert_eq!(super::SLOT_LEN, 0x110);
        assert_eq!(
            size_of_val(&super::SLOTS),
            super::SLOT_COUNT * super::SLOT_LEN
        );
    }
}
