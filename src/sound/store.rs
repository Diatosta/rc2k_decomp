//! The sound module's variables, typed: each is a struct of atomic cells with the game's starting
//! values (`tests::*_start_as_the_games`), one static a piece. The race sounds' objects and the
//! surface sounds hold their fields with nothing between them, as `sound::asm_layout` lays them out
//! for the kept assembly's tests.
//! - `WHEEL_SURFACES`: the two wheel surfaces the sound reads;
//! - `RaceClock`: the race sounds' clock and constants;
//! - `RaceSounds`: the eight sound objects (`SoundObject`) and the cars the sound follows;
//! - `SurfaceSounds`: the factors the engine, speed, revs and throttle sounds scale by, the
//!   wheel sounds across and along, the surface sound volume;
//! - `StageSounds`: the surface sound sets, the stage's one-off, timed, loop and random sounds, and
//!   the spots shared with the sound side;
//! - `SpotPicks`: the spot-voice picks and the sound kinds per spot;
//! - `SettingsApplied`, `SoundOptions` (the kept options, without the tune turn) and `CdAudio`. The
//!   force-feedback effects' state is `input::store::ForceFeedback`'s.
//!
//! The sound commands go to the audio thread through `audio::store`'s queue; these are the
//! game thread's.

use crate::session::{Dword, GameThread, PlainDword, PlainFloat, PlainWord};

/// The surfaces of the wheels the first and the second of a car's two sounds follow
/// (`sound::surface_sounds`). Kept apart from the dwords around them: the save file's four
/// before (`saves::owner::Files::options_kept`), the race record's seal key after
/// (`race::replay::SEAL_KEY`).
pub static WHEEL_SURFACES: GameThread<[PlainDword; 2]> =
    GameThread::new([PlainDword::new(0), PlainDword::new(0)]);

/// One cue of a car's queue: eight dwords.
#[derive(rc2k_derive::Cells)]
pub struct CueEntry {
    /// The cue's first sound.
    pub first: PlainDword,
    /// Its sounds.
    pub count: PlainDword,
    /// The sounds.
    pub sounds: [PlainDword; 6],
}

/// A car's queue of spoken cues, 0x210 bytes.
#[derive(rc2k_derive::Cells)]
pub struct CueQueue {
    /// The first cue queued.
    pub first: PlainDword,
    /// The place in the first cue (-1: none begun).
    pub place: PlainDword,
    /// When the next sound is due.
    pub due: PlainDword,
    /// The cues queued.
    pub count: PlainDword,
    /// The cues.
    pub cues: [CueEntry; 16],
}

/// One of a car's four wheel records in its sound object: five dwords.
#[derive(rc2k_derive::Cells)]
pub struct EngineVoice {
    /// A share the physics sets (`physics::car_sound_state`, or 0); the first wheel sum (`sum_a`)
    /// scales these by 0.4.
    pub share_a: PlainFloat,
    /// 1.0, or the physics' other share (`sound::surface_sounds` reads these: front less back and
    /// left less right, and their sum x 64).
    pub share_b: PlainFloat,
    /// The share x 256.
    pub share_c: PlainFloat,
    /// The surface the wheel runs on.
    pub surface: PlainDword,
}

/// A race sound object: a car's sound state (the revs, the throttle, the place, the timed sound
/// slots) as `sound::car_sounds` and the physics keep it.
#[derive(rc2k_derive::Cells)]
pub struct SoundObject {
    /// Added to a sound set's first sound to pick the car's sound; negative for a computer's
    /// car (the sets are picked in `sound::revs_sound` and `surface_sound`).
    pub sound_offset: PlainDword,
    /// The car's sound kind, an index into `SoundKinds.kinds[0]` (36 of 0x88 bytes).
    pub kind: PlainDword,
    /// 1 when the object is in use (`sound::car_sounds_all`).
    pub in_use: PlainDword,
    /// 0 for the view of the car itself, 1 otherwise; the engine, throttle and speed sounds'
    /// volume is scaled by a constant when it is 0 (`sound::revs_sound`, `throttle_sound`,
    /// `car_slot_sounds`: 1 gets the surface's gain instead).
    pub view_flag: PlainDword,
    /// 1 when a new revs reading (`revs`) came (`sound::car_engine_state`).
    pub new_revs: PlainDword,
    /// The revs (extrapolated between the physics' readings).
    pub revs: PlainDword,
    /// The throttle, 0 ..= 0x100.
    pub throttle: PlainDword,
    /// The car's speed.
    pub speed: PlainFloat,
    /// Set by the physics when the gear changed (cleared by `sound::car_engine_state`).
    pub gear_event: PlainDword,
    /// 1 when the car is in neutral (gear 0, or the gear cue's random bits 0:
    /// `sound::car_engine_state`); 1 from the set-up.
    pub no_gear: PlainDword,
    /// The car's place less the listener's, turned by the view and scaled
    /// (`physics::car_sound_state`).
    pub place: [PlainFloat; 3],
    /// The car's motion, turned by the view.
    pub motion: [PlainFloat; 3],
    /// Four records of five dwords (one per wheel).
    pub voices: [EngineVoice; 4],
    /// The times of the last two revs readings.
    pub reading_times: [PlainDword; 2],
    /// The revs of the last two readings.
    pub reading_revs: [PlainDword; 2],
    /// Two dwords cleared by the set-up (`race::set_up_race_sound_objects`); nothing else uses
    /// them.
    pub unused_a4: [PlainDword; 2],
    /// Set by the physics; the speed sound stops when it is not 0.
    pub flag_ac: PlainDword,
    /// The force feedback's inputs (`input::update_car_effects`), from the view block's sideways
    /// speed, forward speed and driven wheels' speed: sideways x 10, (driven - forward) x 10,
    /// sideways x 10, forward x 10.
    pub effect_in: [PlainDword; 4],
    /// The knock's force (`input::update_car_effects`); cleared by the set-up.
    pub knock: PlainDword,
    /// The knock's inputs (cleared after each use; `knock_in[2]` and `knock_in[0]` give the angle,
    /// `knock_in[3]` the weight).
    pub knock_in: [PlainFloat; 4],
    /// A copy of the car's steering trim made by `physics::share_engine_sound`; nothing reads it.
    pub steer_trim_copy: PlainDword,
    /// The `b` number of the Fourth, Steering and Landing force-feedback requests (the request
    /// word's bits 20 ..); read only by `input::update_car_effects`, never written, so 0.
    pub effect_request_b: PlainDword,
    /// Counts the knocks to 4 before the next effect 3.
    pub knock_count: PlainDword,
    /// The gear.
    pub gear: PlainDword,
    /// The gear when it last changed.
    pub last_gear: PlainDword,
    /// Set with a gear change.
    pub gear_notice: PlainDword,
    /// Counts down to 0; the speed sound waits for it.
    pub countdown: PlainDword,
    /// 1 once the throttle-drop sound has been triggered in this downward run of the throttle
    /// (set when the throttle falls more than 0x14 below its follower, cleared when it rises;
    /// `rc2k_core::sound::engine`), so it plays once per lift; 1 from the set-up.
    pub drop_latched: PlainDword,
    /// The last direction of the revs.
    pub last_direction: PlainDword,
    /// The revs sound's input, (`revs` less the kind's `revs_low`) x its `revs_scale`.
    pub revs_input: PlainFloat,
    /// The throttle sound's input, `throttle` over 256.
    pub throttle_input: PlainFloat,
    /// The speed sound's input, |`speed`| x `SurfaceSounds.speed_scale`.
    pub speed_input: PlainFloat,
    /// 0 (no revs), 1 (up to 1000), 2, or 3 when `gear_event` was set.
    pub state: PlainDword,
    /// The last state.
    pub last_state: PlainDword,
    /// Follows `throttle`.
    pub smooth_throttle: PlainDword,
    /// The throttle's lead over its follower, `throttle` - `smooth_throttle` (negative on a
    /// lift; a gear cue needs 0x32 or more, the drop sound -0x14 or less).
    pub throttle_lead: PlainDword,
    /// The turns going up.
    pub turns_up: PlainDword,
    /// 0, 1 up, 2 down.
    pub direction: PlainDword,
    /// The turns going down.
    pub turns_down: PlainDword,
    /// The listener's distance (the root of the squares of `place`).
    pub distance: PlainFloat,
    /// Its change x the turns a second.
    pub approach: PlainFloat,
    /// The Doppler-like pitch factor, eased.
    pub pitch_factor: PlainFloat,
    /// 1, or 1 / the distance when that is over 1.
    pub volume_factor: PlainFloat,
    /// The sound playing from the surface sets of eight (-2: none; `sound::surface_sound`).
    pub eights_sound: PlainDword,
    /// The sound playing from the surface sets of four (-2: none).
    pub fours_sound: PlainDword,
    /// Seven timed sound slots (`sound::car_slot_sounds` the first five, `sound::car_gear_sounds`
    /// the last two: the sixth armed by `sound::car_engine_state` when the throttle drops at high
    /// revs, the seventh by its countdown at low throttle).
    pub slots: [SoundSlot; 7],
    /// Counts down (back-fires).
    pub backfire_wait: PlainDword,
    /// The gear change's slot (pitch `gear_pitch`).
    pub gear_slot: SoundSlot,
    /// The gear change sound's pitch factor.
    pub gear_pitch: PlainFloat,
    /// The place of `slot_b`.
    pub place_b: [PlainFloat; 3],
    /// A timed slot.
    pub slot_b: SoundSlot,
    /// The hit sound to send once (-2: none), set by the physics.
    pub hit_sound: PlainDword,
    /// Its volume.
    pub hit_volume: PlainDword,
    /// Its place.
    pub hit_place: [PlainFloat; 3],
    /// The second one-off sound (-2: none), set by the physics' cues.
    pub cue_sound: PlainDword,
    /// Its volume.
    pub cue_volume: PlainDword,
    /// Its place.
    pub cue_place: [PlainFloat; 3],
    /// A timed slot.
    pub slot_c: SoundSlot,
    /// 1 after a landing's effects (`input::update_car_effects`).
    pub landed: PlainDword,
    /// The last forward speed x 10 (`effect_in[3]`), kept for the next turn's speed-change
    /// effect (`input::update_car_effects`); cleared by the set-up.
    pub last_forward_speed: PlainDword,
    /// Cleared by the set-up; nothing else reads or writes it.
    pub unused_280: PlainDword,
    /// The first wheel sum x 0.4 (a whole number).
    pub sum_a: PlainDword,
    /// The second wheel sum x 64 (a whole number).
    pub sum_b: PlainDword,
    /// The other wheels' sum x 64 (a whole number).
    pub sum_c: PlainDword,
    /// The surface the first wheel runs on.
    pub surface: PlainDword,
    /// The queue of spoken cues (`sound::play_cue_queue`; `physics::queue_sound_cue` fills it),
    /// 0x210 bytes.
    pub cue_queue: CueQueue,
}

/// The current car's sound object (the current car's number modulo eight).
pub fn current_object<'a>(
    sound: &'a crate::sound::owner::Sound,
    work: &crate::race::work_store::RaceWork,
) -> &'a SoundObject {
    &sound.race.sounds.get().objects[(work.current_car.get() & 7) as usize]
}

impl SoundObject {
    /// Race sound object `n`, refused past the eight.
    pub fn of(
        sound: &crate::sound::owner::Sound,
        n: u32,
    ) -> Result<&SoundObject, crate::errors::BadInput> {
        sound
            .race
            .sounds
            .get()
            .objects
            .get(n as usize)
            .ok_or(crate::errors::BadInput(
                "a race sound object past the eight",
            ))
    }

    /// The timed slot at a byte offset in the 1999 object layout (the core's lists of slots are
    /// by offset). Any other offset is a bug of the caller.
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

/// A sound object as the game starts it, with the car's `sound_offset` (all eight have the
/// kind 0x11 at the start).
pub(crate) const fn initial_object(sound_offset: u32) -> SoundObject {
    SoundObject {
        sound_offset: PlainDword::new(sound_offset),
        kind: PlainDword::new(0x11),
        in_use: PlainDword::new(0),
        view_flag: PlainDword::new(0),
        new_revs: PlainDword::new(0),
        revs: PlainDword::new(0x3E8),
        throttle: PlainDword::new(0),
        speed: PlainFloat::new(0.0),
        gear_event: PlainDword::new(0),
        no_gear: PlainDword::new(0),
        place: [
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
        ],
        motion: [
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
        ],
        voices: [const {
            EngineVoice {
                share_a: PlainFloat::new(0.0),
                share_b: PlainFloat::new(0.0),
                share_c: PlainFloat::new(0.0),
                surface: PlainDword::new(0),
            }
        }; 4],
        reading_times: [PlainDword::new(0), PlainDword::new(0)],
        reading_revs: [PlainDword::new(0), PlainDword::new(0)],
        unused_a4: [PlainDword::new(0), PlainDword::new(0)],
        flag_ac: PlainDword::new(0),
        effect_in: [const { PlainDword::new(0) }; 4],
        knock: PlainDword::new(0),
        knock_in: [const { PlainFloat::new(0.0) }; 4],
        steer_trim_copy: PlainDword::new(0),
        effect_request_b: PlainDword::new(0),
        knock_count: PlainDword::new(0),
        gear: PlainDword::new(0),
        last_gear: PlainDword::new(0),
        gear_notice: PlainDword::new(0),
        countdown: PlainDword::new(0),
        drop_latched: PlainDword::new(1),
        last_direction: PlainDword::new(0),
        revs_input: PlainFloat::new(0.0),
        throttle_input: PlainFloat::new(0.0),
        speed_input: PlainFloat::new(0.0),
        state: PlainDword::new(0),
        last_state: PlainDword::new(0),
        smooth_throttle: PlainDword::new(0),
        throttle_lead: PlainDword::new(0),
        turns_up: PlainDword::new(0),
        direction: PlainDword::new(0),
        turns_down: PlainDword::new(0),
        distance: PlainFloat::new(0.0),
        approach: PlainFloat::new(0.0),
        pitch_factor: PlainFloat::new(1.0),
        volume_factor: PlainFloat::new(0.0),
        eights_sound: PlainDword::new(0xFFFFFFFE),
        fours_sound: PlainDword::new(0xFFFFFFFE),
        slots: [
            SoundSlot {
                sound: PlainDword::new(0xD6),
                start: PlainDword::new(0),
                playing: PlainDword::new(0),
                ends: PlainDword::new(0),
                volume: PlainDword::new(0xFFFFFF91),
                rate: PlainDword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: PlainDword::new(0xD5),
                start: PlainDword::new(0),
                playing: PlainDword::new(0),
                ends: PlainDword::new(0),
                volume: PlainDword::new(0xFFFFFF91),
                rate: PlainDword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: PlainDword::new(0xD3),
                start: PlainDword::new(0),
                playing: PlainDword::new(0),
                ends: PlainDword::new(0),
                volume: PlainDword::new(0xFFFFFF91),
                rate: PlainDword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: PlainDword::new(0xD4),
                start: PlainDword::new(0),
                playing: PlainDword::new(0),
                ends: PlainDword::new(0),
                volume: PlainDword::new(0xFFFFFF91),
                rate: PlainDword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: PlainDword::new(0xD2),
                start: PlainDword::new(0),
                playing: PlainDword::new(0),
                ends: PlainDword::new(0),
                volume: PlainDword::new(0xFFFFFF91),
                rate: PlainDword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: PlainDword::new(0xEB),
                start: PlainDword::new(0),
                playing: PlainDword::new(0),
                ends: PlainDword::new(0),
                volume: PlainDword::new(0xFFFFFF91),
                rate: PlainDword::new(0xFFFFFF91),
            },
            SoundSlot {
                sound: PlainDword::new(0xE9),
                start: PlainDword::new(0),
                playing: PlainDword::new(0),
                ends: PlainDword::new(0),
                volume: PlainDword::new(0xFFFFFF91),
                rate: PlainDword::new(0xFFFFFF91),
            },
        ],
        backfire_wait: PlainDword::new(0),
        gear_slot: SoundSlot {
            sound: PlainDword::new(0xE8),
            start: PlainDword::new(0),
            playing: PlainDword::new(0),
            ends: PlainDword::new(0),
            volume: PlainDword::new(0xFFFFFF91),
            rate: PlainDword::new(0xFFFFFF91),
        },
        gear_pitch: PlainFloat::new(1.0),
        place_b: [
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
        ],
        slot_b: SoundSlot {
            sound: PlainDword::new(0xE3),
            start: PlainDword::new(0),
            playing: PlainDword::new(0),
            ends: PlainDword::new(0),
            volume: PlainDword::new(0xFFFFFF91),
            rate: PlainDword::new(0xFFFFFF91),
        },
        hit_sound: PlainDword::new(0xFFFFFFFE),
        hit_volume: PlainDword::new(0),
        hit_place: [
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
        ],
        cue_sound: PlainDword::new(0xFFFFFFFE),
        cue_volume: PlainDword::new(0),
        cue_place: [
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
            PlainFloat::new(0.0),
        ],
        slot_c: SoundSlot {
            sound: PlainDword::new(0xE4),
            start: PlainDword::new(0),
            playing: PlainDword::new(0),
            ends: PlainDword::new(0),
            volume: PlainDword::new(0xFFFFFF91),
            rate: PlainDword::new(0xFFFFFF91),
        },
        landed: PlainDword::new(0),
        last_forward_speed: PlainDword::new(0),
        unused_280: PlainDword::new(0),
        sum_a: PlainDword::new(0),
        sum_b: PlainDword::new(0),
        sum_c: PlainDword::new(0),
        surface: PlainDword::new(0),
        cue_queue: CueQueue {
            first: PlainDword::new(0),
            place: PlainDword::new(0xFFFFFFFF),
            due: PlainDword::new(0),
            count: PlainDword::new(0),
            cues: [const {
                CueEntry {
                    first: PlainDword::new(0),
                    count: PlainDword::new(0),
                    sounds: [const { PlainDword::new(0) }; 6],
                }
            }; 16],
        },
    }
}

/// The race sounds' objects and the cars they follow.
#[derive(rc2k_derive::Cells)]
pub struct RaceSounds {
    /// The 1999 addresses of the eight objects; nothing reads them, the objects are reached by
    /// number (`objects`).
    pub table: [PlainDword; 8],
    /// The eight objects (`RaceSounds.objects[0].sound_offset` ..).
    pub objects: [SoundObject; 8],
    /// The object numbers of the first and second player's cars.
    pub player_cars: [PlainDword; 2],
    /// The nearest computer car (an object number, -1 for none).
    pub nearest: PlainDword,
    /// The next nearest.
    pub next: PlainDword,
    /// The squared distance `sound::car_sounds_all` works out.
    pub nearest_distance: PlainFloat,
    /// 1 when the cars' voices are marked (`sound::mark_car_voices`).
    pub mark_car_voices: PlainDword,
}

/// The two race sound values the input thread reads (its force-feedback effects follow the
/// objects in use and the player count); the game's thread writes them, the rest of the race
/// sounds stays with it alone (`RaceSounds`).
#[derive(rc2k_derive::Cells)]
pub struct SoundShared {
    /// 1 when one player races.
    pub one_player: Dword,
    /// The objects in use.
    pub count: Dword,
}

impl SoundShared {
    pub const fn new() -> Self {
        SoundShared {
            one_player: Dword::new(1),
            count: Dword::new(1),
        }
    }
}

/// The race sounds' objects (`RaceSounds`), starting as the game does.
impl RaceSounds {
    pub const fn new() -> Self {
        RaceSounds {
            table: [
                PlainDword::new(0x512A30),
                PlainDword::new(0x512ED8),
                PlainDword::new(0x513380),
                PlainDword::new(0x513828),
                PlainDword::new(0x513CD0),
                PlainDword::new(0x514178),
                PlainDword::new(0x514620),
                PlainDword::new(0x514AC8),
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
            player_cars: [PlainDword::new(0), PlainDword::new(1)],
            nearest: PlainDword::new(0),
            next: PlainDword::new(0),
            nearest_distance: PlainFloat::new(0.0),
            mark_car_voices: PlainDword::new(0),
        }
    }
}

/// The race sounds' clock and the constants of the placed and moving sounds. Laid out as the
/// 1999 variables: the recorded assembly's tests lay it over raw bytes.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct RaceClock {
    /// 1 once the race's sounds are set up (`race::sounds`, `race::sound_tables`), 0 at the race's
    /// end; `sound::race_turn` does nothing before.
    pub started: PlainDword,
    /// The clock (ms) of the last sound turn.
    pub last: PlainDword,
    /// Sound turns a second (a distance changed in one turn times it is a speed in units a second).
    pub turns_a_second: PlainFloat,
    /// The turn's length (ms).
    pub step: PlainDword,
    /// The turns counted.
    pub turns: PlainDword,
    /// 337.52 and `RaceClock.over_sound_speed` (its inverse, 0.002963): a moving sound's pitch
    /// is (speed of approach + the first) times the second, the Doppler factor with the speed
    /// of sound (`sound::turn_parts`).
    pub sound_speed: PlainFloat,
    pub over_sound_speed: PlainFloat,
    /// 64 and `RaceClock.pan_per_radian` (128 / pi): a placed sound's pan, the centre and the
    /// pan steps per radian of angle.
    pub pan_centre: PlainFloat,
    pub pan_per_radian: PlainFloat,
}

/// The race sounds' clock (`RaceClock`), starting as the game does
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

/// The surface sounds' constants and work: the factors the engine, speed, revs and throttle sounds
/// and the wheel sounds scale by, and what `sound::surface_sounds` works out.
#[derive(rc2k_derive::Cells)]
pub struct SurfaceSounds {
    /// 1 once `race::prepare_sound_tables` has run.
    pub tables_done: PlainDword,
    /// 0.1: a surface sound's volume is the wheels' sum times it.
    pub volume_scale: PlainFloat,
    /// 0.1: how far a pitch moves to its target in a turn (the engine's and the spot voices'
    /// Doppler pitch).
    pub pitch_blend: PlainFloat,
    /// 0.4: the first wheel sum (`SoundObject.sum_a`) is scaled by it.
    pub across_scale: PlainFloat,
    /// 64: the second wheel sum (`sum_b`, `sum_c`) is scaled by it.
    pub along_scale: PlainFloat,
    /// 160; nothing reads it.
    pub unused_160: PlainFloat,
    /// 0.014: the car speed's (|`SoundObject.speed`|) factor for the speed sound's input
    /// (`speed_input`).
    pub speed_scale: PlainFloat,
    /// 65536.0; nothing reads it.
    pub unused_65536: PlainFloat,
    /// 2.0; nothing reads it.
    pub unused_2: PlainFloat,
    /// 1.4: the speed sound's gain.
    pub speed_gain: PlainFloat,
    /// 0.714: the revs and throttle sounds' gain.
    pub revs_gain: PlainFloat,
    /// 0.625 and 0.25; nothing reads them.
    pub unused_625: PlainFloat,
    pub unused_quarter: PlainFloat,
    /// Front less back of the wheel floats (the signs pick the wheel).
    pub across: PlainFloat,
    /// Left less right of the wheel floats.
    pub along: PlainFloat,
    /// The volume the surface sound is sent with.
    pub volume: PlainFloat,
}

/// The surface sounds' constants and work (`SurfaceSounds`), starting as the game does.
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

/// A timed sound's slot (`sound::play_timed_slot`): a car has ten, the stage one.
#[derive(rc2k_derive::Cells)]
pub struct SoundSlot {
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

/// A timed slot's six dwords (`sound::play_timed_slot`).
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

/// A sound spot shared with the sound side, 0x14 bytes.
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

/// The stage sounds' state and the shared sound spots.
#[derive(rc2k_derive::Cells)]
pub struct StageSounds {
    /// A surface's sound set (an index into `SurfaceSets.sets`' pairs), 16 surfaces.
    pub surface_sets: [PlainDword; 16],
    /// At 1 the stage's one-off sound is played once (the flag is cleared).
    pub once: [PlainDword; 3],
    /// At 1 the timed sound is armed (cleared; due and waiting set).
    pub timed_arm: PlainDword,
    /// The timed sound's place.
    pub timed_place: [PlainFloat; 3],
    /// The delay (ms) from arming to the timed sound.
    pub timed_delay: PlainDword,
    /// The volume (halved) of the sound that plays all the time.
    pub loop_volume_a: PlainDword,
    /// The volume of the second.
    pub loop_volume_b: PlainDword,
    /// The clock the timed sound is due at.
    pub timed_due: PlainDword,
    /// 1 while it waits to be due.
    pub timed_waiting: PlainDword,
    /// The timed sound's slot (sound 0x1C4).
    pub timed_slot: SoundSlot,
    /// At 1 a sound at a random pitch and place is played once.
    pub random: [PlainDword; 2],
    /// The spots the race frame shared this frame (`race::share_sound_spots`).
    pub shared_count: PlainDword,
    /// The spots shared with the sound side, 36 of 0x14 bytes.
    pub shared_spots: [SharedSpot; 36],
}

/// The stage sounds (`StageSounds`), starting as the game does.
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
            timed_slot: SoundSlot {
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

/// The spot voices' pickers' work and the sound kinds per spot.
#[derive(rc2k_derive::Cells)]
pub struct SpotPicks {
    /// The pickers' work (`sound::merge_spot_voices`): the playing voice (its number + 1, 0 none).
    pub pick_playing: PlainDword,
    /// The nearest's distance.
    pub pick_nearest_distance: PlainDword,
    /// The nearest voice (as `pick_playing`).
    pub pick_nearest: PlainDword,
    /// The next's distance.
    pub pick_next_distance: PlainDword,
    /// The next voice (as `pick_playing`).
    pub pick_next: PlainDword,
    /// A spot kind's sound, 22 of them (kind 0 unused).
    pub kind_sounds: [PlainDword; 22],
    /// A kind's repeats, 8 bytes, kind 1 first.
    pub kind_repeats: [KindRepeats; 21],
    /// Cleared at a race's start (`race::reset_race_sounds`); nothing reads it.
    pub spot_work_a: PlainDword,
    /// Nothing reads or writes it (0).
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

/// Set once the kept sound settings are applied.
#[derive(rc2k_derive::Cells)]
pub struct SettingsApplied {
    /// Set by `sound::apply_settings` and `sound::set_up_race_sounds`.
    pub done: PlainDword,
}

impl SettingsApplied {
    pub const fn new() -> Self {
        SettingsApplied {
            done: PlainDword::new(0),
        }
    }
}

/// The kept sound options. The tune turn is the audio thread's too and lives in
/// `audio::store::AudioShared`.
#[derive(rc2k_derive::Cells)]
pub struct SoundOptions {
    /// Kept in `var\ral.dat` (`saves::load_options`), 1 at start; no other code uses it, so what
    /// it sets is unknown.
    pub unread_option_f0: Dword,
    /// The race music on (1) or off: tune 8, the race music, plays only while it is 1
    /// (`sound::play_tune`); the sound page's switch (`OptionSwitch::RaceMusicOn`), kept in
    /// `var\ral.dat`.
    pub race_music_on: Dword,
    /// The music volume, kept.
    pub music_volume: Dword,
    /// The race music's volume (the menu sets it with the music volume; both go to the tunes),
    /// kept.
    pub race_music_volume: Dword,
}

impl SoundOptions {
    pub const fn new() -> Self {
        SoundOptions {
            unread_option_f0: Dword::new(1),
            race_music_on: Dword::new(0),
            music_volume: Dword::new(0x20),
            race_music_volume: Dword::new(0x20),
        }
    }
}

/// The CD audio's state.
#[derive(rc2k_derive::Cells)]
pub struct CdAudio {
    /// `nocdaudio` (`var\ral.cfg`).
    pub no_cd_audio: PlainDword,
    /// The CD audio is open.
    pub open: PlainDword,
    /// The CD track.
    pub track: PlainDword,
    /// Nothing reads or writes it.
    pub work_114: PlainDword,
    /// The CD audio device.
    pub mci_id: PlainDword,
    /// Nothing reads or writes it.
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

/// The car stream's three dwords: the place it is at, its list (a speech row's
/// `texts_store::text_key`), and whether it is on (`sound::play_car_stream`,
/// `sound::step_car_stream`, the stage view's speech).
#[derive(rc2k_derive::Cells)]
pub struct CarStream {
    pub place: PlainDword,
    pub stream: PlainDword,
    pub on: PlainDword,
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

    #[test]
    fn objects_are_by_number_within_the_eight() {
        assert!(SoundObject::of(crate::test_support::own::sound(), 7).is_ok());
        assert!(SoundObject::of(crate::test_support::own::sound(), 8).is_err());
    }
}
