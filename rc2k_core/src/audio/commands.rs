//! The commands the game queues for the audio thread: what each asks, with the values the
//! audio thread acts on, and the queue they wait in. A command has its number and seven words
//! (`record_words`); the audio thread reads only the words each command names.

use std::collections::VecDeque;

/// How many commands the queue holds; one more is dropped.
pub const CAPACITY: usize = 0x400;

/// A list of sounds command 3 loads, by handle (the original sent its address). The menus'
/// list, or a list in the race's sets at an offset from the sets' start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundList {
    Menu,
    Race { offset: u32 },
}

impl SoundList {
    /// The word a command record carries: 0 is no list, 1 the menus', 2 and up the race's
    /// list at that offset less 2.
    pub fn word(self) -> u32 {
        match self {
            SoundList::Menu => 1,
            SoundList::Race { offset } => offset.wrapping_add(2),
        }
    }

    /// The list a record's word names; none for 0.
    pub fn from_word(word: u32) -> Option<SoundList> {
        match word {
            0 => None,
            1 => Some(SoundList::Menu),
            n => Some(SoundList::Race { offset: n - 2 }),
        }
    }
}

/// Command 1: a sound started, changed or stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Start {
    /// The sound: its number in the low word (signed), bit 31 set for "do not start it if it
    /// is not playing".
    pub sound_word: u32,
    /// The volume, or `voices::DEFAULT` for the sound's own.
    pub volume: u32,
    /// The pan (its low byte, signed), or `DEFAULT`.
    pub pan: u32,
    /// The frequency, or `DEFAULT` / `OWN_FREQUENCY`.
    pub frequency: u32,
    /// The place's three floats (bits) in the 3D modes, or `DEFAULT_PLACE`.
    pub place: [u32; 3],
}

impl Start {
    /// The sound's number, negative for none.
    pub fn sound(&self) -> i32 {
        self.sound_word as u16 as i16 as i32
    }
}

/// A command for the audio thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    /// 1: a sound started, changed or stopped.
    Start(Start),
    /// 2: the EAX room asked.
    Room(u32),
    /// 3: a list of sounds loaded (none: nothing).
    Load(Option<SoundList>),
    /// 4: every voice given back.
    Free,
    /// 5: the sound mode.
    Mode(u32),
    /// 7: every playing voice stopped.
    StopAll,
    /// 8: the master volume.
    Volume(u32),
    /// 9: the sound stream.
    Stream { path: u32, volume: u32, action: u32 },
    /// 0xA: the turn's taking ends.
    EndOfTurn,
    /// 0xB: the music.
    Music {
        tune: u32,
        next: u32,
        volume: u32,
        action: u32,
    },
}

impl Command {
    /// The command's number and seven words (the values the game's senders queued; those the
    /// audio thread does not read are 0). The sound hash of a run is made of these, so a held
    /// race's commands can be compared turn by turn; the words keep the order the game
    /// queued them in.
    pub fn record_words(&self) -> (u32, [u32; 7]) {
        match *self {
            Command::Start(s) => (
                1,
                [
                    s.sound_word,
                    s.volume,
                    s.pan,
                    s.frequency,
                    s.place[0],
                    s.place[1],
                    s.place[2],
                ],
            ),
            Command::Room(room) => (2, [room, 0, 0, 0, 0, 0, 0]),
            Command::Load(list) => (3, [list.map_or(0, SoundList::word), 0, 0, 0, 0, 0, 0]),
            Command::Free => (4, [0; 7]),
            Command::Mode(mode) => (5, [mode, 0, 0, 0, 0, 0, 0]),
            Command::StopAll => (7, [0; 7]),
            Command::Volume(volume) => (8, [volume, 0, 0, 0, 0, 0, 0]),
            Command::Stream {
                path,
                volume,
                action,
            } => (9, [path, 0, volume, action, 0, 0, 0]),
            Command::EndOfTurn => (0xA, [0; 7]),
            Command::Music {
                tune,
                next,
                volume,
                action,
            } => (0xB, [tune, next, volume, action, 0, 0, 0]),
        }
    }
}

/// The commands waiting for the audio thread, oldest first. A command stays in the queue
/// while the audio thread acts on it (`front`, then `pop`), so it counts as waiting until it
/// is done; a full queue takes nothing more.
#[derive(Debug, Default)]
pub struct Queue {
    waiting: VecDeque<Command>,
}

impl Queue {
    pub const fn new() -> Self {
        Queue {
            waiting: VecDeque::new(),
        }
    }

    /// Queues `command`; false (and nothing queued) when the queue is full.
    pub fn send(&mut self, command: Command) -> bool {
        if self.waiting.len() >= CAPACITY {
            return false;
        }
        self.waiting.push_back(command);
        true
    }

    /// The oldest command, left in the queue.
    pub fn front(&self) -> Option<Command> {
        self.waiting.front().copied()
    }

    /// Drops the oldest command, done.
    pub fn pop(&mut self) {
        self.waiting.pop_front();
    }

    /// Whether commands are waiting.
    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }

    /// Drops every command.
    pub fn clear(&mut self) {
        self.waiting.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sounds_are_signed_words() {
        let start = |sound_word| Start {
            sound_word,
            volume: 0,
            pan: 0,
            frequency: 0,
            place: [0; 3],
        };
        assert_eq!(start(0x1_FFFE).sound(), -2);
        assert_eq!(start(0x8000_0005).sound(), 5);
    }

    #[test]
    fn a_command_stays_until_popped() {
        let mut q = Queue::new();
        assert!(q.is_empty() && q.front().is_none());
        assert!(q.send(Command::Free));
        assert!(q.send(Command::StopAll));
        assert_eq!(q.front(), Some(Command::Free));
        assert!(!q.is_empty());
        q.pop();
        assert_eq!(q.front(), Some(Command::StopAll));
        q.clear();
        assert!(q.is_empty());
    }
}

#[cfg(test)]
mod golden {
    //! A recorded sequence of commands, and what the audio side hears of each, to the field:
    //! the queue must give the same commands in the same order, whatever it is made of.
    use super::*;

    /// What the audio thread acts on, as text.
    fn heard_text(c: &Command) -> String {
        match c {
            Command::Start(s) => format!(
                "start sound={} word={:#x} volume={} pan={} frequency={} place={:?}",
                s.sound(),
                s.sound_word,
                s.volume,
                s.pan,
                s.frequency,
                s.place
            ),
            Command::Room(n) => format!("room {n}"),
            Command::Load(list) => format!("load {:?}", list.map(SoundList::word)),
            Command::Free => "free".into(),
            Command::Mode(n) => format!("mode {n:#x}"),
            Command::StopAll => "stop all".into(),
            Command::Volume(n) => format!("volume {n}"),
            Command::Stream {
                path,
                volume,
                action,
            } => format!("stream {path} {volume} {action}"),
            Command::EndOfTurn => "end of turn".into(),
            Command::Music {
                tune,
                next,
                volume,
                action,
            } => format!("music {tune} {next} {volume} {action}"),
        }
    }

    /// What the seven words a sender queued ask for command `number`: the inverse of
    /// `record_words`.
    fn command_from(number: u32, r: [u32; 7]) -> Command {
        match number {
            1 => Command::Start(Start {
                sound_word: r[0],
                volume: r[1],
                pan: r[2],
                frequency: r[3],
                place: [r[4], r[5], r[6]],
            }),
            2 => Command::Room(r[0]),
            3 => Command::Load(SoundList::from_word(r[0])),
            4 => Command::Free,
            5 => Command::Mode(r[0]),
            7 => Command::StopAll,
            8 => Command::Volume(r[0]),
            9 => Command::Stream {
                path: r[0],
                volume: r[2],
                action: r[3],
            },
            0xA => Command::EndOfTurn,
            0xB => Command::Music {
                tune: r[0],
                next: r[1],
                volume: r[2],
                action: r[3],
            },
            n => panic!("command {n} is not sent"),
        }
    }

    const SENT: [(u32, [u32; 7]); 12] = [
        (1, [0x1_FFFE, 5, 0x7F, 0x2000, 1, 2, 3]),
        (2, [3, 0, 0, 0, 0, 0, 0]),
        (3, [1, 0, 0, 0, 0, 0, 0]),
        (3, [0x7E, 0, 0, 0, 0, 0, 0]),
        (3, [0, 0, 0, 0, 0, 0, 0]),
        (4, [0; 7]),
        (5, [0x10_0000, 0, 0, 0, 0, 0, 0]),
        (7, [0; 7]),
        (8, [0x40, 0, 0, 0, 0, 0, 0]),
        (9, [5, 0, 0x30, 2, 0, 0, 0]),
        (0xB, [4, 8, 0x30, 2, 0, 0, 0]),
        (0xA, [0; 7]),
    ];

    const HEARD: [&str; 12] = [
        "start sound=-2 word=0x1fffe volume=5 pan=127 frequency=8192 place=[1, 2, 3]",
        "room 3",
        "load Some(1)",
        "load Some(126)",
        "load None",
        "free",
        "mode 0x100000",
        "stop all",
        "volume 64",
        "stream 5 48 2",
        "music 4 8 48 2",
        "end of turn",
    ];

    /// The queue as the senders and the audio thread use it.
    trait Wire {
        fn send(&mut self, number: u32, registers: [u32; 7]);
        fn take(&mut self) -> Option<String>;
    }

    impl Wire for Queue {
        fn send(&mut self, number: u32, registers: [u32; 7]) {
            Queue::send(self, command_from(number, registers));
        }

        fn take(&mut self) -> Option<String> {
            let text = heard_text(&self.front()?);
            self.pop();
            Some(text)
        }
    }

    /// The record of each recorded command is what the old queue stored, so the sound hash
    /// of a run is the same.
    #[test]
    fn records_are_what_the_old_queue_stored() {
        for (number, registers) in SENT {
            assert_eq!(
                command_from(number, registers).record_words(),
                (number, registers)
            );
        }
    }

    fn check_golden(w: &mut dyn Wire) {
        for (n, r) in SENT {
            w.send(n, r);
        }
        for want in HEARD {
            assert_eq!(w.take().as_deref(), Some(want));
        }
        assert_eq!(w.take(), None);
        // Full: 0x400 wait, the rest are dropped without a sound.
        for i in 0..CAPACITY as u32 + 5 {
            w.send(8, [i, 0, 0, 0, 0, 0, 0]);
        }
        // Taking some makes room, and the places run round past the end.
        for i in 0..10 {
            assert_eq!(w.take(), Some(format!("volume {i}")));
        }
        for i in 0..12 {
            w.send(8, [0x1000 + i, 0, 0, 0, 0, 0, 0]);
        }
        for i in 10..CAPACITY as u32 {
            assert_eq!(w.take(), Some(format!("volume {i}")));
        }
        for i in 0..10 {
            assert_eq!(w.take(), Some(format!("volume {}", 0x1000 + i)));
        }
        assert_eq!(w.take(), None);
    }

    #[test]
    fn the_queue_gives_the_recorded_commands() {
        check_golden(&mut Queue::new());
    }
}
