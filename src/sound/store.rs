//! The sound module's variables, typed: each is a struct with the game's starting values
//! (`tests::*_start_as_the_games`), one static a piece. The race sounds' objects are plain
//! structs, each in a `RefCell` borrowed for a statement at a time (never across a sound send);
//! `sound::asm_layout` lays them out as the 1999 program had them, for the kept assembly's tests.
//! - `RaceClock`: the race sounds' clock and constants;
//! - `RaceSounds`: the eight sound objects (`SoundObject`) and the cars the sound follows;
//! - `SurfaceSounds`: the factors the engine, speed, revs and throttle sounds scale by, the
//!   wheel sounds across and along, the surface sound volume;
//! - `StageSounds`: the surface sound sets, the stage's one-off, timed, loop and random sounds, and
//!   the spots shared with the sound side;
//! - `SpotKinds`: the spot-voice picks and the sound kinds per spot;
//! - `SoundOptions` (the kept options, without the tune turn). The
//!   force-feedback effects' state is `input::store::ForceFeedback`'s.
//!
//! The sound commands go to the audio thread through `audio::store`'s queue; these are the
//! game thread's.

use crate::memory::session::{Dword, PlainDword, PlainFloat};
use rc2k_core::sound::cars::CueQueue;
use std::cell::RefCell;

/// One of a car's four wheel records in its sound object: five dwords.
#[derive(Clone, Copy, Default)]
pub struct EngineVoice {
    /// A share the physics sets (`physics::car_sound_state`, or 0); the first wheel sum (`sum_a`)
    /// scales these by 0.4.
    pub share_a: f32,
    /// 1.0, or the physics' other share (`sound::surface_sounds` reads these: front less back and
    /// left less right, and their sum x 64).
    pub share_b: f32,
    /// The share x 256.
    pub share_c: f32,
    /// The surface the wheel runs on.
    pub surface: u32,
}

/// A race sound object: a car's sound state (the revs, the throttle, the place, the timed sound
/// slots) as `sound::car_sounds` and the physics keep it.
#[derive(Clone)]
pub struct SoundObject {
    /// Added to a sound set's first sound to pick the car's sound; negative for a computer's
    /// car (the sets are picked in `sound::revs_sound` and `surface_sound`).
    pub sound_offset: u32,
    /// The car's sound kind, an index into `SoundKinds.kinds[0]` (36 of 0x88 bytes).
    pub kind: u32,
    /// 1 when the object is in use (`sound::car_sounds_all`).
    pub in_use: u32,
    /// 0 for the view of the car itself, 1 otherwise; the engine, throttle and speed sounds'
    /// volume is scaled by a constant when it is 0 (`sound::revs_sound`, `throttle_sound`,
    /// `car_slot_sounds`: 1 gets the surface's gain instead).
    pub view_flag: u32,
    /// 1 when a new revs reading (`revs`) came (`sound::car_engine_state`).
    pub new_revs: u32,
    /// The revs (extrapolated between the physics' readings).
    pub revs: u32,
    /// The throttle, 0 ..= 0x100.
    pub throttle: u32,
    /// The car's speed.
    pub speed: f32,
    /// Set by the physics when the gear changed (cleared by `sound::car_engine_state`).
    pub gear_event: u32,
    /// 1 when the car is in neutral (gear 0, or the gear cue's random bits 0:
    /// `sound::car_engine_state`); 1 from the set-up.
    pub no_gear: u32,
    /// The car's place less the listener's, turned by the view and scaled
    /// (`physics::car_sound_state`).
    pub place: [f32; 3],
    /// The car's motion, turned by the view.
    pub motion: [f32; 3],
    /// Four records of five dwords (one per wheel).
    pub voices: [EngineVoice; 4],
    /// The times of the last two revs readings.
    pub reading_times: [u32; 2],
    /// The revs of the last two readings.
    pub reading_revs: [u32; 2],
    /// Two dwords cleared by the set-up (`race::set_up_race_sound_objects`); nothing else uses
    /// them.
    pub reserved_a4: [u32; 2],
    /// Set by the physics; the speed sound stops when it is not 0.
    pub flag_ac: u32,
    /// The force feedback's inputs (`input::update_car_effects`), from the view block's sideways
    /// speed, forward speed and driven wheels' speed: sideways x 10, (driven - forward) x 10,
    /// sideways x 10, forward x 10.
    pub effect_in: [u32; 4],
    /// The knock's force (`input::update_car_effects`); cleared by the set-up.
    pub knock: u32,
    /// The knock's inputs (cleared after each use; `knock_in[2]` and `knock_in[0]` give the angle,
    /// `knock_in[3]` the weight).
    pub knock_in: [f32; 4],
    /// A copy of the car's steering trim made by `physics::share_engine_sound`; nothing reads it.
    pub steer_trim_copy: u32,
    /// The `b` number of the Fourth, Steering and Landing force-feedback requests (the request
    /// word's bits 20 ..); read only by `input::update_car_effects`, never written, so 0.
    pub effect_request_b: u32,
    /// Counts the knocks to 4 before the next effect 3.
    pub knock_count: u32,
    /// The gear.
    pub gear: u32,
    /// The gear when it last changed.
    pub last_gear: u32,
    /// Set with a gear change.
    pub gear_notice: u32,
    /// Counts down to 0; the speed sound waits for it.
    pub countdown: u32,
    /// 1 once the throttle-drop sound has been triggered in this downward run of the throttle
    /// (set when the throttle falls more than 0x14 below its follower, cleared when it rises;
    /// `rc2k_core::sound::engine`), so it plays once per lift; 1 from the set-up.
    pub drop_latched: u32,
    /// The last direction of the revs.
    pub last_direction: u32,
    /// The revs sound's input, (`revs` less the kind's `revs_low`) x its `revs_scale`.
    pub revs_input: f32,
    /// The throttle sound's input, `throttle` over 256.
    pub throttle_input: f32,
    /// The speed sound's input, |`speed`| x `SurfaceSounds.speed_scale`.
    pub speed_input: f32,
    /// 0 (no revs), 1 (up to 1000), 2, or 3 when `gear_event` was set.
    pub state: u32,
    /// The last state.
    pub last_state: u32,
    /// Follows `throttle`.
    pub smooth_throttle: u32,
    /// The throttle's lead over its follower, `throttle` - `smooth_throttle` (negative on a
    /// lift; a gear cue needs 0x32 or more, the drop sound -0x14 or less).
    pub throttle_lead: u32,
    /// The turns going up.
    pub turns_up: u32,
    /// 0, 1 up, 2 down.
    pub direction: u32,
    /// The turns going down.
    pub turns_down: u32,
    /// The listener's distance (the root of the squares of `place`).
    pub distance: f32,
    /// Its change x the turns a second.
    pub approach: f32,
    /// The Doppler-like pitch factor, eased.
    pub pitch_factor: f32,
    /// 1, or 1 / the distance when that is over 1.
    pub volume_factor: f32,
    /// The sound playing from the surface sets of eight (-2: none; `sound::surface_sound`).
    pub eights_sound: u32,
    /// The sound playing from the surface sets of four (-2: none).
    pub fours_sound: u32,
    /// Seven timed sound slots (`sound::car_slot_sounds` the first five, `sound::car_gear_sounds`
    /// the last two: the sixth armed by `sound::car_engine_state` when the throttle drops at high
    /// revs, the seventh by its countdown at low throttle).
    pub slots: [SoundSlot; 7],
    /// Counts down (back-fires).
    pub backfire_wait: u32,
    /// The gear change's slot (pitch `gear_pitch`).
    pub gear_slot: SoundSlot,
    /// The gear change sound's pitch factor.
    pub gear_pitch: f32,
    /// The place of `slot_b`.
    pub place_b: [f32; 3],
    /// A timed slot.
    pub slot_b: SoundSlot,
    /// The hit sound to send once (-2: none), set by the physics.
    pub hit_sound: u32,
    /// Its volume.
    pub hit_volume: u32,
    /// Its place.
    pub hit_place: [f32; 3],
    /// The second one-off sound (-2: none), set by the physics' cues.
    pub cue_sound: u32,
    /// Its volume.
    pub cue_volume: u32,
    /// Its place.
    pub cue_place: [f32; 3],
    /// A timed slot.
    pub slot_c: SoundSlot,
    /// 1 after a landing's effects (`input::update_car_effects`).
    pub landed: u32,
    /// The last forward speed x 10 (`effect_in[3]`), kept for the next turn's speed-change
    /// effect (`input::update_car_effects`); cleared by the set-up.
    pub last_forward_speed: u32,
    /// Cleared by the set-up; nothing else reads or writes it.
    pub reserved_280: u32,
    /// The first wheel sum x 0.4 (a whole number).
    pub sum_a: u32,
    /// The second wheel sum x 64 (a whole number).
    pub sum_b: u32,
    /// The other wheels' sum x 64 (a whole number).
    pub sum_c: u32,
    /// The surface the first wheel runs on.
    pub surface: u32,
    /// The queue of spoken cues (`sound::play_cue_queue`; `physics::queue_sound_cue` fills it),
    /// 0x210 bytes.
    pub cue_queue: CueQueue,
}

/// A race sound object, borrowed for a statement at a time (never across a sound send).
pub type SoundObjectCell = RefCell<SoundObject>;

/// The eight race sound objects as the game starts them.
pub(crate) const fn initial_objects() -> [RefCell<SoundObject>; 8] {
    [
        RefCell::new(initial_object(0)),
        RefCell::new(initial_object(1)),
        RefCell::new(initial_object(0xFFFFFFFE)),
        RefCell::new(initial_object(0xFFFFFFFD)),
        RefCell::new(initial_object(0xFFFFFFFC)),
        RefCell::new(initial_object(0xFFFFFFFB)),
        RefCell::new(initial_object(0xFFFFFFFA)),
        RefCell::new(initial_object(0xFFFFFFF9)),
    ]
}

/// The current car's sound object (the current car's number modulo eight).
pub fn current_object<'a>(
    sound: &'a crate::sound::owner::Sound,
    work: &crate::race::work_store::RaceWork,
) -> &'a RefCell<SoundObject> {
    &sound.race.objects.get()[(work.current_car.get() & 7) as usize]
}

impl SoundObject {
    /// Race sound object `n`, refused past the eight.
    pub fn of(
        sound: &crate::sound::owner::Sound,
        n: u32,
    ) -> Result<&RefCell<SoundObject>, crate::errors::BadInput> {
        sound
            .race
            .objects
            .get()
            .get(n as usize)
            .ok_or(crate::errors::BadInput(
                "a race sound object past the eight",
            ))
    }

    /// The timed slot at a byte offset in the 1999 object layout (the core's lists of slots are
    /// by offset). Any other offset is a bug of the caller.
    pub(super) fn slot_mut(&mut self, offset: u32) -> &mut SoundSlot {
        match offset {
            0x14C => &mut self.slots[0],
            0x164 => &mut self.slots[1],
            0x17C => &mut self.slots[2],
            0x194 => &mut self.slots[3],
            0x1AC => &mut self.slots[4],
            0x1C4 => &mut self.slots[5],
            0x1DC => &mut self.slots[6],
            0x1F8 => &mut self.gear_slot,
            0x220 => &mut self.slot_b,
            0x260 => &mut self.slot_c,
            _ => panic!("no timed slot at +{offset:#x} of a sound object"),
        }
    }
}

/// A sound object as the game starts it, with the car's `sound_offset` (all eight have the
/// kind 0x11 at the start).
pub(crate) const fn initial_object(sound_offset: u32) -> SoundObject {
    SoundObject {
        sound_offset,
        kind: 0x11,
        in_use: 0,
        view_flag: 0,
        new_revs: 0,
        revs: 0x3E8,
        throttle: 0,
        speed: 0.0,
        gear_event: 0,
        no_gear: 0,
        place: [0.0, 0.0, 0.0],
        motion: [0.0, 0.0, 0.0],
        voices: [const {
            EngineVoice {
                share_a: 0.0,
                share_b: 0.0,
                share_c: 0.0,
                surface: 0,
            }
        }; 4],
        reading_times: [0, 0],
        reading_revs: [0, 0],
        reserved_a4: [0, 0],
        flag_ac: 0,
        effect_in: [0; 4],
        knock: 0,
        knock_in: [const { 0.0 }; 4],
        steer_trim_copy: 0,
        effect_request_b: 0,
        knock_count: 0,
        gear: 0,
        last_gear: 0,
        gear_notice: 0,
        countdown: 0,
        drop_latched: 1,
        last_direction: 0,
        revs_input: 0.0,
        throttle_input: 0.0,
        speed_input: 0.0,
        state: 0,
        last_state: 0,
        smooth_throttle: 0,
        throttle_lead: 0,
        turns_up: 0,
        direction: 0,
        turns_down: 0,
        distance: 0.0,
        approach: 0.0,
        pitch_factor: 1.0,
        volume_factor: 0.0,
        eights_sound: 0xFFFFFFFE,
        fours_sound: 0xFFFFFFFE,
        slots: [
            SoundSlot {
                sound: 0xD6,
                start: 0,
                playing: 0,
                ends: 0,
                volume: 0xFFFFFF91,
                rate: 0xFFFFFF91,
            },
            SoundSlot {
                sound: 0xD5,
                start: 0,
                playing: 0,
                ends: 0,
                volume: 0xFFFFFF91,
                rate: 0xFFFFFF91,
            },
            SoundSlot {
                sound: 0xD3,
                start: 0,
                playing: 0,
                ends: 0,
                volume: 0xFFFFFF91,
                rate: 0xFFFFFF91,
            },
            SoundSlot {
                sound: 0xD4,
                start: 0,
                playing: 0,
                ends: 0,
                volume: 0xFFFFFF91,
                rate: 0xFFFFFF91,
            },
            SoundSlot {
                sound: 0xD2,
                start: 0,
                playing: 0,
                ends: 0,
                volume: 0xFFFFFF91,
                rate: 0xFFFFFF91,
            },
            SoundSlot {
                sound: 0xEB,
                start: 0,
                playing: 0,
                ends: 0,
                volume: 0xFFFFFF91,
                rate: 0xFFFFFF91,
            },
            SoundSlot {
                sound: 0xE9,
                start: 0,
                playing: 0,
                ends: 0,
                volume: 0xFFFFFF91,
                rate: 0xFFFFFF91,
            },
        ],
        backfire_wait: 0,
        gear_slot: SoundSlot {
            sound: 0xE8,
            start: 0,
            playing: 0,
            ends: 0,
            volume: 0xFFFFFF91,
            rate: 0xFFFFFF91,
        },
        gear_pitch: 1.0,
        place_b: [0.0, 0.0, 0.0],
        slot_b: SoundSlot {
            sound: 0xE3,
            start: 0,
            playing: 0,
            ends: 0,
            volume: 0xFFFFFF91,
            rate: 0xFFFFFF91,
        },
        hit_sound: 0xFFFFFFFE,
        hit_volume: 0,
        hit_place: [0.0, 0.0, 0.0],
        cue_sound: 0xFFFFFFFE,
        cue_volume: 0,
        cue_place: [0.0, 0.0, 0.0],
        slot_c: SoundSlot {
            sound: 0xE4,
            start: 0,
            playing: 0,
            ends: 0,
            volume: 0xFFFFFF91,
            rate: 0xFFFFFF91,
        },
        landed: 0,
        last_forward_speed: 0,
        reserved_280: 0,
        sum_a: 0,
        sum_b: 0,
        sum_c: 0,
        surface: 0,
        cue_queue: CueQueue {
            first: 0,
            place: 0xFFFFFFFF,
            due: 0,
            count: 0,
            cues: [const {
                rc2k_core::sound::cars::Cue {
                    first: 0,
                    count: 0,
                    sounds: [0; 6],
                }
            }; 16],
        },
    }
}

/// The cars the race sounds follow (the objects are `RaceSoundState::objects`).
#[derive(rc2k_derive::Cells)]
pub struct RaceSounds {
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

/// The race sounds' cars and nearest-car work, starting as the game does.
impl RaceSounds {
    pub const fn new() -> Self {
        RaceSounds {
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
pub struct RaceClock {
    /// 1 once the race's sounds are set up (`race::sounds`, `race::sound_tables`), 0 at the race's
    /// end; `sound::race_turn` does nothing before.
    pub started: u32,
    /// The clock (ms) of the last sound turn.
    pub last: u32,
    /// Sound turns a second (a distance changed in one turn times it is a speed in units a second).
    pub turns_a_second: f32,
    /// The turn's length (ms).
    pub step: u32,
    /// The turns counted.
    pub turns: u32,
    /// 337.52 and `RaceClock.over_sound_speed` (its inverse, 0.002963): a moving sound's pitch
    /// is (speed of approach + the first) times the second, the Doppler factor with the speed
    /// of sound (`sound::turn_parts`).
    pub sound_speed: f32,
    pub over_sound_speed: f32,
    /// 64 and `RaceClock.pan_per_radian` (128 / pi): a placed sound's pan, the centre and the
    /// pan steps per radian of angle.
    pub pan_centre: f32,
    pub pan_per_radian: f32,
}

/// The race sounds' clock (`RaceClock`), starting as the game does
/// (`tests::race_clock_starts_as_the_games`).
impl RaceClock {
    pub const fn new() -> Self {
        RaceClock {
            started: 0,
            last: 0,
            turns_a_second: 0.0,
            step: 0,
            turns: 0,
            sound_speed: 337.52,
            over_sound_speed: 0.0029627874,
            pan_centre: 64.0,
            pan_per_radian: 40.743664,
        }
    }
}

/// The factors the engine, speed, revs and throttle sounds and the wheel sounds scale by.
#[derive(Clone, Copy)]
pub struct SurfaceConstants {
    /// 0.1: a surface sound's volume is the wheels' sum times it.
    pub volume_scale: f32,
    /// 0.1: how far a pitch moves to its target in a turn (the engine's and the spot voices'
    /// Doppler pitch).
    pub pitch_blend: f32,
    /// 0.4: the first wheel sum (`SoundObject.sum_a`) is scaled by it.
    pub across_scale: f32,
    /// 64: the second wheel sum (`sum_b`, `sum_c`) is scaled by it.
    pub along_scale: f32,
    /// 0.014: the car speed's (|`SoundObject.speed`|) factor for the speed sound's input
    /// (`speed_input`).
    pub speed_scale: f32,
    /// 1.4: the speed sound's gain.
    pub speed_gain: f32,
    /// 0.714: the revs and throttle sounds' gain.
    pub revs_gain: f32,
}

impl SurfaceConstants {
    pub const fn new() -> Self {
        SurfaceConstants {
            volume_scale: 0.1,
            pitch_blend: 0.1,
            across_scale: 0.4,
            along_scale: 64.0,
            speed_scale: 0.0140625,
            speed_gain: 1.4,
            revs_gain: 0.71428573,
        }
    }
}

/// What `sound::surface_sounds` works out, and whether the sound tables are prepared.
#[derive(Clone, Copy)]
pub struct SurfaceWork {
    /// 1 once `race::prepare_sound_tables` has run.
    pub tables_done: u32,
    /// Front less back of the wheel floats (the signs pick the wheel).
    pub across: f32,
    /// Left less right of the wheel floats.
    pub along: f32,
    /// The volume the surface sound is sent with.
    pub volume: f32,
}

impl SurfaceWork {
    pub const fn new() -> Self {
        SurfaceWork {
            tables_done: 0,
            across: 0.0,
            along: 0.0,
            volume: 0.0,
        }
    }
}

/// The surface sounds' constants and work, starting as the game does. The work is the game
/// thread's, borrowed for a statement at a time.
pub struct SurfaceSounds {
    pub constants: SurfaceConstants,
    pub work: RefCell<SurfaceWork>,
}

impl SurfaceSounds {
    pub const fn new() -> Self {
        SurfaceSounds {
            constants: SurfaceConstants::new(),
            work: RefCell::new(SurfaceWork::new()),
        }
    }
}

/// A timed sound's slot (`sound::play_timed_slot`): a car has ten, the stage one. Six dwords:
/// the sound (a sample number), 1 = start it (cleared by the slot's turn), 1 = playing, when it
/// ends (the sound clock, ms), the volume and the rate (-0x6F: the sample's own).
pub type SoundSlot = rc2k_core::sound::cars::TimedSlot;

/// A timed slot read and written whole, the borrow held only for the read or the write.
pub(super) trait TimedCells {
    fn load(&self) -> SoundSlot;
    fn store(&self, slot: &SoundSlot);
}

impl TimedCells for RefCell<SoundSlot> {
    fn load(&self) -> SoundSlot {
        *self.borrow()
    }
    fn store(&self, s: &SoundSlot) {
        *self.borrow_mut() = *s;
    }
}

/// Where in the 1999 object (`SoundObject::slot_mut`) the slots `slot_b`, `gear_slot`, `slots[5]`
/// and `slots[6]` are.
pub(super) const SLOT_B_AT: u32 = 0x220;
pub(super) const GEAR_SLOT_AT: u32 = 0x1F8;
pub(super) const SLOT_FIVE_AT: u32 = 0x1C4;
pub(super) const SLOT_SIX_AT: u32 = 0x1DC;

/// A sound object's timed slot, picked by its place in the 1999 object (see
/// `SoundObject::slot_mut`).
pub(super) struct ObjectSlot<'a> {
    object: &'a RefCell<SoundObject>,
    offset: u32,
}

impl<'a> ObjectSlot<'a> {
    pub(super) fn new(object: &'a RefCell<SoundObject>, offset: u32) -> Self {
        Self { object, offset }
    }
}

impl TimedCells for ObjectSlot<'_> {
    fn load(&self) -> SoundSlot {
        *self.object.borrow_mut().slot_mut(self.offset)
    }
    fn store(&self, s: &SoundSlot) {
        *self.object.borrow_mut().slot_mut(self.offset) = *s;
    }
}

/// A sound spot shared with the sound side.
#[derive(Clone, Copy)]
pub struct SharedSpot {
    /// The spot's id.
    pub id: u32,
    /// Its place.
    pub place: [f32; 3],
    /// Its kind, 1 .. 0x15.
    pub kind: u32,
}

/// A spot kind's repeats.
pub struct KindRepeats {
    /// The most repeats (0: no limit).
    pub most: u16,
    /// The least delay before a repeat.
    pub least_delay: u16,
    /// The most delay before a repeat.
    pub most_delay: u16,
}

/// The stage sounds' state and the shared sound spots (the timed sound's slot is
/// `RaceSoundState::timed_slot`).
pub struct StageSounds {
    /// A surface's sound set (an index into `SurfaceSets.sets`' pairs), 16 surfaces.
    pub surface_sets: [u32; 16],
    /// At 1 the stage's one-off sound is played once (the flag is cleared).
    pub once: [u32; 3],
    /// At 1 the timed sound is armed (cleared; due and waiting set).
    pub timed_arm: u32,
    /// The timed sound's place.
    pub timed_place: [f32; 3],
    /// The delay (ms) from arming to the timed sound.
    pub timed_delay: u32,
    /// The volume (halved) of the sound that plays all the time.
    pub loop_volume_a: u32,
    /// The volume of the second.
    pub loop_volume_b: u32,
    /// The clock the timed sound is due at.
    pub timed_due: u32,
    /// 1 while it waits to be due.
    pub timed_waiting: u32,
    /// At 1 a sound at a random pitch and place is played once.
    pub random: [u32; 2],
    /// The spots the race frame shared this frame (`race::share_sound_spots`).
    pub shared_count: u32,
    /// The spots shared with the sound side, 36 of them.
    pub shared_spots: [SharedSpot; 36],
}

/// The stage sounds (`StageSounds`), starting as the game does.
impl StageSounds {
    pub const fn new() -> Self {
        StageSounds {
            surface_sets: [1, 3, 0, 1, 2, 4, 1, 1, 1, 2, 6, 2, 2, 1, 1, 1],
            once: [0; 3],
            timed_arm: 0,
            timed_place: [0.0, 2.0, 1.0],
            timed_delay: 0,
            loop_volume_a: 0,
            loop_volume_b: 0,
            timed_due: 0,
            timed_waiting: 0,
            random: [0; 2],
            shared_count: 0,
            shared_spots: [SharedSpot {
                id: 0,
                place: [0.0; 3],
                kind: 0,
            }; 36],
        }
    }
}

/// The stage's timed sound's slot (sound 0x1C4), starting as the game does.
pub const fn stage_timed_slot() -> SoundSlot {
    SoundSlot {
        sound: 0x1C4,
        start: 0,
        playing: 0,
        ends: 0,
        volume: 0xFFFFFF91,
        rate: 0xFFFFFF91,
    }
}

/// The sound kinds per spot: what each plays and how often it repeats. Never written after the
/// game starts; the pickers' work between a merge and a thin stays in the core's `Picks`.
pub struct SpotKinds {
    /// A spot kind's sound, 22 of them (kind 0 unused).
    pub kind_sounds: [u32; 22],
    /// A kind's repeats, 8 bytes, kind 1 first.
    pub kind_repeats: [KindRepeats; 21],
}

impl SpotKinds {
    pub const fn new() -> Self {
        SpotKinds {
            kind_sounds: [
                0xFFFFFFFE, 0x1B1, 0x1B2, 0x1B3, 0x1B4, 0x1B5, 0x1B6, 0x1B7, 0x1B9, 0x1BA, 0x1BB,
                0x1B0, 0xFFFFFFFE, 0x1BD, 0x1BE, 0x1BF, 0x1C0, 0x1C1, 0x1C2, 0x1C3, 0x1B8, 0x1BC,
            ],
            kind_repeats: [
                KindRepeats {
                    most: 0,
                    least_delay: 1000,
                    most_delay: 5000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 3000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 2000,
                    most_delay: 6000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 500,
                    most_delay: 3000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 2000,
                    most_delay: 6000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 1000,
                    most_delay: 4000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 1000,
                    most_delay: 5000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 1000,
                    most_delay: 4000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 4000,
                    most_delay: 8000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 3000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 1000,
                    most_delay: 5000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
                KindRepeats {
                    most: 1,
                    least_delay: 0,
                    most_delay: 5000,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
                KindRepeats {
                    most: 0,
                    least_delay: 0,
                    most_delay: 0,
                },
            ],
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

// ---- single variables the game's thread keeps ----

/// The car stream's three dwords: the place it is at, its list (a speech row's
/// `texts_store::text_key`), and whether it is on (`sound::play_car_stream`,
/// `sound::step_car_stream`, the stage view's speech).
pub struct CarStream {
    pub place: u32,
    pub stream: u32,
    pub on: u32,
}

impl CarStream {
    pub const fn new() -> Self {
        CarStream {
            place: 0,
            stream: 0,
            on: 0,
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
