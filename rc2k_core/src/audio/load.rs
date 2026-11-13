//! Command 3, a list of sounds loaded (from `audio::load_sounds`
//! 0x40AEB9): sound numbers, -2 passed over, ended by -1. For each sound whose record has
//! a file (`data_key`) and `copies` above 0 (signed), that many voices while fewer than
//! 0x200 are in use: a voice already made from the file shares its data, else the file is
//! read (failing ends the sound's voices); a new voice (the record's first if it has none),
//! made ready (filled, its mode, prepared, kept, its buffer made and filled) and marked
//! loaded. After each entry of the list the quit switch ends it.

use super::voices::{SAMPLES, SampleRecord};
use crate::Invalid;

/// The list's end and a sound passed over.
pub const END: u32 = 0xFFFF_FFFF;
pub const SKIP: u32 = 0xFFFF_FFFE;
/// The most entries looked at for the end.
pub const LIST_ROOM: usize = 0x1000;

/// The list's entries up to its end. Refused: no end within 0x1000 entries, or a sound
/// of 0x1CD or more.
pub fn entries(list: impl Fn(usize) -> u32) -> Result<Vec<u32>, Invalid> {
    let mut out = Vec::new();
    loop {
        let entry = list(out.len());
        if entry == END {
            return Ok(out);
        }
        if entry != SKIP && entry >= SAMPLES {
            return Err(Invalid("a sound of 0x1CD or more in the list"));
        }
        out.push(entry);
        if out.len() == LIST_ROOM {
            return Err(Invalid("a list without its end"));
        }
    }
}

/// Where a step is: the entry's place in the list, its sound, the voices still to make.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct At {
    pub entry: usize,
    pub sound: u32,
    pub left: u32,
}

/// What loading asks of the audio thread.
pub trait Loader {
    type Error;
    fn record(&mut self, sound: u32) -> SampleRecord;
    fn set_first_voice(&mut self, sound: u32, voice: u32);
    /// All 0x200 voices in use.
    fn full(&mut self) -> bool;
    /// A voice already made from the record's file (0: none).
    fn shared(&mut self, at: At) -> Result<u32, Self::Error>;
    /// The record's file read: false when it failed.
    fn read(&mut self, at: At) -> Result<bool, Self::Error>;
    fn new_voice(&mut self, at: At, shared: u32) -> Result<u32, Self::Error>;
    /// The voice filled (from the file or `shared`), its mode, prepared, kept, its buffer
    /// made and filled.
    fn make_ready(&mut self, at: At, voice: u32, shared: u32) -> Result<(), Self::Error>;
    fn mark_loaded(&mut self, voice: u32);
    fn quitting(&mut self) -> bool;
}

/// The entries loaded.
pub fn load<L: Loader>(l: &mut L, entries: &[u32]) -> Result<(), L::Error> {
    for (entry, &sound) in entries.iter().enumerate() {
        if sound != SKIP {
            let record = l.record(sound);
            if record.data_key != 0 && record.copies > 0 {
                let mut left = record.copies as u32;
                while !l.full() {
                    let at = At { entry, sound, left };
                    let shared = l.shared(at)?;
                    if shared == 0 && !l.read(at)? {
                        break;
                    }
                    let voice = l.new_voice(at, shared)?;
                    if l.record(sound).first_voice == 0 {
                        l.set_first_voice(sound, voice);
                    }
                    l.make_ready(at, voice, shared)?;
                    l.mark_loaded(voice);
                    left -= 1;
                    if left == 0 {
                        break;
                    }
                }
            }
        }
        if l.quitting() {
            return Ok(());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Test {
        records: Vec<SampleRecord>,
        voices: u32,
        log: Vec<String>,
        unreadable: u32,
    }

    impl Loader for Test {
        type Error = ();
        fn record(&mut self, n: u32) -> SampleRecord {
            self.records[n as usize]
        }
        fn set_first_voice(&mut self, n: u32, voice: u32) {
            self.records[n as usize].first_voice = voice;
        }
        fn full(&mut self) -> bool {
            self.voices == 0x200
        }
        fn shared(&mut self, at: At) -> Result<u32, ()> {
            Ok(if at.sound == 2 { 0x99 } else { 0 })
        }
        fn read(&mut self, at: At) -> Result<bool, ()> {
            Ok(at.sound != self.unreadable)
        }
        fn new_voice(&mut self, _: At, _: u32) -> Result<u32, ()> {
            self.voices += 1;
            Ok(0x1000 + self.voices)
        }
        fn make_ready(&mut self, at: At, v: u32, s: u32) -> Result<(), ()> {
            self.log
                .push(format!("{} {} {v:x} {s:x}", at.sound, at.left));
            Ok(())
        }
        fn mark_loaded(&mut self, _: u32) {}
        fn quitting(&mut self) -> bool {
            false
        }
    }

    #[test]
    fn lists() {
        let list = [1, SKIP, 2, 3, END];
        assert_eq!(entries(|n| list[n]).unwrap(), [1, SKIP, 2, 3]);
        assert!(entries(|n| [0x1CD, END][n]).is_err());
        assert!(entries(|_| 0).is_err());
        let mut t = Test {
            records: vec![SampleRecord::default(); 4],
            unreadable: 3,
            ..Test::default()
        };
        for item in t.records.iter_mut().skip(1).take(4 - 1) {
            *item = SampleRecord {
                data_key: 7,
                copies: 2,
                ..SampleRecord::default()
            };
        }
        load(&mut t, &[1, SKIP, 2, 3]).unwrap();
        assert_eq!(
            t.log,
            ["1 2 1001 0", "1 1 1002 0", "2 2 1003 99", "2 1 1004 99"]
        );
        assert_eq!(
            (t.records[1].first_voice, t.records[3].first_voice),
            (0x1001, 0)
        );
    }
}
