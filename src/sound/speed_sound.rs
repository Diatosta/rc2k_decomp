//! `sound::speed_sound` (0x40DE09): a player's car's sound by its speed (`speed_input`,
//! from `sound::car_engine_state`), at the car.
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
) -> Result<(), Refused> {
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    let set = &owner.race.kinds.record(kind)?.speed;
    let sound = so.sound_offset.get().wrapping_add(set.sound.get());
    request.sound.set(sound);
    if speed_stopped(
        so.state.get(),
        || so.no_gear.get(),
        || so.countdown.get(),
        || so.flag_ac.get(),
    ) {
        return crate::sound::turn_parts::stop(request);
    }
    fill_pitch_and_volume(
        so,
        set,
        &so.speed_input,
        &so.speed_input,
        &owner.race.surface.speed_gain,
        request,
    );
    for (cell, place) in request.place.iter().zip(&so.place) {
        cell.set(place.bits());
    }
    crate::sound::turn_parts::placed(request)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::sound::sets_store::SoundSet;

    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    // The original's instructions: the pitch, the volume (on the road: times the surface's gain).
    reference!(ref_off(bx, bp, si) "fld dword ptr [{bx} + 0x108]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]", "fld dword ptr [{bx} + 0x108]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fstp dword ptr [{si} + 0x8]");
    reference!(ref_on(bx, bp, sf, si) "fld dword ptr [{bx} + 0x108]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]", "fld dword ptr [{bx} + 0x108]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fmul dword ptr [{sf} + 0x24]", "fstp dword ptr [{si} + 0x8]");
    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "speed_sound, view flag 1",
                operands: vec![
                    Operand::Buffer {
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
                reference: |a| unsafe { ref_off(a[0], a[1], a[2]) },
                plain: |a| {
                    let bx = view::<SoundObject>(a[0], 0);
                    let set = view::<SoundSet>(a[1], 0);
                    bx.view_flag.set(1);
                    let request = view::<SoundRequest>(a[2], 0);
                    let gain = &&crate::sound::owner::sound().race.surface.speed_gain;
                    fill_pitch_and_volume(bx, set, &bx.speed_input, &bx.speed_input, gain, request);
                },
            },
            Block {
                name: "speed_sound, view flag 0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0xc, In::I), (0x108, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x10, In::F), (0x18, In::F), (0x1C, In::F), (0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(4, Out::F), (8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_on(a[0], a[1], a[2], a[3]) },
                plain: |a| {
                    let bx = view::<SoundObject>(a[0], 0);
                    let set = view::<SoundSet>(a[1], 0);
                    bx.view_flag.set(0);
                    let request = view::<SoundRequest>(a[3], 0);
                    let gain = &view::<crate::sound::store::SurfaceSounds>(a[2], 0).speed_gain;
                    fill_pitch_and_volume(bx, set, &bx.speed_input, &bx.speed_input, gain, request);
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
