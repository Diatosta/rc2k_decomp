//! The player's state as a code: text that QR codes carry, made from the saved games, the car
//! set-ups and the best times, and read back into them. See the wiki page "Save code".
//!
//! The text is `RC2K:` and base 45 letters of a frame and `:`. The frame is a version byte
//! (2), a flags byte (0), a CRC-32 of the rest, then the payload, deflated. The payload is
//! made of sections (saved games, set-ups, best times), each there or not; a section that is
//! there replaces what the importing game has of its kind (set-ups and saved games slot by
//! slot). The settings are not in it. A text too long for one QR code is cut into numbered
//! parts (`parts`). Everything is read into typed values and checked before anything is
//! written; a code is untrusted input.

pub mod base45;
pub mod best;
mod game_pack;
pub mod parts;
pub mod qr;
#[cfg(test)]
mod qr_frames;
mod wire;
use best::{RallyTime, Real, StageRecord};
use wire::{Reader, Writer};

/// The code's text begins with this and ends with `END`.
pub const START: &str = "RC2K:";
pub const END: &str = ":";
/// The frame's version this program writes.
pub const VERSION: u8 = 2;
/// The first version this program still reads. Its payload began with the settings (now left
/// out: read past and ignored) and its saved games were not rearranged.
const FIRST_VERSION: u8 = 1;
/// The most the first version's settings text could be.
const MOST_OLD_SETTINGS: usize = 64 * 1024;
/// A saved game's state in its file: after the 0x30-byte head, this many bytes.
pub const GAME_STATE_LEN: usize = 0x725C;
const _: () = assert!(GAME_STATE_LEN == crate::saves::savegame::LEN);
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
    /// The text is one part of a code cut into parts.
    OnePart,
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
            Self::OnePart => f.write_str("one part of a longer code: read all its parts"),
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
}

/// The player's state: each section there or not.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerState {
    /// When it was made, in seconds from 1970.
    pub made: u32,
    /// The program that made it.
    pub program: String,
    pub games: Option<Vec<Game>>,
    pub setups: Option<Vec<Setup>>,
    pub best: Option<BestTimes>,
}

/// CRC-32 (the one of zip and PNG).
pub(crate) fn crc32(data: &[u8]) -> u32 {
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
    /// The payload's bytes: the saved games first, then the set-ups, then the best times.
    fn payload(&self) -> Vec<u8> {
        let mut w = Writer::default();
        w.u32(self.made);
        w.bytes(self.program.as_bytes());
        let flags = u8::from(self.best.is_some()) << 1
            | u8::from(self.setups.is_some()) << 2
            | u8::from(self.games.is_some()) << 3;
        w.u8(flags);
        if let Some(games) = &self.games {
            w.varint(games.len() as u32);
            for g in games {
                w.u8(g.slot);
                w.raw(&g.name);
                w.raw(&game_pack::pack(&g.state));
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
        w.0
    }

    /// The code's text, whole (a QR code holds it, or `parts::split` cuts it).
    pub fn encode(&self) -> String {
        Self::frame(VERSION, &self.payload())
    }

    fn frame(version: u8, payload: &[u8]) -> String {
        let deflated = miniz_oxide::deflate::compress_to_vec(payload, 10);
        let mut frame = vec![version, 0, 0, 0, 0, 0];
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
        if letters.starts_with(parts::START) {
            return Err(CodeError::OnePart);
        }
        let inner = letters
            .strip_prefix(START)
            .and_then(|t| t.strip_suffix(END))
            .ok_or(CodeError::NotACode)?;
        let frame = base45::decode(inner)?;
        if frame.len() < 7 {
            return Err(CodeError::NotACode);
        }
        let version = frame[0];
        if !(FIRST_VERSION..=VERSION).contains(&version) {
            return Err(CodeError::Version(version));
        }
        if frame[2..6] != crc32(&[&frame[..2], &frame[6..]].concat()).to_le_bytes() {
            return Err(CodeError::Checksum);
        }
        if frame[1] != 0 {
            return Err(CodeError::Damaged("a flag this program does not know"));
        }
        let payload = miniz_oxide::inflate::decompress_to_vec_with_limit(&frame[6..], MOST_PAYLOAD)
            .map_err(|_| CodeError::Damaged("the code does not unpack"))?;
        Self::from_payload(version, &payload)
    }

    fn from_payload(version: u8, payload: &[u8]) -> Result<Self, CodeError> {
        let mut r = Reader::new(payload);
        let made = r.u32()?;
        let program = String::from_utf8(r.bytes(MOST_PROGRAM)?.to_vec())
            .map_err(|_| CodeError::Range("a program name that is not text"))?;
        let flags = r.u8()?;
        let old = version == 1;
        // The first version's flag 1 was the settings, which are read past and ignored.
        if flags > 0xF || (!old && flags & 1 != 0) {
            return Err(CodeError::Damaged("a section this program does not know"));
        }
        let mut state = Self {
            made,
            program,
            ..Self::default()
        };
        if old {
            if flags & 1 != 0 {
                r.bytes(MOST_OLD_SETTINGS)?;
            }
            if flags & 2 != 0 {
                state.best = Some(Self::read_best(&mut r)?);
            }
            if flags & 4 != 0 {
                state.setups = Some(Self::read_setups(&mut r)?);
            }
            if flags & 8 != 0 {
                state.games = Some(Self::read_games(&mut r, false)?);
            }
        } else {
            if flags & 8 != 0 {
                state.games = Some(Self::read_games(&mut r, true)?);
            }
            if flags & 4 != 0 {
                state.setups = Some(Self::read_setups(&mut r)?);
            }
            if flags & 2 != 0 {
                state.best = Some(Self::read_best(&mut r)?);
            }
        }
        r.finish()?;
        Self::check_slots(
            state.setups.as_ref().map(|s| s.iter().map(|x| x.slot)),
            "two set-ups in a slot",
        )?;
        Self::check_slots(
            state.games.as_ref().map(|s| s.iter().map(|x| x.slot)),
            "two saved games in a slot",
        )?;
        Ok(state)
    }

    fn check_slots(
        slots: Option<impl Iterator<Item = u8>>,
        why: &'static str,
    ) -> Result<(), CodeError> {
        let mut slots: Vec<u8> = slots.map(Iterator::collect).unwrap_or_default();
        slots.sort_unstable();
        if slots.windows(2).any(|w| w[0] == w[1]) {
            return Err(CodeError::Range(why));
        }
        Ok(())
    }

    fn read_setups(r: &mut Reader<'_>) -> Result<Vec<Setup>, CodeError> {
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
        Ok(setups)
    }

    /// The saved games; `packed` when their states were rearranged by `game_pack` (the
    /// first version's were not).
    fn read_games(r: &mut Reader<'_>, packed: bool) -> Result<Vec<Game>, CodeError> {
        let n = r.count(SLOTS, "more saved games than slots")?;
        let mut games = Vec::with_capacity(n);
        for _ in 0..n {
            let slot = r.u8()?;
            let name: [u8; NAME_LEN] = r.raw(NAME_LEN)?.try_into().expect("a name");
            check_name(&name)?;
            let bytes = r.raw(GAME_STATE_LEN)?;
            if usize::from(slot) >= SLOTS {
                return Err(CodeError::Range("a saved game of a slot there is not"));
            }
            let state = if packed {
                game_pack::unpack(bytes)
            } else {
                bytes.to_vec()
            };
            games.push(Game { slot, name, state });
        }
        Ok(games)
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

    /// What the state holds, for the question before it is taken in.
    pub fn summary(&self) -> Summary {
        Summary {
            made: self.made,
            program: self.program.clone(),
            best_times: self.best.as_ref().map(BestTimes::count),
            setups: self.setups.as_ref().map(Vec::len),
            games: self
                .games
                .as_ref()
                .map(|g| g.iter().map(|g| (g.slot, name_text(&g.name))).collect()),
        }
    }
}

/// A name's bytes (Windows-1252, to the first NUL) as text.
pub fn name_text(name: &[u8]) -> String {
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    crate::files::ansi::ansi_text(&name[..end])
}

/// What a state holds, to be shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub made: u32,
    pub program: String,
    pub best_times: Option<usize>,
    pub setups: Option<usize>,
    /// The saved games by slot and name.
    pub games: Option<Vec<(u8, String)>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saves::entries::{BestTime, TIME_LEN};
    use crate::saves::savegame::{RECORD_LEN, RECORDS, RECORDS_AT};

    fn name(text: &[u8]) -> [u8; NAME_LEN] {
        let mut n = [0; NAME_LEN];
        n[..text.len()].copy_from_slice(text);
        n
    }

    /// A state that runs like a saved game: counting first dwords in the records.
    fn game_state(seed: usize) -> Vec<u8> {
        let mut s: Vec<u8> = (0..GAME_STATE_LEN)
            .map(|n| ((n + seed) * 31 % 7) as u8)
            .collect();
        for row in 0..RECORDS {
            let at = RECORDS_AT + row * RECORD_LEN;
            s[at..at + 4].copy_from_slice(&(row as u32).to_le_bytes());
        }
        s
    }

    pub fn sample() -> PlayerState {
        let mut real = BestTime::default();
        real.head.player = 1;
        real.head.name[..3].copy_from_slice(b"Ana");
        real.head.key = 123_456;
        let mut entry = vec![0; TIME_LEN];
        real.write(&mut entry);
        PlayerState {
            made: 1_800_000_000,
            program: "0.1.0".into(),
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
                state: game_state(0),
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
        PlayerState::frame(VERSION, payload)
    }

    #[test]
    fn values_out_of_their_range_are_refused() {
        // A set-up value.
        let mut s = sample();
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
        // A best time with a player 9.
        let mut s = sample();
        s.best.as_mut().unwrap().real[0].entry[4] = 9;
        assert!(PlayerState::decode(&framed(&s.payload())).is_err());
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
        let again = |frame: &[u8]| format!("{START}{}{END}", base45::encode(frame));
        for version in [0, 3] {
            frame[0] = version;
            assert_eq!(
                PlayerState::decode(&again(&frame)),
                Err(CodeError::Version(version))
            );
        }
        frame[0] = VERSION;
        frame[10] ^= 1;
        assert_eq!(
            PlayerState::decode(&again(&frame)),
            Err(CodeError::Checksum)
        );
        // This version has no settings section: its flag is refused.
        let mut p = sample().payload();
        let flags_at = 4 + 1 + "0.1.0".len();
        p[flags_at] |= 1;
        assert!(PlayerState::decode(&framed(&p)).is_err());
    }

    /// A first-version payload: the settings first, then the best times, the set-ups and the
    /// saved games as they were laid out (not rearranged).
    fn old_payload(s: &PlayerState, settings: &str) -> Vec<u8> {
        let mut w = Writer::default();
        w.u32(s.made);
        w.bytes(s.program.as_bytes());
        w.u8(0xF);
        w.bytes(settings.as_bytes());
        // The best times are written as they are now: the new payload with the games and
        // set-ups left out holds them after its head.
        let mut only_best = s.clone();
        only_best.games = None;
        only_best.setups = None;
        let tail = &only_best.payload()[4 + 1 + s.program.len() + 1..];
        w.raw(tail);
        w.varint(1);
        let setup = &s.setups.as_ref().unwrap()[0];
        w.u8(setup.slot);
        w.raw(&setup.name);
        for v in setup.values {
            w.varint(v);
        }
        w.varint(1);
        let game = &s.games.as_ref().unwrap()[0];
        w.u8(game.slot);
        w.raw(&game.name);
        w.raw(&game.state);
        w.0
    }

    #[test]
    fn a_first_version_code_reads_and_its_settings_are_ignored() {
        let s = sample();
        let text = PlayerState::frame(1, &old_payload(&s, "[sound]\nmaster_volume = 70\n"));
        assert_eq!(PlayerState::decode(&text).unwrap(), s);
        // Settings that are nothing like settings are read past as well.
        let text = PlayerState::frame(1, &old_payload(&s, "not toml ["));
        assert_eq!(PlayerState::decode(&text).unwrap(), s);
    }

    #[test]
    fn a_deflate_bomb_is_not_unpacked() {
        let bomb = vec![0u8; 8 * 1024 * 1024];
        assert!(PlayerState::decode(&framed(&bomb)).is_err());
    }

    #[test]
    fn the_summary_names_what_is_there() {
        let sum = sample().summary();
        assert_eq!(sum.best_times, Some(1));
        assert_eq!(sum.setups, Some(1));
        assert_eq!(sum.games, Some(vec![(4, "Before Wales".to_string())]));
    }

    #[test]
    fn the_saved_games_come_first_then_the_set_ups_then_the_best_times() {
        let s = sample();
        let head = 4 + 1 + s.program.len() + 1;
        let payload = s.payload();
        // The games: a count of 1, the slot, the name, the state.
        assert_eq!(payload[head..head + 2], [1, 4]);
        let setups_at = head + 1 + 1 + NAME_LEN + GAME_STATE_LEN;
        assert_eq!(payload[setups_at..setups_at + 2], [1, 1]);
        let mut only_best = s.clone();
        only_best.games = None;
        only_best.setups = None;
        assert_eq!(
            payload[payload.len() - (only_best.payload().len() - head)..],
            only_best.payload()[head..]
        );
    }

    #[test]
    fn many_saved_games_become_parts_that_read_back() {
        let mut s = sample();
        let mut x = 0x1234_5678u32;
        let mut noise = || {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (x >> 24) as u8
        };
        s.games = Some(
            (0..3u8)
                .map(|slot| Game {
                    slot,
                    name: name(b"big"),
                    state: (0..GAME_STATE_LEN).map(|_| noise()).collect(),
                })
                .collect(),
        );
        let whole = s.encode();
        let texts = parts::split(&whole, parts::Density::Small).unwrap();
        assert!(texts.len() >= 3);
        let mut assembly = parts::Assembly::default();
        for t in texts.iter().rev() {
            assembly.take(parts::Part::read(t).unwrap());
        }
        let joined = assembly.whole().unwrap().unwrap();
        assert_eq!(PlayerState::decode(&joined).unwrap(), s);
        assert_eq!(PlayerState::decode(&texts[0]), Err(CodeError::OnePart));
        for t in &texts {
            let modules = qr::encode(t).unwrap();
            let (width, pixels) = qr::render(&modules, 3);
            assert_eq!(qr::read(width, width, &pixels).as_deref(), Some(t.as_str()));
        }
    }

    /// The saved games in a savedata folder (`RC2K_SAVEDATA`, the one that holds `savegame\\`),
    /// carried in both densities: each part a QR code read back out of a picture of the size
    /// the page draws it, the parts joined and decoded. Skipped when the folder is not named.
    #[test]
    fn real_saved_games_round_trip_at_both_densities() {
        use parts::Density;
        let Some(dir) = std::env::var_os("RC2K_SAVEDATA") else {
            return;
        };
        let mut games = Vec::new();
        for slot in 0..SLOTS as u8 {
            let path = std::path::Path::new(&dir).join(format!("savegame/svgame{slot:02}"));
            let Ok(file) = std::fs::read(path) else {
                continue;
            };
            let Some(name) = crate::saves::kept::slot_name(
                &file,
                crate::saves::kept::SAVEGAME,
                crate::saves::kept::SAVEGAME_VERSION,
            ) else {
                continue;
            };
            games.push(Game {
                slot,
                name,
                state: file[0x30..0x30 + GAME_STATE_LEN].to_vec(),
            });
        }
        let mut s = sample();
        s.games = Some(games);
        let whole = s.encode();
        for density in [Density::Dense, Density::Small] {
            let texts = parts::split(&whole, density).unwrap();
            println!(
                "{density:?}: {} letters in {} codes, largest side {}",
                whole.len(),
                texts.len(),
                texts
                    .iter()
                    .map(|t| qr::encode(t).unwrap().side)
                    .max()
                    .unwrap()
            );
            let mut assembly = parts::Assembly::default();
            for t in &texts {
                let modules = qr::encode(t).unwrap();
                let (width, pixels) = qr::render(&modules, 4);
                let read = qr::read(width, width, &pixels).unwrap();
                assert_eq!(&read, t);
                if texts.len() > 1 {
                    assembly.take(parts::Part::read(&read).unwrap());
                }
            }
            let joined = if texts.len() > 1 {
                assembly.whole().unwrap().unwrap()
            } else {
                texts[0].clone()
            };
            assert_eq!(PlayerState::decode(&joined).unwrap(), s);
        }
    }

    #[test]
    fn the_qr_round_trip_through_a_picture() {
        let whole = sample().encode();
        let modules = qr::encode(&whole).unwrap();
        let (width, pixels) = qr::render(&modules, 4);
        let read = qr::read(width, width, &pixels).unwrap();
        assert_eq!(PlayerState::decode(&read).unwrap(), sample());
    }
}
