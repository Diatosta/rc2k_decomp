//! The commands the game queues for the audio thread: what each asks, with the values the
//! audio thread acts on, and the queue they wait in. A command has its number and seven words
//! (`record_words`); the audio thread reads only the words each command names.

use std::collections::VecDeque;

/// How many commands the queue holds; one more is dropped.
pub(crate) const CAPACITY: usize = 0x400;

/// The fields of the race's sets of sounds, in the order a list that runs past its field reads on
/// through them (the order of the game's data layout).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RaceField {
    KindSetsFirst,
    FirstSetA,
    FirstSetC,
    KindSetsSecond,
    SecondSetA,
    SecondSetB,
    LastSet,
    FirstSetB,
    ReplaySet,
    Kinds,
    /// The one dword that ends the sets.
    Tail,
}

impl RaceField {
    /// Every field, in the read-on order.
    pub const ALL: [RaceField; 11] = [
        RaceField::KindSetsFirst,
        RaceField::FirstSetA,
        RaceField::FirstSetC,
        RaceField::KindSetsSecond,
        RaceField::SecondSetA,
        RaceField::SecondSetB,
        RaceField::LastSet,
        RaceField::FirstSetB,
        RaceField::ReplaySet,
        RaceField::Kinds,
        RaceField::Tail,
    ];

    /// The dwords of the field.
    pub const fn dwords(self) -> usize {
        [102, 23, 94, 102, 23, 95, 31, 54, 8, 87, 1][self as usize]
    }

    /// The byte offset of the field's start in the game's data layout of the sets: used only for
    /// the command word, whose numbers the sound hash depends on.
    const fn original_offset(self) -> u32 {
        let mut dwords = 0;
        let mut i = 0;
        while i < self as usize {
            dwords += Self::ALL[i].dwords();
            i += 1;
        }
        4 * dwords as u32
    }
}

/// Where one entry of a race list lies: the lists run on through the fields after their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListPlace {
    /// Entry `index` of `field`.
    At { field: RaceField, index: usize },
    /// Past the last field: a list that reads this far has no end.
    PastSets,
    /// The list itself starts past the last field: no list.
    StartsPastSets,
}

impl RaceField {
    /// Where entry `n` of the list that starts at entry `entry` of `self` lies, reading on through
    /// the fields after `self` in their order when the list runs past its own.
    pub fn list_place(self, entry: u32, n: usize) -> ListPlace {
        let mut left = (entry as usize).saturating_add(n);
        for field in Self::ALL.into_iter().skip(self as usize) {
            if left < field.dwords() {
                return ListPlace::At { field, index: left };
            }
            left -= field.dwords();
        }
        let after: usize = Self::ALL[self as usize..].iter().map(|f| f.dwords()).sum();
        if (entry as usize) >= after {
            ListPlace::StartsPastSets
        } else {
            ListPlace::PastSets
        }
    }
}

/// A list of sounds command 3 loads, by handle. The menus' list, or a list of the race's sets:
/// entry `entry` of a field (the list reads on into the fields after it until its -1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundList {
    Menu,
    Race {
        field: RaceField,
        entry: u32,
    },
    /// One sound alone: the third and fourth cars' copies of an engine sound, which no list of
    /// the race's sets holds.
    One(u32),
    /// Sounds `first` and the `count` (below 0x100) after it: the third and fourth players'
    /// copies of the co-driver's voices.
    Range {
        first: u32,
        count: u32,
    },
}

impl SoundList {
    /// The word a command record carries: 0 is no list, 1 the menus', 2 and up a race list's
    /// `original_offset` plus 2. Kept: the sound check (`RC2K_SOUND_HASH`) hashes these words,
    /// so its numbers stay comparable.
    fn word(self) -> u32 {
        match self {
            SoundList::Menu => 1,
            // Beyond every race list's word.
            SoundList::One(sound) => 0x4000_0000 | sound,
            SoundList::Range { first, count } => 0x2000_0000 | (first << 8) | count,
            SoundList::Race { field, entry } => field
                .original_offset()
                .wrapping_add(entry.wrapping_mul(4))
                .wrapping_add(2),
        }
    }

    /// The list a record's word names; none for 0, and for a word that names no dword of the sets
    /// (`word` makes no such word).
    #[cfg(test)]
    fn from_word(word: u32) -> Option<SoundList> {
        match word {
            0 => None,
            1 => Some(SoundList::Menu),
            n if n & 0x4000_0000 != 0 => Some(SoundList::One(n & 0x3FFF_FFFF)),
            n if n & 0x2000_0000 != 0 => Some(SoundList::Range {
                first: (n & 0x1FFF_FFFF) >> 8,
                count: n & 0xFF,
            }),
            n => {
                let offset = n - 2;
                RaceField::ALL.into_iter().rev().find_map(|field| {
                    let start = field.original_offset();
                    (offset >= start
                        && offset % 4 == 0
                        && offset - start < 4 * field.dwords() as u32)
                        .then(|| SoundList::Race {
                            field,
                            entry: (offset - start) / 4,
                        })
                })
            }
        }
    }
}

/// "As the sample record says" in a command word.
const OWN_WORD: u32 = 0xFFFF_FF91;
/// "The voice's own frequency" in a command word.
const VOICE_FREQUENCY_WORD: u32 = 0xFFFF_FF90;
/// "As the sample record says" for one coordinate of a place, in a command word.
const OWN_PLACE_WORD: u32 = 0xFF80_0000;

/// A volume or pan a command sets: the sample record's, or a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Setting {
    /// As the sample record says.
    Own,
    /// The value as the sender gave it (a volume 0 ..= 0x40; a pan whose low byte is signed).
    Value(u32),
}

impl Setting {
    /// The setting a command word names.
    pub(crate) fn from_word(word: u32) -> Setting {
        if word == OWN_WORD {
            Setting::Own
        } else {
            Setting::Value(word)
        }
    }

    /// The command word of the setting.
    pub fn word(self) -> u32 {
        match self {
            Setting::Own => OWN_WORD,
            Setting::Value(value) => value,
        }
    }

    /// The value, or `own` for the sample record's.
    pub fn resolve(self, own: u32) -> u32 {
        match self {
            Setting::Own => own,
            Setting::Value(value) => value,
        }
    }
}

/// The frequency a command sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// As the sample record says.
    Own,
    /// The frequency the voice has.
    Voice,
    /// The value as the sender gave it, in Hz.
    Value(u32),
}

impl Frequency {
    /// The frequency a command word names.
    pub(crate) fn from_word(word: u32) -> Frequency {
        match word {
            OWN_WORD => Frequency::Own,
            VOICE_FREQUENCY_WORD => Frequency::Voice,
            value => Frequency::Value(value),
        }
    }

    /// The command word of the frequency.
    pub fn word(self) -> u32 {
        match self {
            Frequency::Own => OWN_WORD,
            Frequency::Voice => VOICE_FREQUENCY_WORD,
            Frequency::Value(value) => value,
        }
    }

    /// The word the voice is given: `own` for the sample record's (which may itself be
    /// `VOICE_FREQUENCY_WORD`, the voice's own).
    pub fn resolve(self, own: u32) -> u32 {
        match self {
            Frequency::Own => own,
            other => other.word(),
        }
    }
}

/// A place's three coordinates as float bits; 0xFF80_0000 in one is the sample record's.
pub(crate) type PlaceBits = [u32; 3];

/// Command 1: a sound started, changed or stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Start {
    /// The sound's number, negative for none.
    pub sound: i16,
    /// Nothing is started when the sound is not playing already (a playing one is changed).
    pub only_if_playing: bool,
    pub volume: Setting,
    pub pan: Setting,
    pub frequency: Frequency,
    /// The place's three coordinates (float bits), used in the 3D modes; none: the sample
    /// record's.
    pub place: Option<PlaceBits>,
}

impl Start {
    /// The command a sender's words make: `sound_word` holds the number in its low word and bit
    /// 31 for `only_if_playing`; a place whose three coordinates are all 0xFF80_0000 ("as the
    /// sample record says") is none.
    pub fn from_words(
        sound_word: u32,
        volume: u32,
        pan: u32,
        frequency: u32,
        place: PlaceBits,
    ) -> Start {
        Start {
            sound: sound_word as u16 as i16,
            only_if_playing: sound_word & 0x8000_0000 != 0,
            volume: Setting::from_word(volume),
            pan: Setting::from_word(pan),
            frequency: Frequency::from_word(frequency),
            place: (place != [OWN_PLACE_WORD; 3]).then_some(place),
        }
    }

    /// The words a record holds: the inverse of `from_words`.
    fn words(&self) -> [u32; 7] {
        let [x, y, z] = self.place.unwrap_or([OWN_PLACE_WORD; 3]);
        [
            self.sound as u16 as u32 | (self.only_if_playing as u32) << 31,
            self.volume.word(),
            self.pan.word(),
            self.frequency.word(),
            x,
            y,
            z,
        ]
    }
}

/// What a stream or music command asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StreamAction {
    /// Stop it.
    Stop,
    /// The audio thread's own turn: feed what plays.
    Turn,
    /// Play a file or tune.
    Play,
    /// Change the volume of what plays.
    Volume,
}

impl StreamAction {
    #[cfg(test)]
    /// The action a command word names (a word that is none of 0, 1, 3 plays).
    pub(crate) fn from_word(word: u32) -> StreamAction {
        match word {
            0 => StreamAction::Stop,
            1 => StreamAction::Turn,
            3 => StreamAction::Volume,
            _ => StreamAction::Play,
        }
    }

    /// The command word of the action.
    pub fn word(self) -> u32 {
        match self {
            StreamAction::Stop => 0,
            StreamAction::Turn => 1,
            StreamAction::Play => 2,
            StreamAction::Volume => 3,
        }
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
    Stream {
        path: u32,
        volume: u32,
        action: StreamAction,
    },
    /// 0xA: the turn's taking ends.
    EndOfTurn,
    /// 0xB: the music.
    Music {
        tune: u32,
        next: u32,
        volume: u32,
        action: StreamAction,
    },
}

impl Command {
    /// The command's number and seven words (the values the game's senders queued; those the
    /// audio thread does not read are 0). The sound hash of a run is made of these, so a held
    /// race's commands can be compared turn by turn; the words keep the order the game
    /// queued them in.
    pub fn record_words(&self) -> (u32, [u32; 7]) {
        match *self {
            Command::Start(s) => (1, s.words()),
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
            } => (9, [path, 0, volume, action.word(), 0, 0, 0]),
            Command::EndOfTurn => (0xA, [0; 7]),
            Command::Music {
                tune,
                next,
                volume,
                action,
            } => (0xB, [tune, next, volume, action.word(), 0, 0, 0]),
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

    /// Frozen: the byte offsets of the fields in the game's data layout of the sets, and its size.
    const FIELD_OFFSETS: [u32; 11] = [
        0x0, 0x198, 0x1F4, 0x36C, 0x504, 0x560, 0x6DC, 0x758, 0x830, 0x850, 0x9AC,
    ];
    const SETS_BYTES: u32 = 0x9B0;

    #[test]
    fn fields_keep_their_layout() {
        for (field, offset) in RaceField::ALL.into_iter().zip(FIELD_OFFSETS) {
            assert_eq!(field.original_offset(), offset, "{field:?}");
        }
        assert_eq!(RaceField::Tail.original_offset() + 4, SETS_BYTES);
    }

    /// Every list start reads on through the next fields exactly as a read of the flat dwords of
    /// the layout does, and ends at the sets' end.
    #[test]
    fn lists_read_on_as_flat_dwords() {
        let total = SETS_BYTES as usize / 4;
        for (k, field) in RaceField::ALL.into_iter().enumerate() {
            for entry in 0..field.dwords() as u32 {
                let start = FIELD_OFFSETS[k] as usize / 4 + entry as usize;
                for n in 0..3 {
                    let flat = start + n;
                    let want = if flat < total {
                        let at = FIELD_OFFSETS.iter().rposition(|&o| o as usize / 4 <= flat);
                        let at = at.unwrap();
                        ListPlace::At {
                            field: RaceField::ALL[at],
                            index: flat - FIELD_OFFSETS[at] as usize / 4,
                        }
                    } else {
                        ListPlace::PastSets
                    };
                    assert_eq!(field.list_place(entry, n), want, "{field:?} {entry} {n}");
                }
            }
        }
        assert_eq!(RaceField::Tail.list_place(1, 0), ListPlace::StartsPastSets);
        assert_eq!(
            RaceField::KindSetsFirst.list_place(u32::MAX, usize::MAX),
            ListPlace::StartsPastSets
        );
    }

    /// The word of a range of sounds names it back.
    #[test]
    fn a_range_keeps_its_word() {
        let list = SoundList::Range {
            first: 0x4C8,
            count: 190,
        };
        assert_eq!(SoundList::from_word(list.word()), Some(list));
    }

    /// A race list's command word is its old offset plus 2, and the word names the list back.
    #[test]
    fn race_lists_keep_their_words() {
        for (k, field) in RaceField::ALL.into_iter().enumerate() {
            for entry in 0..field.dwords() as u32 {
                let list = SoundList::Race { field, entry };
                let word = FIELD_OFFSETS[k] + 4 * entry + 2;
                assert_eq!(list.word(), word);
                assert_eq!(SoundList::from_word(word), Some(list));
                assert_eq!(
                    Command::Load(Some(list)).record_words(),
                    (3, [word, 0, 0, 0, 0, 0, 0])
                );
            }
        }
        assert_eq!(SoundList::from_word(0), None);
        assert_eq!(SoundList::from_word(1), Some(SoundList::Menu));
        assert_eq!(SoundList::from_word(SETS_BYTES + 2), None);
    }

    #[test]
    fn sounds_are_signed_words() {
        let start = |sound_word| Start::from_words(sound_word, 0, 0, 0, [0; 3]);
        assert_eq!(start(0x1_FFFE).sound, -2);
        assert_eq!(start(0x8000_0005).sound, 5);
        assert!(start(0x8000_0005).only_if_playing && !start(0x1_FFFE).only_if_playing);
    }

    #[test]
    fn settings_keep_their_words() {
        for word in [0, 0x40, 0xFFFF_FF90, 0xFFFF_FF91, 0xFFFF_FFC0] {
            assert_eq!(Setting::from_word(word).word(), word);
            assert_eq!(Frequency::from_word(word).word(), word);
        }
        assert_eq!(Setting::from_word(0xFFFF_FF91), Setting::Own);
        assert_eq!(Frequency::from_word(0xFFFF_FF90), Frequency::Voice);
        assert_eq!(Frequency::Voice.resolve(7), 0xFFFF_FF90);
        assert_eq!(Frequency::Own.resolve(7), 7);
        assert_eq!(Setting::Value(3).resolve(7), 3);
        assert_eq!(Start::from_words(1, 2, 3, 4, [0xFF80_0000; 3]).place, None);
        assert_eq!(
            Start::from_words(1, 2, 3, 4, [0xFF80_0000, 0, 0]).place,
            Some([0xFF80_0000, 0, 0])
        );
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
                "start sound={} only_if_playing={} volume={:?} pan={:?} frequency={:?} place={:?}",
                s.sound, s.only_if_playing, s.volume, s.pan, s.frequency, s.place
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
            } => format!("stream {path} {volume} {}", action.word()),
            Command::EndOfTurn => "end of turn".into(),
            Command::Music {
                tune,
                next,
                volume,
                action,
            } => format!("music {tune} {next} {volume} {}", action.word()),
        }
    }

    /// What the seven words a sender queued ask for command `number`: the inverse of
    /// `record_words`.
    fn command_from(number: u32, r: [u32; 7]) -> Command {
        match number {
            1 => Command::Start(Start::from_words(
                r[0],
                r[1],
                r[2],
                r[3],
                [r[4], r[5], r[6]],
            )),
            2 => Command::Room(r[0]),
            3 => Command::Load(SoundList::from_word(r[0])),
            4 => Command::Free,
            5 => Command::Mode(r[0]),
            7 => Command::StopAll,
            8 => Command::Volume(r[0]),
            9 => Command::Stream {
                path: r[0],
                volume: r[2],
                action: StreamAction::from_word(r[3]),
            },
            0xA => Command::EndOfTurn,
            0xB => Command::Music {
                tune: r[0],
                next: r[1],
                volume: r[2],
                action: StreamAction::from_word(r[3]),
            },
            n => panic!("command {n} is not sent"),
        }
    }

    const SENT: [(u32, [u32; 7]); 12] = [
        (1, [0xFFFE, 5, 0x7F, 0x2000, 1, 2, 3]),
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
        "start sound=-2 only_if_playing=false volume=Value(5) pan=Value(127) \
         frequency=Value(8192) place=Some([1, 2, 3])",
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

#[cfg(test)]
mod frozen_words {
    //! The words each command records, as the code before the typed values produced them
    //! (literals, kept): the sound hash of a run is made of them.
    use super::*;

    fn start(sound_word: u32, volume: u32, pan: u32, frequency: u32, place: [u32; 3]) -> Command {
        Command::Start(Start::from_words(sound_word, volume, pan, frequency, place))
    }

    #[test]
    fn start_words() {
        assert_eq!(
            start(5, 0x20, 0x7F, 0x2000, [1, 2, 3]).record_words(),
            (1, [5, 0x20, 0x7F, 0x2000, 1, 2, 3])
        );
        assert_eq!(
            start(0x8000_0005, 0, 0, 0, [0xFF80_0000; 3]).record_words(),
            (
                1,
                [0x8000_0005, 0, 0, 0, 0xFF80_0000, 0xFF80_0000, 0xFF80_0000]
            )
        );
        assert_eq!(
            start(
                0xFFFE,
                0xFFFF_FF91,
                0xFFFF_FF91,
                0xFFFF_FF91,
                [0xFF80_0000, 1, 2]
            )
            .record_words(),
            (
                1,
                [
                    0xFFFE,
                    0xFFFF_FF91,
                    0xFFFF_FF91,
                    0xFFFF_FF91,
                    0xFF80_0000,
                    1,
                    2
                ]
            )
        );
        assert_eq!(
            start(0x1CB, 0x40, 0xFFFF_FFC0, 0xFFFF_FF90, [0; 3]).record_words(),
            (1, [0x1CB, 0x40, 0xFFFF_FFC0, 0xFFFF_FF90, 0, 0, 0])
        );
    }

    #[test]
    fn stream_words() {
        let stream = |path, volume, action| Command::Stream {
            path,
            volume,
            action: StreamAction::from_word(action),
        };
        assert_eq!(
            stream(5, 0x30, 2).record_words(),
            (9, [5, 0, 0x30, 2, 0, 0, 0])
        );
        assert_eq!(stream(0, 0, 0).record_words(), (9, [0; 7]));
    }

    #[test]
    fn music_words() {
        let music = |tune, next, volume, action| Command::Music {
            tune,
            next,
            volume,
            action: StreamAction::from_word(action),
        };
        assert_eq!(
            music(4, 8, 0x30, 2).record_words(),
            (0xB, [4, 8, 0x30, 2, 0, 0, 0])
        );
        assert_eq!(
            music(0, 0, 0x20, 3).record_words(),
            (0xB, [0, 0, 0x20, 3, 0, 0, 0])
        );
        assert_eq!(music(0, 0, 0, 0).record_words(), (0xB, [0; 7]));
    }
}
