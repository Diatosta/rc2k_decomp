//! `sound::throttle_sound` (0x40DD96): a player's car's engine sound by its throttle,
//! placed behind the car.
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
) -> Result<(), Refused> {
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    let set = &crate::sound::sets_store::SoundKinds::record(kind)?.throttle;
    request
        .sound
        .set(so.sound_offset.get().wrapping_add(set.sound.get()));
    send_engine_sound(so, set, &so.throttle_input, EnginePlace::Behind, request)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::sound::revs_sound::EnginePlace;
    use crate::sound::revs_sound::fill_request;
    use crate::sound::sets_store::SoundSet;

    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    // The original's instructions: the pitch, the volume (on the road: times the surface's gain), the place.
    reference!(ref_off(bx, bp, si) "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]", "fld dword ptr [{bx} + 0x104]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fstp dword ptr [{si} + 0x8]", "fld dword ptr [{bx} + 0x2c]", "fsub dword ptr [{bx} + 0x38]", "fstp dword ptr [{si} + 0xc]", "fld dword ptr [{bx} + 0x30]", "fsub dword ptr [{bx} + 0x3c]", "fstp dword ptr [{si} + 0x10]", "fld dword ptr [{bx} + 0x34]", "fsub dword ptr [{bx} + 0x40]", "fstp dword ptr [{si} + 0x14]");
    reference!(ref_on(bx, bp, sf, si) "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]", "fld dword ptr [{bx} + 0x104]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fmul dword ptr [{sf} + 0x28]", "fstp dword ptr [{si} + 0x8]", "fld dword ptr [{bx} + 0x2c]", "fsub dword ptr [{bx} + 0x38]", "fstp dword ptr [{si} + 0xc]", "fld dword ptr [{bx} + 0x30]", "fsub dword ptr [{bx} + 0x3c]", "fstp dword ptr [{si} + 0x10]", "fld dword ptr [{bx} + 0x34]", "fsub dword ptr [{bx} + 0x40]", "fstp dword ptr [{si} + 0x14]");
    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "throttle_sound, view flag 1",
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
                reference: |a| unsafe { ref_off(a[0], a[1], a[2]) },
                plain: |a| {
                    let bx = view::<crate::sound::store::SoundObject>(a[0], 0);
                    let set = view::<SoundSet>(a[1], 0);
                    bx.view_flag.set(1);
                    let request = view::<SoundRequest>(a[2], 0);
                    fill_request(
                        bx,
                        set,
                        &bx.throttle_input,
                        EnginePlace::Behind,
                        &crate::sound::store::SURFACE_SOUNDS,
                        request,
                    );
                },
            },
            Block {
                name: "throttle_sound, view flag 0",
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
                            (0x104, In::F),
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
                        &bx.throttle_input,
                        EnginePlace::Behind,
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
