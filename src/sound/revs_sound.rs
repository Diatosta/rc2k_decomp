//! `sound::revs_sound`: a car's engine sound by its revs, placed ahead of the car; and the request
//! filling it shares with `sound::throttle_sound`.
//!
//! The sound is the set's first sound plus the object's `sound_offset` (a computer's car,
//! offset negative: its bit 0, from the computers' set). With the engine stopped (state 0
//! or 3) the sound is stopped. Else the request gets the pitch from the revs, the volume,
//! and the place, and is played. Refuses a null object, and a kind wanted but null.

use crate::errors::Refused;
use crate::math::x87_float::{X, ld, st, stf};
use crate::memory::session::{PlainDword, PlainFloat};
use crate::sound::sets_store::{SoundRequest, SoundSet};
use crate::sound::store::{SoundObject, SurfaceSounds};
use rc2k_core::sound::engine::{computers, revs_sound_number, revs_stopped};

/// The engine's sound for object `so` of sound kind number `kind`, through `request`.
pub fn revs_sound(
    so: &SoundObject,
    kind: u32,
    request: &SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let object = so.sound_offset.get();
    let set = if computers(object) {
        sound.race.kinds.get().computers_set()
    } else {
        &sound.race.kinds.get().record(kind)?.revs
    };
    request
        .sound
        .set(revs_sound_number(object, set.sound.get()));
    send_engine_sound(
        so,
        set,
        &so.revs_input,
        EnginePlace::Ahead,
        request,
        sound,
        audio,
    )
}

/// Which side of the car the engine sound is heard: the car's place plus its motion, or
/// less it.
#[derive(Clone, Copy)]
pub enum EnginePlace {
    /// The revs sound: place + motion.
    Ahead,
    /// The throttle sound: place - motion.
    Behind,
}

/// The end of `revs_sound` and `throttle_sound`, with the request's sound number set:
/// stops the sound when the engine is stopped; else fills the request (pitch from the revs,
/// volume from `volume_input`) and plays it.
pub fn send_engine_sound(
    so: &SoundObject,
    set: &SoundSet,
    volume_input: &PlainFloat,
    side: EnginePlace,
    request: &SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if revs_stopped(so.state.get()) {
        return crate::sound::turn_parts::stop(request, audio);
    }
    fill_request(
        so,
        set,
        volume_input,
        side,
        sound.race.surface.get(),
        request,
    );
    crate::sound::turn_parts::placed(request, sound, audio)
}

/// The request's pitch, volume and place for the engine sound.
pub(super) fn fill_request(
    so: &SoundObject,
    set: &SoundSet,
    volume_input: &PlainFloat,
    side: EnginePlace,
    surface: &SurfaceSounds,
    request: &SoundRequest,
) {
    fill_pitch_and_volume(
        so,
        set,
        &so.revs_input,
        volume_input,
        &surface.revs_gain,
        request,
    );
    for n in 0..3 {
        let place = ld(so.place[n].get());
        let motion = ld(so.motion[n].get());
        request.place[n].set(stf(match side {
            EnginePlace::Ahead => place + motion,
            EnginePlace::Behind => place - motion,
        }));
    }
}

/// The request's pitch (the set's curve over `pitch_input`) and volume (its curve over
/// `volume_input`, times the surface's `gain` when the view is not the car's own). Each value is
/// rounded to 32 bits as the x87 does. Shared by the car's engine sounds (`sound::speed_sound`
/// too).
pub fn fill_pitch_and_volume(
    so: &SoundObject,
    set: &SoundSet,
    pitch_input: &PlainFloat,
    volume_input: &PlainFloat,
    gain: &PlainFloat,
    request: &SoundRequest,
) {
    request
        .pitch
        .set(st(linear(pitch_input, &set.pitch_scale, &set.pitch_low)));
    let volume = linear(volume_input, &set.volume_scale, &set.volume_low);
    if so.view_flag.get() == 0 {
        // On the road the surface's gain scales the volume, before it is rounded.
        request.volume.set(st(volume * ld(gain.get())));
    } else {
        request.volume.set(st(volume));
    }
}

/// `input * slope + start`, left on the x87 stack unrounded.
fn linear(input: &PlainFloat, slope: &PlainDword, start: &PlainDword) -> X {
    ld(input.get()) * ld(slope.get()) + ld(start.get())
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::scene::x87_mem::view;
    use crate::sound::asm_layout::{Laid, with_laid};
    use crate::sound::store::{SoundObject, SurfaceSounds};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse_locked};

    // The blocks: the pitch, the volume (on the road: times the surface's gain), the place.
    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "revs_sound, view flag 1",
                operands: vec![
                    Operand::Sized {
                        len: SoundObject::EXTENT,
                        reads: &[
                            (0xc, In::I),
                            (0x2c, In::F),
                            (0x30, In::F),
                            (0x34, In::F),
                            (0x38, In::F),
                            (0x3c, In::F),
                            (0x40, In::F),
                            (0x100, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x10, In::F), (0x18, In::F), (0x1C, In::F), (0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[
                            (4, Out::F),
                            (8, Out::F),
                            (0xC, Out::F),
                            (0x10, Out::F),
                            (0x14, Out::F),
                        ],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    with_laid::<SoundObject, _>(a[0], |bx| {
                        let set = view::<SoundSet>(a[1], 0);
                        bx.view_flag.set(1);
                        let request = view::<SoundRequest>(a[2], 0);
                        fill_request(
                            bx,
                            set,
                            &bx.revs_input,
                            EnginePlace::Ahead,
                            crate::test_support::own::sound().race.surface.get(),
                            request,
                        );
                    });
                },
            },
            Block {
                name: "revs_sound, view flag 0",
                operands: vec![
                    Operand::Sized {
                        len: SoundObject::EXTENT,
                        reads: &[
                            (0xc, In::I),
                            (0x2c, In::F),
                            (0x30, In::F),
                            (0x34, In::F),
                            (0x38, In::F),
                            (0x3c, In::F),
                            (0x40, In::F),
                            (0x100, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x10, In::F), (0x18, In::F), (0x1C, In::F), (0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Sized {
                        len: SurfaceSounds::EXTENT,
                        reads: &[(0x28, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[
                            (4, Out::F),
                            (8, Out::F),
                            (0xC, Out::F),
                            (0x10, Out::F),
                            (0x14, Out::F),
                        ],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    with_laid::<SoundObject, _>(a[0], |bx| {
                        with_laid::<SurfaceSounds, _>(a[2], |sf| {
                            let set = view::<SoundSet>(a[1], 0);
                            bx.view_flag.set(0);
                            let request = view::<SoundRequest>(a[3], 0);
                            fill_request(bx, set, &bx.revs_input, EnginePlace::Ahead, sf, request);
                        });
                    });
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse_locked("sound.revs_sound", &blocks(), 1200, 100);
    }
}
