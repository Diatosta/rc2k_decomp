//! The commands the game queues for the audio thread (from
//! `audio::take_commands` 0x40A8A4): a record of 0x20 bytes, the number and then the
//! sender's eax, ebx, ecx, edx, esi, edi, ebp; 0x400 records, read round.

/// The queue's records.
pub const RECORDS: u32 = 0x400;

/// A list of sounds command 3 loads, by handle (the original sent its address). The menus'
/// list, or a list in the race's sets at an offset from the sets' start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// A record, by its number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// 1: a sound started, changed or stopped (the sender's ax, signed).
    Start(i32),
    /// 2: the EAX room asked.
    Room(u32),
    /// 3: a list of sounds loaded (none: nothing).
    Load(Option<SoundList>),
    /// 4: every voice given back.
    Free,
    /// 5: the sound mode.
    Mode(u32),
    /// 6: nothing.
    Six,
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
    /// Any other: nothing.
    Other(u32),
}

/// What a record asks.
pub fn decode(record: &[u32; 8]) -> Command {
    let [number, eax, ebx, ecx, edx, ..] = *record;
    match number {
        1 => Command::Start(eax as u16 as i16 as i32),
        2 => Command::Room(eax),
        3 => Command::Load(SoundList::from_word(eax)),
        4 => Command::Free,
        5 => Command::Mode(eax),
        6 => Command::Six,
        7 => Command::StopAll,
        8 => Command::Volume(eax),
        9 => Command::Stream {
            path: eax,
            volume: ecx,
            action: edx,
        },
        0xA => Command::EndOfTurn,
        0xB => Command::Music {
            tune: eax,
            next: ebx,
            volume: ecx,
            action: edx,
        },
        n => Command::Other(n),
    }
}

/// The read place after `place` (round at 0x400).
pub fn next_place(place: u32) -> u32 {
    if place + 1 >= RECORDS { 0 } else { place + 1 }
}

/// Where a command goes into the queue (from `sound::send_command`
/// 0x409C45): without audio the count is cleared and nothing is queued; a full queue
/// takes nothing; else the record at `write`, and the write place after it (round).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Put {
    NoAudio,
    Full,
    At { slot: u32, next: u32 },
}

pub fn put(no_audio: bool, count: u32, write: u32) -> Put {
    if no_audio {
        Put::NoAudio
    } else if count == RECORDS {
        Put::Full
    } else {
        let next = write.wrapping_add(1);
        Put::At {
            slot: write,
            next: if next >= RECORDS { 0 } else { next },
        }
    }
}

/// A command's record: its number and the sender's eax, ebx, ecx, edx, esi, edi, ebp.
pub fn record(command: u32, registers: [u32; 7]) -> [u32; 8] {
    let mut r = [command; 8];
    r[1..].copy_from_slice(&registers);
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue() {
        assert_eq!(
            (put(true, 0, 0), put(false, RECORDS, 0)),
            (Put::NoAudio, Put::Full)
        );
        assert_eq!(
            put(false, 1, RECORDS - 1),
            Put::At {
                slot: RECORDS - 1,
                next: 0
            }
        );
        assert_eq!(record(9, [1, 2, 3, 4, 5, 6, 7])[..2], [9, 1]);
    }

    #[test]
    fn records() {
        assert_eq!(decode(&[1, 0x1_FFFE, 0, 0, 0, 0, 0, 0]), Command::Start(-2));
        assert_eq!(
            decode(&[0xB, 4, 8, 0x30, 2, 0, 0, 0]),
            Command::Music {
                tune: 4,
                next: 8,
                volume: 0x30,
                action: 2
            }
        );
        assert_eq!(decode(&[3, 0, 0, 0, 0, 0, 0, 0]), Command::Load(None));
        for list in [
            SoundList::Menu,
            SoundList::Race { offset: 0 },
            SoundList::Race { offset: 0x7C },
        ] {
            assert_eq!(
                decode(&[3, list.word(), 0, 0, 0, 0, 0, 0]),
                Command::Load(Some(list))
            );
        }
        assert_eq!(decode(&[0xC, 0, 0, 0, 0, 0, 0, 0]), Command::Other(0xC));
        assert_eq!((next_place(5), next_place(0x3FF)), (6, 0));
    }
}
