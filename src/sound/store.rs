//! The sound module's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`):
//! the race sounds' clock and counters (IDA 0x5124E4 .. 0x512508), and the block from the
//! spot-voice picks to the CD audio's (0x517F40 .. 0x518120: the spot picks, the sound kinds
//! per spot, the force-feedback effects' clock, the effect kinds and constants, the kept
//! options `g_tuneTurn`, `g_musicVolume` and the others `var\ral.dat` keeps, the CD audio's
//! state).
//!
//! The stage sounds' state and the shared sound spots (0x51655C .. 0x5168C0: the surface
//! sound set, the once-only and timed flags, the random and loop sounds, the spots' count
//! and table) is a third.
//!
//! The race sounds' objects (0x512A10 .. 0x514F90: `g_raceSoundObjects`, the eight pointers,
//! then the eight objects of 0x4A8 bytes from 0x512A30, then the cars the sound follows and
//! the nearest computer car's work) are another. The eight pointers name the objects as the
//! image holds them (IDA's addresses): `point_sound_objects` sets them to where the objects
//! are now (the standalone's start-up calls it).
//!
//! The kept wheel options (0x4F66DD .. 0x4F66FD: four dwords `var\ral.dat` keeps, the
//! two wheel surfaces the sound reads, and the seal key of the race's record) is the
//! first.
//!
//! The surface sounds' constants and work (0x516358 .. 0x51639C: the factors the engine,
//! speed, revs and throttle sounds scale by, the wheel sounds across and along, the
//! surface sound volume) is a fourth.
//!
//! Each is one block with the image's layout (`crate::store`); the first values are the
//! image's (`store_data.rs`, generated). The sound commands go to the audio thread through
//! `audio::store`'s queue; these are the game thread's.

use crate::block;

block!(pub mod wheels {
    start: 0x4F66DD,
    end: 0x4F66FD,
    runs: crate::sound::store_data::WHEELS_RUNS,
});

block!(pub mod objects {
    start: 0x512A10,
    end: 0x514F90,
    runs: crate::sound::store_data::OBJECTS_RUNS,
});

block!(pub mod clock {
    start: 0x5124E4,
    end: 0x512508,
    runs: crate::sound::store_data::CLOCK_RUNS,
});

block!(pub mod surface {
    start: 0x516358,
    end: 0x51639C,
    runs: crate::sound::store_data::SURFACE_RUNS,
});

block!(pub mod stage {
    start: 0x51655C,
    end: 0x5168C0,
    runs: crate::sound::store_data::STAGE_RUNS,
});

block!(pub mod effects {
    start: 0x517F40,
    end: 0x518120,
    runs: crate::sound::store_data::EFFECTS_RUNS,
});

pub type WheelsSlot = wheels::Slot;
pub type ObjectsSlot = objects::Slot;
pub type ClockSlot = clock::Slot;
pub type SurfaceSlot = surface::Slot;
pub type StageSlot = stage::Slot;
pub type EffectsSlot = effects::Slot;

// ---- single variables the game's thread keeps ----

use crate::session::Dword;

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

/// `g_raceSoundObjects`: the eight pointers.
pub const OBJECT_TABLE: ObjectsSlot = ObjectsSlot::at(0x512A10);
/// The first object (0x4A8 bytes each, eight of them).
pub const FIRST_OBJECT: ObjectsSlot = ObjectsSlot::at(0x512A30);
pub const OBJECT_LEN: usize = 0x4A8;

/// Sets `g_raceSoundObjects` to the objects' places now.
pub fn point_sound_objects() {
    for k in 0..8 {
        unsafe {
            crate::store::game_write(
                OBJECT_TABLE + 4 * k,
                (FIRST_OBJECT + OBJECT_LEN * k).address() as u32,
            );
        }
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
}
