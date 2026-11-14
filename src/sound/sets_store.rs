//! The sound module's sets and tables (the settings page's menu sounds, the race's sets of sounds
//! and the commands that name them, the cues' lists, the car sound kinds' records, the surface
//! sound sets and the spot voices), typed. The sets are lists of sound numbers ending in -1 that
//! the audio thread reads; the tables of pointers name them by handle: the race's sets by
//! `RACE_LIST_TAG` and an offset in the table they lie in; the cues' and surface sets' lists by
//! their byte offset in the block of lists after the table, the kinds' records by their number.
//!
//! Structs of atomic cells with the game's starting values (`tests`); the race's sets keep
//! the 1999 layout, as their lists' handles are offsets in it.

use crate::session::{Bytes, Cell32, Dword, PlainBytes, PlainDword, PlainFloat, with_runs};
use rc2k_core::audio::commands::RaceField;

/// Handle of the race's sets: in place of a list's address the table holds
/// `TAG | field << 16 | entry` (the field's number in `RaceField::ALL`, the entry in it);
/// 0 does not occur. The list runs on through the fields after its own as far as the
/// sounds read (`RaceSoundSets::list_entry`).
pub const RACE_LIST_TAG: u32 = 0xC400_0000;

/// The handle of the list that starts at entry `entry` of `field`.
const fn race_list(field: RaceField, entry: u32) -> u32 {
    assert!((entry as usize) < field.dwords());
    RACE_LIST_TAG | (field as u32) << 16 | entry
}

/// The field and entry a handle of the race's sets names (None: no handle of the sets).
fn race_list_of(handle: u32) -> Option<(RaceField, u32)> {
    let rest = handle.checked_sub(RACE_LIST_TAG)?;
    let field = *RaceField::ALL.get((rest >> 16) as usize)?;
    let entry = rest & 0xFFFF;
    ((entry as usize) < field.dwords()).then_some((field, entry))
}

/// The settings page's menu sounds list.
#[derive(rc2k_derive::Cells)]
pub struct MenuSounds {
    /// Padding that keeps every field's 1999 alignment (the struct is 16-byte aligned).
    pub _lead: Bytes<4>,
    /// The list `sound::apply_settings` passes as the menu sounds' third argument.
    pub list: Bytes<40>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: Bytes<4>,
}

impl MenuSounds {
    pub const fn new() -> Self {
        MenuSounds {
    _lead: Bytes::new([0; 4]),
    list: Bytes::new(with_runs::<40>(&[(0, b"\xc7\x01\x00\x00\xc8\x01\x00\x00\xc9\x01\x00\x00\xca\x01\x00\x00c\x01\x00\x00\x97\x01\x00\x00\xcb\x01\x00\x00\xe4\x00\x00\x00\xcc\x01\x00\x00\xff\xff\xff\xff")])),
    _tail: Bytes::new([0; 4]),
        }
    }
}

impl MenuSounds {
    /// Entry `n` of the list, none past its ten dwords (it ends in -1 at the tenth).
    pub fn sound(&self, n: usize) -> Option<u32> {
        (n < self.list.atomics().len() / 4).then(|| self.list.get_u32(4 * n))
    }
}

/// The sets of sounds a race loads (`sound::load_race_sounds`) and the request it sends. The fields
/// are in the 1999 order: a list runs on into the fields after it (`list_entry`, `RaceField`).
#[derive(rc2k_derive::Cells)]
pub struct RaceSoundSets {
    /// The first car's sets by kind of car, three dwords each (two sounds, -1).
    pub kind_sets_first: [Dword; 102],
    /// A set of the first car (sound numbers ending in -1).
    pub first_set_a: [Dword; 23],
    /// Another of the first car.
    pub first_set_c: [Dword; 94],
    /// The second car's sets by kind of car, three dwords each.
    pub kind_sets_second: [Dword; 102],
    /// A set of the second car.
    pub second_set_a: [Dword; 23],
    /// Another of the second car.
    pub second_set_b: [Dword; 95],
    /// The set loaded when the two cars differ.
    pub last_set: [Dword; 31],
    /// Another of the first car.
    pub first_set_b: [Dword; 54],
    /// The only set of a replay.
    pub replay_set: [Dword; 8],
    /// Two sets (pointers) a kind of car.
    pub kinds: [Dword; 87],
    /// Padding that fills the record to its 16-byte alignment (the sound request follows it in
    /// the 1999 layout; it is the game thread's now, `SoundRequest`).
    pub _tail: Bytes<4>,
}

/// The ten dwords the race's sounds are sent with (`sound::play_placed_sound`,
/// `sound::stop_sound`); a work area each turn fills again. The game thread's
/// (`sound::owner::RaceSoundState::request`). Laid out as the 1999 variables: the recorded
/// assembly's tests lay it over raw bytes.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct SoundRequest {
    /// The sound (bit 0x8000: none).
    pub sound: PlainDword,
    /// The pitch, a float (`sound::revs_sound::fill_pitch_and_volume`), then the whole number
    /// sent.
    pub pitch: PlainDword,
    /// The volume, a float, then (without 3D) the whole number sent.
    pub volume: PlainDword,
    /// Where the sound is, three floats.
    pub place: [PlainFloat; 3],
    /// Without 3D, the angle (the arctangent of the place's third float over its first).
    pub angle: PlainDword,
    /// Without 3D, the place between left and right (a whole number).
    pub pan: PlainDword,
    /// The pitch's factor, a float.
    pub pitch_factor: PlainFloat,
    /// The volume's factor, a float.
    pub volume_factor: PlainFloat,
}

impl SoundRequest {
    /// The request as the game starts: all 0.
    pub const fn new() -> Self {
        Self {
            sound: PlainDword::new(0),
            pitch: PlainDword::new(0),
            volume: PlainDword::new(0),
            place: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            angle: PlainDword::new(0),
            pan: PlainDword::new(0),
            pitch_factor: PlainFloat::new(0.0),
            volume_factor: PlainFloat::new(0.0),
        }
    }

    /// Stores `bits` in dword `n` (+4n bytes in the 1999 layout), for the cores that name the words
    /// by number; None past the ten.
    pub fn set_word(&self, n: usize, bits: u32) -> Option<()> {
        let cell: &dyn Cell32 = match n {
            0 => &self.sound,
            1 => &self.pitch,
            2 => &self.volume,
            3..=5 => &self.place[n - 3],
            6 => &self.angle,
            7 => &self.pan,
            8 => &self.pitch_factor,
            9 => &self.volume_factor,
            _ => return None,
        };
        cell.set_bits(bits);
        Some(())
    }
}

impl RaceSoundSets {
    pub const fn new() -> Self {
        RaceSoundSets {
            kind_sets_first: [
                Dword::new(0x0),
                Dword::new(0x2),
                Dword::new(0xffffffff),
                Dword::new(0x4),
                Dword::new(0x6),
                Dword::new(0xffffffff),
                Dword::new(0x8),
                Dword::new(0xa),
                Dword::new(0xffffffff),
                Dword::new(0xc),
                Dword::new(0xe),
                Dword::new(0xffffffff),
                Dword::new(0x10),
                Dword::new(0x12),
                Dword::new(0xffffffff),
                Dword::new(0x14),
                Dword::new(0x16),
                Dword::new(0xffffffff),
                Dword::new(0x18),
                Dword::new(0x1a),
                Dword::new(0xffffffff),
                Dword::new(0x1c),
                Dword::new(0x1e),
                Dword::new(0xffffffff),
                Dword::new(0x20),
                Dword::new(0x22),
                Dword::new(0xffffffff),
                Dword::new(0x24),
                Dword::new(0x26),
                Dword::new(0xffffffff),
                Dword::new(0x28),
                Dword::new(0x2a),
                Dword::new(0xffffffff),
                Dword::new(0x2c),
                Dword::new(0x2e),
                Dword::new(0xffffffff),
                Dword::new(0x30),
                Dword::new(0x32),
                Dword::new(0xffffffff),
                Dword::new(0x34),
                Dword::new(0x36),
                Dword::new(0xffffffff),
                Dword::new(0x38),
                Dword::new(0x3a),
                Dword::new(0xffffffff),
                Dword::new(0x3c),
                Dword::new(0x3e),
                Dword::new(0xffffffff),
                Dword::new(0x40),
                Dword::new(0x42),
                Dword::new(0xffffffff),
                Dword::new(0x44),
                Dword::new(0x46),
                Dword::new(0xffffffff),
                Dword::new(0x48),
                Dword::new(0x4a),
                Dword::new(0xffffffff),
                Dword::new(0x4c),
                Dword::new(0x4e),
                Dword::new(0xffffffff),
                Dword::new(0x50),
                Dword::new(0x52),
                Dword::new(0xffffffff),
                Dword::new(0x54),
                Dword::new(0x56),
                Dword::new(0xffffffff),
                Dword::new(0x58),
                Dword::new(0x5a),
                Dword::new(0xffffffff),
                Dword::new(0x5c),
                Dword::new(0x5e),
                Dword::new(0xffffffff),
                Dword::new(0x60),
                Dword::new(0x62),
                Dword::new(0xffffffff),
                Dword::new(0x64),
                Dword::new(0x66),
                Dword::new(0xffffffff),
                Dword::new(0x68),
                Dword::new(0x6a),
                Dword::new(0xffffffff),
                Dword::new(0x6c),
                Dword::new(0x6e),
                Dword::new(0xffffffff),
                Dword::new(0x70),
                Dword::new(0x72),
                Dword::new(0xffffffff),
                Dword::new(0x74),
                Dword::new(0x76),
                Dword::new(0xffffffff),
                Dword::new(0x78),
                Dword::new(0x7a),
                Dword::new(0xffffffff),
                Dword::new(0x7c),
                Dword::new(0x7e),
                Dword::new(0xffffffff),
                Dword::new(0x80),
                Dword::new(0x82),
                Dword::new(0xffffffff),
                Dword::new(0x84),
                Dword::new(0x86),
                Dword::new(0xffffffff),
            ],
            first_set_a: [
                Dword::new(0x88),
                Dword::new(0x8c),
                Dword::new(0x8e),
                Dword::new(0x92),
                Dword::new(0x96),
                Dword::new(0x98),
                Dword::new(0x9c),
                Dword::new(0xa0),
                Dword::new(0xa2),
                Dword::new(0xa6),
                Dword::new(0xaa),
                Dword::new(0xac),
                Dword::new(0xb0),
                Dword::new(0xb4),
                Dword::new(0xb6),
                Dword::new(0xba),
                Dword::new(0xbe),
                Dword::new(0xc0),
                Dword::new(0xc4),
                Dword::new(0xc8),
                Dword::new(0xca),
                Dword::new(0xce),
                Dword::new(0xffffffff),
            ],
            first_set_c: [
                Dword::new(0xf5),
                Dword::new(0xf3),
                Dword::new(0xf1),
                Dword::new(0xef),
                Dword::new(0xed),
                Dword::new(0xf7),
                Dword::new(0xf9),
                Dword::new(0xfb),
                Dword::new(0xfd),
                Dword::new(0xff),
                Dword::new(0x101),
                Dword::new(0x103),
                Dword::new(0x105),
                Dword::new(0x107),
                Dword::new(0x109),
                Dword::new(0x10b),
                Dword::new(0x111),
                Dword::new(0x10f),
                Dword::new(0x113),
                Dword::new(0x10d),
                Dword::new(0x115),
                Dword::new(0x117),
                Dword::new(0x119),
                Dword::new(0x11b),
                Dword::new(0x11d),
                Dword::new(0x11f),
                Dword::new(0x121),
                Dword::new(0x123),
                Dword::new(0x125),
                Dword::new(0x127),
                Dword::new(0x129),
                Dword::new(0x12b),
                Dword::new(0x12d),
                Dword::new(0x12f),
                Dword::new(0x131),
                Dword::new(0x133),
                Dword::new(0x135),
                Dword::new(0x137),
                Dword::new(0x139),
                Dword::new(0x13b),
                Dword::new(0x13d),
                Dword::new(0x141),
                Dword::new(0x143),
                Dword::new(0x17d),
                Dword::new(0x17b),
                Dword::new(0x169),
                Dword::new(0x179),
                Dword::new(0x16b),
                Dword::new(0x16f),
                Dword::new(0x16d),
                Dword::new(0x177),
                Dword::new(0x17f),
                Dword::new(0x171),
                Dword::new(0x173),
                Dword::new(0x175),
                Dword::new(0x181),
                Dword::new(0x187),
                Dword::new(0x183),
                Dword::new(0x189),
                Dword::new(0x18b),
                Dword::new(0x185),
                Dword::new(0x195),
                Dword::new(0x18f),
                Dword::new(0x193),
                Dword::new(0x191),
                Dword::new(0x19b),
                Dword::new(0x19d),
                Dword::new(0x1a1),
                Dword::new(0x1a5),
                Dword::new(0x197),
                Dword::new(0x1a3),
                Dword::new(0x1a7),
                Dword::new(0x1a9),
                Dword::new(0x199),
                Dword::new(0x19f),
                Dword::new(0x145),
                Dword::new(0x147),
                Dword::new(0x149),
                Dword::new(0x14b),
                Dword::new(0x14d),
                Dword::new(0x14f),
                Dword::new(0x151),
                Dword::new(0x153),
                Dword::new(0x155),
                Dword::new(0x157),
                Dword::new(0x159),
                Dword::new(0x15b),
                Dword::new(0x15d),
                Dword::new(0x15f),
                Dword::new(0x161),
                Dword::new(0x163),
                Dword::new(0x13f),
                Dword::new(0x165),
                Dword::new(0x167),
            ],
            kind_sets_second: [
                Dword::new(0x1),
                Dword::new(0x3),
                Dword::new(0xffffffff),
                Dword::new(0x5),
                Dword::new(0x7),
                Dword::new(0xffffffff),
                Dword::new(0x9),
                Dword::new(0xb),
                Dword::new(0xffffffff),
                Dword::new(0xd),
                Dword::new(0xf),
                Dword::new(0xffffffff),
                Dword::new(0x11),
                Dword::new(0x13),
                Dword::new(0xffffffff),
                Dword::new(0x15),
                Dword::new(0x17),
                Dword::new(0xffffffff),
                Dword::new(0x19),
                Dword::new(0x1b),
                Dword::new(0xffffffff),
                Dword::new(0x1d),
                Dword::new(0x1f),
                Dword::new(0xffffffff),
                Dword::new(0x21),
                Dword::new(0x23),
                Dword::new(0xffffffff),
                Dword::new(0x25),
                Dword::new(0x27),
                Dword::new(0xffffffff),
                Dword::new(0x29),
                Dword::new(0x2b),
                Dword::new(0xffffffff),
                Dword::new(0x2d),
                Dword::new(0x2f),
                Dword::new(0xffffffff),
                Dword::new(0x31),
                Dword::new(0x33),
                Dword::new(0xffffffff),
                Dword::new(0x35),
                Dword::new(0x37),
                Dword::new(0xffffffff),
                Dword::new(0x39),
                Dword::new(0x3b),
                Dword::new(0xffffffff),
                Dword::new(0x3d),
                Dword::new(0x3f),
                Dword::new(0xffffffff),
                Dword::new(0x41),
                Dword::new(0x43),
                Dword::new(0xffffffff),
                Dword::new(0x45),
                Dword::new(0x47),
                Dword::new(0xffffffff),
                Dword::new(0x49),
                Dword::new(0x4b),
                Dword::new(0xffffffff),
                Dword::new(0x4d),
                Dword::new(0x4f),
                Dword::new(0xffffffff),
                Dword::new(0x51),
                Dword::new(0x53),
                Dword::new(0xffffffff),
                Dword::new(0x55),
                Dword::new(0x57),
                Dword::new(0xffffffff),
                Dword::new(0x59),
                Dword::new(0x5b),
                Dword::new(0xffffffff),
                Dword::new(0x5d),
                Dword::new(0x5f),
                Dword::new(0xffffffff),
                Dword::new(0x61),
                Dword::new(0x63),
                Dword::new(0xffffffff),
                Dword::new(0x65),
                Dword::new(0x67),
                Dword::new(0xffffffff),
                Dword::new(0x69),
                Dword::new(0x6b),
                Dword::new(0xffffffff),
                Dword::new(0x6d),
                Dword::new(0x6f),
                Dword::new(0xffffffff),
                Dword::new(0x71),
                Dword::new(0x73),
                Dword::new(0xffffffff),
                Dword::new(0x75),
                Dword::new(0x77),
                Dword::new(0xffffffff),
                Dword::new(0x79),
                Dword::new(0x7b),
                Dword::new(0xffffffff),
                Dword::new(0x7d),
                Dword::new(0x7f),
                Dword::new(0xffffffff),
                Dword::new(0x81),
                Dword::new(0x83),
                Dword::new(0xffffffff),
                Dword::new(0x85),
                Dword::new(0x87),
                Dword::new(0xffffffff),
            ],
            second_set_a: [
                Dword::new(0x89),
                Dword::new(0x8d),
                Dword::new(0x8f),
                Dword::new(0x93),
                Dword::new(0x97),
                Dword::new(0x99),
                Dword::new(0x9d),
                Dword::new(0xa1),
                Dword::new(0xa3),
                Dword::new(0xa7),
                Dword::new(0xab),
                Dword::new(0xad),
                Dword::new(0xb1),
                Dword::new(0xb5),
                Dword::new(0xb7),
                Dword::new(0xbb),
                Dword::new(0xbf),
                Dword::new(0xc1),
                Dword::new(0xc5),
                Dword::new(0xc9),
                Dword::new(0xcb),
                Dword::new(0xcf),
                Dword::new(0xffffffff),
            ],
            second_set_b: [
                Dword::new(0xf6),
                Dword::new(0xf4),
                Dword::new(0xf2),
                Dword::new(0xf0),
                Dword::new(0xee),
                Dword::new(0xf8),
                Dword::new(0xfa),
                Dword::new(0xfc),
                Dword::new(0xfe),
                Dword::new(0x100),
                Dword::new(0x102),
                Dword::new(0x104),
                Dword::new(0x106),
                Dword::new(0x108),
                Dword::new(0x10a),
                Dword::new(0x10c),
                Dword::new(0x112),
                Dword::new(0x110),
                Dword::new(0x114),
                Dword::new(0x10e),
                Dword::new(0x116),
                Dword::new(0x118),
                Dword::new(0x11a),
                Dword::new(0x11c),
                Dword::new(0x11e),
                Dword::new(0x120),
                Dword::new(0x122),
                Dword::new(0x124),
                Dword::new(0x126),
                Dword::new(0x128),
                Dword::new(0x12a),
                Dword::new(0x12c),
                Dword::new(0x12e),
                Dword::new(0x130),
                Dword::new(0x132),
                Dword::new(0x134),
                Dword::new(0x136),
                Dword::new(0x138),
                Dword::new(0x13a),
                Dword::new(0x13c),
                Dword::new(0x13e),
                Dword::new(0x142),
                Dword::new(0x144),
                Dword::new(0x17e),
                Dword::new(0x17c),
                Dword::new(0x16a),
                Dword::new(0x17a),
                Dword::new(0x16c),
                Dword::new(0x170),
                Dword::new(0x16e),
                Dword::new(0x178),
                Dword::new(0x180),
                Dword::new(0x172),
                Dword::new(0x174),
                Dword::new(0x176),
                Dword::new(0x182),
                Dword::new(0x188),
                Dword::new(0x184),
                Dword::new(0x18a),
                Dword::new(0x18c),
                Dword::new(0x186),
                Dword::new(0x196),
                Dword::new(0x190),
                Dword::new(0x194),
                Dword::new(0x192),
                Dword::new(0x19c),
                Dword::new(0x19e),
                Dword::new(0x1a2),
                Dword::new(0x1a6),
                Dword::new(0x198),
                Dword::new(0x1a4),
                Dword::new(0x1a8),
                Dword::new(0x1aa),
                Dword::new(0x19a),
                Dword::new(0x1a0),
                Dword::new(0x146),
                Dword::new(0x148),
                Dword::new(0x14a),
                Dword::new(0x14c),
                Dword::new(0x14e),
                Dword::new(0x150),
                Dword::new(0x152),
                Dword::new(0x154),
                Dword::new(0x156),
                Dword::new(0x158),
                Dword::new(0x15a),
                Dword::new(0x15c),
                Dword::new(0x15e),
                Dword::new(0x160),
                Dword::new(0x162),
                Dword::new(0x164),
                Dword::new(0x140),
                Dword::new(0x166),
                Dword::new(0x168),
                Dword::new(0xffffffff),
            ],
            last_set: [
                Dword::new(0x8a),
                Dword::new(0x8b),
                Dword::new(0x90),
                Dword::new(0x91),
                Dword::new(0x94),
                Dword::new(0x95),
                Dword::new(0x9a),
                Dword::new(0x9b),
                Dword::new(0x9e),
                Dword::new(0x9f),
                Dword::new(0xa4),
                Dword::new(0xa5),
                Dword::new(0xa8),
                Dword::new(0xa9),
                Dword::new(0xae),
                Dword::new(0xaf),
                Dword::new(0xb2),
                Dword::new(0xb3),
                Dword::new(0xb8),
                Dword::new(0xb9),
                Dword::new(0xbc),
                Dword::new(0xbd),
                Dword::new(0xc2),
                Dword::new(0xc3),
                Dword::new(0xc6),
                Dword::new(0xc7),
                Dword::new(0xcc),
                Dword::new(0xcd),
                Dword::new(0xd0),
                Dword::new(0xd1),
                Dword::new(0xffffffff),
            ],
            first_set_b: [
                Dword::new(0xd2),
                Dword::new(0xd3),
                Dword::new(0xd4),
                Dword::new(0xd5),
                Dword::new(0xd6),
                Dword::new(0xd7),
                Dword::new(0xd8),
                Dword::new(0xd9),
                Dword::new(0xda),
                Dword::new(0xdb),
                Dword::new(0xdc),
                Dword::new(0xdd),
                Dword::new(0xde),
                Dword::new(0xdf),
                Dword::new(0xe0),
                Dword::new(0xe1),
                Dword::new(0xe2),
                Dword::new(0xe5),
                Dword::new(0xe6),
                Dword::new(0xe7),
                Dword::new(0xe4),
                Dword::new(0xe3),
                Dword::new(0xe8),
                Dword::new(0xe9),
                Dword::new(0xea),
                Dword::new(0xeb),
                Dword::new(0xec),
                Dword::new(0x1b0),
                Dword::new(0x1b1),
                Dword::new(0x1b2),
                Dword::new(0x1b3),
                Dword::new(0x1b4),
                Dword::new(0x1b5),
                Dword::new(0x1b6),
                Dword::new(0x1b7),
                Dword::new(0x1b8),
                Dword::new(0x1b9),
                Dword::new(0x1ba),
                Dword::new(0x1bb),
                Dword::new(0x1bc),
                Dword::new(0x1bd),
                Dword::new(0x1be),
                Dword::new(0x1bf),
                Dword::new(0x1c0),
                Dword::new(0x1c1),
                Dword::new(0x1c2),
                Dword::new(0x1c3),
                Dword::new(0x1ab),
                Dword::new(0x1ac),
                Dword::new(0x1ad),
                Dword::new(0x1c4),
                Dword::new(0x1c5),
                Dword::new(0x1c6),
                Dword::new(0xffffffff),
            ],
            replay_set: [
                Dword::new(0x1af),
                Dword::new(0x1ae),
                Dword::new(0x1bb),
                Dword::new(0x1bc),
                Dword::new(0x1b2),
                Dword::new(0x1b7),
                Dword::new(0x1b8),
                Dword::new(0xffffffff),
            ],
            kinds: [
                Dword::new(race_list(RaceField::KindSetsFirst, 0)),
                Dword::new(race_list(RaceField::KindSetsSecond, 0)),
                Dword::new(race_list(RaceField::KindSetsFirst, 3)),
                Dword::new(race_list(RaceField::KindSetsSecond, 3)),
                Dword::new(race_list(RaceField::KindSetsFirst, 6)),
                Dword::new(race_list(RaceField::KindSetsSecond, 6)),
                Dword::new(race_list(RaceField::KindSetsFirst, 9)),
                Dword::new(race_list(RaceField::KindSetsSecond, 9)),
                Dword::new(race_list(RaceField::KindSetsFirst, 12)),
                Dword::new(race_list(RaceField::KindSetsSecond, 12)),
                Dword::new(race_list(RaceField::KindSetsFirst, 15)),
                Dword::new(race_list(RaceField::KindSetsSecond, 15)),
                Dword::new(race_list(RaceField::KindSetsFirst, 18)),
                Dword::new(race_list(RaceField::KindSetsSecond, 18)),
                Dword::new(race_list(RaceField::KindSetsFirst, 21)),
                Dword::new(race_list(RaceField::KindSetsSecond, 21)),
                Dword::new(race_list(RaceField::KindSetsFirst, 24)),
                Dword::new(race_list(RaceField::KindSetsSecond, 24)),
                Dword::new(race_list(RaceField::KindSetsFirst, 27)),
                Dword::new(race_list(RaceField::KindSetsSecond, 27)),
                Dword::new(race_list(RaceField::KindSetsFirst, 30)),
                Dword::new(race_list(RaceField::KindSetsSecond, 30)),
                Dword::new(race_list(RaceField::KindSetsFirst, 33)),
                Dword::new(race_list(RaceField::KindSetsSecond, 33)),
                Dword::new(race_list(RaceField::KindSetsFirst, 36)),
                Dword::new(race_list(RaceField::KindSetsSecond, 36)),
                Dword::new(race_list(RaceField::KindSetsFirst, 39)),
                Dword::new(race_list(RaceField::KindSetsSecond, 39)),
                Dword::new(race_list(RaceField::KindSetsFirst, 42)),
                Dword::new(race_list(RaceField::KindSetsSecond, 42)),
                Dword::new(race_list(RaceField::KindSetsFirst, 45)),
                Dword::new(race_list(RaceField::KindSetsSecond, 45)),
                Dword::new(race_list(RaceField::KindSetsFirst, 48)),
                Dword::new(race_list(RaceField::KindSetsSecond, 48)),
                Dword::new(race_list(RaceField::KindSetsFirst, 51)),
                Dword::new(race_list(RaceField::KindSetsSecond, 51)),
                Dword::new(race_list(RaceField::KindSetsFirst, 54)),
                Dword::new(race_list(RaceField::KindSetsSecond, 54)),
                Dword::new(race_list(RaceField::KindSetsFirst, 57)),
                Dword::new(race_list(RaceField::KindSetsSecond, 57)),
                Dword::new(race_list(RaceField::KindSetsFirst, 60)),
                Dword::new(race_list(RaceField::KindSetsSecond, 60)),
                Dword::new(race_list(RaceField::KindSetsFirst, 63)),
                Dword::new(race_list(RaceField::KindSetsSecond, 63)),
                Dword::new(race_list(RaceField::KindSetsFirst, 66)),
                Dword::new(race_list(RaceField::KindSetsSecond, 66)),
                Dword::new(race_list(RaceField::KindSetsFirst, 69)),
                Dword::new(race_list(RaceField::KindSetsSecond, 69)),
                Dword::new(race_list(RaceField::KindSetsFirst, 72)),
                Dword::new(race_list(RaceField::KindSetsSecond, 72)),
                Dword::new(race_list(RaceField::KindSetsFirst, 75)),
                Dword::new(race_list(RaceField::KindSetsSecond, 75)),
                Dword::new(race_list(RaceField::KindSetsFirst, 78)),
                Dword::new(race_list(RaceField::KindSetsSecond, 78)),
                Dword::new(race_list(RaceField::KindSetsFirst, 81)),
                Dword::new(race_list(RaceField::KindSetsSecond, 81)),
                Dword::new(race_list(RaceField::KindSetsFirst, 84)),
                Dword::new(race_list(RaceField::KindSetsSecond, 84)),
                Dword::new(race_list(RaceField::KindSetsFirst, 87)),
                Dword::new(race_list(RaceField::KindSetsSecond, 87)),
                Dword::new(race_list(RaceField::KindSetsFirst, 90)),
                Dword::new(race_list(RaceField::KindSetsSecond, 90)),
                Dword::new(race_list(RaceField::KindSetsFirst, 93)),
                Dword::new(race_list(RaceField::KindSetsSecond, 93)),
                Dword::new(race_list(RaceField::KindSetsFirst, 96)),
                Dword::new(race_list(RaceField::KindSetsSecond, 96)),
                Dword::new(race_list(RaceField::KindSetsFirst, 99)),
                Dword::new(race_list(RaceField::KindSetsSecond, 99)),
                Dword::new(race_list(RaceField::KindSetsFirst, 51)),
                Dword::new(race_list(RaceField::KindSetsSecond, 51)),
                Dword::new(race_list(RaceField::KindSetsFirst, 51)),
                Dword::new(race_list(RaceField::KindSetsSecond, 51)),
                Dword::new(0x0),
                Dword::new(0x10),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x4),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x2),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x2),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x4),
                Dword::new(0x0),
            ],
            _tail: Bytes::new([0; 4]),
        }
    }
}

/// The sets the race loader names directly (the others come from the kind table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedSet {
    Replay,
    FirstA,
    FirstB,
    FirstC,
    SecondA,
    SecondB,
    Last,
}

impl NamedSet {
    #[cfg(test)]
    pub const ALL: [NamedSet; 7] = [
        NamedSet::Replay,
        NamedSet::FirstA,
        NamedSet::FirstB,
        NamedSet::FirstC,
        NamedSet::SecondA,
        NamedSet::SecondB,
        NamedSet::Last,
    ];

    /// The set as the handle a load command carries.
    pub fn list(self) -> rc2k_core::audio::commands::SoundList {
        let field = match self {
            NamedSet::Replay => RaceField::ReplaySet,
            NamedSet::FirstA => RaceField::FirstSetA,
            NamedSet::FirstB => RaceField::FirstSetB,
            NamedSet::FirstC => RaceField::FirstSetC,
            NamedSet::SecondA => RaceField::SecondSetA,
            NamedSet::SecondB => RaceField::SecondSetB,
            NamedSet::Last => RaceField::LastSet,
        };
        rc2k_core::audio::commands::SoundList::Race { field, entry: 0 }
    }
}

impl RaceSoundSets {
    /// The sets of the kind-table entry `n` (two a kind of car), as a handle.
    #[cfg(test)]
    pub fn kind_list_of(
        n: usize,
    ) -> Result<rc2k_core::audio::commands::SoundList, crate::errors::BadInput> {
        crate::sound::owner::sound().race.sets.kind_list(n)
    }

    /// The sets of the kind-table entry `n` (two a kind of car), as a handle.
    pub fn kind_list(
        &self,
        n: usize,
    ) -> Result<rc2k_core::audio::commands::SoundList, crate::errors::BadInput> {
        let handle = self
            .kinds
            .get(n)
            .filter(|_| n < Self::KIND_ENTRIES)
            .ok_or(crate::errors::BadInput("a race sound kind past the table"))?
            .get();
        match race_list_of(handle) {
            Some((field, entry)) => {
                Ok(rc2k_core::audio::commands::SoundList::Race { field, entry })
            }
            None => Err(crate::errors::BadInput(
                "a sound table's entry off its piece",
            )),
        }
    }

    /// Entry `n` of the list that starts at entry `entry` of `field`. A list is read on until its
    /// -1, through the next fields of the sets in their order where it runs past its own; a read
    /// past the sets is none. The list is named at run time (a handle the race sound loader is
    /// given), so any of the fields can follow.
    pub fn list_entry(
        &self,
        field: RaceField,
        entry: u32,
        n: usize,
    ) -> Result<Option<u32>, crate::errors::BadInput> {
        let runs = self.runs();
        let mut left = (entry as usize).saturating_add(n);
        for (run, own) in runs.iter().zip(RaceField::ALL).skip(field as usize) {
            debug_assert_eq!(run.count(), own.dwords());
            if left < run.count() {
                return Ok(Some(run.bits(left)));
            }
            left -= run.count();
        }
        // Past the sets: a list that starts there is refused, a read that runs there is none.
        if (entry as usize)
            >= RaceField::ALL[field as usize..]
                .iter()
                .map(|f| f.dwords())
                .sum()
        {
            return Err(crate::errors::BadInput("a sound list off the race's sets"));
        }
        Ok(None)
    }

    /// The fields in the record's order, each a run of dwords.
    fn runs(&self) -> [&dyn crate::session::Run; 11] {
        [
            &self.kind_sets_first,
            &self.first_set_a,
            &self.first_set_c,
            &self.kind_sets_second,
            &self.second_set_a,
            &self.second_set_b,
            &self.last_set,
            &self.first_set_b,
            &self.replay_set,
            &self.kinds,
            &self._tail,
        ]
    }

    const KIND_ENTRIES: usize = 72;
}

/// The cues' lists.
#[derive(rc2k_derive::Cells)]
pub struct CueLists {
    /// Padding that keeps every field's 1999 alignment (the struct is 16-byte aligned).
    pub _lead: PlainBytes<8>,
    /// Where each cue's list of sounds starts, as a byte offset in `lists`
    /// (`physics::queue_sound_cue`).
    pub table: [PlainDword; 113],
    /// The lists.
    pub lists: PlainBytes<836>,
}

impl CueLists {
    pub const fn new() -> Self {
        CueLists {
    _lead: PlainBytes::new([0; 8]),
    table: [PlainDword::new(0), PlainDword::new(4), PlainDword::new(12), PlainDword::new(20), PlainDword::new(28), PlainDword::new(36), PlainDword::new(44), PlainDword::new(52), PlainDword::new(60), PlainDword::new(68), PlainDword::new(80), PlainDword::new(92), PlainDword::new(104), PlainDword::new(116), PlainDword::new(0), PlainDword::new(0), PlainDword::new(0), PlainDword::new(128), PlainDword::new(136), PlainDword::new(144), PlainDword::new(152), PlainDword::new(160), PlainDword::new(168), PlainDword::new(176), PlainDword::new(184), PlainDword::new(192), PlainDword::new(204), PlainDword::new(216), PlainDword::new(228), PlainDword::new(240), PlainDword::new(252), PlainDword::new(260), PlainDword::new(268), PlainDword::new(276), PlainDword::new(284), PlainDword::new(292), PlainDword::new(300), PlainDword::new(308), PlainDword::new(316), PlainDword::new(324), PlainDword::new(332), PlainDword::new(340), PlainDword::new(0), PlainDword::new(0), PlainDword::new(0), PlainDword::new(0), PlainDword::new(0), PlainDword::new(348), PlainDword::new(364), PlainDword::new(372), PlainDword::new(380), PlainDword::new(388), PlainDword::new(396), PlainDword::new(404), PlainDword::new(412), PlainDword::new(420), PlainDword::new(508), PlainDword::new(516), PlainDword::new(524), PlainDword::new(532), PlainDword::new(540), PlainDword::new(548), PlainDword::new(556), PlainDword::new(564), PlainDword::new(428), PlainDword::new(444), PlainDword::new(436), PlainDword::new(452), PlainDword::new(460), PlainDword::new(468), PlainDword::new(476), PlainDword::new(484), PlainDword::new(492), PlainDword::new(500), PlainDword::new(0), PlainDword::new(0), PlainDword::new(0), PlainDword::new(0), PlainDword::new(0), PlainDword::new(0), PlainDword::new(572), PlainDword::new(588), PlainDword::new(580), PlainDword::new(596), PlainDword::new(604), PlainDword::new(612), PlainDword::new(620), PlainDword::new(628), PlainDword::new(636), PlainDword::new(644), PlainDword::new(652), PlainDword::new(660), PlainDword::new(668), PlainDword::new(676), PlainDword::new(684), PlainDword::new(692), PlainDword::new(700), PlainDword::new(708), PlainDword::new(716), PlainDword::new(724), PlainDword::new(732), PlainDword::new(740), PlainDword::new(748), PlainDword::new(756), PlainDword::new(764), PlainDword::new(772), PlainDword::new(780), PlainDword::new(788), PlainDword::new(796), PlainDword::new(804), PlainDword::new(812), PlainDword::new(820), PlainDword::new(828)],
    lists: PlainBytes::new(with_runs::<836>(&[(0, b"\xff\xff\xff\xffE\x01\x00\x00\xff\xff\xff\xffG\x01\x00\x00\xff\xff\xff\xffI\x01\x00\x00\xff\xff\xff\xffK\x01\x00\x00\xff\xff\xff\xffM\x01\x00\x00\xff\xff\xff\xffO\x01\x00\x00\xff\xff\xff\xffQ\x01\x00\x00\xff\xff\xff\xffS\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00E\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00G\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00I\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00K\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00M\x01\x00\x00\xff\xff\xff\xffU\x01\x00\x00\xff\xff\xff\xffW\x01\x00\x00\xff\xff\xff\xffY\x01\x00\x00\xff\xff\xff\xff[\x01\x00\x00\xff\xff\xff\xff]\x01\x00\x00\xff\xff\xff\xff_\x01\x00\x00\xff\xff\xff\xffa\x01\x00\x00\xff\xff\xff\xffc\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00U\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00W\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00Y\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00[\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00]\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00\xff\xff\xff\xffg\x01\x00\x00\xff\xff\xff\xff\xf9\x00\x00\x00\xff\xff\xff\xff\xfb\x00\x00\x00\xff\xff\xff\xff\xfd\x00\x00\x00\xff\xff\xff\xff\xff\x00\x00\x00\xff\xff\xff\xff\x01\x01\x00\x00\xff\xff\xff\xff\x03\x01\x00\x00\xff\xff\xff\xff\x05\x01\x00\x00\xff\xff\xff\xff\x07\x01\x00\x00\xff\xff\xff\xff\t\x01\x00\x00\xff\xff\xff\xff\x0b\x01\x00\x00\xff\xff\xff\xff1\x01\x00\x00!\x01\x00\x00?\x01\x00\x00\xff\xff\xff\xff\x11\x01\x00\x00\xff\xff\xff\xff\x0f\x01\x00\x00\xff\xff\xff\xff\x13\x01\x00\x00\xff\xff\xff\xff\r\x01\x00\x00\xff\xff\xff\xff1\x01\x00\x00\xff\xff\xff\xffA\x01\x00\x00\xff\xff\xff\xff=\x01\x00\x00\xff\xff\xff\xff3\x01\x00\x00\xff\xff\xff\xff\x1b\x01\x00\x00\xff\xff\xff\xff)\x01\x00\x00\xff\xff\xff\xff\x1d\x01\x00\x00\xff\xff\xff\xff#\x01\x00\x00\xff\xff\xff\xff+\x01\x00\x00\xff\xff\xff\xffC\x01\x00\x00\xff\xff\xff\xff\x15\x01\x00\x00\xff\xff\xff\xff\x17\x01\x00\x00\xff\xff\xff\xff\x1f\x01\x00\x00\xff\xff\xff\xff%\x01\x00\x00\xff\xff\xff\xff\x19\x01\x00\x00\xff\xff\xff\xff'\x01\x00\x00\xff\xff\xff\xff-\x01\x00\x00\xff\xff\xff\xff/\x01\x00\x00\xff\xff\xff\xff5\x01\x00\x00\xff\xff\xff\xff7\x01\x00\x00\xff\xff\xff\xff9\x01\x00\x00\xff\xff\xff\xff;\x01\x00\x00\xff\xff\xff\xff}\x01\x00\x00\xff\xff\xff\xff\x81\x01\x00\x00\xff\xff\xff\xff\x7f\x01\x00\x00\xff\xff\xff\xffs\x01\x00\x00\xff\xff\xff\xffi\x01\x00\x00\xff\xff\xff\xffk\x01\x00\x00\xff\xff\xff\xffm\x01\x00\x00\xff\xff\xff\xffo\x01\x00\x00\xff\xff\xff\xffq\x01\x00\x00\xff\xff\xff\xffu\x01\x00\x00\xff\xff\xff\xffw\x01\x00\x00\xff\xff\xff\xffy\x01\x00\x00\xff\xff\xff\xff{\x01\x00\x00\xff\xff\xff\xff\x83\x01\x00\x00\xff\xff\xff\xff\x85\x01\x00\x00\xff\xff\xff\xff\x87\x01\x00\x00\xff\xff\xff\xff\x89\x01\x00\x00\xff\xff\xff\xff\x8b\x01\x00\x00\xff\xff\xff\xff\x8d\x01\x00\x00\xff\xff\xff\xff\x8f\x01\x00\x00\xff\xff\xff\xff\x91\x01\x00\x00\xff\xff\xff\xff\x93\x01\x00\x00\xff\xff\xff\xff\x95\x01\x00\x00\xff\xff\xff\xff\x97\x01\x00\x00\xff\xff\xff\xff\x99\x01\x00\x00\xff\xff\xff\xff\x9b\x01\x00\x00\xff\xff\xff\xff\x9d\x01\x00\x00\xff\xff\xff\xff\x9f\x01\x00\x00\xff\xff\xff\xff\xa1\x01\x00\x00\xff\xff\xff\xff\xa3\x01\x00\x00\xff\xff\xff\xff\xa5\x01\x00\x00\xff\xff\xff\xff\xa7\x01\x00\x00\xff\xff\xff\xff\xa9\x01\x00\x00\xff\xff\xff\xff")])),
        }
    }
}

/// A list of dwords inside a block of bytes, read by position and bounded to the block.
pub struct ListIn<'a, const N: usize> {
    bytes: &'a PlainBytes<N>,
    from: usize,
}

impl<const N: usize> ListIn<'_, N> {
    /// Entry `k`; refused past the block (a list there has no end).
    pub fn get(&self, k: usize) -> Result<u32, crate::errors::BadInput> {
        k.checked_mul(4)
            .and_then(|k| k.checked_add(self.from))
            .filter(|at| at + 4 <= N)
            .map(|at| self.bytes.get_u32(at))
            .ok_or(crate::errors::BadInput(
                "a sound list read past the lists that hold it",
            ))
    }
}

impl CueLists {
    /// Cue `cue`'s list of sounds, in the lists after the table (each ends in -1 inside them).
    pub fn list(&self, cue: u32) -> Result<ListIn<'_, 836>, crate::errors::BadInput> {
        let lists = self;
        let cell = lists
            .table
            .get(cue as usize)
            .ok_or(crate::errors::BadInput("a sound cue past the table"))?;
        Ok(ListIn {
            bytes: &lists.lists,
            from: cell.get() as usize,
        })
    }
}

/// One sound set of a kind's record (0x28 bytes; `sound::revs_sound`, `throttle_sound`,
/// `speed_sound`): a sound number, three more values, and a pitch and a volume as a start, an end
/// and the span between them (`race::sound_tables` makes the whole numbers floats and the spans
/// once). The sound at the object's pitch is `input * scale + low`. The values are kept as bits: a
/// whole number until the tables are made ready. Laid out as the 1999 variables: the recorded
/// assembly's tests lay it over raw bytes.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct SoundSet {
    /// The sound's number, added to the object's `sound_offset`.
    pub sound: PlainDword,
    /// Read by no sound code and 0 in every set of the shipped data, so not converted to a
    /// float (a spare word, probably an input range's start).
    pub unused_float_04: PlainDword,
    /// As `unused_float_04` (probably the range's end).
    pub unused_float_08: PlainDword,
    /// As `unused_float_04` (probably the range's span).
    pub unused_float_0c: PlainDword,
    /// The pitch at input 0.
    pub pitch_low: PlainDword,
    /// The pitch's end (the span's second value).
    pub pitch_high: PlainDword,
    /// The pitch span, `pitch_high - pitch_low`: the pitch's slope.
    pub pitch_scale: PlainDword,
    /// The volume at input 0.
    pub volume_low: PlainDword,
    /// The volume's end.
    pub volume_high: PlainDword,
    /// The volume span: the volume's slope.
    pub volume_scale: PlainDword,
}

const SOUND_SET_LEN: usize = 0x28;

impl SoundSet {
    /// The set in `b` from byte `at` (each dword little-endian).
    const fn from_slice(b: &[u8], at: usize) -> Self {
        const fn word(b: &[u8], at: usize) -> PlainDword {
            PlainDword::new(u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]))
        }
        Self {
            sound: word(b, at),
            unused_float_04: word(b, at + 0x04),
            unused_float_08: word(b, at + 0x08),
            unused_float_0c: word(b, at + 0x0C),
            pitch_low: word(b, at + 0x10),
            pitch_high: word(b, at + 0x14),
            pitch_scale: word(b, at + 0x18),
            volume_low: word(b, at + 0x1C),
            volume_high: word(b, at + 0x20),
            volume_scale: word(b, at + 0x24),
        }
    }

    /// A set of zeros with the byte runs `runs` ((offset, bytes), as `with_runs`).
    const fn from_runs(runs: &[(usize, &[u8])]) -> Self {
        Self::from_slice(&with_runs::<SOUND_SET_LEN>(runs), 0)
    }
}

/// A sound kind's record (0x88 bytes): the revs' range and slope, then the engine's three sets.
/// Laid out as the 1999 variables: the recorded assembly's tests lay it over raw bytes.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct KindRecord {
    /// The kind's flags (`sound::car_engine_state`'s `state` reads them).
    pub flags: PlainDword,
    /// The revs the engine's input starts from (a whole number made a float).
    pub revs_low: PlainDword,
    /// The revs' end.
    pub revs_high: PlainDword,
    /// First the span (`revs_high - revs_low`), then its reciprocal (`race::sound_tables`): the
    /// engine input's slope.
    pub revs_scale: PlainDword,
    /// The engine's revs sound (`sound::revs_sound`).
    pub revs: SoundSet,
    /// The throttle sound (`sound::throttle_sound`).
    pub throttle: SoundSet,
    /// The speed sound (`sound::speed_sound`).
    pub speed: SoundSet,
}

const KIND_RECORD_LEN: usize = 0x88;

impl KindRecord {
    /// A record of zeros with the byte runs `runs` ((offset, bytes), as `with_runs`).
    const fn from_runs(runs: &[(usize, &[u8])]) -> Self {
        let b = with_runs::<KIND_RECORD_LEN>(runs);
        const fn word(b: &[u8], at: usize) -> PlainDword {
            PlainDword::new(u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]))
        }
        Self {
            flags: word(&b, 0),
            revs_low: word(&b, 4),
            revs_high: word(&b, 8),
            revs_scale: word(&b, 0xC),
            revs: SoundSet::from_slice(&b, 0x10),
            throttle: SoundSet::from_slice(&b, 0x38),
            speed: SoundSet::from_slice(&b, 0x60),
        }
    }
}

/// The car sound kinds' records, the computer cars' set and the sine table.
#[derive(rc2k_derive::Cells)]
pub struct SoundKinds {
    /// Each sound kind's record, by its number (`sound::car_engine_state`).
    pub kinds: [PlainDword; 36],
    /// The sound kinds' records of 0x88 bytes (`race::sound_tables` works the floats out).
    pub records: [KindRecord; 34],
    /// The computer's cars' set (`sound::revs_sound`).
    pub computers: SoundSet,
    /// 0x40 floats, 1 + 0.05 sin x (`race::sound_tables`).
    pub sines: [PlainFloat; 64],
    /// Padding that fills the record to its 16-byte alignment.
    pub _tail: PlainBytes<8>,
}

impl SoundKinds {
    pub const fn new() -> Self {
        SoundKinds {
            kinds: [
                PlainDword::new(0),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(3),
                PlainDword::new(4),
                PlainDword::new(5),
                PlainDword::new(6),
                PlainDword::new(7),
                PlainDword::new(8),
                PlainDword::new(9),
                PlainDword::new(10),
                PlainDword::new(11),
                PlainDword::new(12),
                PlainDword::new(13),
                PlainDword::new(14),
                PlainDword::new(15),
                PlainDword::new(16),
                PlainDword::new(17),
                PlainDword::new(18),
                PlainDword::new(19),
                PlainDword::new(20),
                PlainDword::new(21),
                PlainDword::new(22),
                PlainDword::new(23),
                PlainDword::new(24),
                PlainDword::new(25),
                PlainDword::new(26),
                PlainDword::new(27),
                PlainDword::new(28),
                PlainDword::new(29),
                PlainDword::new(30),
                PlainDword::new(31),
                PlainDword::new(32),
                PlainDword::new(33),
                PlainDword::new(17),
                PlainDword::new(17),
            ],
            records: [
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xd0\x07\x00\x00\x88\x13"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x02"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xd0\x07\x00\x00\x88\x13"),
                    (16, b"\x04"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x06"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xec\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"\x08"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\n"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xec \x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"\x0c"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x0e"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xeb\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"\x10"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x12"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"\x14"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x16"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xeb\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"\x18"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x1a"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xec\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"\x1c"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x1e"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xd0\x07\x00\x00\x88\x13"),
                    (16, b" "),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\""),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xd0\x07\x00\x00\x88\x13"),
                    (16, b"$"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"&"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xea\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"("),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"*"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b","),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"."),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"0"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"2"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"4"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"6"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xec\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"8"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b":"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"<"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b">"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xd0\x07\x00\x00\x88\x13"),
                    (16, b"@"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"B"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xd0\x07\x00\x00\x88\x13"),
                    (16, b"D"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"F"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"H"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"J"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"L"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"N"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xd0\x07\x00\x00\x88\x13"),
                    (16, b"P"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"R"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"T"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"V"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xec\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"X"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"Z"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xec\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"\\"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"^"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xeb\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"`"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"b"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"d"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"f"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"h"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"j"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xec\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"l"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"n"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (0, b"\xea\x10\x00\x00\xd0\x07\x00\x00\x94\x11"),
                    (16, b"p"),
                    (32, b"\x98:\x00\x00\xe8\x80"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"r"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xd0\x07\x00\x00\xa0\x0f"),
                    (16, b"t"),
                    (32, b"\xf0U\x00\x00\xe0\xab\x00\x00x\xe6"),
                    (48, b"\x0e\x00\x00\x00 \x00\x00\x00v"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (1, b" \x00\x00\xb8\x0b\x00\x00\xa0\x0f"),
                    (16, b"x"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"z"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"|"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"~"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\xa0\x0f"),
                    (16, b"\x80"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x82"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
                KindRecord::from_runs(&[
                    (4, b"\xd0\x07\x00\x00\x88\x13"),
                    (16, b"\x84"),
                    (32, b"(#\x00\x00x\xe6"),
                    (44, b"\x0e\x00\x00\x00 "),
                    (56, b"\x86"),
                    (72, b"\x98:\x00\x00\xf8$\x01"),
                    (84, b"\x04\x00\x00\x00 "),
                    (96, b"\x88"),
                    (112, b"PF\x00\x00\x80\xbb"),
                    (124, b"\x04\x00\x00\x00\x14"),
                ]),
            ],
            computers: SoundSet::from_runs(&[
                (0, b"\x8a"),
                (
                    17,
                    b"`jF\x00|\x92G\x00`jG\x00\x00\x80A\x00\x00\x80B\x00\x00@B",
                ),
            ]),
            sines: [const { PlainFloat::new(0.0) }; 64],
            _tail: PlainBytes::new([0; 8]),
        }
    }
}

impl SoundKinds {
    /// Kind `n`'s record. The kind past the table, or a table entry off the records, is
    /// refused.
    pub fn record(&self, n: u32) -> Result<&KindRecord, crate::errors::BadInput> {
        let cell = self
            .kinds
            .get(n as usize)
            .ok_or(crate::errors::BadInput("a sound kind past the table"))?;
        self.records
            .get(cell.get() as usize)
            .ok_or(crate::errors::BadInput(
                "a sound table's entry off its piece",
            ))
    }

    /// The computers' cars' set (40 bytes).
    pub fn computers_set(&self) -> &SoundSet {
        &self.computers
    }
}

/// The surface sound sets.
#[derive(rc2k_derive::Cells)]
pub struct SurfaceSets {
    /// Padding that keeps every field's 1999 alignment (the struct is 16-byte aligned).
    pub _lead: PlainBytes<12>,
    /// The sets (seven pairs; each is the byte offset in `lists` where its set starts;
    /// `sound::surface_sounds`).
    pub sets: [PlainDword; 14],
    /// The lists.
    pub lists: PlainBytes<392>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: PlainBytes<4>,
}

impl SurfaceSets {
    pub const fn new() -> Self {
        SurfaceSets {
    _lead: PlainBytes::new([0; 12]),
    sets: [PlainDword::new(0), PlainDword::new(36), PlainDword::new(56), PlainDword::new(92), PlainDword::new(112), PlainDword::new(148), PlainDword::new(168), PlainDword::new(204), PlainDword::new(280), PlainDword::new(316), PlainDword::new(224), PlainDword::new(260), PlainDword::new(336), PlainDword::new(372)],
    lists: PlainBytes::new(with_runs::<392>(&[(0, b"\x01\x00\x00\x00\x8c\x00\x00\x00\x8d\x00\x00\x00\x90\x00\x00\x00\x91\x00\x00\x00\x8e\x00\x00\x00\x8f\x00\x00\x00\x90\x00\x00\x00\x91\x00\x00\x00\x02\x00\x00\x00\x92\x00\x00\x00\x93\x00\x00\x00\x94\x00\x00\x00\x95\x00\x00\x00\x01\x00\x00\x00\x96\x00\x00\x00\x97\x00\x00\x00\x9a\x00\x00\x00\x9b\x00\x00\x00\x98\x00\x00\x00\x99\x00\x00\x00\x9a\x00\x00\x00\x9b\x00\x00\x00\x02\x00\x00\x00\x9c\x00\x00\x00\x9d\x00\x00\x00\x9e\x00\x00\x00\x9f\x00\x00\x00\x01\x00\x00\x00\xa0\x00\x00\x00\xa1\x00\x00\x00\xa4\x00\x00\x00\xa5\x00\x00\x00\xa2\x00\x00\x00\xa3\x00\x00\x00\xa4\x00\x00\x00\xa5\x00\x00\x00\x02\x00\x00\x00\xa6\x00\x00\x00\xa7\x00\x00\x00\xa8\x00\x00\x00\xa9\x00\x00\x00\x01\x00\x00\x00\xaa\x00\x00\x00\xab\x00\x00\x00\xae\x00\x00\x00\xaf\x00\x00\x00\xac\x00\x00\x00\xad\x00\x00\x00\xae\x00\x00\x00\xaf\x00\x00\x00\x02\x00\x00\x00\xb0\x00\x00\x00\xb1\x00\x00\x00\xb2\x00\x00\x00\xb3\x00\x00\x00\x01\x00\x00\x00\xb4\x00\x00\x00\xb5\x00\x00\x00\xb8\x00\x00\x00\xb9\x00\x00\x00\xb6\x00\x00\x00\xb7\x00\x00\x00\xb8\x00\x00\x00\xb9\x00\x00\x00\x02\x00\x00\x00\xba\x00\x00\x00\xbb\x00\x00\x00\xbc\x00\x00\x00\xbd\x00\x00\x00\x01\x00\x00\x00\xbe\x00\x00\x00\xbf\x00\x00\x00\xc2\x00\x00\x00\xc3\x00\x00\x00\xc0\x00\x00\x00\xc1\x00\x00\x00\xc2\x00\x00\x00\xc3\x00\x00\x00\x02\x00\x00\x00\xc4\x00\x00\x00\xc5\x00\x00\x00\xc6\x00\x00\x00\xc7\x00\x00\x00\x01\x00\x00\x00\xc8\x00\x00\x00\xc9\x00\x00\x00\xcc\x00\x00\x00\xcd\x00\x00\x00\xca\x00\x00\x00\xcb\x00\x00\x00\xcc\x00\x00\x00\xcd\x00\x00\x00\x02\x00\x00\x00\xce\x00\x00\x00\xcf\x00\x00\x00\xd0\x00\x00\x00\xd1")])),
    _tail: PlainBytes::new([0; 4]),
        }
    }
}

impl SurfaceSets {
    /// Surface sound set `n`'s list: the flag, then its sounds (nine dwords for a set of eight,
    /// five for a set of four; the sets follow one another in the lists).
    pub fn set(&self, n: usize) -> Result<ListIn<'_, 392>, crate::errors::BadInput> {
        let sets = self;
        let cell = sets.sets.get(n).ok_or(crate::errors::BadInput(
            "a surface sound set past the table",
        ))?;
        Ok(ListIn {
            bytes: &sets.lists,
            from: cell.get() as usize,
        })
    }
}

/// The spot voices.
#[repr(align(16))]
#[derive(rc2k_derive::Cells)]
pub struct SpotVoices {
    /// 0x48 records of 0x50 bytes (`sound::spot_voices`).
    pub voices: [crate::sound::spot_voices::SpotVoice; 72],
}

impl SpotVoices {
    pub const fn new() -> Self {
        SpotVoices {
            voices: [const { crate::sound::spot_voices::SpotVoice::free() }; 72],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Byte offsets in the 1999 layout of the fields, and of the 72 lists the kinds table names.
    const OLD_FIELD_OFFSETS: [u32; 11] = [
        0x0, 0x198, 0x1F4, 0x36C, 0x504, 0x560, 0x6DC, 0x758, 0x830, 0x850, 0x9AC,
    ];
    const OLD_KIND_OFFSETS: [u32; 72] = [
        0, 876, 12, 888, 24, 900, 36, 912, 48, 924, 60, 936, 72, 948, 84, 960, 96, 972, 108, 984,
        120, 996, 132, 1008, 144, 1020, 156, 1032, 168, 1044, 180, 1056, 192, 1068, 204, 1080, 216,
        1092, 228, 1104, 240, 1116, 252, 1128, 264, 1140, 276, 1152, 288, 1164, 300, 1176, 312,
        1188, 324, 1200, 336, 1212, 348, 1224, 360, 1236, 372, 1248, 384, 1260, 396, 1272, 204,
        1080, 204, 1080,
    ];

    /// The flat dwords of the 1999 layout: every field in order.
    fn flat(sets: &RaceSoundSets) -> Vec<u32> {
        sets.runs()
            .iter()
            .flat_map(|run| (0..run.count()).map(|i| run.bits(i)))
            .collect()
    }

    #[test]
    fn field_offsets_are_the_old_layout() {
        for (field, old) in RaceField::ALL.into_iter().zip(OLD_FIELD_OFFSETS) {
            assert_eq!(field.original_offset(), old, "{field:?}");
        }
        assert_eq!(
            RaceField::Tail.original_offset() + 4,
            core::mem::size_of::<RaceSoundSets>() as u32
        );
    }

    /// Every entry of every list start, to the end of the sets and past it, reads as a read of
    /// the 1999 layout's bytes does; the word (the hash's input) is the old offset + 2, and
    /// names the same list back.
    #[test]
    fn list_entries_read_as_the_byte_read() {
        let sets = Box::new(RaceSoundSets::new());
        for (k, run) in sets.runs().iter().enumerate() {
            for i in 0..run.count() {
                run.set_bits(i, 0x7000_0000 + (OLD_FIELD_OFFSETS[k] / 4) + i as u32);
            }
        }
        let filled = flat(&sets);
        for field in RaceField::ALL {
            for entry in 0..field.dwords() as u32 {
                let offset = field.original_offset() + 4 * entry;
                let list = rc2k_core::audio::commands::SoundList::Race { field, entry };
                assert_eq!(list.word(), offset + 2);
                assert_eq!(
                    rc2k_core::audio::commands::SoundList::from_word(offset + 2),
                    Some(list)
                );
                for n in 0..3 {
                    let old = filled.get(offset as usize / 4 + n).copied();
                    assert_eq!(
                        sets.list_entry(field, entry, n).unwrap(),
                        old,
                        "{field:?} {entry} {n}"
                    );
                }
            }
        }
        assert!(sets.list_entry(RaceField::Tail, 1, 0).is_err());
        assert!(sets.list_entry(RaceField::Tail, 0, 0).is_ok());
        assert_eq!(sets.list_entry(RaceField::Tail, 0, 1).unwrap(), None);
    }

    /// The lists the kinds table and the named sets start: the same words as the 1999 offsets, and
    /// the lists read to their -1 without reaching the kinds table (whose values changed with
    /// their encoding).
    #[test]
    fn started_lists_keep_their_hash_words_and_end_before_the_kinds() {
        use rc2k_core::audio::commands::SoundList;
        let sets = &crate::sound::owner::sound().race.sets;
        let flat = flat(sets);
        let kinds_start = RaceField::Kinds.original_offset() as usize / 4;
        let mut lists: Vec<(SoundList, Option<u32>)> = (0..72)
            .map(|n| (sets.kind_list(n).unwrap(), Some(OLD_KIND_OFFSETS[n])))
            .collect();
        let named_old = [
            (NamedSet::Replay, 0x830),
            (NamedSet::FirstA, 0x198),
            (NamedSet::FirstB, 0x758),
            (NamedSet::FirstC, 0x1F4),
            (NamedSet::SecondA, 0x504),
            (NamedSet::SecondB, 0x560),
            (NamedSet::Last, 0x6DC),
        ];
        lists.extend(named_old.map(|(set, old)| (set.list(), Some(old))));
        for (list, old) in lists {
            assert_eq!(list.word(), old.unwrap() + 2, "{list:?}");
            let SoundList::Race { field, entry } = list else {
                panic!()
            };
            let start = old.unwrap() as usize / 4;
            let mut n = 0;
            loop {
                let got = sets.list_entry(field, entry, n).unwrap();
                assert_eq!(got, flat.get(start + n).copied());
                assert!(start + n < kinds_start, "{list:?} reaches the kinds");
                if got == Some(u32::MAX) {
                    break;
                }
                n += 1;
            }
        }
    }

    #[test]
    fn racesoundsets_kind_lists_are_their_handles() {
        for n in 0..72 {
            let handle = crate::sound::owner::sound().race.sets.kinds[n].get();
            let at = RaceSoundSets::kind_list_of(n).unwrap();
            let (field, entry) = race_list_of(handle).unwrap();
            assert_eq!(
                at,
                rc2k_core::audio::commands::SoundList::Race { field, entry }
            );
        }
        assert!(RaceSoundSets::kind_list_of(72).is_err());
    }

    #[test]
    fn cuelists_end_in_minus_one() {
        // Every cue's list is one of the lists, and ends.
        for cue in 0..113 {
            let list = crate::sound::owner::sound()
                .race
                .cue_lists
                .get()
                .list(cue)
                .unwrap();
            // The list ends in -1 inside the lists.
            assert!((0..6).any(|k| list.get(k) == Ok(u32::MAX)), "{cue}");
        }
        assert!(
            crate::sound::owner::sound()
                .race
                .cue_lists
                .get()
                .list(113)
                .is_err()
        );
    }

    #[test]
    fn surfacesets_lists_are_inside_the_store() {
        for n in 0..14 {
            let set = crate::sound::owner::sound()
                .race
                .surface_sets
                .get()
                .set(n)
                .unwrap();
            // A set of eight (flag 1) holds nine dwords, a set of four five, inside the lists.
            let dwords = if set.get(0).unwrap() == 1 { 9 } else { 5 };
            assert!(set.get(dwords - 1).is_ok(), "{n}");
        }
        let kinds = crate::sound::owner::sound().race.kinds.get();
        for n in 0..36 {
            kinds.record(n).unwrap();
        }
        // The records follow the table of their 36 addresses, in the record's field order.
        let table = crate::test_support::paths::field_start(kinds, ".kinds").unwrap();
        let records = crate::test_support::paths::field_start(kinds, ".records").unwrap();
        assert!(records - table >= 36 * 4);
    }
}
