//! `sound::speed_sound`: a player's car's sound by its speed (`speed_input`, from
//! `sound::car_engine_state`), at the car.
//!
//! Nothing for a computer's car (sound offset negative). The sound is the speed set's
//! first sound plus the object's `sound_offset`. It is stopped in engine state 0 or 1, in
//! no gear, while the hold-off counts or the object's `flag_ac` is set. Else the request
//! gets the pitch and volume from the speed (`revs_sound::fill_pitch_and_volume`, with the
//! surface's speed gain) and the car's place as it is, and is played. Refuses a null
//! object, and a kind wanted but null.

use crate::errors::Refused;
use crate::sound::revs_sound::fill_pitch_and_volume;
use crate::sound::sets_store::SoundRequest;
use crate::sound::store::SoundObject;
use rc2k_core::sound::engine::{computers, speed_stopped};

/// A player's sound by the speed for object `so` of sound kind number `kind`, through
/// `request`.
pub fn speed_sound(
    so: &SoundObject,
    kind: u32,
    request: &SoundRequest,
    owner: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    let set = &owner.race.kinds.get().record(kind)?.speed;
    let sound = rc2k_core::sound::engine::player_sound(so.sound_offset.get(), set.sound.get());
    request.sound.set(sound);
    if speed_stopped(
        so.state.get(),
        || so.no_gear.get(),
        || so.countdown.get(),
        || so.flag_ac.get(),
    ) {
        return crate::sound::turn_parts::stop(request, audio);
    }
    fill_pitch_and_volume(
        so,
        set,
        &so.speed_input,
        &so.speed_input,
        &owner.race.surface.get().speed_gain,
        request,
    );
    for (cell, place) in request.place.iter().zip(&so.place) {
        cell.set(place.get());
    }
    crate::sound::turn_parts::placed(request, owner, audio)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::sound::asm_layout::{Laid, with_laid};
    use crate::sound::sets_store::SoundSet;
    use crate::sound::store::{SoundObject, SurfaceSounds};

    use crate::math::x87_mem::view;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    // The blocks: the pitch, the volume (on the road: times the surface's gain).
    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "speed_sound, view flag 1",
                operands: vec![
                    Operand::Sized {
                        len: SoundObject::EXTENT,
                        reads: &[(0xc, In::I), (0x108, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x10, In::F), (0x18, In::F), (0x1C, In::F), (0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(4, Out::F), (8, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    with_laid::<SoundObject, _>(a[0], |bx| {
                        let set = view::<SoundSet>(a[1], 0);
                        bx.view_flag.set(1);
                        let request = view::<SoundRequest>(a[2], 0);
                        let gain = &&crate::test_support::own::sound()
                            .race
                            .surface
                            .get()
                            .speed_gain;
                        fill_pitch_and_volume(
                            bx,
                            set,
                            &bx.speed_input,
                            &bx.speed_input,
                            gain,
                            request,
                        );
                    });
                },
            },
            Block {
                name: "speed_sound, view flag 0",
                operands: vec![
                    Operand::Sized {
                        len: SoundObject::EXTENT,
                        reads: &[(0xc, In::I), (0x108, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x10, In::F), (0x18, In::F), (0x1C, In::F), (0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Sized {
                        len: SurfaceSounds::EXTENT,
                        reads: &[(0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(4, Out::F), (8, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    with_laid::<SoundObject, _>(a[0], |bx| {
                        with_laid::<SurfaceSounds, _>(a[2], |sf| {
                            let set = view::<SoundSet>(a[1], 0);
                            bx.view_flag.set(0);
                            let request = view::<SoundRequest>(a[3], 0);
                            let gain = &sf.speed_gain;
                            fill_pitch_and_volume(
                                bx,
                                set,
                                &bx.speed_input,
                                &bx.speed_input,
                                gain,
                                request,
                            );
                        });
                    });
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.speed_sound", &blocks(), 1200, 100);
    }
}
