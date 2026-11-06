//! `sound::revs_sound` (0x40DD19): a car's engine sound by its revs, placed ahead of the
//! car; and the request filling it shares with `sound::throttle_sound`.
//!
//! The sound is the set's first sound plus the object's `sound_offset` (a computer's car,
//! offset negative: its bit 0, from the computers' set). With the engine stopped (state 0
//! or 3) the sound is stopped. Else the request gets the pitch from the revs, the volume,
//! and the place, and is played. Refuses a null object, and a kind wanted but null.

use crate::errors::Refused;
use crate::math::x87_float::{X, ld, st};
use crate::session::{Dword, Float};
use crate::sound::sets_store::{SoundRequest, SoundSet};
use crate::sound::store::{SoundObject, SurfaceSounds};
use rc2k_core::sound::engine::{computers, revs_sound_number, revs_stopped};

/// The engine's sound for object `so` of sound kind number `kind`, through `request`.
pub fn revs_sound(so: &SoundObject, kind: u32, request: &SoundRequest) -> Result<(), Refused> {
    let object = so.sound_offset.get();
    let set = if computers(object) {
        crate::sound::sets_store::SoundKinds::computers_set()
    } else {
        &crate::sound::sets_store::SoundKinds::record(kind)?.revs
    };
    request
        .sound
        .set(revs_sound_number(object, set.sound.get()));
    send_engine_sound(so, set, &so.revs_input, EnginePlace::Ahead, request)
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
    volume_input: &Float,
    side: EnginePlace,
    request: &SoundRequest,
) -> Result<(), Refused> {
    if revs_stopped(so.state.get()) {
        return crate::sound::turn_parts::stop(request);
    }
    fill_request(
        so,
        set,
        volume_input,
        side,
        &crate::sound::owner::sound().race.surface,
        request,
    );
    crate::sound::turn_parts::placed(request)
}

/// The request's pitch, volume and place for the engine sound.
pub(super) fn fill_request(
    so: &SoundObject,
    set: &SoundSet,
    volume_input: &Float,
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
        let place = ld(so.place[n].bits());
        let motion = ld(so.motion[n].bits());
        request.place[n].set_bits(st(match side {
            EnginePlace::Ahead => place + motion,
            EnginePlace::Behind => place - motion,
        }));
    }
}

/// The request's pitch (the set's curve over `pitch_input`) and volume (its curve over
/// `volume_input`, times the surface's `gain` when the view is not the car's own). Each
/// value is rounded to 32 bits as the original's x87 does. Shared by the car's engine
/// sounds (`sound::speed_sound` too).
pub fn fill_pitch_and_volume(
    so: &SoundObject,
    set: &SoundSet,
    pitch_input: &Float,
    volume_input: &Float,
    gain: &Float,
    request: &SoundRequest,
) {
    request
        .pitch
        .set_bits(st(linear(pitch_input, &set.pitch_scale, &set.pitch_low)));
    let volume = linear(volume_input, &set.volume_scale, &set.volume_low);
    if so.view_flag.get() == 0 {
        // On the road the surface's gain scales the volume, before it is rounded.
        request.volume.set_bits(st(volume * ld(gain.bits())));
    } else {
        request.volume.set_bits(st(volume));
    }
}

/// `input * slope + start`, left on the x87 stack unrounded.
fn linear(input: &Float, slope: &Dword, start: &Dword) -> X {
    ld(input.bits()) * ld(slope.get()) + ld(start.get())
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    // The original's instructions: the pitch, the volume (on the road: times the surface's gain), the place.
    reference!(ref_off(bx, bp, si) "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]", "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fstp dword ptr [{si} + 0x8]", "fld dword ptr [{bx} + 0x2c]", "fadd dword ptr [{bx} + 0x38]", "fstp dword ptr [{si} + 0xc]", "fld dword ptr [{bx} + 0x30]", "fadd dword ptr [{bx} + 0x3c]", "fstp dword ptr [{si} + 0x10]", "fld dword ptr [{bx} + 0x34]", "fadd dword ptr [{bx} + 0x40]", "fstp dword ptr [{si} + 0x14]");
    reference!(ref_on(bx, bp, sf, si) "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]", "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fmul dword ptr [{sf} + 0x28]", "fstp dword ptr [{si} + 0x8]", "fld dword ptr [{bx} + 0x2c]", "fadd dword ptr [{bx} + 0x38]", "fstp dword ptr [{si} + 0xc]", "fld dword ptr [{bx} + 0x30]", "fadd dword ptr [{bx} + 0x3c]", "fstp dword ptr [{si} + 0x10]", "fld dword ptr [{bx} + 0x34]", "fadd dword ptr [{bx} + 0x40]", "fstp dword ptr [{si} + 0x14]");
    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "revs_sound, view flag 1",
                operands: vec![
                    Operand::Buffer {
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
                reference: |a| unsafe { ref_off(a[0], a[1], a[2]) },
                plain: |a| {
                    let bx = view::<crate::sound::store::SoundObject>(a[0], 0);
                    let set = view::<SoundSet>(a[1], 0);
                    bx.view_flag.set(1);
                    let request = view::<SoundRequest>(a[2], 0);
                    fill_request(
                        bx,
                        set,
                        &bx.revs_input,
                        EnginePlace::Ahead,
                        &crate::sound::owner::sound().race.surface,
                        request,
                    );
                },
            },
            Block {
                name: "revs_sound, view flag 0",
                operands: vec![
                    Operand::Buffer {
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
                reference: |a| unsafe { ref_on(a[0], a[1], a[2], a[3]) },
                plain: |a| {
                    let bx = view::<crate::sound::store::SoundObject>(a[0], 0);
                    let set = view::<SoundSet>(a[1], 0);
                    bx.view_flag.set(0);
                    let request = view::<SoundRequest>(a[3], 0);
                    fill_request(
                        bx,
                        set,
                        &bx.revs_input,
                        EnginePlace::Ahead,
                        view::<crate::sound::store::SurfaceSounds>(a[2], 0),
                        request,
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
