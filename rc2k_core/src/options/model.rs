//! The options by area. Every value of the old file (`var\ral.dat`, 69 pieces, 0xB656 bytes
//! after its head) has a field here; `Options::visit` lists them in the file's order, the one
//! place that order is written.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The pieces' lengths together.
pub const PIECES_LEN: usize = 0xB656;
/// How many pieces the old file has.
pub const PIECE_COUNT: usize = 69;

/// A value of the old file, one piece.
pub trait Field {
    /// The piece's length in the old file.
    fn byte_len(&self) -> usize;
    /// Appends the piece's bytes.
    fn put(&self, out: &mut Vec<u8>);
    /// Takes the piece's bytes (`byte_len()` of them).
    fn take(&mut self, bytes: &[u8]);
}

impl Field for u32 {
    fn byte_len(&self) -> usize {
        4
    }
    fn put(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_le_bytes());
    }
    fn take(&mut self, bytes: &[u8]) {
        *self = u32::from_le_bytes(bytes.try_into().expect("a piece of its length"));
    }
}

/// A run of bytes (a name, a record, a table), as hex text in the new file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bytes<const N: usize>(pub Vec<u8>);

impl<const N: usize> Default for Bytes<N> {
    fn default() -> Self {
        Self(vec![0; N])
    }
}

impl<const N: usize> Field for Bytes<N> {
    fn byte_len(&self) -> usize {
        N
    }
    fn put(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.0);
    }
    fn take(&mut self, bytes: &[u8]) {
        self.0.copy_from_slice(bytes);
    }
}

impl<const N: usize> Serialize for Bytes<N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut text = String::with_capacity(2 * N);
        for b in &self.0 {
            text.push_str(&format!("{b:02x}"));
        }
        s.serialize_str(&text)
    }
}

impl<'de, const N: usize> Deserialize<'de> for Bytes<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        if text.len() != 2 * N || !text.is_ascii() {
            return Err(D::Error::custom(format!(
                "expected {N} bytes as {} hex digits",
                2 * N
            )));
        }
        let mut bytes = vec![0u8; N];
        for (n, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&text[2 * n..2 * n + 2], 16)
                .map_err(|_| D::Error::custom("a hex digit expected"))?;
        }
        Ok(Self(bytes))
    }
}

/// A list of dwords (a page's switches, the car numbers), as a list of numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Words<const N: usize>(pub Vec<u32>);

impl<const N: usize> Default for Words<N> {
    fn default() -> Self {
        Self(vec![0; N])
    }
}

impl<const N: usize> Field for Words<N> {
    fn byte_len(&self) -> usize {
        4 * N
    }
    fn put(&self, out: &mut Vec<u8>) {
        for w in &self.0 {
            out.extend_from_slice(&w.to_le_bytes());
        }
    }
    fn take(&mut self, bytes: &[u8]) {
        for (w, b) in self.0.iter_mut().zip(bytes.as_chunks::<4>().0) {
            *w = u32::from_le_bytes(*b);
        }
    }
}

impl<const N: usize> Serialize for Words<N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de, const N: usize> Deserialize<'de> for Words<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let list = Vec::<u32>::deserialize(d)?;
        if list.len() != N {
            return Err(D::Error::invalid_length(list.len(), &"the list's length"));
        }
        Ok(Self(list))
    }
}

/// The screen modes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Display {
    pub menu_mode: u32,
    pub race_mode: u32,
    /// Mipmapping, off or on.
    pub mipmapping: u32,
}

/// The sound settings.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sound {
    pub menu_sound_mode: u32,
    pub race_sound_mode: u32,
    pub master_volume: u32,
    /// Kept in the old file and read by no other code.
    pub unread_option_f0: u32,
    pub race_music_on: u32,
    /// Which tune comes next.
    pub tune_turn: u32,
    pub music_volume: u32,
    pub race_music_volume: u32,
    pub master_volume_shown: u32,
    pub music_level: u32,
    /// Read by the music "up" step.
    pub other_volume: u32,
    pub menu_sounds_c: u32,
    pub menu_sounds_a: u32,
    pub speech_level: u32,
    pub sounds_a: u32,
    pub sounds_b: u32,
}

/// The network settings.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    pub machine_name: Bytes<0x40>,
    /// The most players a hosted session takes.
    pub most_players: u32,
    pub host_address: Bytes<0x40>,
    pub port: u32,
    /// The saved connection's GUID.
    pub saved_connection: Bytes<0x10>,
}

/// The options pages' own settings and what the front end keeps.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pages {
    /// The pages' work copies, the first switch up to the headlights.
    pub switches: Words<11>,
    /// The audio quality, the 3D switches and the channels.
    pub audio: Words<6>,
    /// The services counted.
    pub services: u32,
    /// Kept by the event races.
    pub event_kept: u32,
    /// The race settings' first word.
    pub race_settings: u32,
    /// The arcade levels won, a byte each.
    pub arcade_levels_won: Bytes<6>,
}

/// The car number lists.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cars {
    pub first_numbers: Words<40>,
    /// 20 entries.
    pub second_numbers: Words<20>,
}

/// The four players: their records and names.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Players {
    pub records: [Bytes<0x28C>; 4],
    pub names: [Bytes<0x20>; 4],
}

/// The best times and records kept in the options file.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Records {
    pub tyre_change_holder: Bytes<0x20>,
    pub tyre_change_time: u32,
    pub championship: Bytes<0x2A48>,
    /// The twelve events' tables, by event number.
    pub events: [Bytes<0x47C>; 12],
    /// The six rallies' best times.
    pub rallies: [Bytes<0xC30>; 6],
}

/// Words the old file keeps and no other code reads.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kept {
    pub runs: u32,
    pub saved_games_a: u32,
    pub saved_games_b: u32,
    pub set_ups: u32,
    pub options: [u32; 4],
}

/// Everything the old file holds.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
    pub display: Display,
    pub sound: Sound,
    pub network: Network,
    pub pages: Pages,
    pub cars: Cars,
    pub players: Players,
    pub records: Records,
    pub kept: Kept,
}

impl Options {
    /// Calls `f` with every value, in the old file's order.
    #[rustfmt::skip]
    pub fn visit(&mut self, f: &mut dyn FnMut(&mut dyn Field)) {
        f(&mut self.kept.runs);
        f(&mut self.display.menu_mode);
        f(&mut self.display.race_mode);
        f(&mut self.display.mipmapping);
        f(&mut self.sound.menu_sound_mode);
        f(&mut self.sound.race_sound_mode);
        f(&mut self.sound.master_volume);
        f(&mut self.sound.unread_option_f0);
        f(&mut self.sound.race_music_on);
        f(&mut self.sound.tune_turn);
        f(&mut self.sound.music_volume);
        f(&mut self.sound.race_music_volume);
        f(&mut self.network.machine_name);
        f(&mut self.network.most_players);
        f(&mut self.network.host_address);
        f(&mut self.network.port);
        f(&mut self.network.saved_connection);
        f(&mut self.kept.saved_games_a);
        f(&mut self.kept.saved_games_b);
        f(&mut self.kept.set_ups);
        f(&mut self.records.tyre_change_holder);
        f(&mut self.records.tyre_change_time);
        for item in self.kept.options.iter_mut().take(4) {
            f(item);
        }
        for item in self.players.records.iter_mut().take(4) {
            f(item);
        }
        f(&mut self.sound.master_volume_shown);
        f(&mut self.sound.music_level);
        f(&mut self.sound.other_volume);
        f(&mut self.sound.menu_sounds_c);
        f(&mut self.sound.menu_sounds_a);
        f(&mut self.sound.speech_level);
        f(&mut self.sound.sounds_a);
        f(&mut self.sound.sounds_b);
        f(&mut self.pages.switches);
        f(&mut self.pages.audio);
        f(&mut self.pages.services);
        f(&mut self.pages.event_kept);
        f(&mut self.pages.race_settings);
        f(&mut self.pages.arcade_levels_won);
        f(&mut self.cars.first_numbers);
        f(&mut self.cars.second_numbers);
        f(&mut self.records.championship);
        // The events' tables are in the file in this order.
        for n in [5, 4, 3, 2, 1, 0, 6, 7, 8, 9, 10, 11] {
            f(&mut self.records.events[n]);
        }
        for item in self.records.rallies.iter_mut().take(6) {
            f(item);
        }
        for item in self.players.names.iter_mut().take(4) {
            f(item);
        }
    }

    /// The pieces' bytes, one `Vec` for each, in the old file's order.
    pub fn to_pieces(&self) -> Vec<Vec<u8>> {
        let mut pieces = Vec::with_capacity(PIECE_COUNT);
        self.clone().visit(&mut |field| {
            let mut bytes = Vec::with_capacity(field.byte_len());
            field.put(&mut bytes);
            pieces.push(bytes);
        });
        pieces
    }

    /// The lengths of the pieces, in the old file's order.
    pub fn piece_lengths() -> Vec<usize> {
        let mut lengths = Vec::with_capacity(PIECE_COUNT);
        Self::default().visit(&mut |field| lengths.push(field.byte_len()));
        lengths
    }

    /// Takes the pieces (as `to_pieces` gives them) into the options; the number and lengths
    /// must be right.
    pub fn take_pieces(&mut self, pieces: &[&[u8]]) -> bool {
        let mut at = 0;
        let mut right = true;
        self.visit(&mut |field| match pieces.get(at) {
            Some(bytes) if bytes.len() == field.byte_len() => {
                field.take(bytes);
                at += 1;
            }
            _ => right = false,
        });
        right && at == pieces.len()
    }
}
