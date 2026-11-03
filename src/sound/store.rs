//! The sound module's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`),
//! typed (phase 7): each is a `#[repr(C)]` struct of atomic cells with the image's layout
//! (the first values the image's, `tests::*_start_as_the_games`), one static a piece, and
//! `moved::MOVED` points the image's addresses at it:
//! - `WHEEL_SURFACES` (IDA 0x4F66F1 .. 0x4F66F9): the two wheel surfaces the sound reads;
//! - `RaceClock` (0x5124E4 .. 0x512508): the race sounds' clock and constants;
//! - `RaceSounds` (0x512A10 .. 0x514F90): the eight sound objects (`SoundObject`, 0x4A8 bytes
//!   each, with the table of their addresses) and the cars the sound follows. The table
//!   names the objects as the image holds them (IDA's addresses): `point_sound_objects`
//!   sets it to where they are now (the standalone's start-up calls it);
//! - `SurfaceSounds` (0x516358 .. 0x51639C): the factors the engine, speed, revs and
//!   throttle sounds scale by, the wheel sounds across and along, the surface sound volume;
//! - `StageSounds` (0x51655C .. 0x5168C0): the surface sound sets, the stage's one-off,
//!   timed, loop and random sounds, and the spots shared with the sound side;
//! - `SpotPicks` (0x517F40 .. 0x518080): the spot-voice picks and the sound kinds per spot;
//!   `SettingsApplied` (0x518080); `SoundOptions` (0x5180F0 .. 0x518104): the kept options
//!   (`g_tuneTurn`, `g_musicVolume` ..); `CdAudio` (0x518104 .. 0x518120). Between them in the
//!   image, the force-feedback effects' state is `input::store::ForceFeedback`'s (split
//!   2026-10-25).
//!
//! The sound commands go to the audio thread through `audio::store`'s queue; these are the
//! game thread's.

use crate::session::{Dword, Float, Word};

/// `g_wheelSurfaceA`, `g_wheelSurfaceB` (IDA 0x4F66F1, 0x4F66F5): the surfaces of the wheels
/// the first and the second of a car's two sounds follow (`sound::surface_sounds`). Split
/// 2026-10-25 from the dwords around them: the save file's four before
/// (`saves::load_options::KEPT`), the race record's seal key after (`race::replay::SEAL_KEY`).
pub static WHEEL_SURFACES: [Dword; 2] = [Dword::new(0), Dword::new(0)];

/// One cue of a car's queue: eight dwords.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CueEntry {
    /// The cue's first sound.
    pub first: Dword,
    /// Its sounds.
    pub count: Dword,
    /// The sounds.
    pub sounds: [Dword; 6],
}

/// A car's queue of spoken cues, 0x210 bytes.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CueQueue {
    /// The first cue queued.
    pub first: Dword,
    /// The place in the first cue (-1: none begun).
    pub place: Dword,
    /// When the next sound is due.
    pub due: Dword,
    /// The cues queued.
    pub count: Dword,
    /// The cues.
    pub cues: [CueEntry; 16],
}

/// One of a car's four wheel records in its sound object: five dwords.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct EngineVoice {
    /// +0: a share the physics sets (`physics::car_sound_state`: +0x294 of the work car, or 0); the first wheel sum (`int` +0x284 of the object) scales these by 0.4.
    pub share_a: Float,
    /// +4: nothing reads or writes it.
    pub _gap_4: Dword,
    /// +8: 1.0, or the share +0x28C (`sound::surface_sounds` reads these: front less back and left less right, and their sum x 64).
    pub share_b: Float,
    /// +0xC: the share x 256.
    pub share_c: Float,
    /// +0x10: the surface the wheel runs on (`g_car` +0x930).
    pub surface: Dword,
}

/// A race sound object, 0x4A8 bytes: a car's sound state (the revs, the throttle, the place, the timed sound slots) as `sound::car_sounds` and the physics keep it. Offsets in the docs are the image's (+0x..); fields not worked out are gaps.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SoundObject {
    /// +0: added to a sound set's first sound to pick the car's sound; negative for a computer's car (a reading).
    pub sound_offset: Dword,
    /// +4: the car's sound kind, an index into `off_514F90` (36 of 0x88 bytes).
    pub kind: Dword,
    /// +8: 1 when the object is in use (`sound::car_sounds_all`).
    pub in_use: Dword,
    /// +0xC: `g_car` +0x6F0 xor 1; the engine, throttle and speed sounds' volume is scaled by a constant when it is 0 (a reading).
    pub view_flag: Dword,
    /// +0x10: 1 when a new revs reading (+0x14) came (`sound::car_engine_state`).
    pub new_revs: Dword,
    /// +0x14: the revs (extrapolated between the physics' readings).
    pub revs: Dword,
    /// +0x18: the throttle, 0 ..= 0x100.
    pub throttle: Dword,
    /// +0x1C: the car's speed.
    pub speed: Float,
    /// +0x20: set by the physics when the gear changed (cleared by `sound::car_engine_state`).
    pub gear_event: Dword,
    /// +0x24: 1 when the car is in no gear (a reading).
    pub no_gear: Dword,
    /// +0x28: not worked out (1 dwords).
    pub _gap_28: [Dword; 1],
    /// +0x2C: the car's place less the listener's, turned by the view and scaled (`physics::car_sound_state`).
    pub place: [Float; 3],
    /// +0x38: the car's motion, turned by the view.
    pub motion: [Float; 3],
    /// +0x44: four records of five dwords (one per wheel).
    pub voices: [EngineVoice; 4],
    /// +0x94: the times of the last two revs readings.
    pub reading_times: [Dword; 2],
    /// +0x9C: the revs of the last two readings.
    pub reading_revs: [Dword; 2],
    /// +0xA4: cleared by the set-up (`race::set_up_race_sound_objects`); nothing else reads or writes them.
    pub cleared_a4: [Dword; 2],
    /// +0xAC: set by the physics (`g_car` +0x248 and +0x24C); the speed sound stops when it is not 0.
    pub flag_ac: Dword,
    /// +0xB0: the force feedback's inputs (`input::update_car_effects`): +0x1A4 x 10, (+0x1B8 - +0x1AC) x 10, +0x1A4 x 10, +0x1AC x 10 of the view block.
    pub effect_in: [Dword; 4],
    /// +0xC0: the knock's force (`input::update_car_effects`); cleared by the set-up.
    pub knock: Dword,
    /// +0xC4: the knock's inputs (cleared after each use; +0xCC and +0xC4 give the angle, +0xD0 the weight).
    pub knock_in: [Dword; 4],
    /// +0xD4: set by `physics::share_engine_sound` (a reading: nothing in the ported code reads it).
    pub knock_d4: Dword,
    /// +0xD8: the third value of the effect requests (a reading).
    pub effect_d8: Dword,
    /// +0xDC: counts the knocks to 4 before the next effect 3.
    pub knock_count: Dword,
    /// +0xE0: not worked out (2 dwords).
    pub _gap_e0: [Dword; 2],
    /// +0xE8: the gear.
    pub gear: Dword,
    /// +0xEC: the gear when it last changed.
    pub last_gear: Dword,
    /// +0xF0: set with a gear change.
    pub gear_notice: Dword,
    /// +0xF4: counts down to 0; the speed sound waits for it.
    pub countdown: Dword,
    /// +0xF8: back-fire wait flag (a reading).
    pub wait_f8: Dword,
    /// +0xFC: the last direction of the revs.
    pub last_direction: Dword,
    /// +0x100: the revs sound's input, (+0x14 - kind +4) x kind +0xC.
    pub revs_input: Float,
    /// +0x104: the throttle sound's input, +0x18 over 256.
    pub throttle_input: Float,
    /// +0x108: the speed sound's input, |+0x1C| x `flt_516370`.
    pub speed_input: Float,
    /// +0x10C: 0 (no revs), 1 (up to 1000), 2, or 3 when +0x20 was set.
    pub state: Dword,
    /// +0x110: the last state.
    pub last_state: Dword,
    /// +0x114: not worked out (1 dwords).
    pub _gap_114: [Dword; 1],
    /// +0x118: follows +0x18.
    pub smooth_throttle: Dword,
    /// +0x11C: the gap (a reading).
    pub gap_time: Dword,
    /// +0x120: not worked out (2 dwords).
    pub _gap_120: [Dword; 2],
    /// +0x128: the turns going up.
    pub turns_up: Dword,
    /// +0x12C: 0, 1 up, 2 down.
    pub direction: Dword,
    /// +0x130: the turns going down.
    pub turns_down: Dword,
    /// +0x134: the listener's distance (the root of +0x2C .. +0x34 squared).
    pub distance: Float,
    /// +0x138: its change x the turns a second.
    pub approach: Float,
    /// +0x13C: the Doppler-like pitch factor, eased.
    pub pitch_factor: Float,
    /// +0x140: 1, or 1 / the distance when that is over 1.
    pub volume_factor: Float,
    /// +0x144: not worked out (2 dwords).
    pub _gap_144: [Dword; 2],
    /// +0x14C: seven timed sound slots of 0x18 bytes (`sound::car_slot_sounds` the first five, `sound::car_gear_sounds` the last two: +0x1C4 armed by `sound::car_engine_state` when the throttle drops at high revs, +0x1DC by its countdown at low throttle).
    pub slots: [SoundSlot; 7],
    /// +0x1F4: counts down (back-fires).
    pub backfire_wait: Dword,
    /// +0x1F8: the gear change's slot (pitch +0x210).
    pub gear_slot: SoundSlot,
    /// +0x210: the gear change sound's pitch factor.
    pub gear_pitch: Float,
    /// +0x214: the place of the slot at +0x220.
    pub place_b: [Float; 3],
    /// +0x220: a timed slot.
    pub slot_b: SoundSlot,
    /// +0x238: the hit sound to send once (-2: none), set by the physics.
    pub hit_sound: Dword,
    /// +0x23C: its volume.
    pub hit_volume: Dword,
    /// +0x240: its place.
    pub hit_place: [Float; 3],
    /// +0x24C: the second one-off sound (-2: none), set by the physics' cues.
    pub cue_sound: Dword,
    /// +0x250: its volume.
    pub cue_volume: Dword,
    /// +0x254: its place.
    pub cue_place: [Float; 3],
    /// +0x260: a timed slot.
    pub slot_c: SoundSlot,
    /// +0x278: 1 after a landing's effects (`input::update_car_effects`).
    pub landed: Dword,
    /// +0x27C: the last +0xBC, for effect 5 (a reading).
    pub last_effect_bc: Dword,
    /// +0x280: cleared by the set-up; nothing else reads or writes it.
    pub cleared_280: Dword,
    /// +0x284: the first wheel sum x 0.4 (a whole number).
    pub sum_a: Dword,
    /// +0x288: not worked out (1 dwords).
    pub _gap_288: [Dword; 1],
    /// +0x28C: the second wheel sum x 64 (a whole number).
    pub sum_b: Dword,
    /// +0x290: the other wheels' sum x 64 (a whole number).
    pub sum_c: Dword,
    /// +0x294: the surface the first wheel runs on.
    pub surface: Dword,
    /// +0x298: the queue of spoken cues (`sound::play_cue_queue`; `physics::queue_sound_cue` fills it), 0x210 bytes.
    pub cue_queue: CueQueue,
}

impl SoundObject {
    /// The object at `address`, as the game's tables and the physics hold it (a number),
    /// when all of it lies in one region of the game's memory.
    pub fn at<'a>(address: u32) -> Result<&'a SoundObject, crate::errors::BadInput> {
        crate::pointers::GamePtr::new(address).record::<SoundObject>("a sound object")
    }

    /// The timed slot the image names by its offset in the object (the core's lists of
    /// slots are by offset). Any other offset is a bug of the caller.
    pub fn slot_at(&self, offset: u32) -> &SoundSlot {
        match offset {
            0x14C => &self.slots[0],
            0x164 => &self.slots[1],
            0x17C => &self.slots[2],
            0x194 => &self.slots[3],
            0x1AC => &self.slots[4],
            0x1C4 => &self.slots[5],
            0x1DC => &self.slots[6],
            0x1F8 => &self.gear_slot,
            0x220 => &self.slot_b,
            0x260 => &self.slot_c,
            _ => panic!("no timed slot at +{offset:#x} of a sound object"),
        }
    }
}

/// A sound object as the image has it, with the car's `sound_offset` (all eight have the
/// kind 0x11 at the start).
const fn initial_object(sound_offset: u32) -> SoundObject {
    SoundObject {
        sound_offset: Dword::new(sound_offset),
        kind: Dword::new(0x11),
        in_use: Dword::new(0),
        view_flag: Dword::new(0),
        new_revs: Dword::new(0),
        revs: Dword::new(0x3E8),
        throttle: Dword::new(0),
        speed: Float::from_bits(0),
        gear_event: Dword::new(0),
        no_gear: Dword::new(0),
        _gap_28: [Dword::new(0)],
        place: [
            Float::from_bits(0),
            Float::from_bits(0),
            Float::from_bits(0),
        ],
        motion: [
            Float::from_bits(0),
            Float::from_bits(0),
            Float::from_bits(0),
        ],
        voices: [const {
            EngineVoice {
                share_a: Float::from_bits(0),
                _gap_4: Dword::new(0),
                share_b: Float::from_bits(0),
                share_c: Float::from_bits(0),
                surface: Dword::new(0),
            }
        }; 4],
        reading_times: [Dword::new(0), Dword::new(0)],
        reading_revs: [Dword::new(0), Dword::new(0)],
        cleared_a4: [Dword::new(0), Dword::new(0)],
        flag_ac: Dword::new(0),
        effect_in: [const { Dword::new(0) }; 4],
        knock: Dword::new(0),
        knock_in: [const { Dword::new(0) }; 4],
        knock_d4: Dword::new(0),
        effect_d8: Dword::new(0),
        knock_count: Dword::new(0),
        _gap_e0: [Dword::new(0), Dword::new(0)],
        gear: Dword::new(0),
        last_gear: Dword::new(0),
        gear_notice: Dword::new(0),
        countdown: Dword::new(0),
        wait_f8: Dword::new(1),
        last_direction: Dword::new(0),
        revs_input: Float::from_bits(0),
        throttle_input: Float::from_bits(0),
        speed_input: Float::from_bits(0),
        state: Dword::new(0),
        last_state: Dword::new(0),
        _gap_114: [Dword::new(0)],
        smooth_throttle: Dword::new(0),
        gap_time: Dword::new(0),
        _gap_120: [Dword::new(0), Dword::new(0)],
        turns_up: Dword::new(0),
        direction: Dword::new(0),
        turns_down: Dword::new(0),
        distance: Float::from_bits(0),
        approach: Float::from_bits(0),
        pitch_factor: Float::from_bits(0x3F800000),
        volume_factor: Float::from_bits(0),
        _gap_144: [Dword::new(0xFFFFFFFE), Dword::new(0xFFFFFFFE)],
        slots: [
            SoundSlot {
                sound: Dword::new(0xD6),
                start: Dword::new(0),
                playing: Dword::new(0),
                ends: Dword::new(0),
                volume: Dword::new(0xFFFFFF91),
                rate: Dword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: Dword::new(0xD5),
                start: Dword::new(0),
                playing: Dword::new(0),
                ends: Dword::new(0),
                volume: Dword::new(0xFFFFFF91),
                rate: Dword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: Dword::new(0xD3),
                start: Dword::new(0),
                playing: Dword::new(0),
                ends: Dword::new(0),
                volume: Dword::new(0xFFFFFF91),
                rate: Dword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: Dword::new(0xD4),
                start: Dword::new(0),
                playing: Dword::new(0),
                ends: Dword::new(0),
                volume: Dword::new(0xFFFFFF91),
                rate: Dword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: Dword::new(0xD2),
                start: Dword::new(0),
                playing: Dword::new(0),
                ends: Dword::new(0),
                volume: Dword::new(0xFFFFFF91),
                rate: Dword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: Dword::new(0xEB),
                start: Dword::new(0),
                playing: Dword::new(0),
                ends: Dword::new(0),
                volume: Dword::new(0xFFFFFF91),
                rate: Dword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: Dword::new(0xE9),
                start: Dword::new(0),
                playing: Dword::new(0),
                ends: Dword::new(0),
                volume: Dword::new(0xFFFFFF91),
                rate: Dword::new(0xFFFFFF91),
            },
        ],
        backfire_wait: Dword::new(0),
        gear_slot: SoundSlot {
            sound: Dword::new(0xE8),
            start: Dword::new(0),
            playing: Dword::new(0),
            ends: Dword::new(0),
            volume: Dword::new(0xFFFFFF91),
            rate: Dword::new(0xFFFFFF91),
        },
        gear_pitch: Float::from_bits(0x3F800000),
        place_b: [
            Float::from_bits(0),
            Float::from_bits(0),
            Float::from_bits(0),
        ],
        slot_b: SoundSlot {
            sound: Dword::new(0xE3),
            start: Dword::new(0),
            playing: Dword::new(0),
            ends: Dword::new(0),
            volume: Dword::new(0xFFFFFF91),
            rate: Dword::new(0xFFFFFF91),
        },
        hit_sound: Dword::new(0xFFFFFFFE),
        hit_volume: Dword::new(0),
        hit_place: [
            Float::from_bits(0),
            Float::from_bits(0),
            Float::from_bits(0),
        ],
        cue_sound: Dword::new(0xFFFFFFFE),
        cue_volume: Dword::new(0),
        cue_place: [
            Float::from_bits(0),
            Float::from_bits(0),
            Float::from_bits(0),
        ],
        slot_c: SoundSlot {
            sound: Dword::new(0xE4),
            start: Dword::new(0),
            playing: Dword::new(0),
            ends: Dword::new(0),
            volume: Dword::new(0xFFFFFF91),
            rate: Dword::new(0xFFFFFF91),
        },
        landed: Dword::new(0),
        last_effect_bc: Dword::new(0),
        cleared_280: Dword::new(0),
        sum_a: Dword::new(0),
        _gap_288: [Dword::new(0)],
        sum_b: Dword::new(0),
        sum_c: Dword::new(0),
        surface: Dword::new(0),
        cue_queue: CueQueue {
            first: Dword::new(0),
            place: Dword::new(0xFFFFFFFF),
            due: Dword::new(0),
            count: Dword::new(0),
            cues: [const {
                CueEntry {
                    first: Dword::new(0),
                    count: Dword::new(0),
                    sounds: [const { Dword::new(0) }; 6],
                }
            }; 16],
        },
    }
}

/// The race sounds' objects and the cars they follow (IDA 0x512A10 .. 0x514F90, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct RaceSounds {
    /// `g_raceSoundObjects`: the eight objects' addresses (IDA's addresses at start; `point_sound_objects` sets them to where the objects are now).
    pub table: [Dword; 8],
    /// The eight objects (`dword_512A30` ..), 0x4A8 bytes each.
    pub objects: [SoundObject; 8],
    /// `g_playerCarA`, `g_playerCarB`: the object numbers of the first and second player's cars.
    pub player_cars: [Dword; 2],
    /// `g_onePlayer`: 1 when one player races.
    pub one_player: Dword,
    /// `g_raceSoundObjectCount`: the objects in use.
    pub count: Dword,
    /// `g_nearestComputerCar`: the nearest computer car (an object number, -1 for none).
    pub nearest: Dword,
    /// `g_nextComputerCar`: the next nearest.
    pub next: Dword,
    /// `g_computerCarDistance`: the squared distance `sound::car_sounds_all` works out.
    pub nearest_distance: Float,
    /// `g_markCarVoices`: 1 when the cars' voices are marked (`sound::mark_car_voices`).
    pub mark_car_voices: Dword,
}

/// The race sounds' objects (`RaceSounds`); first values the image's.
pub static RACE_SOUNDS: RaceSounds = RaceSounds {
    table: [
        Dword::new(0x512A30),
        Dword::new(0x512ED8),
        Dword::new(0x513380),
        Dword::new(0x513828),
        Dword::new(0x513CD0),
        Dword::new(0x514178),
        Dword::new(0x514620),
        Dword::new(0x514AC8),
    ],
    objects: [
        initial_object(0),
        initial_object(1),
        initial_object(0xFFFFFFFE),
        initial_object(0xFFFFFFFD),
        initial_object(0xFFFFFFFC),
        initial_object(0xFFFFFFFB),
        initial_object(0xFFFFFFFA),
        initial_object(0xFFFFFFF9),
    ],
    player_cars: [Dword::new(0), Dword::new(1)],
    one_player: Dword::new(1),
    count: Dword::new(1),
    nearest: Dword::new(0),
    next: Dword::new(0),
    nearest_distance: Float::from_bits(0),
    mark_car_voices: Dword::new(0),
};

/// Where the race sounds' block is (`moved::MOVED`'s home for IDA 0x512A10).
pub fn race_sounds_addr() -> u32 {
    crate::pointers::game_address(&raw const RACE_SOUNDS)
}

/// The race sounds' clock and the constants of the placed and moving sounds (IDA 0x5124E4
/// .. 0x512508, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct RaceClock {
    /// `dword_5124E4`: 1 once the race's sounds are set up (`race::sounds`,
    /// `race::sound_tables`), 0 at the race's end; `sound::race_turn` does nothing before.
    pub started: Dword,
    /// `dword_5124E8`: the clock (ms) of the last sound turn.
    pub last: Dword,
    /// `flt_5124EC`: sound turns a second (a distance changed in one turn times it is a
    /// speed in units a second).
    pub turns_a_second: Float,
    /// `dword_5124F0`: the turn's length (ms).
    pub step: Dword,
    /// `dword_5124F4`: the turns counted.
    pub turns: Dword,
    /// `flt_5124F8` (337.52) and `flt_5124FC` (its inverse, 0.002963): a moving sound's
    /// pitch is (speed of approach + the first) times the second, the Doppler factor
    /// with the speed of sound (a reading).
    pub sound_speed: Float,
    pub over_sound_speed: Float,
    /// `flt_512500` (64) and `flt_512504` (128 / pi): a placed sound's pan, the centre and
    /// the pan steps per radian of angle.
    pub pan_centre: Float,
    pub pan_per_radian: Float,
}

/// The race sounds' clock (`RaceClock`); first values the image's
/// (`tests::race_clock_starts_as_the_games`).
pub static RACE_CLOCK: RaceClock = RaceClock {
    started: Dword::new(0),
    last: Dword::new(0),
    turns_a_second: Float::from_bits(0),
    step: Dword::new(0),
    turns: Dword::new(0),
    sound_speed: Float::from_bits(0x43A8_C28F),
    over_sound_speed: Float::from_bits(0x3B42_2B53),
    pan_centre: Float::from_bits(0x4280_0000),
    pan_per_radian: Float::from_bits(0x4222_F983),
};

/// Where the race clock is (`moved::MOVED`'s home for IDA 0x5124E4).
pub fn race_clock_addr() -> u32 {
    crate::pointers::game_address(&raw const RACE_CLOCK)
}

/// The surface sounds' constants and work (IDA 0x516358 .. 0x51639C, the image's layout;
/// phase 7, typed state): the factors the engine, speed, revs and throttle sounds and the
/// wheel sounds scale by, and what `sound::surface_sounds` works out.
#[repr(C)]
pub struct SurfaceSounds {
    /// `dword_516358`: 1 once `race::prepare_sound_tables` has run.
    pub tables_done: Dword,
    /// `flt_51635C` (0.1): a surface sound's volume is the wheels' sum times it.
    pub volume_scale: Float,
    /// `flt_516360` (0.1): how far a pitch moves to its target in a turn (the engine's
    /// and the spot voices' Doppler pitch).
    pub pitch_blend: Float,
    /// `flt_516364` (0.4): the first wheel sum (+0x284) is scaled by it.
    pub across_scale: Float,
    /// `flt_516368` (64): the second wheel sum (+0x28C, +0x290) is scaled by it.
    pub along_scale: Float,
    /// `flt_51636C` (160): not read by any ported code.
    pub unused_160: Float,
    /// `flt_516370` (0.014): the car speed's (|+0x1C|) factor for the speed sound's
    /// input (+0x108).
    pub speed_scale: Float,
    /// `0x516374` (65536.0): not read by any ported code.
    pub unused_65536: Float,
    /// `0x516378` (2.0): not read by any ported code.
    pub unused_2: Float,
    /// `flt_51637C` (1.4): the speed sound's gain.
    pub speed_gain: Float,
    /// `flt_516380` (0.714): the revs and throttle sounds' gain.
    pub revs_gain: Float,
    /// `0x516384` (0.625) and `0x516388` (0.25): not read by any ported code.
    pub unused_625: Float,
    pub unused_quarter: Float,
    /// `g_wheelSoundAcross`: front less back of the wheel floats (the signs pick the wheel).
    pub across: Float,
    /// `0x516390`: nothing reads or writes it (a gap).
    pub _gap_390: Dword,
    /// `g_wheelSoundAlong`: left less right of the wheel floats.
    pub along: Float,
    /// `g_surfaceSoundVolume`: the volume the surface sound is sent with.
    pub volume: Float,
}

/// The surface sounds' constants and work (`SurfaceSounds`); first values the image's.
pub static SURFACE_SOUNDS: SurfaceSounds = SurfaceSounds {
    tables_done: Dword::new(0),
    volume_scale: Float::from_bits(0x3DCC_CCCD),
    pitch_blend: Float::from_bits(0x3DCC_CCCD),
    across_scale: Float::from_bits(0x3ECC_CCCD),
    along_scale: Float::from_bits(0x4280_0000),
    unused_160: Float::from_bits(0x4320_0000),
    speed_scale: Float::from_bits(0x3C66_6666),
    unused_65536: Float::from_bits(0x4780_0000),
    unused_2: Float::from_bits(0x4000_0000),
    speed_gain: Float::from_bits(0x3FB3_3333),
    revs_gain: Float::from_bits(0x3F36_DB6E),
    unused_625: Float::from_bits(0x3F20_0000),
    unused_quarter: Float::from_bits(0x3E80_0000),
    across: Float::from_bits(0),
    _gap_390: Dword::new(0),
    along: Float::from_bits(0),
    volume: Float::from_bits(0),
};

/// Where the surface sounds' block is (`moved::MOVED`'s home for IDA 0x516358).
pub fn surface_sounds_addr() -> u32 {
    crate::pointers::game_address(&raw const SURFACE_SOUNDS)
}

/// A timed sound's slot: six dwords (`sound::play_timed_slot`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SoundSlot {
    /// The sound (a sample number); +0 of the slot `sound::play_timed_slot` takes.
    pub sound: Dword,
    /// 1 = start it (cleared by the slot's turn).
    pub start: Dword,
    /// 1 = playing.
    pub playing: Dword,
    /// When it ends (the sound clock, ms).
    pub ends: Dword,
    /// The volume (-0x6F: the sample's own).
    pub volume: Dword,
    /// The rate (-0x6F: the sample's own).
    pub rate: Dword,
}

/// A sound spot shared with the sound side, 0x14 bytes.
#[repr(C)]
pub struct SharedSpot {
    /// The spot's id.
    pub id: Dword,
    /// Its place.
    pub place: [Float; 3],
    /// Its kind, 1 .. 0x15.
    pub kind: Dword,
}

/// A spot kind's repeats, 8 bytes.
#[repr(C)]
pub struct KindRepeats {
    /// Nothing reads it (a gap).
    pub _word_0: Word,
    /// The most repeats (0: no limit).
    pub most: Word,
    /// The least delay before a repeat.
    pub least_delay: Word,
    /// The most delay before a repeat.
    pub most_delay: Word,
}

/// The stage sounds' state and the shared sound spots (IDA 0x51655C .. 0x5168C0, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct StageSounds {
    /// `g_surfaceSoundSet` (`dword_51655C`): a surface's sound set (an index into `off_51639C`'s pairs), 16 surfaces.
    pub surface_sets: [Dword; 16],
    /// `g_stageSoundOnce1` .. `3`: at 1 the stage's one-off sound (0x1AB, 0x1AC, 0x1AD) is played once (the flag is cleared).
    pub once: [Dword; 3],
    /// `g_stageTimedArm`: at 1 the timed sound is armed (cleared; due and waiting set).
    pub timed_arm: Dword,
    /// `g_stageTimedPlace` and the two after it: the timed sound's place.
    pub timed_place: [Float; 3],
    /// `g_stageTimedDelay`: the delay (ms) from arming to the timed sound.
    pub timed_delay: Dword,
    /// `g_stageLoopVolumeA`: the volume (halved) of the sound that plays all the time (0x1C5).
    pub loop_volume_a: Dword,
    /// `g_stageLoopVolumeB`: the volume of the second (0x1C6).
    pub loop_volume_b: Dword,
    /// `g_stageTimedDue`: the clock the timed sound is due at.
    pub timed_due: Dword,
    /// `g_stageTimedWaiting`: 1 while it waits to be due.
    pub timed_waiting: Dword,
    /// `g_stageTimedSlot` (`unk_5165CC`): the timed sound's slot (sound 0x1C4).
    pub timed_slot: SoundSlot,
    /// `g_stageRandomSound1`, `g_stageRandomSound2`: at 1 a sound (0x1AE, 0x1AF) at a random pitch and place is played once.
    pub random: [Dword; 2],
    /// `g_sharedSoundSpotCount`: the spots the race frame shared this frame (`race::share_sound_spots`).
    pub shared_count: Dword,
    /// `g_sharedSoundSpots`: the spots shared with the sound side, 36 of 0x14 bytes.
    pub shared_spots: [SharedSpot; 36],
}

/// The stage sounds (`StageSounds`); first values the image's.
pub static STAGE_SOUNDS: StageSounds = StageSounds {
    surface_sets: [
        Dword::new(1),
        Dword::new(3),
        Dword::new(0),
        Dword::new(1),
        Dword::new(2),
        Dword::new(4),
        Dword::new(1),
        Dword::new(1),
        Dword::new(1),
        Dword::new(2),
        Dword::new(6),
        Dword::new(2),
        Dword::new(2),
        Dword::new(1),
        Dword::new(1),
        Dword::new(1),
    ],
    once: [Dword::new(0), Dword::new(0), Dword::new(0)],
    timed_arm: Dword::new(0),
    timed_place: [
        Float::from_bits(0),
        Float::from_bits(0x40000000),
        Float::from_bits(0x3F800000),
    ],
    timed_delay: Dword::new(0),
    loop_volume_a: Dword::new(0),
    loop_volume_b: Dword::new(0),
    timed_due: Dword::new(0),
    timed_waiting: Dword::new(0),
    timed_slot: SoundSlot {
        sound: Dword::new(0x1C4),
        start: Dword::new(0),
        playing: Dword::new(0),
        ends: Dword::new(0),
        volume: Dword::new(0xFFFFFF91),
        rate: Dword::new(0xFFFFFF91),
    },
    random: [Dword::new(0), Dword::new(0)],
    shared_count: Dword::new(0),
    shared_spots: [const {
        SharedSpot {
            id: Dword::new(0),
            place: [
                Float::from_bits(0),
                Float::from_bits(0),
                Float::from_bits(0),
            ],
            kind: Dword::new(0),
        }
    }; 36],
};

/// Where the stage sounds are (`moved::MOVED`'s home for IDA 0x51655C).
pub fn stage_sounds_addr() -> u32 {
    crate::pointers::game_address(&raw const STAGE_SOUNDS)
}

/// The spot voices' pickers' work and the sound kinds per spot (IDA 0x517f40 .. 0x518080, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SpotPicks {
    /// `g_spotPickPlaying`: the pickers' work (`sound::merge_spot_voices`): the playing voice (an address).
    pub pick_playing: Dword,
    /// `g_spotPickNearestDistance`: the nearest's distance.
    pub pick_nearest_distance: Dword,
    /// `g_spotPickNearest`: the nearest voice (an address).
    pub pick_nearest: Dword,
    /// `g_spotPickNextDistance`: the next's distance.
    pub pick_next_distance: Dword,
    /// `g_spotPickNext`: the next voice (an address).
    pub pick_next: Dword,
    /// `g_spotKindSounds` (`dword_517F54`): a spot kind's sound, 22 of them (kind 0 unused).
    pub kind_sounds: [Dword; 22],
    /// `g_spotKindRepeats` (`unk_517FAC`): a kind's repeats, 8 bytes, kind 1 first.
    pub kind_repeats: [KindRepeats; 21],
    /// `dword_518054`: cleared at a race's start (`race::reset_race_sounds`); read by a sound function not ported by name (0x40EC85); a reading.
    pub spot_work_a: Dword,
    /// `dword_518058`: read at 0x40EC8E (a reading).
    pub spot_work_b: Dword,
    /// `0x51805C`: nine dwords nothing ported reads or writes (1, 2, 3, 4, 5, 6, 9, 7, 8 at start).
    pub _gap_5c: [Dword; 9],
}

pub static SPOT_PICKS: SpotPicks = SpotPicks {
    pick_playing: Dword::new(0),
    pick_nearest_distance: Dword::new(0xFFFFFFFF),
    pick_nearest: Dword::new(0),
    pick_next_distance: Dword::new(0xFFFFFFFF),
    pick_next: Dword::new(0),
    kind_sounds: [
        Dword::new(0xFFFFFFFE),
        Dword::new(0x1B1),
        Dword::new(0x1B2),
        Dword::new(0x1B3),
        Dword::new(0x1B4),
        Dword::new(0x1B5),
        Dword::new(0x1B6),
        Dword::new(0x1B7),
        Dword::new(0x1B9),
        Dword::new(0x1BA),
        Dword::new(0x1BB),
        Dword::new(0x1B0),
        Dword::new(0xFFFFFFFE),
        Dword::new(0x1BD),
        Dword::new(0x1BE),
        Dword::new(0x1BF),
        Dword::new(0x1C0),
        Dword::new(0x1C1),
        Dword::new(0x1C2),
        Dword::new(0x1C3),
        Dword::new(0x1B8),
        Dword::new(0x1BC),
    ],
    kind_repeats: [
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(1000),
            most_delay: Word::new(5000),
        },
        KindRepeats {
            _word_0: Word::new(1),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(3000),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(2000),
            most_delay: Word::new(6000),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(500),
            most_delay: Word::new(3000),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(2000),
            most_delay: Word::new(6000),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(1000),
            most_delay: Word::new(4000),
        },
        KindRepeats {
            _word_0: Word::new(1),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(1000),
            most_delay: Word::new(5000),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(1000),
            most_delay: Word::new(4000),
        },
        KindRepeats {
            _word_0: Word::new(1),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(4000),
            most_delay: Word::new(8000),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(3000),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(1000),
            most_delay: Word::new(5000),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
        KindRepeats {
            _word_0: Word::new(0),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
        KindRepeats {
            _word_0: Word::new(1),
            most: Word::new(1),
            least_delay: Word::new(0),
            most_delay: Word::new(5000),
        },
        KindRepeats {
            _word_0: Word::new(1),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
        KindRepeats {
            _word_0: Word::new(1),
            most: Word::new(0),
            least_delay: Word::new(0),
            most_delay: Word::new(0),
        },
    ],
    spot_work_a: Dword::new(0),
    spot_work_b: Dword::new(0),
    _gap_5c: [
        Dword::new(1),
        Dword::new(2),
        Dword::new(3),
        Dword::new(4),
        Dword::new(5),
        Dword::new(6),
        Dword::new(9),
        Dword::new(7),
        Dword::new(8),
    ],
};

impl SpotPicks {
    pub const START: usize = 0x517f40;
    pub const LEN: usize = 0x140;

    /// `moved::MOVED`'s home for `START`.
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const SPOT_PICKS) as usize
    }
}

/// Set once the kept sound settings are applied (IDA 0x518080 .. 0x518084, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SettingsApplied {
    /// `dword_518080`: set once the kept sound settings are applied (`sound::apply_settings`, `sound::set_up_race_sounds`).
    pub done: Dword,
}

pub static SETTINGS_APPLIED: SettingsApplied = SettingsApplied {
    done: Dword::new(0),
};

impl SettingsApplied {
    pub const START: usize = 0x518080;
    pub const LEN: usize = 0x4;

    /// `moved::MOVED`'s home for `START`.
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const SETTINGS_APPLIED) as usize
    }
}

/// The kept sound options (IDA 0x5180f0 .. 0x518104, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SoundOptions {
    /// `dword_5180F0`: kept in `var\ral.dat` (`saves::load_options`); what it sets is not worked out.
    pub kept_f0: Dword,
    /// `dword_5180F4`: the driver names switch (`menu::toggle_5180f4`), kept in `var\ral.dat`.
    pub name_switch: Dword,
    /// `g_tuneTurn`: which of tunes 10 .. 15 the next tune 0xA is (0 .. 5), kept.
    pub tune_turn: Dword,
    /// `g_musicVolume`, kept.
    pub music_volume: Dword,
    /// `dword_518100`: the race music's volume (the menu sets it with `g_musicVolume`; both go to the tunes), kept.
    pub race_music_volume: Dword,
}

pub static SOUND_OPTIONS: SoundOptions = SoundOptions {
    kept_f0: Dword::new(1),
    name_switch: Dword::new(0),
    tune_turn: Dword::new(0),
    music_volume: Dword::new(0x20),
    race_music_volume: Dword::new(0x20),
};

impl SoundOptions {
    pub const START: usize = 0x5180f0;
    pub const LEN: usize = 0x14;

    /// `moved::MOVED`'s home for `START`.
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const SOUND_OPTIONS) as usize
    }
}

/// The CD audio's state (IDA 0x518104 .. 0x518120, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct CdAudio {
    /// `dword_518104`: `nocdaudio` (`var\ral.cfg`).
    pub no_cd_audio: Dword,
    /// `g_cdAudioOpen`: the CD audio is open.
    pub open: Dword,
    /// `0x51810C`: nothing reads or writes it.
    pub _gap_10c: Dword,
    /// `g_cdTrack`.
    pub track: Dword,
    /// `dword_518114` (a reading).
    pub work_114: Dword,
    /// `mciId`: the CD audio device.
    pub mci_id: Dword,
    /// `dword_51811C` (a reading).
    pub work_11c: Dword,
}

pub static CD_AUDIO: CdAudio = CdAudio {
    no_cd_audio: Dword::new(1),
    open: Dword::new(0),
    _gap_10c: Dword::new(0),
    track: Dword::new(0),
    work_114: Dword::new(0),
    mci_id: Dword::new(0),
    work_11c: Dword::new(0),
};

impl CdAudio {
    pub const START: usize = 0x518104;
    pub const LEN: usize = 0x1c;

    /// `moved::MOVED`'s home for `START`.
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const CD_AUDIO) as usize
    }
}

// ---- single variables the game's thread keeps ----

/// `g_nextMenuSoundTime` (IDA 0x511B0C): the clock the next menu sound may be heard at.
pub static NEXT_MENU_SOUND_TIME: Dword = Dword::new(0);

/// `g_raceSoundsOn` (0x6218F0): 1 at start; the race's sounds switched.
pub static RACE_SOUNDS_ON: Dword = Dword::new(1);

/// The car stream's three dwords (0xA8C180 .. 0xA8C18C): the place it is at, its list, and
/// whether it is on (`sound::play_car_stream`, `sound::step_car_stream`, the stage
/// view's speech).
#[repr(C)]
pub struct CarStream {
    pub place: Dword,  // dword_A8C180
    pub stream: Dword, // dword_A8C184
    pub on: Dword,     // dword_A8C188
}

pub static CAR_STREAM: CarStream = CarStream {
    place: Dword::new(0),
    stream: Dword::new(0),
    on: Dword::new(0),
};

pub const CAR_STREAM_IDA: usize = 0xA8C180;

/// Sets `g_raceSoundObjects` to the objects' places now.
pub fn point_sound_objects() {
    for (pointer, object) in RACE_SOUNDS.table.iter().zip(&RACE_SOUNDS.objects) {
        pointer.set(crate::pointers::game_address(std::ptr::from_ref::<
            SoundObject,
        >(object)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;

    fn dword(d: &Dword) -> Vec<u8> {
        d.get().to_le_bytes().to_vec()
    }

    #[test]
    fn single_variables_start_as_the_games() {
        assert_eq!(dword(&NEXT_MENU_SOUND_TIME), image_bytes(0x511B0C, 4));
        assert_eq!(dword(&RACE_SOUNDS_ON), image_bytes(0x6218F0, 4));
        let mut stream = Vec::new();
        for d in [&CAR_STREAM.place, &CAR_STREAM.stream, &CAR_STREAM.on] {
            stream.extend(dword(d));
        }
        assert_eq!(stream, image_bytes(CAR_STREAM_IDA, 12));
        assert_eq!(core::mem::size_of::<CarStream>(), 12);
    }

    #[test]
    fn wheel_surfaces_start_as_the_games() {
        let image = crate::moved::image_bytes(0x4F66F1, 8);
        let mine = unsafe { std::slice::from_raw_parts(WHEEL_SURFACES.as_ptr().cast::<u8>(), 8) };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn surface_sounds_start_as_the_games() {
        assert_eq!(size_of::<super::SurfaceSounds>(), 0x51639C - 0x516358);
        let image = crate::moved::image_bytes(0x516358, 0x44);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(surface_sounds_addr()).cast_const(),
                0x44,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn stage_sounds_start_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<super::StageSounds>(), 0x5168C0 - 0x51655C);
        assert_eq!(offset_of!(StageSounds, once), 0x51659C - 0x51655C);
        assert_eq!(offset_of!(StageSounds, timed_arm), 0x5165A8 - 0x51655C);
        assert_eq!(offset_of!(StageSounds, timed_delay), 0x5165B8 - 0x51655C);
        assert_eq!(offset_of!(StageSounds, timed_slot), 0x5165CC - 0x51655C);
        assert_eq!(offset_of!(StageSounds, random), 0x5165E4 - 0x51655C);
        assert_eq!(offset_of!(StageSounds, shared_count), 0x5165EC - 0x51655C);
        assert_eq!(offset_of!(StageSounds, shared_spots), 0x5165F0 - 0x51655C);
        let image = crate::moved::image_bytes(0x51655C, 0x364);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(stage_sounds_addr()).cast_const(),
                0x364,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn spot_picks_starts_as_the_games() {
        assert_eq!(size_of::<SpotPicks>(), SpotPicks::LEN);
        let image = crate::moved::image_bytes(SpotPicks::START, SpotPicks::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(SpotPicks::home()).cast_const(),
                SpotPicks::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn settings_applied_starts_as_the_games() {
        assert_eq!(size_of::<SettingsApplied>(), SettingsApplied::LEN);
        let image = crate::moved::image_bytes(SettingsApplied::START, SettingsApplied::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(SettingsApplied::home()).cast_const(),
                SettingsApplied::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn sound_options_starts_as_the_games() {
        assert_eq!(size_of::<SoundOptions>(), SoundOptions::LEN);
        let image = crate::moved::image_bytes(SoundOptions::START, SoundOptions::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(SoundOptions::home()).cast_const(),
                SoundOptions::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn cd_audio_starts_as_the_games() {
        assert_eq!(size_of::<CdAudio>(), CdAudio::LEN);
        let image = crate::moved::image_bytes(CdAudio::START, CdAudio::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(CdAudio::home()).cast_const(),
                CdAudio::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn race_sounds_start_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<super::SoundObject>(), 0x4A8);
        assert_eq!(size_of::<super::RaceSounds>(), 0x514F90 - 0x512A10);
        assert_eq!(offset_of!(RaceSounds, objects), 0x512A30 - 0x512A10);
        assert_eq!(offset_of!(RaceSounds, player_cars), 0x514F70 - 0x512A10);
        assert_eq!(offset_of!(RaceSounds, one_player), 0x514F78 - 0x512A10);
        assert_eq!(offset_of!(RaceSounds, count), 0x514F7C - 0x512A10);
        assert_eq!(offset_of!(RaceSounds, nearest), 0x514F80 - 0x512A10);
        assert_eq!(
            offset_of!(RaceSounds, nearest_distance),
            0x514F88 - 0x512A10
        );
        assert_eq!(offset_of!(RaceSounds, mark_car_voices), 0x514F8C - 0x512A10);
        assert_eq!(offset_of!(SoundObject, state), 0x10C);
        assert_eq!(offset_of!(SoundObject, slots), 0x14C);
        assert_eq!(offset_of!(SoundObject, gear_slot), 0x1F8);
        assert_eq!(offset_of!(SoundObject, gear_pitch), 0x210);
        assert_eq!(offset_of!(SoundObject, hit_sound), 0x238);
        assert_eq!(offset_of!(SoundObject, cue_sound), 0x24C);
        assert_eq!(offset_of!(SoundObject, slot_c), 0x260);
        assert_eq!(offset_of!(SoundObject, surface), 0x294);
        let image = crate::moved::image_bytes(0x512A10, 0x2580);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(race_sounds_addr()).cast_const(),
                0x2580,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn race_clock_starts_as_the_games() {
        assert_eq!(size_of::<super::RaceClock>(), 0x512508 - 0x5124E4);
        let image = crate::moved::image_bytes(0x5124E4, 0x24);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(super::race_clock_addr()).cast_const(),
                0x24,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
