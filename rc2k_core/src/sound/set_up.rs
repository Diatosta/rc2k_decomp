//! The audio thread put in its modes, and the waits for it (from
//! `sound::apply_settings` 0x40EFF3, `sound::set_up_race_sounds` 0x40F02E,
//! `sound::wait_for_commands` 0x409C1E and `sound::play_car_stream` 0x469AA1).
//!
//! Both set-ups mark which they are (`dword_518080`: 1 the menus', 2 the race's), stop the
//! music, queue their commands, wait for the thread to take them and let sounds play again
//! (`dword_506C84` 0). The registers go into the commands as the caller left them, eax set
//! by each command for those after it.

/// `dword_518080` after the menus' set-up.
pub const MENUS: u32 = 1;
/// `dword_518080` after the race's.
pub const RACE: u32 = 2;

/// The commands the set-ups queue (`audio::take_commands`' numbers).
pub const LOAD: u32 = 3;
pub const MODE: u32 = 5;
pub const VOLUME: u32 = 8;

/// A set-up's steps, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// `sound::stop_music`.
    StopMusic,
    /// A command queued with eax (kept for the steps after it).
    Send { command: u32, eax: u32 },
    /// `sound::load_race_sounds`.
    LoadRaceSounds,
    /// `sound::wait_for_commands`.
    Wait,
}

/// The menus' set-up: the mode (`dword_4F82C4`), the sounds' list (`unk_511AE4`) and the master volume (`dword_755684`).
pub fn menus(mode: u32, list: crate::audio::commands::SoundList, volume: u32) -> [Step; 5] {
    [
        Step::StopMusic,
        Step::Send {
            command: MODE,
            eax: mode,
        },
        Step::Send {
            command: LOAD,
            eax: list.word(),
        },
        Step::Send {
            command: VOLUME,
            eax: volume,
        },
        Step::Wait,
    ]
}

/// The race's set-up: the race's mode (`dword_4F82C8`).
pub fn race(mode: u32) -> [Step; 4] {
    [
        Step::StopMusic,
        Step::Send {
            command: MODE,
            eax: mode,
        },
        Step::LoadRaceSounds,
        Step::Wait,
    ]
}

/// What the wait for the audio thread is made of.
pub trait Waiting {
    /// Queues an "end of turn" command.
    fn end_turn(&mut self);
    /// Whether commands are still waiting.
    fn pending(&mut self) -> bool;
    fn sleep_a_millisecond(&mut self);
    fn pump_messages(&mut self);
    /// The quit switch is 1.
    fn quitting(&mut self) -> bool;
}

/// `sound::wait_for_commands`: two "end of turn" commands, then while commands wait a
/// millisecond's sleep and the window's messages; quitting ends the wait.
pub fn wait_for_commands(w: &mut dyn Waiting) {
    w.end_turn();
    w.end_turn();
    while w.pending() {
        w.sleep_a_millisecond();
        w.pump_messages();
        if w.quitting() {
            break;
        }
    }
}

/// The car screens' stream started (`sound::play_car_stream`): on (`dword_A8C188` 1), its
/// list (`dword_A8C184`), the place in it 0 (`dword_A8C180`), and `sound::start_stream`
/// with the list's first entry (eax) and the volume (ecx, `dword_755698`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarStream {
    pub on: u32,
    pub list: u32,
    pub place: u32,
    pub first: u32,
    pub volume: u32,
}

/// The car's stream list (`off_75A31E[car]`); `first` reads its first dword.
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
                Step::Send {
                    command: MODE,
                    eax: 1
                },
                Step::Send {
                    command: LOAD,
                    eax: 1
                },
                Step::Send {
                    command: VOLUME,
                    eax: 3
                }
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
        fn pending(&mut self) -> bool {
            self.calls.push('p');
            let waiting = self.waits > 0;
            self.waits = self.waits.saturating_sub(1);
            waiting
        }
        fn sleep_a_millisecond(&mut self) {
            self.calls.push('s');
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
        assert_eq!(s.calls.iter().collect::<String>(), "eepsmpsmp");
        let mut s = Script {
            waits: 5,
            quit_after: 1,
            calls: Vec::new(),
        };
        wait_for_commands(&mut s);
        assert_eq!(s.calls.iter().collect::<String>(), "eepsm");
    }
}
