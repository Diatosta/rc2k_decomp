//! `sound::throttle_sound`: a player's car's engine sound by its throttle, placed behind the car.
//!
//! Nothing for a computer's car (sound offset negative). Else as `sound::revs_sound`
//! (whose `send_engine_sound` it shares) with the throttle's set, the volume from the
//! throttle, and the place less the motion. Refuses a null object, and a kind wanted but
//! null.

use crate::errors::Refused;
use crate::sound::revs_sound::{EnginePlace, send_engine_sound};
use crate::sound::sets_store::SoundRequest;
use rc2k_core::sound::engine::computers;

/// A player's engine sound by the throttle for object `so` of sound kind number `kind`,
/// through `request`.
pub fn throttle_sound(
    so: &crate::sound::store::SoundObject,
    kind: u32,
    request: &SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    let set = &sound.race.kinds.get().record(kind)?.throttle;
    request
        .sound
        .set(so.sound_offset.get().wrapping_add(set.sound.get()));
    send_engine_sound(
        so,
        set,
        &so.throttle_input,
        EnginePlace::Behind,
        request,
        sound,
        audio,
    )
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::sound::asm_layout::{Laid, with_laid};
    use crate::sound::revs_sound::EnginePlace;
    use crate::sound::revs_sound::fill_request;
    use crate::sound::sets_store::SoundSet;
    use crate::sound::store::{SoundObject, SurfaceSounds};

    use crate::scene::x87_mem::view;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse_locked};

    // The blocks: the pitch, the volume (on the road: times the surface's gain), the place.
    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "throttle_sound, view flag 1",
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
                            (0x104, In::F),
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
                            &bx.throttle_input,
                            EnginePlace::Behind,
                            crate::test_support::own::sound().race.surface.get(),
                            request,
                        );
                    });
                },
            },
            Block {
                name: "throttle_sound, view flag 0",
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
                            (0x104, In::F),
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
                            fill_request(
                                bx,
                                set,
                                &bx.throttle_input,
                                EnginePlace::Behind,
                                sf,
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
        sparse_locked("sound.throttle_sound", &blocks(), 1200, 100);
    }
}
