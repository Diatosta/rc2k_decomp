//! The game's four thread slots (the input, audio, net and shutdown threads'). A slot: the
//! thread's routine, its thread id, its handle, its flag (1 running, 0xFF suspended) and the
//! path buffer `files::resolve_path` uses for the thread.

use crate::errors::{BadInput, Refused};
use crate::memory::session::{Byte, Bytes, Dword};

#[derive(rc2k_derive::Cells)]
pub struct ThreadSlot {
    /// The routine, by its game address (`run_routine` finds its Rust function by it).
    pub routine: Dword,
    pub thread_id: Dword,
    /// The thread's handle.
    pub handle: Dword,
    /// 1 running, 0xFF suspended.
    pub flag: Byte,
    _pad: Bytes<3>,
    /// The buffer `files::resolve_path` uses for the thread.
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

pub static SLOTS: [ThreadSlot; SLOT_COUNT] = [
    ThreadSlot::new(0x4078DA),
    ThreadSlot::new(0x409CCE),
    ThreadSlot::new(0x4183C3),
    ThreadSlot::new(0x410C17),
];

/// The owners the slot threads work with. Every one is built at the start and kept for the run; its state the
/// other threads may read (atomic cells); none holds a `GameThread` state, which only the game's
/// thread may reach.
#[derive(Clone, Copy)]
pub struct SlotOwners<'a> {
    pub audio: &'a crate::audio::owner::Audio,
    pub input: &'a crate::input::owner::Input,
    pub sound: &'a crate::sound::owner::Sound,
    pub system: &'a crate::system::owner::System,
    pub race: &'a crate::race::owner::Race,
    pub net: &'a crate::net::owner::Net,
}

impl<'a> SlotOwners<'a> {
    /// The slot threads' share of the owners, taken on the game's thread.
    pub fn of(o: crate::owners::Owners<'a>) -> Self {
        SlotOwners {
            audio: o.audio,
            input: o.input,
            sound: o.sound,
            system: o.system,
            race: o.race,
            net: o.net,
        }
    }
}

/// The owners published for the fatal-error reporter, which runs on any thread with no caller to
/// hand it owners, even from a fault before most state exists. The only global handle to owners
/// that stays by design; `publish` fills it once at the start.
static PUBLISHED: std::sync::OnceLock<SlotOwners<'static>> = std::sync::OnceLock::new();

/// Publishes the owners for `published`; the first call counts.
pub fn publish(owners: SlotOwners<'static>) {
    let _ = PUBLISHED.set(owners);
}

/// The published owners, or `None` before `publish` ran.
pub fn published() -> Option<SlotOwners<'static>> {
    PUBLISHED.get().copied()
}

/// A slot's routine run by its Rust function; None for an address that is none of the four.
pub fn run_routine(routine: u32, o: SlotOwners<'static>) -> Option<Result<(), Refused>> {
    match routine {
        0x4078DA => Some(crate::input::input_thread::input_thread(
            o.input,
            &o.sound.threads.shared,
            o.system,
            o.race,
        )),
        0x409CCE => Some(crate::audio::thread::audio_thread(
            crate::audio::thread::AudioInputs {
                audio: o.audio,
                system: o.system,
                sets: &o.sound.threads.sets,
                menu_sounds: &o.sound.threads.menu_sounds,
            },
        )),
        0x4183C3 => Some(crate::net::receive_thread::receive_thread(o.net, o.system)),
        0x410C17 => Some(crate::threading::shutdown_thread::shutdown_thread()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_published_owners_are_the_ones_given_to_the_slot_threads() {
        let o = crate::owners::Owners::the();
        let owners = SlotOwners::of(o);
        publish(owners);
        let got = published().expect("published");
        assert!(std::ptr::eq(got.system, o.system));
        assert!(std::ptr::eq(got.net, o.net));
        // The first publish counts: a second one changes nothing.
        publish(SlotOwners::of(o));
        assert!(std::ptr::eq(published().unwrap().race, o.race));
    }
}
