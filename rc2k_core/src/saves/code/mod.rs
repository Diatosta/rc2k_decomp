//! The player's state as a code: text that a QR code carries, made from the settings, the best
//! times, the car set-ups and the saved games, and read back into them. See the wiki page
//! "Save code".
//!
//! The text is `RC2K:` and base 45 letters of a frame and `:`. The frame is a version byte
//! (1), a flags byte (0), a CRC-32 of the rest, then the payload, deflated. The payload is
//! made of sections (settings, best times, set-ups, saved games), each there or not; a
//! section that is there replaces what the importing game has of its kind (set-ups and saved
//! games slot by slot). Everything is read into typed values and checked before anything is
//! written; a code is untrusted input.

pub mod base45;
pub mod best;
pub mod qr;
pub mod settings;
mod wire;

use crate::options::Options;
use best::{RallyTime, Real, StageRecord};
use wire::{Reader, Writer};

/// The code's text begins with this and ends with `END`.
pub const START: &str = "RC2K:";
pub const END: &str = ":";
/// The frame's version this program writes and reads.
pub const VERSION: u8 = 1;
/// A saved game's state in its file: after the 0x30-byte head, this many bytes.
pub const GAME_STATE_LEN: usize = 0x725C;
/// The car set-ups and saved games have this many slots.
pub const SLOTS: usize = 100;
/// A name's room in a slot.
pub const NAME_LEN: usize = 0x20;
/// How many values a set-up has.
pub const SETUP_VALUES: usize = 6;
/// The payload, inflated, may not be longer than this (what a hundred saved games come to).
const MOST_PAYLOAD: usize = 4 * 1024 * 1024;
const MOST_PROGRAM: usize = 32;
const MOST_ENTRIES: usize = 4096;
/// A set-up's values are small numbers (the choices of its page).
const MOST_SETUP_VALUE: u32 = 0x1000;

/// What went wrong with a code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeError {
    /// The text is not a code at all.
    NotACode,
    /// A code of a version this program does not know.
    Version(u8),
    /// The check number does not match: the text was changed or cut.
    Checksum,
    /// The code is not made as a code is.
    Damaged(&'static str),
    /// A value out of its range.
    Range(&'static str),
}

impl std::fmt::Display for CodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotACode => f.write_str("not a save code"),
            Self::Version(v) => write!(f, "a save code of version {v}"),
            Self::Checksum => f.write_str("the save code's check number is wrong"),
            Self::Damaged(why) | Self::Range(why) => f.write_str(why),
        }
    }
}

impl std::error::Error for CodeError {}

/// A car set-up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setup {
    pub slot: u8,
    pub name: [u8; NAME_LEN],
    pub values: [u32; SETUP_VALUES],
}

/// A saved game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub slot: u8,
    pub name: [u8; NAME_LEN],
    /// The state, the bytes of the game's file after its head.
    pub state: Vec<u8>,
}

/// The best times of the code.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BestTimes {
    pub real: Vec<Real>,
    pub rallies: Vec<RallyTime>,
    pub records: Vec<StageRecord>,
    /// The tyre change's record holder and time.
    pub tyre_change: Option<([u8; NAME_LEN], u32)>,
}

impl BestTimes {
    /// How many entries a player made.
    pub fn count(&self) -> usize {
        self.real.len() + self.rallies.len() + self.records.len()
    }

    /// The times without their checkpoint lines (the sixteen split times of a best time, the
    /// bulk of an entry and numbers that do not compress): a time then has no splits to be
    /// compared with.
    pub fn without_lines(&self) -> Self {
        let cut = |entry: &mut Vec<u8>| {
            if entry.len() == crate::saves::entries::TIME_LEN {
                entry[0x5C..].fill(0);
            }
        };
        let mut out = self.clone();
        out.real.iter_mut().for_each(|r| cut(&mut r.entry));
        out.rallies.iter_mut().for_each(|r| cut(&mut r.entry));
        out
    }
}

/// The player's state: each section there or not.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerState {
    /// When it was made, in seconds from 1970.
    pub made: u32,
    /// The program that made it.
    pub program: String,
    /// The settings' text (`settings`).
    pub settings: Option<String>,
    pub best: Option<BestTimes>,
    pub setups: Option<Vec<Setup>>,
    pub games: Option<Vec<Game>>,
}

/// CRC-32 (the one of zip and PNG).
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn check_name(name: &[u8]) -> Result<(), CodeError> {
    let text = name.iter().take_while(|&&b| b != 0);
    if !name.contains(&0) || text.clone().any(|&b| b < 0x20) {
        return Err(CodeError::Range("a name that is not one"));
    }
    Ok(())
}

impl PlayerState {
    /// The payload's bytes.
    fn payload(&self) -> Vec<u8> {
        let mut w = Writer::default();
        w.u32(self.made);
        w.bytes(self.program.as_bytes());
        let flags = u8::from(self.settings.is_some())
            | u8::from(self.best.is_some()) << 1
            | u8::from(self.setups.is_some()) << 2
            | u8::from(self.games.is_some()) << 3;
        w.u8(flags);
        if let Some(text) = &self.settings {
            w.bytes(text.as_bytes());
        }
        if let Some(best) = &self.best {
            w.varint(best.real.len() as u32);
            for r in &best.real {
                w.u8(r.table);
                w.varint(u32::from(r.list));
                w.bytes(&r.entry);
            }
            w.varint(best.rallies.len() as u32);
            for r in &best.rallies {
                w.u8(r.rally);
                w.u8(r.list);
                w.bytes(&r.entry);
            }
            w.varint(best.records.len() as u32);
            for r in &best.records {
                w.u8(r.group);
                w.u8(r.event);
                w.varint(u32::from(r.index));
                w.bytes(&r.bytes);
            }
            match &best.tyre_change {
                Some((name, time)) => {
                    w.u8(1);
                    w.raw(name);
                    w.varint(*time);
                }
                None => w.u8(0),
            }
        }
        if let Some(setups) = &self.setups {
            w.varint(setups.len() as u32);
            for s in setups {
                w.u8(s.slot);
                w.raw(&s.name);
                for v in s.values {
                    w.varint(v);
                }
            }
        }
        if let Some(games) = &self.games {
            w.varint(games.len() as u32);
            for g in games {
                w.u8(g.slot);
                w.raw(&g.name);
                w.raw(&g.state);
            }
        }
        w.0
    }

    /// The code's text.
    pub fn encode(&self) -> String {
        let deflated = miniz_oxide::deflate::compress_to_vec(&self.payload(), 10);
        let mut frame = vec![VERSION, 0, 0, 0, 0, 0];
        frame.extend_from_slice(&deflated);
        let crc = crc32(&[&frame[..2], &frame[6..]].concat());
        frame[2..6].copy_from_slice(&crc.to_le_bytes());
        format!("{START}{}{END}", base45::encode(&frame))
    }

    /// The state a code's text gives. Refused: text that is not a code, a version this program
    /// does not know, a changed or cut text, a value out of its range. Nothing is done with
    /// the state; the caller writes it where it goes.
    pub fn decode(text: &str) -> Result<Self, CodeError> {
        let letters: String = text.chars().filter(|c| !"\r\n\t".contains(*c)).collect();
        let letters = letters.trim();
        let inner = letters
            .strip_prefix(START)
            .and_then(|t| t.strip_suffix(END))
            .ok_or(CodeError::NotACode)?;
        let frame = base45::decode(inner)?;
        if frame.len() < 7 {
            return Err(CodeError::NotACode);
        }
        if frame[0] != VERSION {
            return Err(CodeError::Version(frame[0]));
        }
        if frame[2..6] != crc32(&[&frame[..2], &frame[6..]].concat()).to_le_bytes() {
            return Err(CodeError::Checksum);
        }
        if frame[1] != 0 {
            return Err(CodeError::Damaged("a flag this program does not know"));
        }
        let payload = miniz_oxide::inflate::decompress_to_vec_with_limit(&frame[6..], MOST_PAYLOAD)
            .map_err(|_| CodeError::Damaged("the code does not unpack"))?;
        Self::from_payload(&payload)
    }

    fn from_payload(payload: &[u8]) -> Result<Self, CodeError> {
        let mut r = Reader::new(payload);
        let made = r.u32()?;
        let program = String::from_utf8(r.bytes(MOST_PROGRAM)?.to_vec())
            .map_err(|_| CodeError::Range("a program name that is not text"))?;
        let flags = r.u8()?;
        if flags > 0xF {
            return Err(CodeError::Damaged("a section this program does not know"));
        }
        let mut state = Self {
            made,
            program,
            ..Self::default()
        };
        if flags & 1 != 0 {
            let text = r.bytes(settings::MOST_TEXT)?;
            state.settings = Some(
                String::from_utf8(text.to_vec())
                    .map_err(|_| CodeError::Range("settings that are not text"))?,
            );
        }
        if flags & 2 != 0 {
            state.best = Some(Self::read_best(&mut r)?);
        }
        if flags & 4 != 0 {
            let n = r.count(SLOTS, "more set-ups than slots")?;
            let mut setups = Vec::with_capacity(n);
            for _ in 0..n {
                let slot = r.u8()?;
                let name: [u8; NAME_LEN] = r.raw(NAME_LEN)?.try_into().expect("a name");
                check_name(&name)?;
                let mut values = [0; SETUP_VALUES];
                for v in &mut values {
                    *v = r.varint()?;
                    if *v > MOST_SETUP_VALUE {
                        return Err(CodeError::Range("a set-up value out of its range"));
                    }
                }
                if usize::from(slot) >= SLOTS {
                    return Err(CodeError::Range("a set-up of a slot there is not"));
                }
                setups.push(Setup { slot, name, values });
            }
            state.setups = Some(setups);
        }
        if flags & 8 != 0 {
            let n = r.count(SLOTS, "more saved games than slots")?;
            let mut games = Vec::with_capacity(n);
            for _ in 0..n {
                let slot = r.u8()?;
                let name: [u8; NAME_LEN] = r.raw(NAME_LEN)?.try_into().expect("a name");
                check_name(&name)?;
                let bytes = r.raw(GAME_STATE_LEN)?.to_vec();
                if usize::from(slot) >= SLOTS {
                    return Err(CodeError::Range("a saved game of a slot there is not"));
                }
                games.push(Game {
                    slot,
                    name,
                    state: bytes,
                });
            }
            state.games = Some(games);
        }
        r.finish()?;
        for (list, what) in [
            (
                state
                    .setups
                    .as_ref()
                    .map(|s| s.iter().map(|x| x.slot).collect::<Vec<_>>()),
                "set-ups",
            ),
            (
                state
                    .games
                    .as_ref()
                    .map(|s| s.iter().map(|x| x.slot).collect::<Vec<_>>()),
                "saved games",
            ),
        ] {
            let mut slots = list.unwrap_or_default();
            slots.sort_unstable();
            if slots.windows(2).any(|w| w[0] == w[1]) {
                return Err(CodeError::Range(match what {
                    "set-ups" => "two set-ups in a slot",
                    _ => "two saved games in a slot",
                }));
            }
        }
        Ok(state)
    }

    fn read_best(r: &mut Reader<'_>) -> Result<BestTimes, CodeError> {
        let mut best = BestTimes::default();
        for _ in 0..r.count(MOST_ENTRIES, "too many best times")? {
            let table = r.u8()?;
            let list = u16::try_from(r.varint()?)
                .map_err(|_| CodeError::Range("a best time of a list there is not"))?;
            let entry = r.bytes(0x9C)?.to_vec();
            best.real.push(Real { table, list, entry });
        }
        for _ in 0..r.count(MOST_ENTRIES, "too many rally times")? {
            let (rally, list) = (r.u8()?, r.u8()?);
            let entry = r.bytes(0x9C)?.to_vec();
            best.rallies.push(RallyTime { rally, list, entry });
        }
        for _ in 0..r.count(MOST_ENTRIES, "too many stage records")? {
            let (group, event) = (r.u8()?, r.u8()?);
            let index = u16::try_from(r.varint()?)
                .map_err(|_| CodeError::Range("a stage record of a place there is not"))?;
            let bytes = r.bytes(best::RECORD_LEN)?.to_vec();
            best.records.push(StageRecord {
                group,
                event,
                index,
                bytes,
            });
        }
        if r.u8()? == 1 {
            let name: [u8; NAME_LEN] = r.raw(NAME_LEN)?.try_into().expect("a name");
            check_name(&name)?;
            best.tyre_change = Some((name, r.varint()?));
        }
        best::check(&best.real)?;
        best::check_rallies(&best.rallies)?;
        best::check_records(&best.records)?;
        Ok(best)
    }

    /// The state cut down to `level` of a QR code's room (0 the whole state).
    pub fn trimmed(&self, level: Trim) -> Self {
        let mut out = self.clone();
        if level >= Trim::NoGames {
            out.games = None;
        }
        if level >= Trim::NoLines {
            out.best = out.best.map(|b| b.without_lines());
        }
        if level >= Trim::NoRecords
            && let Some(b) = &mut out.best
        {
            b.rallies.clear();
            b.records.clear();
        }
        if level >= Trim::NoBestTimes {
            out.best = None;
        }
        out
    }

    /// The code for a QR code: the whole state if it fits, else the state cut down step by
    /// step (`Trim`) until it does. With the cut that was made.
    pub fn for_qr(&self) -> (String, Trim) {
        for level in [Trim::Whole, Trim::NoGames, Trim::NoLines, Trim::NoRecords] {
            let text = self.trimmed(level).encode();
            if text.len() <= qr::MOST_LETTERS && qr::encode(&text).is_some() {
                return (text, level);
            }
        }
        (self.trimmed(Trim::NoBestTimes).encode(), Trim::NoBestTimes)
    }

    pub fn sections(&self) -> Sections {
        Sections {
            settings: self.settings.is_some(),
            best_times: self.best.is_some(),
            records: self
                .best
                .as_ref()
                .is_some_and(|b| !b.records.is_empty() || !b.rallies.is_empty()),
            setups: self.setups.is_some(),
            games: self.games.is_some(),
        }
    }

    /// What the state holds, for the question before it is taken in. `current` gives the
    /// options the settings are read over (the settings' text may be damaged: refused).
    pub fn summary(&self, current: &Options) -> Result<Summary, CodeError> {
        let options = match &self.settings {
            Some(text) => Some(settings::options_of(text, current)?),
            None => None,
        };
        Ok(Summary {
            made: self.made,
            program: self.program.clone(),
            players: options.as_ref().map(names_of).unwrap_or_default(),
            runs: options.as_ref().map(|o| o.kept.runs),
            best_times: self.best.as_ref().map(BestTimes::count),
            setups: self.setups.as_ref().map(Vec::len),
            games: self
                .games
                .as_ref()
                .map(|g| g.iter().map(|g| (g.slot, name_text(&g.name))).collect()),
        })
    }
}

/// The four players' names in `options`.
pub fn names_of(o: &Options) -> Vec<String> {
    o.players.names.iter().map(|n| name_text(&n.0)).collect()
}

/// A name's bytes (Windows-1252, to the first NUL) as text.
pub fn name_text(name: &[u8]) -> String {
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    crate::files::ansi::ansi_text(&name[..end])
}

/// How much of a state a QR code leaves out, each step with the ones before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trim {
    Whole,
    /// Without the saved games.
    NoGames,
    /// And the best times without their checkpoint lines.
    NoLines,
    /// And without the rally times and stage records.
    NoRecords,
    /// And without any best times.
    NoBestTimes,
}

/// Which sections a code holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sections {
    pub settings: bool,
    pub best_times: bool,
    /// The rally times and stage records among the best times.
    pub records: bool,
    pub setups: bool,
    pub games: bool,
}

/// What a state holds, to be shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub made: u32,
    pub program: String,
    /// The four players' names; none when the state has no settings.
    pub players: Vec<String>,
    pub runs: Option<u32>,
    pub best_times: Option<usize>,
    pub setups: Option<usize>,
    /// The saved games by slot and name.
    pub games: Option<Vec<(u8, String)>>,
}

/// The state of a fresh install's size and a played one's, for the tests and the wiki.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::saves::entries::{BestTime, TIME_LEN};

    fn name(text: &[u8]) -> [u8; NAME_LEN] {
        let mut n = [0; NAME_LEN];
        n[..text.len()].copy_from_slice(text);
        n
    }

    pub fn sample() -> PlayerState {
        let mut options = Options::default();
        options.players.names[0].0[..4].copy_from_slice(b"Ana\0");
        options.sound.master_volume = 64;
        options.kept.runs = 12;
        let mut real = BestTime::default();
        real.head.player = 1;
        real.head.name[..3].copy_from_slice(b"Ana");
        real.head.key = 123_456;
        let mut entry = vec![0; TIME_LEN];
        real.write(&mut entry);
        PlayerState {
            made: 1_800_000_000,
            program: "0.1.0".into(),
            settings: Some(settings::text_of(&options).unwrap()),
            best: Some(BestTimes {
                real: vec![Real {
                    table: 5,
                    list: 3,
                    entry,
                }],
                tyre_change: Some((name(b"Ana"), 4321)),
                ..BestTimes::default()
            }),
            setups: Some(vec![Setup {
                slot: 1,
                name: name(b"Gravel"),
                values: [1, 2, 3, 4, 5, 6],
            }]),
            games: Some(vec![Game {
                slot: 4,
                name: name(b"Before Wales"),
                state: (0..GAME_STATE_LEN).map(|n| (n * 31 % 7) as u8).collect(),
            }]),
        }
    }

    #[test]
    fn a_state_survives_its_text() {
        let s = sample();
        let text = s.encode();
        assert!(text.starts_with("RC2K:") && text.ends_with(':'));
        assert_eq!(PlayerState::decode(&text).unwrap(), s);
        // Line breaks (a wrapped paste) and the spaces around do not matter.
        let wrapped: String = text
            .as_bytes()
            .chunks(40)
            .map(|c| format!("{}\r\n", std::str::from_utf8(c).unwrap()))
            .collect();
        assert_eq!(PlayerState::decode(&format!("  {wrapped}\n")).unwrap(), s);
    }

    #[test]
    fn a_changed_text_is_refused() {
        let text = sample().encode();
        let mut accepted = 0;
        for at in 0..text.len() {
            for c in "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:".chars() {
                let mut bytes = text.clone().into_bytes();
                if char::from(bytes[at]) == c {
                    continue;
                }
                bytes[at] = c as u8;
                let changed = String::from_utf8(bytes).unwrap();
                // A changed letter is never taken: the check number sees it (the start and the
                // end, which are not in it, make it no code at all).
                if PlayerState::decode(&changed).is_ok() {
                    accepted += 1;
                }
            }
        }
        assert_eq!(accepted, 0);
        assert_eq!(
            PlayerState::decode(&text[..text.len() - 3]),
            Err(CodeError::NotACode)
        );
        assert!(PlayerState::decode("hello").is_err());
        assert!(PlayerState::decode("RC2K::").is_err());
    }

    /// Frames with a good check number but a bad payload.
    fn framed(payload: &[u8]) -> String {
        let deflated = miniz_oxide::deflate::compress_to_vec(payload, 6);
        let mut frame = vec![VERSION, 0, 0, 0, 0, 0];
        frame.extend_from_slice(&deflated);
        let crc = crc32(&[&frame[..2], &frame[6..]].concat());
        frame[2..6].copy_from_slice(&crc.to_le_bytes());
        format!("{START}{}{END}", base45::encode(&frame))
    }

    #[test]
    fn values_out_of_their_range_are_refused() {
        let mut s = sample();
        // A set-up value.
        s.setups.as_mut().unwrap()[0].values[2] = 99_999;
        assert!(matches!(
            PlayerState::decode(&framed(&s.payload())),
            Err(CodeError::Range(_))
        ));
        // A slot past the hundred.
        let mut s = sample();
        s.games.as_mut().unwrap()[0].slot = 100;
        assert!(PlayerState::decode(&framed(&s.payload())).is_err());
        // Two games in a slot.
        let mut s = sample();
        let g = s.games.as_ref().unwrap()[0].clone();
        s.games.as_mut().unwrap().push(g);
        assert!(PlayerState::decode(&framed(&s.payload())).is_err());
        // A name without its end.
        let mut s = sample();
        s.setups.as_mut().unwrap()[0].name = [b'x'; NAME_LEN];
        assert!(PlayerState::decode(&framed(&s.payload())).is_err());
        // A best time with a player 9; settings out of range.
        let mut s = sample();
        s.best.as_mut().unwrap().real[0].entry[4] = 9;
        assert!(PlayerState::decode(&framed(&s.payload())).is_err());
        let mut s = sample();
        s.settings = Some("[sound]\nmaster_volume = 70000\n".into());
        let back = PlayerState::decode(&framed(&s.payload())).unwrap();
        assert!(back.summary(&Options::default()).is_err());
        // Bytes left over, and a payload cut short.
        let mut p = sample().payload();
        p.push(0);
        assert!(PlayerState::decode(&framed(&p)).is_err());
        p.truncate(p.len() - 30);
        assert!(PlayerState::decode(&framed(&p)).is_err());
    }

    #[test]
    fn the_version_and_the_flags_are_checked() {
        let text = sample().encode();
        let inner = text.strip_prefix(START).unwrap().strip_suffix(END).unwrap();
        let mut frame = base45::decode(inner).unwrap();
        frame[0] = 2;
        let again = |frame: &[u8]| format!("{START}{}{END}", base45::encode(frame));
        assert_eq!(
            PlayerState::decode(&again(&frame)),
            Err(CodeError::Version(2))
        );
        frame[0] = 1;
        frame[10] ^= 1;
        assert_eq!(
            PlayerState::decode(&again(&frame)),
            Err(CodeError::Checksum)
        );
    }

    #[test]
    fn a_deflate_bomb_is_not_unpacked() {
        let bomb = vec![0u8; 8 * 1024 * 1024];
        assert!(PlayerState::decode(&framed(&bomb)).is_err());
    }

    #[test]
    fn the_summary_names_what_is_there() {
        let s = sample();
        let sum = s.summary(&Options::default()).unwrap();
        assert_eq!(sum.players[0], "Ana");
        assert_eq!(sum.runs, Some(12));
        assert_eq!(sum.best_times, Some(1));
        assert_eq!(sum.setups, Some(1));
        assert_eq!(sum.games, Some(vec![(4, "Before Wales".to_string())]));
    }

    #[test]
    fn the_qr_text_drops_the_saved_games_when_they_do_not_fit() {
        let mut s = sample();
        // Saved games that do not compress.
        let mut x = 0x1234_5678u32;
        let noise = |x: &mut u32| {
            *x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (*x >> 24) as u8
        };
        s.games = Some(vec![Game {
            slot: 0,
            name: name(b"big"),
            state: (0..GAME_STATE_LEN).map(|_| noise(&mut x)).collect(),
        }]);
        let (text, trim) = s.for_qr();
        assert_eq!(trim, Trim::NoGames);
        assert!(qr::encode(&text).is_some());
        let (_, trim) = sample().for_qr();
        assert_eq!(trim, Trim::Whole);
    }

    #[test]
    fn the_qr_round_trip_through_a_picture() {
        let (text, _) = sample().for_qr();
        let modules = qr::encode(&text).unwrap();
        let (width, pixels) = qr::render(&modules, 4);
        let read = qr::read(width, width, &pixels).unwrap();
        assert_eq!(
            PlayerState::decode(&read)
                .unwrap()
                .summary(&Options::default())
                .unwrap()
                .players[0],
            "Ana"
        );
    }
}
