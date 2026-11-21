//! The sound module's sets and tables (the settings page's menu sounds, the race's sets of sounds
//! and the commands that name them, the cues' lists, the car sound kinds' records, the surface
//! sound sets and the spot voices), typed. The sets are lists of sound numbers ending in -1 that
//! the audio thread reads; the tables of pointers name them by handle: the race's sets by
//! `RACE_LIST_TAG` and an offset in the table they lie in; the cues' and surface sets' lists by
//! their byte offset in the block of lists after the table, the kinds' records by their number.
//!
//! Structs of atomic cells with the game's starting values (`tests`). The race's sets are named
//! lists; the read-on order of `RaceField` is what the sound hash's list words rely on.

use crate::memory::session::{Dword, with_runs};
use rc2k_core::audio::commands::RaceField;

/// Handle of the race's sets: in place of a list's address the table holds
/// `TAG | field << 16 | entry` (the field's number in `RaceField::ALL`, the entry in it);
/// 0 does not occur. The list runs on through the fields after its own as far as the
/// sounds read (`SoundSets::list_entry`).
const RACE_LIST_TAG: u32 = 0xC400_0000;

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

/// The settings page's menu sounds list: constants both the game's thread and the audio thread read.
pub struct MenuSounds {
    /// The list `sound::apply_settings` passes as the menu sounds' third argument: nine sound
    /// numbers, then -1.
    pub list: [u32; 10],
}

impl MenuSounds {
    pub const fn new() -> Self {
        MenuSounds {
            list: [
                0x1c7,
                0x1c8,
                0x1c9,
                0x1ca,
                0x163,
                0x197,
                0x1cb,
                0xe4,
                0x1cc,
                0xffff_ffff,
            ],
        }
    }

    /// Entry `n` of the list, none past its ten dwords (it ends in -1 at the tenth).
    pub fn sound(&self, n: usize) -> Option<u32> {
        self.list.get(n).copied()
    }
}

/// The named lists of sounds a race loads (`sound::load_race_sounds`). Lists end in -1; one that
/// runs past its field reads on into the next in `RaceField` order (`list_entry` is the only
/// reader that knows that order).
#[derive(rc2k_derive::Cells)]
pub struct SoundSets {
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
}

/// The ten dwords the race's sounds are sent with (`sound::play_placed_sound`,
/// `sound::stop_sound`); a work area each turn fills again. The game thread's, behind one
/// `RefCell` (`sound::owner::RaceSoundState::request`) that a turn's parts borrow once and pass
/// down. The words keep the order the sound command's dwords had (`set_word` names them by
/// number).
#[derive(Clone, Copy, Default)]
pub struct SoundRequest {
    /// The sound (bit 0x8000: none).
    pub sound: u32,
    /// The pitch, a float's bits (`sound::revs_sound::fill_pitch_and_volume`), then the whole
    /// number sent.
    pub pitch: u32,
    /// The volume, a float's bits, then (without 3D) the whole number sent.
    pub volume: u32,
    /// Where the sound is, three floats.
    pub place: [f32; 3],
    /// Without 3D, the angle (the arctangent of the place's third float over its first).
    pub angle: u32,
    /// Without 3D, the place between left and right (a whole number).
    pub pan: u32,
    /// The pitch's factor, a float.
    pub pitch_factor: f32,
    /// The volume's factor, a float.
    pub volume_factor: f32,
}

impl SoundRequest {
    /// The request as the game starts: all 0.
    pub const fn new() -> Self {
        Self {
            sound: 0,
            pitch: 0,
            volume: 0,
            place: [0.0; 3],
            angle: 0,
            pan: 0,
            pitch_factor: 0.0,
            volume_factor: 0.0,
        }
    }

    /// Stores `bits` in dword `n` (the sound command's order), for the cores that name the words
    /// by number; None past the ten.
    pub fn set_word(&mut self, n: usize, bits: u32) -> Option<()> {
        match n {
            0 => self.sound = bits,
            1 => self.pitch = bits,
            2 => self.volume = bits,
            3..=5 => self.place[n - 3] = f32::from_bits(bits),
            6 => self.angle = bits,
            7 => self.pan = bits,
            8 => self.pitch_factor = f32::from_bits(bits),
            9 => self.volume_factor = f32::from_bits(bits),
            _ => return None,
        }
        Some(())
    }
}

/// The starting values of `kind_sets_first`.
#[rustfmt::skip]
const KIND_SETS_FIRST: [u32; 102] = [
    0, 2, 0xFFFFFFFF, 4, 6, 0xFFFFFFFF, 8, 10, 0xFFFFFFFF, 12, 14, 0xFFFFFFFF, 16, 18, 0xFFFFFFFF,
    20, 22, 0xFFFFFFFF, 24, 26, 0xFFFFFFFF, 28, 30, 0xFFFFFFFF, 32, 34, 0xFFFFFFFF, 36, 38,
    0xFFFFFFFF, 40, 42, 0xFFFFFFFF, 44, 46, 0xFFFFFFFF, 48, 50, 0xFFFFFFFF, 52, 54, 0xFFFFFFFF, 56,
    58, 0xFFFFFFFF, 60, 62, 0xFFFFFFFF, 64, 66, 0xFFFFFFFF, 68, 70, 0xFFFFFFFF, 72, 74, 0xFFFFFFFF,
    76, 78, 0xFFFFFFFF, 80, 82, 0xFFFFFFFF, 84, 86, 0xFFFFFFFF, 88, 90, 0xFFFFFFFF, 92, 94,
    0xFFFFFFFF, 96, 98, 0xFFFFFFFF, 100, 102, 0xFFFFFFFF, 104, 106, 0xFFFFFFFF, 108, 110,
    0xFFFFFFFF, 112, 114, 0xFFFFFFFF, 116, 118, 0xFFFFFFFF, 120, 122, 0xFFFFFFFF, 124, 126,
    0xFFFFFFFF, 128, 130, 0xFFFFFFFF, 132, 134, 0xFFFFFFFF,
];

/// The starting values of `first_set_a`.
#[rustfmt::skip]
const FIRST_SET_A: [u32; 23] = [
    136, 140, 142, 146, 150, 152, 156, 160, 162, 166, 170, 172, 176, 180, 182, 186, 190, 192, 196,
    200, 202, 206, 0xFFFFFFFF,
];

/// The starting values of `first_set_c`.
#[rustfmt::skip]
const FIRST_SET_C: [u32; 94] = [
    245, 243, 241, 239, 237, 247, 249, 251, 253, 255, 0x101, 0x103, 0x105, 0x107, 0x109, 0x10B,
    0x111, 0x10F, 0x113, 0x10D, 0x115, 0x117, 0x119, 0x11B, 0x11D, 0x11F, 0x121, 0x123, 0x125,
    0x127, 0x129, 0x12B, 0x12D, 0x12F, 0x131, 0x133, 0x135, 0x137, 0x139, 0x13B, 0x13D, 0x141,
    0x143, 0x17D, 0x17B, 0x169, 0x179, 0x16B, 0x16F, 0x16D, 0x177, 0x17F, 0x171, 0x173, 0x175,
    0x181, 0x187, 0x183, 0x189, 0x18B, 0x185, 0x195, 0x18F, 0x193, 0x191, 0x19B, 0x19D, 0x1A1,
    0x1A5, 0x197, 0x1A3, 0x1A7, 0x1A9, 0x199, 0x19F, 0x145, 0x147, 0x149, 0x14B, 0x14D, 0x14F,
    0x151, 0x153, 0x155, 0x157, 0x159, 0x15B, 0x15D, 0x15F, 0x161, 0x163, 0x13F, 0x165, 0x167,
];

/// The starting values of `kind_sets_second`.
#[rustfmt::skip]
const KIND_SETS_SECOND: [u32; 102] = [
    1, 3, 0xFFFFFFFF, 5, 7, 0xFFFFFFFF, 9, 11, 0xFFFFFFFF, 13, 15, 0xFFFFFFFF, 17, 19, 0xFFFFFFFF,
    21, 23, 0xFFFFFFFF, 25, 27, 0xFFFFFFFF, 29, 31, 0xFFFFFFFF, 33, 35, 0xFFFFFFFF, 37, 39,
    0xFFFFFFFF, 41, 43, 0xFFFFFFFF, 45, 47, 0xFFFFFFFF, 49, 51, 0xFFFFFFFF, 53, 55, 0xFFFFFFFF, 57,
    59, 0xFFFFFFFF, 61, 63, 0xFFFFFFFF, 65, 67, 0xFFFFFFFF, 69, 71, 0xFFFFFFFF, 73, 75, 0xFFFFFFFF,
    77, 79, 0xFFFFFFFF, 81, 83, 0xFFFFFFFF, 85, 87, 0xFFFFFFFF, 89, 91, 0xFFFFFFFF, 93, 95,
    0xFFFFFFFF, 97, 99, 0xFFFFFFFF, 101, 103, 0xFFFFFFFF, 105, 107, 0xFFFFFFFF, 109, 111,
    0xFFFFFFFF, 113, 115, 0xFFFFFFFF, 117, 119, 0xFFFFFFFF, 121, 123, 0xFFFFFFFF, 125, 127,
    0xFFFFFFFF, 129, 131, 0xFFFFFFFF, 133, 135, 0xFFFFFFFF,
];

/// The starting values of `second_set_a`.
#[rustfmt::skip]
const SECOND_SET_A: [u32; 23] = [
    137, 141, 143, 147, 151, 153, 157, 161, 163, 167, 171, 173, 177, 181, 183, 187, 191, 193, 197,
    201, 203, 207, 0xFFFFFFFF,
];

/// The starting values of `second_set_b`.
#[rustfmt::skip]
const SECOND_SET_B: [u32; 95] = [
    246, 244, 242, 240, 238, 248, 250, 252, 254, 0x100, 0x102, 0x104, 0x106, 0x108, 0x10A, 0x10C,
    0x112, 0x110, 0x114, 0x10E, 0x116, 0x118, 0x11A, 0x11C, 0x11E, 0x120, 0x122, 0x124, 0x126,
    0x128, 0x12A, 0x12C, 0x12E, 0x130, 0x132, 0x134, 0x136, 0x138, 0x13A, 0x13C, 0x13E, 0x142,
    0x144, 0x17E, 0x17C, 0x16A, 0x17A, 0x16C, 0x170, 0x16E, 0x178, 0x180, 0x172, 0x174, 0x176,
    0x182, 0x188, 0x184, 0x18A, 0x18C, 0x186, 0x196, 0x190, 0x194, 0x192, 0x19C, 0x19E, 0x1A2,
    0x1A6, 0x198, 0x1A4, 0x1A8, 0x1AA, 0x19A, 0x1A0, 0x146, 0x148, 0x14A, 0x14C, 0x14E, 0x150,
    0x152, 0x154, 0x156, 0x158, 0x15A, 0x15C, 0x15E, 0x160, 0x162, 0x164, 0x140, 0x166, 0x168,
    0xFFFFFFFF,
];

/// The starting values of `last_set`.
#[rustfmt::skip]
const LAST_SET: [u32; 31] = [
    138, 139, 144, 145, 148, 149, 154, 155, 158, 159, 164, 165, 168, 169, 174, 175, 178, 179, 184,
    185, 188, 189, 194, 195, 198, 199, 204, 205, 208, 209, 0xFFFFFFFF,
];

/// The starting values of `first_set_b`.
#[rustfmt::skip]
const FIRST_SET_B: [u32; 54] = [
    210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 229, 230,
    231, 228, 227, 232, 233, 234, 235, 236, 0x1B0, 0x1B1, 0x1B2, 0x1B3, 0x1B4, 0x1B5, 0x1B6, 0x1B7,
    0x1B8, 0x1B9, 0x1BA, 0x1BB, 0x1BC, 0x1BD, 0x1BE, 0x1BF, 0x1C0, 0x1C1, 0x1C2, 0x1C3, 0x1AB,
    0x1AC, 0x1AD, 0x1C4, 0x1C5, 0x1C6, 0xFFFFFFFF,
];

/// The starting values of `replay_set`.
#[rustfmt::skip]
const REPLAY_SET: [u32; 8] = [
    0x1AF, 0x1AE, 0x1BB, 0x1BC, 0x1B2, 0x1B7, 0x1B8, 0xFFFFFFFF,
];

/// The starting values of `kinds`.
#[rustfmt::skip]
const RACE_KINDS: [u32; 87] = [
    race_list(RaceField::KindSetsFirst, 0), race_list(RaceField::KindSetsSecond, 0),
    race_list(RaceField::KindSetsFirst, 3), race_list(RaceField::KindSetsSecond, 3),
    race_list(RaceField::KindSetsFirst, 6), race_list(RaceField::KindSetsSecond, 6),
    race_list(RaceField::KindSetsFirst, 9), race_list(RaceField::KindSetsSecond, 9),
    race_list(RaceField::KindSetsFirst, 12), race_list(RaceField::KindSetsSecond, 12),
    race_list(RaceField::KindSetsFirst, 15), race_list(RaceField::KindSetsSecond, 15),
    race_list(RaceField::KindSetsFirst, 18), race_list(RaceField::KindSetsSecond, 18),
    race_list(RaceField::KindSetsFirst, 21), race_list(RaceField::KindSetsSecond, 21),
    race_list(RaceField::KindSetsFirst, 24), race_list(RaceField::KindSetsSecond, 24),
    race_list(RaceField::KindSetsFirst, 27), race_list(RaceField::KindSetsSecond, 27),
    race_list(RaceField::KindSetsFirst, 30), race_list(RaceField::KindSetsSecond, 30),
    race_list(RaceField::KindSetsFirst, 33), race_list(RaceField::KindSetsSecond, 33),
    race_list(RaceField::KindSetsFirst, 36), race_list(RaceField::KindSetsSecond, 36),
    race_list(RaceField::KindSetsFirst, 39), race_list(RaceField::KindSetsSecond, 39),
    race_list(RaceField::KindSetsFirst, 42), race_list(RaceField::KindSetsSecond, 42),
    race_list(RaceField::KindSetsFirst, 45), race_list(RaceField::KindSetsSecond, 45),
    race_list(RaceField::KindSetsFirst, 48), race_list(RaceField::KindSetsSecond, 48),
    race_list(RaceField::KindSetsFirst, 51), race_list(RaceField::KindSetsSecond, 51),
    race_list(RaceField::KindSetsFirst, 54), race_list(RaceField::KindSetsSecond, 54),
    race_list(RaceField::KindSetsFirst, 57), race_list(RaceField::KindSetsSecond, 57),
    race_list(RaceField::KindSetsFirst, 60), race_list(RaceField::KindSetsSecond, 60),
    race_list(RaceField::KindSetsFirst, 63), race_list(RaceField::KindSetsSecond, 63),
    race_list(RaceField::KindSetsFirst, 66), race_list(RaceField::KindSetsSecond, 66),
    race_list(RaceField::KindSetsFirst, 69), race_list(RaceField::KindSetsSecond, 69),
    race_list(RaceField::KindSetsFirst, 72), race_list(RaceField::KindSetsSecond, 72),
    race_list(RaceField::KindSetsFirst, 75), race_list(RaceField::KindSetsSecond, 75),
    race_list(RaceField::KindSetsFirst, 78), race_list(RaceField::KindSetsSecond, 78),
    race_list(RaceField::KindSetsFirst, 81), race_list(RaceField::KindSetsSecond, 81),
    race_list(RaceField::KindSetsFirst, 84), race_list(RaceField::KindSetsSecond, 84),
    race_list(RaceField::KindSetsFirst, 87), race_list(RaceField::KindSetsSecond, 87),
    race_list(RaceField::KindSetsFirst, 90), race_list(RaceField::KindSetsSecond, 90),
    race_list(RaceField::KindSetsFirst, 93), race_list(RaceField::KindSetsSecond, 93),
    race_list(RaceField::KindSetsFirst, 96), race_list(RaceField::KindSetsSecond, 96),
    race_list(RaceField::KindSetsFirst, 99), race_list(RaceField::KindSetsSecond, 99),
    race_list(RaceField::KindSetsFirst, 51), race_list(RaceField::KindSetsSecond, 51),
    race_list(RaceField::KindSetsFirst, 51), race_list(RaceField::KindSetsSecond, 51), 0, 16, 0, 0,
    4, 0, 0, 2, 0, 0, 2, 0, 0, 4, 0,
];

impl SoundSets {
    pub const fn new() -> Self {
        SoundSets {
            kind_sets_first: Dword::array(KIND_SETS_FIRST),
            first_set_a: Dword::array(FIRST_SET_A),
            first_set_c: Dword::array(FIRST_SET_C),
            kind_sets_second: Dword::array(KIND_SETS_SECOND),
            second_set_a: Dword::array(SECOND_SET_A),
            second_set_b: Dword::array(SECOND_SET_B),
            last_set: Dword::array(LAST_SET),
            first_set_b: Dword::array(FIRST_SET_B),
            replay_set: Dword::array(REPLAY_SET),
            kinds: Dword::array(RACE_KINDS),
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

impl SoundSets {
    /// The sets of the kind-table entry `n` (two a kind of car), as a handle.
    #[cfg(test)]
    pub fn kind_list_of(
        n: usize,
    ) -> Result<rc2k_core::audio::commands::SoundList, crate::errors::BadInput> {
        crate::test_support::own::sound().threads.sets.kind_list(n)
    }

    /// The sets of the kind-table entry `n` (two a kind of car), as a handle.
    pub(super) fn kind_list(
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
        use rc2k_core::audio::commands::ListPlace;
        match field.list_place(entry, n) {
            ListPlace::At {
                field: RaceField::Tail,
                ..
            } => Ok(Some(0)),
            ListPlace::At { field, index } => {
                let run = self.runs()[field as usize];
                debug_assert_eq!(run.count(), field.dwords());
                Ok(Some(run.bits(index)))
            }
            ListPlace::PastSets => Ok(None),
            ListPlace::StartsPastSets => {
                Err(crate::errors::BadInput("a sound list off the race's sets"))
            }
        }
    }

    /// The stored fields in the read-on order, each a run of dwords (`RaceField::Tail` holds
    /// nothing: it reads as 0).
    fn runs(&self) -> [&dyn crate::memory::session::Run; 10] {
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
        ]
    }

    const KIND_ENTRIES: usize = 72;
}

/// The cues' lists.
/// Never written after the game starts.
pub struct CueLists {
    /// Where each cue's list of sounds starts, as a byte offset in `lists`
    /// (`physics::queue_sound_cue`).
    pub table: [u32; 113],
    /// The lists.
    pub lists: [u8; 836],
}

/// The starting values of `table`.
#[rustfmt::skip]
const CUE_STARTS: [u32; 113] = [
    0, 4, 12, 20, 28, 36, 44, 52, 60, 68, 80, 92, 104, 116, 0, 0, 0, 128, 136, 144, 152, 160, 168,
    176, 184, 192, 204, 216, 228, 240, 252, 0x104, 0x10c, 0x114, 0x11c, 0x124, 0x12c, 0x134, 0x13c,
    0x144, 0x14c, 0x154, 0, 0, 0, 0, 0, 0x15c, 0x16c, 0x174, 0x17c, 0x184, 0x18c, 0x194, 0x19c,
    0x1a4, 0x1fc, 0x204, 0x20c, 0x214, 0x21c, 0x224, 0x22c, 0x234, 0x1ac, 0x1bc, 0x1b4, 0x1c4,
    0x1cc, 0x1d4, 0x1dc, 0x1e4, 0x1ec, 0x1f4, 0, 0, 0, 0, 0, 0, 0x23c, 0x24c, 0x244, 0x254, 0x25c,
    0x264, 0x26c, 0x274, 0x27c, 0x284, 0x28c, 0x294, 0x29c, 0x2a4, 0x2ac, 0x2b4, 0x2bc, 0x2c4,
    0x2cc, 0x2d4, 0x2dc, 0x2e4, 0x2ec, 0x2f4, 0x2fc, 0x304, 0x30c, 0x314, 0x31c, 0x324, 0x32c,
    0x334, 0x33c,
];

impl CueLists {
    pub const fn new() -> Self {
        CueLists {
    table: CUE_STARTS,
    lists: with_runs::<836>(&[(0, b"\xff\xff\xff\xffE\x01\x00\x00\xff\xff\xff\xffG\x01\x00\x00\xff\xff\xff\xffI\x01\x00\x00\xff\xff\xff\xffK\x01\x00\x00\xff\xff\xff\xffM\x01\x00\x00\xff\xff\xff\xffO\x01\x00\x00\xff\xff\xff\xffQ\x01\x00\x00\xff\xff\xff\xffS\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00E\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00G\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00I\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00K\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00M\x01\x00\x00\xff\xff\xff\xffU\x01\x00\x00\xff\xff\xff\xffW\x01\x00\x00\xff\xff\xff\xffY\x01\x00\x00\xff\xff\xff\xff[\x01\x00\x00\xff\xff\xff\xff]\x01\x00\x00\xff\xff\xff\xff_\x01\x00\x00\xff\xff\xff\xffa\x01\x00\x00\xff\xff\xff\xffc\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00U\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00W\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00Y\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00[\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00]\x01\x00\x00\xff\xff\xff\xffe\x01\x00\x00\xff\xff\xff\xffg\x01\x00\x00\xff\xff\xff\xff\xf9\x00\x00\x00\xff\xff\xff\xff\xfb\x00\x00\x00\xff\xff\xff\xff\xfd\x00\x00\x00\xff\xff\xff\xff\xff\x00\x00\x00\xff\xff\xff\xff\x01\x01\x00\x00\xff\xff\xff\xff\x03\x01\x00\x00\xff\xff\xff\xff\x05\x01\x00\x00\xff\xff\xff\xff\x07\x01\x00\x00\xff\xff\xff\xff\t\x01\x00\x00\xff\xff\xff\xff\x0b\x01\x00\x00\xff\xff\xff\xff1\x01\x00\x00!\x01\x00\x00?\x01\x00\x00\xff\xff\xff\xff\x11\x01\x00\x00\xff\xff\xff\xff\x0f\x01\x00\x00\xff\xff\xff\xff\x13\x01\x00\x00\xff\xff\xff\xff\r\x01\x00\x00\xff\xff\xff\xff1\x01\x00\x00\xff\xff\xff\xffA\x01\x00\x00\xff\xff\xff\xff=\x01\x00\x00\xff\xff\xff\xff3\x01\x00\x00\xff\xff\xff\xff\x1b\x01\x00\x00\xff\xff\xff\xff)\x01\x00\x00\xff\xff\xff\xff\x1d\x01\x00\x00\xff\xff\xff\xff#\x01\x00\x00\xff\xff\xff\xff+\x01\x00\x00\xff\xff\xff\xffC\x01\x00\x00\xff\xff\xff\xff\x15\x01\x00\x00\xff\xff\xff\xff\x17\x01\x00\x00\xff\xff\xff\xff\x1f\x01\x00\x00\xff\xff\xff\xff%\x01\x00\x00\xff\xff\xff\xff\x19\x01\x00\x00\xff\xff\xff\xff'\x01\x00\x00\xff\xff\xff\xff-\x01\x00\x00\xff\xff\xff\xff/\x01\x00\x00\xff\xff\xff\xff5\x01\x00\x00\xff\xff\xff\xff7\x01\x00\x00\xff\xff\xff\xff9\x01\x00\x00\xff\xff\xff\xff;\x01\x00\x00\xff\xff\xff\xff}\x01\x00\x00\xff\xff\xff\xff\x81\x01\x00\x00\xff\xff\xff\xff\x7f\x01\x00\x00\xff\xff\xff\xffs\x01\x00\x00\xff\xff\xff\xffi\x01\x00\x00\xff\xff\xff\xffk\x01\x00\x00\xff\xff\xff\xffm\x01\x00\x00\xff\xff\xff\xffo\x01\x00\x00\xff\xff\xff\xffq\x01\x00\x00\xff\xff\xff\xffu\x01\x00\x00\xff\xff\xff\xffw\x01\x00\x00\xff\xff\xff\xffy\x01\x00\x00\xff\xff\xff\xff{\x01\x00\x00\xff\xff\xff\xff\x83\x01\x00\x00\xff\xff\xff\xff\x85\x01\x00\x00\xff\xff\xff\xff\x87\x01\x00\x00\xff\xff\xff\xff\x89\x01\x00\x00\xff\xff\xff\xff\x8b\x01\x00\x00\xff\xff\xff\xff\x8d\x01\x00\x00\xff\xff\xff\xff\x8f\x01\x00\x00\xff\xff\xff\xff\x91\x01\x00\x00\xff\xff\xff\xff\x93\x01\x00\x00\xff\xff\xff\xff\x95\x01\x00\x00\xff\xff\xff\xff\x97\x01\x00\x00\xff\xff\xff\xff\x99\x01\x00\x00\xff\xff\xff\xff\x9b\x01\x00\x00\xff\xff\xff\xff\x9d\x01\x00\x00\xff\xff\xff\xff\x9f\x01\x00\x00\xff\xff\xff\xff\xa1\x01\x00\x00\xff\xff\xff\xff\xa3\x01\x00\x00\xff\xff\xff\xff\xa5\x01\x00\x00\xff\xff\xff\xff\xa7\x01\x00\x00\xff\xff\xff\xff\xa9\x01\x00\x00\xff\xff\xff\xff")]),
        }
    }
}

/// A list of dwords inside a block of bytes, read by position and bounded to the block.
pub struct ListIn<'a, const N: usize> {
    bytes: &'a [u8; N],
    from: usize,
}

impl<const N: usize> ListIn<'_, N> {
    /// Entry `k`; refused past the block (a list there has no end).
    pub fn get(&self, k: usize) -> Result<u32, crate::errors::BadInput> {
        k.checked_mul(4)
            .and_then(|k| k.checked_add(self.from))
            .filter(|at| at + 4 <= N)
            .map(|at| {
                u32::from_le_bytes([
                    self.bytes[at],
                    self.bytes[at + 1],
                    self.bytes[at + 2],
                    self.bytes[at + 3],
                ])
            })
            .ok_or(crate::errors::BadInput(
                "a sound list read past the lists that hold it",
            ))
    }
}

impl CueLists {
    /// Cue `cue`'s list of sounds, in the lists after the table (each ends in -1 inside them).
    pub fn list(&self, cue: u32) -> Result<ListIn<'_, 836>, crate::errors::BadInput> {
        let lists = self;
        let start = lists
            .table
            .get(cue as usize)
            .ok_or(crate::errors::BadInput("a sound cue past the table"))?;
        Ok(ListIn {
            bytes: &lists.lists,
            from: *start as usize,
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
#[derive(Clone, Copy)]
pub struct SoundSet {
    /// The sound's number, added to the object's `sound_offset`.
    pub sound: u32,
    /// Read by no sound code and 0 in every set of the shipped data, so not converted to a
    /// float (a spare word, probably an input range's start).
    pub unused_float_04: u32,
    /// As `unused_float_04` (probably the range's end).
    pub unused_float_08: u32,
    /// As `unused_float_04` (probably the range's span).
    pub unused_float_0c: u32,
    /// The pitch at input 0.
    pub pitch_low: u32,
    /// The pitch's end (the span's second value).
    pub pitch_high: u32,
    /// The pitch span, `pitch_high - pitch_low`: the pitch's slope.
    pub pitch_scale: u32,
    /// The volume at input 0.
    pub volume_low: u32,
    /// The volume's end.
    pub volume_high: u32,
    /// The volume span: the volume's slope.
    pub volume_scale: u32,
}

const SOUND_SET_LEN: usize = 0x28;

impl SoundSet {
    /// The set in `b` from byte `at` (each dword little-endian).
    const fn from_slice(b: &[u8], at: usize) -> Self {
        const fn word(b: &[u8], at: usize) -> u32 {
            u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
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

    /// The set in `d` from dword `at`.
    const fn from_words(d: &[u32], at: usize) -> Self {
        Self {
            sound: d[at],
            unused_float_04: d[at + 1],
            unused_float_08: d[at + 2],
            unused_float_0c: d[at + 3],
            pitch_low: d[at + 4],
            pitch_high: d[at + 5],
            pitch_scale: d[at + 6],
            volume_low: d[at + 7],
            volume_high: d[at + 8],
            volume_scale: d[at + 9],
        }
    }

    /// A set of zeros with the byte runs `runs` ((offset, bytes), as `with_runs`).
    const fn from_runs(runs: &[(usize, &[u8])]) -> Self {
        Self::from_slice(&with_runs::<SOUND_SET_LEN>(runs), 0)
    }

    /// The ten dwords in order (the starting bytes' checksum test).
    #[cfg(test)]
    fn words(&self) -> [u32; 10] {
        [
            self.sound,
            self.unused_float_04,
            self.unused_float_08,
            self.unused_float_0c,
            self.pitch_low,
            self.pitch_high,
            self.pitch_scale,
            self.volume_low,
            self.volume_high,
            self.volume_scale,
        ]
    }
}

/// A sound kind's record (0x88 bytes): the revs' range and slope, then the engine's three sets.
/// Laid out as the 1999 variables: the recorded assembly's tests lay it over raw bytes.
#[cfg_attr(test, repr(C))]
#[derive(Clone, Copy)]
pub struct KindRecord {
    /// The kind's flags (`sound::car_engine_state`'s `state` reads them).
    pub flags: u32,
    /// The revs the engine's input starts from (a whole number made a float).
    pub revs_low: u32,
    /// The revs' end.
    pub revs_high: u32,
    /// First the span (`revs_high - revs_low`), then its reciprocal (`race::sound_tables`): the
    /// engine input's slope.
    pub revs_scale: u32,
    /// The engine's revs sound (`sound::revs_sound`).
    pub revs: SoundSet,
    /// The throttle sound (`sound::throttle_sound`).
    pub throttle: SoundSet,
    /// The speed sound (`sound::speed_sound`).
    pub speed: SoundSet,
}

impl KindRecord {
    /// A record from its 34 dwords.
    const fn from_words(d: &[u32; 34]) -> Self {
        Self {
            flags: d[0],
            revs_low: d[1],
            revs_high: d[2],
            revs_scale: d[3],
            revs: SoundSet::from_words(d, 4),
            throttle: SoundSet::from_words(d, 14),
            speed: SoundSet::from_words(d, 24),
        }
    }

    /// The records of `rows`, in order.
    const fn rows(rows: &[[u32; 34]; 34]) -> [Self; 34] {
        let mut records: [Self; 34] = [const { Self::from_words(&[0; 34]) }; 34];
        let mut n = 0;
        while n < 34 {
            records[n] = Self::from_words(&rows[n]);
            n += 1;
        }
        records
    }

    /// The 34 dwords in order (the starting bytes' checksum test).
    #[cfg(test)]
    fn words(&self) -> Vec<u32> {
        let mut words = vec![self.flags, self.revs_low, self.revs_high, self.revs_scale];
        for set in [&self.revs, &self.throttle, &self.speed] {
            words.extend(set.words());
        }
        words
    }
}

/// The 34 kind records' starting values: 34 dwords each, in the order of `KindRecord`'s fields
/// (the three sets' ten dwords after the first four).
#[rustfmt::skip]
const KIND_ROWS: [[u32; 34]; 34] = [
    [
        0x2000, 0x7d0, 0x1388, 0, 0, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 2, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x2000, 0x7d0, 0x1388, 0, 4, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 6, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10ec, 0x7d0, 0x1194, 0, 8, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 10, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x20ec, 0x7d0, 0x1194, 0, 12, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 14, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10eb, 0x7d0, 0x1194, 0, 16, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 18, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 20, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 22, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10eb, 0x7d0, 0x1194, 0, 24, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 26, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10ec, 0x7d0, 0x1194, 0, 28, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 30, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x2000, 0x7d0, 0x1388, 0, 32, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 34, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x2000, 0x7d0, 0x1388, 0, 36, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 38, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10ea, 0x7d0, 0x1194, 0, 40, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 42, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 44, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 46, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 48, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 50, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 52, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 54, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10ec, 0x7d0, 0x1194, 0, 56, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 58, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 60, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 62, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x2000, 0x7d0, 0x1388, 0, 64, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 66, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x2000, 0x7d0, 0x1388, 0, 68, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 70, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 72, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 74, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 76, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 78, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x2000, 0x7d0, 0x1388, 0, 80, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 82, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 84, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 86, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10ec, 0x7d0, 0x1194, 0, 88, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 90, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10ec, 0x7d0, 0x1194, 0, 92, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 94, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10eb, 0x7d0, 0x1194, 0, 96, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 98, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 100, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 102, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 104, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 106, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10ec, 0x7d0, 0x1194, 0, 108, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 110, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x10ea, 0x7d0, 0x1194, 0, 112, 0, 0, 0, 0x3a98, 0x80e8, 0, 14, 32, 0, 114, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x2000, 0x7d0, 0xfa0, 0, 116, 0, 0, 0, 0x55f0, 0xabe0, 0xe678, 0, 14, 32, 118, 0, 0, 0,
        0x3a98, 0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0x2000, 0xbb8, 0xfa0, 0, 120, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 122, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 124, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 126, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0xfa0, 0, 128, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 130, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
    [
        0, 0x7d0, 0x1388, 0, 132, 0, 0, 0, 0x2328, 0xe678, 0, 14, 32, 0, 134, 0, 0, 0, 0x3a98,
        0x124f8, 0, 4, 32, 0, 136, 0, 0, 0, 0x4650, 0xbb80, 0, 4, 20, 0,
    ],
];

/// The car sound kinds' records and the computer cars' set.
pub struct SoundKinds {
    /// Each sound kind's record, by its number (`sound::car_engine_state`).
    pub kinds: [u32; 36],
    /// The sound kinds' records (`race::sound_tables` works the floats out once, when the race's
    /// sound objects are first set up); the game thread's, borrowed for a statement at a time.
    pub records: std::cell::RefCell<[KindRecord; KIND_RECORDS]>,
    /// The computer's cars' set (`sound::revs_sound`).
    pub computers: SoundSet,
}

/// The number of kind records.
const KIND_RECORDS: usize = 34;

/// The starting values of `kinds`.
#[rustfmt::skip]
const KIND_NUMBERS: [u32; 36] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 17, 17,
];

impl SoundKinds {
    pub const fn new() -> Self {
        SoundKinds {
            kinds: KIND_NUMBERS,
            records: std::cell::RefCell::new(KindRecord::rows(&KIND_ROWS)),
            computers: SoundSet::from_runs(&[
                (0, b"\x8a"),
                (
                    17,
                    b"`jF\x00|\x92G\x00`jG\x00\x00\x80A\x00\x00\x80B\x00\x00@B",
                ),
            ]),
        }
    }

    /// The place in `records` of kind `n`'s record. The kind past the table, or a table entry
    /// off the records, is refused.
    fn record_place(&self, n: u32) -> Result<usize, crate::errors::BadInput> {
        let number = *self
            .kinds
            .get(n as usize)
            .ok_or(crate::errors::BadInput("a sound kind past the table"))?;
        if number as usize >= KIND_RECORDS {
            return Err(crate::errors::BadInput(
                "a sound table's entry off its piece",
            ));
        }
        Ok(number as usize)
    }

    /// A copy of kind `n`'s record (refused as `record_place` says).
    pub fn record(&self, n: u32) -> Result<KindRecord, crate::errors::BadInput> {
        Ok(self.records.borrow()[self.record_place(n)?])
    }

    /// Kind `n`'s record changed by `change` (refused as `record_place` says, before anything).
    pub fn update_record(
        &self,
        n: u32,
        change: impl FnOnce(&mut KindRecord),
    ) -> Result<(), crate::errors::BadInput> {
        let place = self.record_place(n)?;
        change(&mut self.records.borrow_mut()[place]);
        Ok(())
    }

    /// The computers' cars' set.
    pub(super) fn computers_set(&self) -> SoundSet {
        self.computers
    }
}

/// The surface sound sets.
/// Never written after the game starts.
pub struct SurfaceSets {
    /// The sets (seven pairs; each is the byte offset in `lists` where its set starts;
    /// `sound::surface_sounds`).
    pub sets: [u32; 14],
    /// The lists.
    pub lists: [u8; 392],
}

/// The starting values of `sets`.
#[rustfmt::skip]
const SURFACE_SET_STARTS: [u32; 14] = [
    0, 36, 56, 92, 112, 148, 168, 204, 0x118, 0x13c, 224, 0x104, 0x150, 0x174,
];

impl SurfaceSets {
    pub const fn new() -> Self {
        SurfaceSets {
    sets: SURFACE_SET_STARTS,
    lists: with_runs::<392>(&[(0, b"\x01\x00\x00\x00\x8c\x00\x00\x00\x8d\x00\x00\x00\x90\x00\x00\x00\x91\x00\x00\x00\x8e\x00\x00\x00\x8f\x00\x00\x00\x90\x00\x00\x00\x91\x00\x00\x00\x02\x00\x00\x00\x92\x00\x00\x00\x93\x00\x00\x00\x94\x00\x00\x00\x95\x00\x00\x00\x01\x00\x00\x00\x96\x00\x00\x00\x97\x00\x00\x00\x9a\x00\x00\x00\x9b\x00\x00\x00\x98\x00\x00\x00\x99\x00\x00\x00\x9a\x00\x00\x00\x9b\x00\x00\x00\x02\x00\x00\x00\x9c\x00\x00\x00\x9d\x00\x00\x00\x9e\x00\x00\x00\x9f\x00\x00\x00\x01\x00\x00\x00\xa0\x00\x00\x00\xa1\x00\x00\x00\xa4\x00\x00\x00\xa5\x00\x00\x00\xa2\x00\x00\x00\xa3\x00\x00\x00\xa4\x00\x00\x00\xa5\x00\x00\x00\x02\x00\x00\x00\xa6\x00\x00\x00\xa7\x00\x00\x00\xa8\x00\x00\x00\xa9\x00\x00\x00\x01\x00\x00\x00\xaa\x00\x00\x00\xab\x00\x00\x00\xae\x00\x00\x00\xaf\x00\x00\x00\xac\x00\x00\x00\xad\x00\x00\x00\xae\x00\x00\x00\xaf\x00\x00\x00\x02\x00\x00\x00\xb0\x00\x00\x00\xb1\x00\x00\x00\xb2\x00\x00\x00\xb3\x00\x00\x00\x01\x00\x00\x00\xb4\x00\x00\x00\xb5\x00\x00\x00\xb8\x00\x00\x00\xb9\x00\x00\x00\xb6\x00\x00\x00\xb7\x00\x00\x00\xb8\x00\x00\x00\xb9\x00\x00\x00\x02\x00\x00\x00\xba\x00\x00\x00\xbb\x00\x00\x00\xbc\x00\x00\x00\xbd\x00\x00\x00\x01\x00\x00\x00\xbe\x00\x00\x00\xbf\x00\x00\x00\xc2\x00\x00\x00\xc3\x00\x00\x00\xc0\x00\x00\x00\xc1\x00\x00\x00\xc2\x00\x00\x00\xc3\x00\x00\x00\x02\x00\x00\x00\xc4\x00\x00\x00\xc5\x00\x00\x00\xc6\x00\x00\x00\xc7\x00\x00\x00\x01\x00\x00\x00\xc8\x00\x00\x00\xc9\x00\x00\x00\xcc\x00\x00\x00\xcd\x00\x00\x00\xca\x00\x00\x00\xcb\x00\x00\x00\xcc\x00\x00\x00\xcd\x00\x00\x00\x02\x00\x00\x00\xce\x00\x00\x00\xcf\x00\x00\x00\xd0\x00\x00\x00\xd1")]),
        }
    }
}

impl SurfaceSets {
    /// Surface sound set `n`'s list: the flag, then its sounds (nine dwords for a set of eight,
    /// five for a set of four; the sets follow one another in the lists).
    pub fn set(&self, n: usize) -> Result<ListIn<'_, 392>, crate::errors::BadInput> {
        let sets = self;
        let start = sets.sets.get(n).ok_or(crate::errors::BadInput(
            "a surface sound set past the table",
        ))?;
        Ok(ListIn {
            bytes: &sets.lists,
            from: *start as usize,
        })
    }
}

/// The spot voices.
pub struct SpotVoices {
    /// 0x48 records (`sound::spot_voices`), the game thread's, borrowed for a statement at a time.
    pub voices: std::cell::RefCell<[crate::sound::spot_voices::SpotVoice; 72]>,
}

impl SpotVoices {
    pub const fn new() -> Self {
        SpotVoices {
            voices: std::cell::RefCell::new(
                [const { crate::sound::spot_voices::SpotVoice::free() }; 72],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every starting table's whole bytes, as a checksum: a wrong dword in a row shows here.
    #[test]
    fn the_sound_tables_keep_the_bytes_of_the_image() {
        use crate::memory::cells::flatten;
        let fnv = |bytes: Vec<u8>| {
            let mut hash = 0xcbf2_9ce4_8422_2325_u64;
            for byte in &bytes {
                hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
            }
            (bytes.len(), hash)
        };
        assert_eq!(
            fnv(MenuSounds::new()
                .list
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect()),
            (40, 0xea1e_8bf9_7ac9_6646)
        );
        assert_eq!(
            fnv(flatten(&SoundSets::new())),
            (2476, 0xd0a0_f5d7_dfd4_e733)
        );
        let cues = CueLists::new();
        let mut cue_bytes = vec![0; 8];
        cue_bytes.extend(cues.table.iter().flat_map(|w| w.to_le_bytes()));
        cue_bytes.extend(cues.lists);
        assert_eq!(fnv(cue_bytes), (1296, 0x801b_361e_3c58_d21a));
        let kinds = SoundKinds::new();
        let kind_words: Vec<u32> = kinds
            .kinds
            .iter()
            .copied()
            .chain(kinds.records.borrow().iter().flat_map(KindRecord::words))
            .chain(kinds.computers.words())
            .chain([0; 66])
            .collect();
        assert_eq!(
            fnv(kind_words.iter().flat_map(|w| w.to_le_bytes()).collect()),
            (5072, 0x318f_fbc9_6e4c_8c24)
        );
        let surfaces = SurfaceSets::new();
        let mut surface_bytes = vec![0; 12];
        surface_bytes.extend(surfaces.sets.iter().flat_map(|w| w.to_le_bytes()));
        surface_bytes.extend(surfaces.lists);
        surface_bytes.extend([0; 4]);
        assert_eq!(fnv(surface_bytes), (464, 0x82c0_ea45_99e5_7e4d));
        let voices = SpotVoices::new();
        let voice_bytes: Vec<u8> = voices
            .voices
            .borrow()
            .iter()
            .flat_map(|v| v.to_words())
            .flat_map(u32::to_le_bytes)
            .collect();
        assert_eq!(fnv(voice_bytes), (5760, 0xd258_8b64_db91_c525));
        let request = SoundRequest::new();
        let request_words = [
            request.sound,
            request.pitch,
            request.volume,
            request.place[0].to_bits(),
            request.place[1].to_bits(),
            request.place[2].to_bits(),
            request.angle,
            request.pan,
            request.pitch_factor.to_bits(),
            request.volume_factor.to_bits(),
        ];
        assert_eq!(
            fnv(request_words.iter().flat_map(|w| w.to_le_bytes()).collect()),
            (40, 0x40d6_9e0c_f0f6_5c45)
        );
    }

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
    fn flat(sets: &SoundSets) -> Vec<u32> {
        sets.runs()
            .iter()
            .flat_map(|run| (0..run.count()).map(|i| run.bits(i)))
            .chain([0])
            .collect()
    }

    /// Every entry of every list start, to the end of the sets and past it, reads as a read of
    /// the 1999 layout's bytes does.
    #[test]
    fn list_entries_read_as_the_byte_read() {
        let sets = Box::new(SoundSets::new());
        for (k, run) in sets.runs().iter().enumerate() {
            for i in 0..run.count() {
                run.set_bits(i, 0x7000_0000 + (OLD_FIELD_OFFSETS[k] / 4) + i as u32);
            }
        }
        let filled = flat(&sets);
        for field in RaceField::ALL {
            for entry in 0..field.dwords() as u32 {
                let offset = OLD_FIELD_OFFSETS[field as usize] + 4 * entry;
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
        use rc2k_core::audio::commands::{Command, SoundList};
        let sets = &crate::test_support::own::sound().threads.sets;
        let flat = flat(sets);
        let kinds_start = OLD_FIELD_OFFSETS[RaceField::Kinds as usize] as usize / 4;
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
            assert_eq!(
                Command::Load(Some(list)).record_words().1[0],
                old.unwrap() + 2,
                "{list:?}"
            );
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
            let handle = crate::test_support::own::sound().threads.sets.kinds[n].get();
            let at = SoundSets::kind_list_of(n).unwrap();
            let (field, entry) = race_list_of(handle).unwrap();
            assert_eq!(
                at,
                rc2k_core::audio::commands::SoundList::Race { field, entry }
            );
        }
        assert!(SoundSets::kind_list_of(72).is_err());
    }

    #[test]
    fn cuelists_end_in_minus_one() {
        // Every cue's list is one of the lists, and ends.
        for cue in 0..113 {
            let list = crate::test_support::own::sound()
                .race
                .cue_lists
                .get()
                .list(cue)
                .unwrap();
            // The list ends in -1 inside the lists.
            assert!((0..6).any(|k| list.get(k) == Ok(u32::MAX)), "{cue}");
        }
        assert!(
            crate::test_support::own::sound()
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
            let set = crate::test_support::own::sound()
                .race
                .surface_sets
                .get()
                .set(n)
                .unwrap();
            // A set of eight (flag 1) holds nine dwords, a set of four five, inside the lists.
            let dwords = if set.get(0).unwrap() == 1 { 9 } else { 5 };
            assert!(set.get(dwords - 1).is_ok(), "{n}");
        }
        let kinds = crate::test_support::own::sound().race.kinds.get();
        for n in 0..36 {
            kinds.record(n).unwrap();
        }
    }
}
