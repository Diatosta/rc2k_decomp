//! The audio thread put in its modes, and the waits for it.
//!
//! Both set-ups stop the music, queue their commands, wait for the thread to take them and let sounds play again
//! (`AudioShared.sounds_held_back` 0). Each command's values go in as the caller left them; a
//! command's result is set for those after it.

use crate::audio::commands::Command;

/// A set-up's steps, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// `sound::stop_music`.
    StopMusic,
    /// A command queued.
    Send(Command),
    /// `sound::load_race_sounds`.
    LoadRaceSounds,
    /// `sound::wait_for_commands`.
    Wait,
}

/// The menus' set-up: the mode, the sounds' list and the master volume.
pub fn menus(mode: u32, list: crate::audio::commands::SoundList, volume: u32) -> [Step; 5] {
    [
        Step::StopMusic,
        Step::Send(Command::Mode(mode)),
        Step::Send(Command::Load(Some(list))),
        Step::Send(Command::Volume(volume)),
        Step::Wait,
    ]
}

/// The race's set-up: the race's mode.
pub fn race(mode: u32) -> [Step; 4] {
    [
        Step::StopMusic,
        Step::Send(Command::Mode(mode)),
        Step::LoadRaceSounds,
        Step::Wait,
    ]
}

/// What the wait for the audio thread is made of.
pub trait Waiting {
    /// Queues an "end of turn" command.
    fn end_turn(&mut self);
    /// Blocks until the audio thread has taken every command and ended the turn it took the
    /// last in, or until a short while has passed: true when it has taken them all (or
    /// there is no audio thread to take them).
    fn wait_a_while(&mut self) -> bool;
    fn pump_messages(&mut self);
    /// Whether the quit switch is set to 1.
    fn quitting(&mut self) -> bool;
}

/// `sound::wait_for_commands`: an "end of turn" command, then the wait for the audio thread,
/// in short stretches; between them the window's messages, and quitting ends the wait.
pub fn wait_for_commands(w: &mut dyn Waiting) {
    w.end_turn();
    while !w.wait_a_while() {
        w.pump_messages();
        if w.quitting() {
            break;
        }
    }
}

/// The car screens' stream started (`sound::play_car_stream`): on, its list, the place in it 0, and
/// `sound::start_stream` with the list's first entry and the volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarStream {
    pub on: u32,
    pub list: u32,
    pub place: u32,
    pub first: u32,
    pub volume: u32,
}

/// The car's stream list (`MenuLists.car_streams[0][car]`); `first` reads its first dword.
pub fn car_stream(list: u32, first: impl FnOnce(u32) -> u32, volume: u32) -> CarStream {
    CarStream {
        on: 1,
        list,
        place: 0,
        first: first(list),
        volume,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_ups() {
        assert_eq!(
            menus(1, crate::audio::commands::SoundList::Menu, 3)[1..4],
            [
                Step::Send(Command::Mode(1)),
                Step::Send(Command::Load(Some(crate::audio::commands::SoundList::Menu))),
                Step::Send(Command::Volume(3))
            ]
        );
        assert_eq!(race(7)[2..], [Step::LoadRaceSounds, Step::Wait]);
        let s = car_stream(0x100, |l| l + 1, 9);
        assert_eq!((s.on, s.place, s.first, s.volume), (1, 0, 0x101, 9));
    }

    struct Script {
        waits: u32,
        quit_after: u32,
        calls: Vec<char>,
    }

    impl Waiting for Script {
        fn end_turn(&mut self) {
            self.calls.push('e');
        }
        fn wait_a_while(&mut self) -> bool {
            self.calls.push('w');
            let waiting = self.waits > 0;
            self.waits = self.waits.saturating_sub(1);
            !waiting
        }
        fn pump_messages(&mut self) {
            self.calls.push('m');
        }
        fn quitting(&mut self) -> bool {
            self.quit_after = self.quit_after.saturating_sub(1);
            self.quit_after == 0
        }
    }

    #[test]
    fn waits() {
        let mut s = Script {
            waits: 2,
            quit_after: 9,
            calls: Vec::new(),
        };
        wait_for_commands(&mut s);
        assert_eq!(s.calls.iter().collect::<String>(), "ewmwmw");
        let mut s = Script {
            waits: 5,
            quit_after: 1,
            calls: Vec::new(),
        };
        wait_for_commands(&mut s);
        assert_eq!(s.calls.iter().collect::<String>(), "ewm");
    }
}
