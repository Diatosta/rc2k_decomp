//! The game's four own threads (input, audio, network receive, shutdown) and their slots. A
//! slot: the thread's id, its handle, whether its routine runs, and the path buffer
//! `files::resolve_path` uses for the thread.

use crate::errors::{BadInput, FatalError, Recover, Refused};
use crate::memory::session::Bytes;
use crate::program_log::log;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// One of the game's own threads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotThread {
    Input,
    Audio,
    NetReceive,
    Shutdown,
}

impl SlotThread {
    pub const ALL: [SlotThread; 4] = [
        SlotThread::Input,
        SlotThread::Audio,
        SlotThread::NetReceive,
        SlotThread::Shutdown,
    ];
    /// The threads started with the game, in the order they are started; the shutdown waits
    /// for these.
    pub const WORKERS: [SlotThread; 3] =
        [SlotThread::Input, SlotThread::Audio, SlotThread::NetReceive];

    /// The thread's name, as a debugger, a profiler or a fault report shows it.
    pub const fn name(self) -> &'static str {
        match self {
            SlotThread::Input => "input",
            SlotThread::Audio => "audio",
            SlotThread::NetReceive => "net receive",
            SlotThread::Shutdown => "shutdown",
        }
    }

    /// The thread's slot.
    pub fn slot(self) -> &'static ThreadSlot {
        &SLOTS[self as usize]
    }

    /// The one of the four the calling thread is, None on any other thread (the game's own
    /// among them).
    pub fn current() -> Option<SlotThread> {
        let id = crate::platform::sync::thread_id();
        Self::ALL
            .into_iter()
            .find(|thread| thread.slot().thread_id.load(Ordering::SeqCst) == id)
    }

    /// The thread's routine, run to its end on the calling thread.
    pub fn run(self, o: SlotOwners<'static>) -> Result<(), Refused> {
        match self {
            SlotThread::Input => crate::input::input_thread::input_thread(
                o.input,
                &o.sound.threads.shared,
                o.system,
                o.race,
            ),
            SlotThread::Audio => {
                crate::audio::thread::audio_thread(crate::audio::thread::AudioInputs {
                    audio: o.audio,
                    system: o.system,
                    sets: &o.sound.threads.sets,
                    menu_sounds: &o.sound.threads.menu_sounds,
                })
            }
            SlotThread::NetReceive => crate::net::receive_thread::receive_thread(o.net, o.system),
            SlotThread::Shutdown => crate::threading::shutdown_thread::shutdown_thread(),
        }
    }

    /// The thread started: it runs its routine with `owners`; its id and handle are kept in
    /// its slot. A routine that raises a fatal error or refuses ends the program. A thread
    /// that cannot be made is fatal error 0x80108601.
    pub fn start(self, owners: SlotOwners<'static>) -> Result<(), FatalError> {
        let slot = self.slot();
        slot.running.store(false, Ordering::SeqCst);
        // The thread's id goes into the slot before the thread can look for its slot.
        let id = crate::platform::sync::new_thread_id();
        slot.thread_id.store(id, Ordering::SeqCst);
        let Some(handle) = crate::platform::sync::spawn(self.name(), id, move || {
            let name = self.name();
            match self.run(owners) {
                Ok(()) => {}
                Err(Refused::Fatal(error)) => {
                    log(&format!(
                        "threading: the {name} thread raises fatal error {:#x} {:?}",
                        error.code, error.message
                    ));
                    crate::fatal::raise::raise_bare(error);
                }
                Err(Refused::BadInput(BadInput(why)) | Refused::Recoverable(Recover(why))) => {
                    crate::standalone::finish(
                        &format!("threading: the {name} thread refused: {why}"),
                        1,
                    )
                }
            }
        }) else {
            return Err(FatalError {
                code: 0x8010_8601,
                message: "",
            });
        };
        slot.handle.store(handle, Ordering::SeqCst);
        Ok(())
    }
}

/// The input, audio and network receive threads started, in that order.
pub fn start_workers(owners: SlotOwners<'static>) -> Result<(), FatalError> {
    for thread in SlotThread::WORKERS {
        thread.start(owners)?;
    }
    Ok(())
}

pub struct ThreadSlot {
    pub thread_id: AtomicU32,
    /// The thread's handle (`platform::sync`).
    pub handle: AtomicU32,
    /// Whether the thread's routine is at work. A routine entered again while it is (after a
    /// fatal error on its own thread) goes straight to its end.
    pub running: AtomicBool,
    /// The buffer `files::resolve_path` uses for the thread.
    pub path: Bytes<0x100>,
}

impl ThreadSlot {
    const fn new() -> Self {
        Self {
            thread_id: AtomicU32::new(0),
            handle: AtomicU32::new(0),
            running: AtomicBool::new(false),
            path: Bytes::new([0; 0x100]),
        }
    }
}

static SLOTS: [ThreadSlot; SlotThread::ALL.len()] = [
    ThreadSlot::new(),
    ThreadSlot::new(),
    ThreadSlot::new(),
    ThreadSlot::new(),
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
