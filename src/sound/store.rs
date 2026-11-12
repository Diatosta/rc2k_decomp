//! The sound module's variables, out of RAL.EXE's image,
//! typed: each is a `#[repr(C)]` struct of atomic cells (the first values the image's,
//! `tests::*_start_as_the_games`; the race sounds' objects and the surface sounds hold their fields
//! with nothing between them: `sound::asm_layout` has the 1999 offsets the kept assembly's tests use), one static a piece:
//! - `WHEEL_SURFACES` (IDA 0x4F66F1 .. 0x4F66F9): the two wheel surfaces the sound reads;
//! - `RaceClock` (0x5124E4 .. 0x512508): the race sounds' clock and constants;
//! - `RaceSounds` (0x512A10 .. 0x514F90): the eight sound objects (`SoundObject`, 0x4A8 bytes
//!   each in the image, with the table of their addresses) and the cars the sound follows. The table
//!   keeps the image's IDA addresses: nothing reads it (the objects are reached by number);
//! - `SurfaceSounds` (0x516358 .. 0x51639C): the factors the engine, speed, revs and
//!   throttle sounds scale by, the wheel sounds across and along, the surface sound volume;
//! - `StageSounds` (0x51655C .. 0x5168C0): the surface sound sets, the stage's one-off,
//!   timed, loop and random sounds, and the spots shared with the sound side;
//! - `SpotPicks` (0x517F40 .. 0x518080): the spot-voice picks and the sound kinds per spot;
//!   `SettingsApplied` (0x518080); `SoundOptions` (0x5180F0 .. 0x518104): the kept options
//!   (`g_tuneTurn`, `g_musicVolume` ..); `CdAudio` (0x518104 .. 0x518120). Between them in the
//!   image, the force-feedback effects' state is `input::store::ForceFeedback`'s.
//!
//! The sound commands go to the audio thread through `audio::store`'s queue; these are the
//! game thread's.

use crate::session::{Dword, Float, GameThread, PlainDword, PlainFloat, PlainWord};

/// `g_wheelSurfaceA`, `g_wheelSurfaceB` (IDA 0x4F66F1, 0x4F66F5): the surfaces of the wheels
/// the first and the second of a car's two sounds follow (`sound::surface_sounds`). Kept apart
/// from the dwords around them: the save file's four before
/// (`saves::owner::Files::options_kept`), the race record's seal key after (`race::replay::SEAL_KEY`).
pub static WHEEL_SURFACES: GameThread<[PlainDword; 2]> =
    GameThread::new([PlainDword::new(0), PlainDword::new(0)]);

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
    /// A share the physics sets (`physics::car_sound_state`: +0x294 of the work car, or 0); the first wheel sum (`sum_a`) scales these by 0.4.
    pub share_a: Float,
    /// 1.0, or the share +0x28C (`sound::surface_sounds` reads these: front less back and left less right, and their sum x 64).
    pub share_b: Float,
    /// The share x 256.
    pub share_c: Float,
    /// The surface the wheel runs on (`g_car` +0x930).
    pub surface: Dword,
}

/// A race sound object (0x4A8 bytes in the image): a car's sound state (the revs, the throttle, the place, the timed sound slots) as `sound::car_sounds` and the physics keep it.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SoundObject {
    /// Added to a sound set's first sound to pick the car's sound; negative for a computer's car (the sets are picked in `sound::revs_sound` and `surface_sound`).
    pub sound_offset: Dword,
    /// The car's sound kind, an index into `SoundKinds.kinds[0]` (36 of 0x88 bytes).
    pub kind: Dword,
    /// 1 when the object is in use (`sound::car_sounds_all`).
    pub in_use: Dword,
    /// `g_car` +0x6F0 xor 1; the engine, throttle and speed sounds' volume is scaled by a constant when it is 0 (`sound::revs_sound`, `throttle_sound`, `car_slot_sounds`: 1 is the view of the car itself, which gets the surface's gain instead).
    pub view_flag: Dword,
    /// 1 when a new revs reading (`revs`) came (`sound::car_engine_state`).
    pub new_revs: Dword,
    /// The revs (extrapolated between the physics' readings).
    pub revs: Dword,
    /// The throttle, 0 ..= 0x100.
    pub throttle: Dword,
    /// The car's speed.
    pub speed: Float,
    /// Set by the physics when the gear changed (cleared by `sound::car_engine_state`).
    pub gear_event: Dword,
    /// 1 when the car is in neutral (gear 0, or the gear cue's random bits 0: `sound::car_engine_state`); 1 from the set-up.
    pub no_gear: Dword,
    /// The car's place less the listener's, turned by the view and scaled (`physics::car_sound_state`).
    pub place: [Float; 3],
    /// The car's motion, turned by the view.
    pub motion: [Float; 3],
    /// Four records of five dwords (one per wheel).
    pub voices: [EngineVoice; 4],
    /// The times of the last two revs readings.
    pub reading_times: [Dword; 2],
    /// The revs of the last two readings.
    pub reading_revs: [Dword; 2],
    /// Two dwords cleared by the set-up (`race::set_up_race_sound_objects`); nothing else reads or writes them.
    pub unused_a4: [Dword; 2],
    /// Set by the physics (`g_car` +0x248 and +0x24C); the speed sound stops when it is not 0.
    pub flag_ac: Dword,
    /// The force feedback's inputs (`input::update_car_effects`; of the view block +0x1A4 the sideways speed, +0x1AC the forward speed, +0x1B8 the driven wheels' speed): +0x1A4 x 10, (+0x1B8 - +0x1AC) x 10, +0x1A4 x 10, +0x1AC x 10 of the view block.
    pub effect_in: [Dword; 4],
    /// The knock's force (`input::update_car_effects`); cleared by the set-up.
    pub knock: Dword,
    /// The knock's inputs (cleared after each use; `knock_in[2]` and `knock_in[0]` give the angle, `knock_in[3]` the weight).
    pub knock_in: [Dword; 4],
    /// A copy of the car's steering trim (`g_car` `steer_trim`) made by `physics::share_engine_sound`; nothing reads it.
    pub steer_trim_copy: Dword,
    /// The `b` number of the Fourth, Steering and Landing force-feedback requests (`input::update_car_effects`: the request word's bits 20 ..); never written, so 0 (read only by that function).
    pub effect_request_b: Dword,
    /// Counts the knocks to 4 before the next effect 3.
    pub knock_count: Dword,
    /// The gear.
    pub gear: Dword,
    /// The gear when it last changed.
    pub last_gear: Dword,
    /// Set with a gear change.
    pub gear_notice: Dword,
    /// Counts down to 0; the speed sound waits for it.
    pub countdown: Dword,
    /// 1 once the throttle-drop sound has been triggered in this downward run of the throttle (set when the throttle falls more than 0x14 below its follower, cleared when it rises; `rc2k_core::sound::engine`), so it plays once per lift; 1 from the set-up.
    pub drop_latched: Dword,
    /// The last direction of the revs.
    pub last_direction: Dword,
    /// The revs sound's input, (`revs` less the kind's `revs_low`) x its `revs_scale`.
    pub revs_input: Float,
    /// The throttle sound's input, `throttle` over 256.
    pub throttle_input: Float,
    /// The speed sound's input, |`speed`| x `SurfaceSounds.speed_scale`.
    pub speed_input: Float,
    /// 0 (no revs), 1 (up to 1000), 2, or 3 when `gear_event` was set.
    pub state: Dword,
    /// The last state.
    pub last_state: Dword,
    /// Follows `throttle`.
    pub smooth_throttle: Dword,
    /// The throttle's lead over its follower, `throttle` - `smooth_throttle` (negative on a lift; a gear cue needs 0x32 or more, the drop sound -0x14 or less).
    pub throttle_lead: Dword,
    /// The turns going up.
    pub turns_up: Dword,
    /// 0, 1 up, 2 down.
    pub direction: Dword,
    /// The turns going down.
    pub turns_down: Dword,
    /// The listener's distance (the root of the squares of `place`).
    pub distance: Float,
    /// Its change x the turns a second.
    pub approach: Float,
    /// The Doppler-like pitch factor, eased.
    pub pitch_factor: Float,
    /// 1, or 1 / the distance when that is over 1.
    pub volume_factor: Float,
    /// The sound playing from the surface sets of eight (-2: none; `sound::surface_sound`).
    pub eights_sound: Dword,
    /// The sound playing from the surface sets of four (-2: none).
    pub fours_sound: Dword,
    /// Seven timed sound slots of 0x18 bytes (`sound::car_slot_sounds` the first five, `sound::car_gear_sounds` the last two: the sixth armed by `sound::car_engine_state` when the throttle drops at high revs, the seventh by its countdown at low throttle).
    pub slots: [SoundSlot; 7],
    /// Counts down (back-fires).
    pub backfire_wait: Dword,
    /// The gear change's slot (pitch `gear_pitch`).
    pub gear_slot: SoundSlot,
    /// The gear change sound's pitch factor.
    pub gear_pitch: Float,
    /// The place of `slot_b`.
    pub place_b: [Float; 3],
    /// A timed slot.
    pub slot_b: SoundSlot,
    /// The hit sound to send once (-2: none), set by the physics.
    pub hit_sound: Dword,
    /// Its volume.
    pub hit_volume: Dword,
    /// Its place.
    pub hit_place: [Float; 3],
    /// The second one-off sound (-2: none), set by the physics' cues.
    pub cue_sound: Dword,
    /// Its volume.
    pub cue_volume: Dword,
    /// Its place.
    pub cue_place: [Float; 3],
    /// A timed slot.
    pub slot_c: SoundSlot,
    /// 1 after a landing's effects (`input::update_car_effects`).
    pub landed: Dword,
    /// The last forward speed x 10 (`effect_in[3]`), kept for the next turn's speed-change effect (effect 5, `input::update_car_effects`); cleared by the set-up.
    pub last_forward_speed: Dword,
    /// Cleared by the set-up; nothing else reads or writes it.
    pub unused_280: Dword,
    /// The first wheel sum x 0.4 (a whole number).
    pub sum_a: Dword,
    /// The second wheel sum x 64 (a whole number).
    pub sum_b: Dword,
    /// The other wheels' sum x 64 (a whole number).
    pub sum_c: Dword,
    /// The surface the first wheel runs on.
    pub surface: Dword,
    /// The queue of spoken cues (`sound::play_cue_queue`; `physics::queue_sound_cue` fills it), 0x210 bytes.
    pub cue_queue: CueQueue,
}

/// The current car's sound object (`g_raceSoundObjects[g_currentCar & 7]`).
pub fn current_object<'a>(
    sound: &'a crate::sound::owner::Sound,
    work: &crate::race::work_store::RaceWork,
) -> &'a SoundObject {
    &sound.race.sounds.objects[(work.current_car.get() & 7) as usize]
}

impl SoundObject {
    /// Race sound object `n` (the original kept its address in `g_raceSoundObjects`),
    /// refused past the eight.
    pub fn of(
        sound: &crate::sound::owner::Sound,
        n: u32,
    ) -> Result<&SoundObject, crate::errors::BadInput> {
        sound
            .race
            .sounds
            .objects
            .get(n as usize)
            .ok_or(crate::errors::BadInput(
                "a race sound object past the eight",
            ))
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
pub(crate) const fn initial_object(sound_offset: u32) -> SoundObject {
    SoundObject {
        sound_offset: Dword::new(sound_offset),
        kind: Dword::new(0x11),
        in_use: Dword::new(0),
        view_flag: Dword::new(0),
        new_revs: Dword::new(0),
        revs: Dword::new(0x3E8),
        throttle: Dword::new(0),
        speed: Float::new(0.0),
        gear_event: Dword::new(0),
        no_gear: Dword::new(0),
        place: [Float::new(0.0), Float::new(0.0), Float::new(0.0)],
        motion: [Float::new(0.0), Float::new(0.0), Float::new(0.0)],
        voices: [const {
            EngineVoice {
                share_a: Float::new(0.0),
                share_b: Float::new(0.0),
                share_c: Float::new(0.0),
                surface: Dword::new(0),
            }
        }; 4],
        reading_times: [Dword::new(0), Dword::new(0)],
        reading_revs: [Dword::new(0), Dword::new(0)],
        unused_a4: [Dword::new(0), Dword::new(0)],
        flag_ac: Dword::new(0),
        effect_in: [const { Dword::new(0) }; 4],
        knock: Dword::new(0),
        knock_in: [const { Dword::new(0) }; 4],
        steer_trim_copy: Dword::new(0),
        effect_request_b: Dword::new(0),
        knock_count: Dword::new(0),
        gear: Dword::new(0),
        last_gear: Dword::new(0),
        gear_notice: Dword::new(0),
        countdown: Dword::new(0),
        drop_latched: Dword::new(1),
        last_direction: Dword::new(0),
        revs_input: Float::new(0.0),
        throttle_input: Float::new(0.0),
        speed_input: Float::new(0.0),
        state: Dword::new(0),
        last_state: Dword::new(0),
        smooth_throttle: Dword::new(0),
        throttle_lead: Dword::new(0),
        turns_up: Dword::new(0),
        direction: Dword::new(0),
        turns_down: Dword::new(0),
        distance: Float::new(0.0),
        approach: Float::new(0.0),
        pitch_factor: Float::new(1.0),
        volume_factor: Float::new(0.0),
        eights_sound: Dword::new(0xFFFFFFFE),
        fours_sound: Dword::new(0xFFFFFFFE),
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
        gear_pitch: Float::new(1.0),
        place_b: [Float::new(0.0), Float::new(0.0), Float::new(0.0)],
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
        hit_place: [Float::new(0.0), Float::new(0.0), Float::new(0.0)],
        cue_sound: Dword::new(0xFFFFFFFE),
        cue_volume: Dword::new(0),
        cue_place: [Float::new(0.0), Float::new(0.0), Float::new(0.0)],
        slot_c: SoundSlot {
            sound: Dword::new(0xE4),
            start: Dword::new(0),
            playing: Dword::new(0),
            ends: Dword::new(0),
            volume: Dword::new(0xFFFFFF91),
            rate: Dword::new(0xFFFFFF91),
        },
        landed: Dword::new(0),
        last_forward_speed: Dword::new(0),
        unused_280: Dword::new(0),
        sum_a: Dword::new(0),
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

/// The race sounds' objects and the cars they follow (IDA 0x512A10 .. 0x514F90).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceSounds {
    /// `g_raceSoundObjects`: the eight objects' addresses as the image names them (IDA's); read by nothing, the objects are reached by number (`objects`).
    pub table: [Dword; 8],
    /// The eight objects (`RaceSounds.objects[0].sound_offset` ..).
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
impl RaceSounds {
    pub const fn new() -> Self {
        RaceSounds {
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
            nearest_distance: Float::new(0.0),
            mark_car_voices: Dword::new(0),
        }
    }
}

/// The race sounds' clock and the constants of the placed and moving sounds (IDA 0x5124E4
/// .. 0x512508).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceClock {
    /// `dword_5124E4`: 1 once the race's sounds are set up (`race::sounds`,
    /// `race::sound_tables`), 0 at the race's end; `sound::race_turn` does nothing before.
    pub started: PlainDword,
    /// `dword_5124E8`: the clock (ms) of the last sound turn.
    pub last: PlainDword,
    /// `dword_5124EC`: sound turns a second (a distance changed in one turn times it is a
    /// speed in units a second).
    pub turns_a_second: PlainFloat,
    /// `dword_5124F0`: the turn's length (ms).
    pub step: PlainDword,
    /// `dword_5124F4`: the turns counted.
    pub turns: PlainDword,
    /// `flt_5124F8` (337.52) and `RaceClock.over_sound_speed` (its inverse, 0.002963): a moving sound's
    /// pitch is (speed of approach + the first) times the second, the Doppler factor
    /// with the speed of sound (the Doppler factor, `sound::turn_parts`).
    pub sound_speed: PlainFloat,
    pub over_sound_speed: PlainFloat,
    /// `flt_512500` (64) and `RaceClock.pan_per_radian` (128 / pi): a placed sound's pan, the centre and
    /// the pan steps per radian of angle.
    pub pan_centre: PlainFloat,
    pub pan_per_radian: PlainFloat,
}

/// The race sounds' clock (`RaceClock`); first values the image's
/// (`tests::race_clock_starts_as_the_games`).
impl RaceClock {
    pub const fn new() -> Self {
        RaceClock {
            started: PlainDword::new(0),
            last: PlainDword::new(0),
            turns_a_second: PlainFloat::new(0.0),
            step: PlainDword::new(0),
            turns: PlainDword::new(0),
            sound_speed: PlainFloat::new(337.52),
            over_sound_speed: PlainFloat::new(0.0029627874),
            pan_centre: PlainFloat::new(64.0),
            pan_per_radian: PlainFloat::new(40.743664),
        }
    }
}

/// The surface sounds' constants and work (IDA 0x516358 .. 0x51639C):
/// the factors the engine, speed, revs and throttle sounds and the
/// wheel sounds scale by, and what `sound::surface_sounds` works out.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SurfaceSounds {
    /// `dword_516358`: 1 once `race::prepare_sound_tables` has run.
    pub tables_done: PlainDword,
    /// `flt_51635C` (0.1): a surface sound's volume is the wheels' sum times it.
    pub volume_scale: PlainFloat,
    /// `flt_516360` (0.1): how far a pitch moves to its target in a turn (the engine's
    /// and the spot voices' Doppler pitch).
    pub pitch_blend: PlainFloat,
    /// `flt_516364` (0.4): the first wheel sum (`SoundObject.sum_a`) is scaled by it.
    pub across_scale: PlainFloat,
    /// `flt_516368` (64): the second wheel sum (`sum_b`, `sum_c`) is scaled by it.
    pub along_scale: PlainFloat,
    /// `flt_51636C` (160): not read by any ported code.
    pub unused_160: PlainFloat,
    /// `flt_516370` (0.014): the car speed's (|`SoundObject.speed`|) factor for the speed sound's
    /// input (`speed_input`).
    pub speed_scale: PlainFloat,
    /// `0x516374` (65536.0): not read by any ported code.
    pub unused_65536: PlainFloat,
    /// `0x516378` (2.0): not read by any ported code.
    pub unused_2: PlainFloat,
    /// `flt_51637C` (1.4): the speed sound's gain.
    pub speed_gain: PlainFloat,
    /// `flt_516380` (0.714): the revs and throttle sounds' gain.
    pub revs_gain: PlainFloat,
    /// `0x516384` (0.625) and `0x516388` (0.25): not read by any ported code.
    pub unused_625: PlainFloat,
    pub unused_quarter: PlainFloat,
    /// `g_wheelSoundAcross`: front less back of the wheel floats (the signs pick the wheel).
    pub across: PlainFloat,
    /// `g_wheelSoundAlong`: left less right of the wheel floats.
    pub along: PlainFloat,
    /// `g_surfaceSoundVolume`: the volume the surface sound is sent with.
    pub volume: PlainFloat,
}

/// The surface sounds' constants and work (`SurfaceSounds`); first values the image's.
impl SurfaceSounds {
    pub const fn new() -> Self {
        SurfaceSounds {
            tables_done: PlainDword::new(0),
            volume_scale: PlainFloat::new(0.1),
            pitch_blend: PlainFloat::new(0.1),
            across_scale: PlainFloat::new(0.4),
            along_scale: PlainFloat::new(64.0),
            unused_160: PlainFloat::new(160.0),
            speed_scale: PlainFloat::new(0.0140625),
            unused_65536: PlainFloat::new(65536.0),
            unused_2: PlainFloat::new(2.0),
            speed_gain: PlainFloat::new(1.4),
            revs_gain: PlainFloat::new(0.71428573),
            unused_625: PlainFloat::new(0.625),
            unused_quarter: PlainFloat::new(0.25),
            across: PlainFloat::new(0.0),
            along: PlainFloat::new(0.0),
            volume: PlainFloat::new(0.0),
        }
    }
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

/// The stage's timed sound slot: `SoundSlot`'s six dwords, plain (the game thread's alone; the
/// race sound objects' slots sit among atomic cells).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct StageSlot {
    /// The sound (a sample number).
    pub sound: PlainDword,
    /// 1 = start it (cleared by the slot's turn).
    pub start: PlainDword,
    /// 1 = playing.
    pub playing: PlainDword,
    /// When it ends (the sound clock, ms).
    pub ends: PlainDword,
    /// The volume (-0x6F: the sample's own).
    pub volume: PlainDword,
    /// The rate (-0x6F: the sample's own).
    pub rate: PlainDword,
}

/// A timed slot's six dwords, whichever kind of cells hold them (`sound::play_timed_slot`).
pub trait TimedCells {
    fn load(&self) -> rc2k_core::sound::cars::TimedSlot;
    fn store(&self, slot: &rc2k_core::sound::cars::TimedSlot);
}

impl TimedCells for SoundSlot {
    fn load(&self) -> rc2k_core::sound::cars::TimedSlot {
        rc2k_core::sound::cars::TimedSlot {
            sound: self.sound.get(),
            start: self.start.get(),
            playing: self.playing.get(),
            ends: self.ends.get(),
            volume: self.volume.get(),
            rate: self.rate.get(),
        }
    }
    fn store(&self, s: &rc2k_core::sound::cars::TimedSlot) {
        self.sound.set(s.sound);
        self.start.set(s.start);
        self.playing.set(s.playing);
        self.ends.set(s.ends);
        self.volume.set(s.volume);
        self.rate.set(s.rate);
    }
}

impl TimedCells for StageSlot {
    fn load(&self) -> rc2k_core::sound::cars::TimedSlot {
        rc2k_core::sound::cars::TimedSlot {
            sound: self.sound.get(),
            start: self.start.get(),
            playing: self.playing.get(),
            ends: self.ends.get(),
            volume: self.volume.get(),
            rate: self.rate.get(),
        }
    }
    fn store(&self, s: &rc2k_core::sound::cars::TimedSlot) {
        self.sound.set(s.sound);
        self.start.set(s.start);
        self.playing.set(s.playing);
        self.ends.set(s.ends);
        self.volume.set(s.volume);
        self.rate.set(s.rate);
    }
}

/// A sound spot shared with the sound side, 0x14 bytes.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SharedSpot {
    /// The spot's id.
    pub id: PlainDword,
    /// Its place.
    pub place: [PlainFloat; 3],
    /// Its kind, 1 .. 0x15.
    pub kind: PlainDword,
}

/// A spot kind's repeats, 8 bytes.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KindRepeats {
    /// Nothing reads it (a gap).
    pub _word_0: PlainWord,
    /// The most repeats (0: no limit).
    pub most: PlainWord,
    /// The least delay before a repeat.
    pub least_delay: PlainWord,
    /// The most delay before a repeat.
    pub most_delay: PlainWord,
}

/// The stage sounds' state and the shared sound spots (IDA 0x51655C .. 0x5168C0).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct StageSounds {
    /// `g_surfaceSoundSet` (`dword_51655C`): a surface's sound set (an index into `off_51639C`'s pairs), 16 surfaces.
    pub surface_sets: [PlainDword; 16],
    /// `g_stageSoundOnce1` .. `3`: at 1 the stage's one-off sound (0x1AB, 0x1AC, 0x1AD) is played once (the flag is cleared).
    pub once: [PlainDword; 3],
    /// `g_stageTimedArm`: at 1 the timed sound is armed (cleared; due and waiting set).
    pub timed_arm: PlainDword,
    /// `g_stageTimedPlace` and the two after it: the timed sound's place.
    pub timed_place: [PlainFloat; 3],
    /// `g_stageTimedDelay`: the delay (ms) from arming to the timed sound.
    pub timed_delay: PlainDword,
    /// `g_stageLoopVolumeA`: the volume (halved) of the sound that plays all the time (0x1C5).
    pub loop_volume_a: PlainDword,
    /// `g_stageLoopVolumeB`: the volume of the second (0x1C6).
    pub loop_volume_b: PlainDword,
    /// `g_stageTimedDue`: the clock the timed sound is due at.
    pub timed_due: PlainDword,
    /// `g_stageTimedWaiting`: 1 while it waits to be due.
    pub timed_waiting: PlainDword,
    /// `g_stageTimedSlot` (`StageSounds.timed_slot.sound`): the timed sound's slot (sound 0x1C4).
    pub timed_slot: StageSlot,
    /// `g_stageRandomSound1`, `g_stageRandomSound2`: at 1 a sound (0x1AE, 0x1AF) at a random pitch and place is played once.
    pub random: [PlainDword; 2],
    /// `g_sharedSoundSpotCount`: the spots the race frame shared this frame (`race::share_sound_spots`).
    pub shared_count: PlainDword,
    /// `g_sharedSoundSpots`: the spots shared with the sound side, 36 of 0x14 bytes.
    pub shared_spots: [SharedSpot; 36],
}

/// The stage sounds (`StageSounds`); first values the image's.
impl StageSounds {
    pub const fn new() -> Self {
        StageSounds {
            surface_sets: [
                PlainDword::new(1),
                PlainDword::new(3),
                PlainDword::new(0),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(4),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(6),
                PlainDword::new(2),
                PlainDword::new(2),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
            ],
            once: [PlainDword::new(0), PlainDword::new(0), PlainDword::new(0)],
            timed_arm: PlainDword::new(0),
            timed_place: [
                PlainFloat::new(0.0),
                PlainFloat::new(2.0),
                PlainFloat::new(1.0),
            ],
            timed_delay: PlainDword::new(0),
            loop_volume_a: PlainDword::new(0),
            loop_volume_b: PlainDword::new(0),
            timed_due: PlainDword::new(0),
            timed_waiting: PlainDword::new(0),
            timed_slot: StageSlot {
                sound: PlainDword::new(0x1C4),
                start: PlainDword::new(0),
                playing: PlainDword::new(0),
                ends: PlainDword::new(0),
                volume: PlainDword::new(0xFFFFFF91),
                rate: PlainDword::new(0xFFFFFF91),
            },
            random: [PlainDword::new(0), PlainDword::new(0)],
            shared_count: PlainDword::new(0),
            shared_spots: [const {
                SharedSpot {
                    id: PlainDword::new(0),
                    place: [
                        PlainFloat::new(0.0),
                        PlainFloat::new(0.0),
                        PlainFloat::new(0.0),
                    ],
                    kind: PlainDword::new(0),
                }
            }; 36],
        }
    }
}

/// The spot voices' pickers' work and the sound kinds per spot (IDA 0x517f40 .. 0x518080).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SpotPicks {
    /// `g_spotPickPlaying`: the pickers' work (`sound::merge_spot_voices`): the playing voice (its number + 1, 0 none).
    pub pick_playing: PlainDword,
    /// `g_spotPickNearestDistance`: the nearest's distance.
    pub pick_nearest_distance: PlainDword,
    /// `g_spotPickNearest`: the nearest voice (as `pick_playing`).
    pub pick_nearest: PlainDword,
    /// `g_spotPickNextDistance`: the next's distance.
    pub pick_next_distance: PlainDword,
    /// `g_spotPickNext`: the next voice (as `pick_playing`).
    pub pick_next: PlainDword,
    /// `g_spotKindSounds` (`dword_517F54`): a spot kind's sound, 22 of them (kind 0 unused).
    pub kind_sounds: [PlainDword; 22],
    /// `g_spotKindRepeats` (`SpotPicks.kind_repeats[0]._word_0`): a kind's repeats, 8 bytes, kind 1 first.
    pub kind_repeats: [KindRepeats; 21],
    /// `dword_518054`: the original cleared it at a race's start (`race::reset_race_sounds`); no ported code reads or writes it (the original's 0x40EC85 read it).
    pub spot_work_a: PlainDword,
    /// `dword_518058`: read by the original at 0x40EC8E; no ported code reads or writes it (0 in the image).
    pub spot_work_b: PlainDword,
}

impl SpotPicks {
    pub const fn new() -> Self {
        SpotPicks {
            pick_playing: PlainDword::new(0),
            pick_nearest_distance: PlainDword::new(0xFFFFFFFF),
            pick_nearest: PlainDword::new(0),
            pick_next_distance: PlainDword::new(0xFFFFFFFF),
            pick_next: PlainDword::new(0),
            kind_sounds: [
                PlainDword::new(0xFFFFFFFE),
                PlainDword::new(0x1B1),
                PlainDword::new(0x1B2),
                PlainDword::new(0x1B3),
                PlainDword::new(0x1B4),
                PlainDword::new(0x1B5),
                PlainDword::new(0x1B6),
                PlainDword::new(0x1B7),
                PlainDword::new(0x1B9),
                PlainDword::new(0x1BA),
                PlainDword::new(0x1BB),
                PlainDword::new(0x1B0),
                PlainDword::new(0xFFFFFFFE),
                PlainDword::new(0x1BD),
                PlainDword::new(0x1BE),
                PlainDword::new(0x1BF),
                PlainDword::new(0x1C0),
                PlainDword::new(0x1C1),
                PlainDword::new(0x1C2),
                PlainDword::new(0x1C3),
                PlainDword::new(0x1B8),
                PlainDword::new(0x1BC),
            ],
            kind_repeats: [
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(1000),
                    most_delay: PlainWord::new(5000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(1),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(3000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(2000),
                    most_delay: PlainWord::new(6000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(500),
                    most_delay: PlainWord::new(3000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(2000),
                    most_delay: PlainWord::new(6000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(1000),
                    most_delay: PlainWord::new(4000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(1),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(1000),
                    most_delay: PlainWord::new(5000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(1000),
                    most_delay: PlainWord::new(4000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(1),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(4000),
                    most_delay: PlainWord::new(8000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(3000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(1000),
                    most_delay: PlainWord::new(5000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
                KindRepeats {
                    _word_0: PlainWord::new(0),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
                KindRepeats {
                    _word_0: PlainWord::new(1),
                    most: PlainWord::new(1),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(5000),
                },
                KindRepeats {
                    _word_0: PlainWord::new(1),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
                KindRepeats {
                    _word_0: PlainWord::new(1),
                    most: PlainWord::new(0),
                    least_delay: PlainWord::new(0),
                    most_delay: PlainWord::new(0),
                },
            ],
            spot_work_a: PlainDword::new(0),
            spot_work_b: PlainDword::new(0),
        }
    }
}

/// Set once the kept sound settings are applied (IDA 0x518080 .. 0x518084).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SettingsApplied {
    /// `dword_518080`: set once the kept sound settings are applied (`sound::apply_settings`, `sound::set_up_race_sounds`).
    pub done: PlainDword,
}

impl SettingsApplied {
    pub const fn new() -> Self {
        SettingsApplied {
            done: PlainDword::new(0),
        }
    }
}

/// The kept sound options (IDA 0x5180f0 .. 0x518104).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SoundOptions {
    /// +0 (0x5180F0) `dword_5180F0`: kept in `var\ral.dat` (`saves::load_options`), 1 at start; read and written by no other code, so what it sets is unknown.
    pub unread_option_f0: Dword,
    /// `dword_5180F4`: the race music on (1) or off: tune 8, the race music, plays only while it is 1 (`sound::play_tune`); the sound page's switch (`OptionSwitch::RaceMusicOn`), kept in `var\ral.dat`.
    pub race_music_on: Dword,
    /// `g_tuneTurn`: which of tunes 10 .. 15 the next tune 0xA is (0 .. 5), kept.
    pub tune_turn: Dword,
    /// `g_musicVolume`, kept.
    pub music_volume: Dword,
    /// `dword_518100`: the race music's volume (the menu sets it with `g_musicVolume`; both go to the tunes), kept.
    pub race_music_volume: Dword,
}

impl SoundOptions {
    pub const fn new() -> Self {
        SoundOptions {
            unread_option_f0: Dword::new(1),
            race_music_on: Dword::new(0),
            tune_turn: Dword::new(0),
            music_volume: Dword::new(0x20),
            race_music_volume: Dword::new(0x20),
        }
    }
}

/// The CD audio's state (IDA 0x518104 .. 0x518120).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CdAudio {
    /// `dword_518104`: `nocdaudio` (`var\ral.cfg`).
    pub no_cd_audio: PlainDword,
    /// `g_cdAudioOpen`: the CD audio is open.
    pub open: PlainDword,
    /// `g_cdTrack`.
    pub track: PlainDword,
    /// `dword_518114`: no code reads or writes it.
    pub work_114: PlainDword,
    /// `mciId`: the CD audio device.
    pub mci_id: PlainDword,
    /// `dword_51811C`: no code reads or writes it.
    pub work_11c: PlainDword,
}

impl CdAudio {
    pub const fn new() -> Self {
        CdAudio {
            no_cd_audio: PlainDword::new(1),
            open: PlainDword::new(0),
            track: PlainDword::new(0),
            work_114: PlainDword::new(0),
            mci_id: PlainDword::new(0),
            work_11c: PlainDword::new(0),
        }
    }
}

// ---- single variables the game's thread keeps ----

/// The car stream's three dwords (0xA8C180 .. 0xA8C18C): the place it is at, its list (a
/// speech row's `texts_store::text_key`; the original kept its address), and
/// whether it is on (`sound::play_car_stream`, `sound::step_car_stream`, the stage
/// view's speech).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CarStream {
    pub place: PlainDword,  // CarStream.place
    pub stream: PlainDword, // CarStream.stream
    pub on: PlainDword,     // CarStream.on
}

impl CarStream {
    pub const fn new() -> Self {
        CarStream {
            place: PlainDword::new(0),
            stream: PlainDword::new(0),
            on: PlainDword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dword(d: &PlainDword) -> Vec<u8> {
        d.get().to_le_bytes().to_vec()
    }

    #[test]
    fn single_variables_start_as_the_games() {
        assert_eq!(
            dword(crate::sound::owner::sound().menu.next_sound_time.get()),
            crate::test_support::first_values::bytes("sound.owner.SOUND.menu.next_sound_time/4")
        );
        assert_eq!(
            dword(crate::sound::owner::sound().race.sounds_on.get()),
            crate::test_support::first_values::bytes("sound.owner.SOUND.race.sounds_on/4")
        );
        let mut stream = Vec::new();
        for d in [
            &crate::sound::owner::sound().race.car_stream.get().place,
            &crate::sound::owner::sound().race.car_stream.get().stream,
            &crate::sound::owner::sound().race.car_stream.get().on,
        ] {
            stream.extend(dword(d));
        }
        assert_eq!(
            stream,
            crate::test_support::first_values::bytes("sound.owner.SOUND.race.car_stream/12")
        );
        assert_eq!(core::mem::size_of::<CarStream>(), 12);
    }

    #[test]
    fn objects_are_by_number_within_the_eight() {
        assert!(SoundObject::of(crate::sound::owner::sound(), 7).is_ok());
        assert!(SoundObject::of(crate::sound::owner::sound(), 8).is_err());
    }

    #[test]
    fn wheel_surfaces_start_as_the_games() {
        let image = crate::test_support::first_values::bytes("sound.store.WHEEL_SURFACES/8");
        let mine: Vec<u8> = WHEEL_SURFACES
            .get()
            .iter()
            .flat_map(|cell| cell.get().to_le_bytes())
            .collect();
        assert_eq!(mine, image);
    }

    #[test]
    fn stage_sounds_start_as_the_games() {
        assert_eq!(size_of::<super::StageSounds>(), 0x364);
    }

    #[test]
    fn spot_picks_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::sound::owner::SOUND.race.spot_picks.get(),
        );
    }

    #[test]
    fn settings_applied_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::sound::owner::SOUND.menu.settings_applied.get(),
        );
    }

    #[test]
    fn sound_options_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::sound::owner::SOUND.menu.options,
        );
    }

    #[test]
    fn cd_audio_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::sound::owner::SOUND.menu.cd_audio.get(),
        );
    }

    #[test]
    fn race_clock_starts_as_the_games() {
        assert_eq!(size_of::<super::RaceClock>(), 0x24);
    }
}
