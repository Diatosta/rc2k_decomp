//! `physics::ease_view_blend` (0x4385EB): with the mode's record's blend (+0x58) and
//! change time (+0x3C) not 0: the share done (`physics/camera_view.rs`; 1 less it for a
//! blend below 0) between `flt_608B20` and `flt_608B24` into `unk_71A6A0`, and between
//! `flt_608B28` and `flt_608B2C`, x `flt_5C5D98`, negated, `fistp`ed into
//! `dword_608B38`.
//!
//! In:  nothing
//! Out: esi the mode's record; eax the time held when it ran; the rest kept; no flags
//!      read
//!
//! Caller: `sub_437A53`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87_float::X;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::camera_view::{record, share_done};
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;

/// The Rust version (the exact one, for `physics::update_camera`).
pub fn ease_view_blend(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let esi = record(&race.shared)?;
    if esi.blend.get() == 0 || esi.change_time.get() == 0 {
        return Ok(());
    }
    share_done(&race.shared, esi, scene);
    if (esi.blend.get() as i32) < 0 {
        x87_1(&race.shared);
    }
    x87_2(race);
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x205260] = 1 - [rc + 0x205260]`
fn x87_1(shared: &Shared) {
    let t0 = X(1.0);
    let t1 = ld(shared.tables.view_share.bits());
    let t2 = t0 - t1;
    shared.tables.view_share.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A6A0] = (([rc + 0x208B24] - [rc + 0x208B20]) * [rc + 0x205260]) + [rc + 0x208B20]`
/// `int [rc + 0x208B38] = -(((([rc + 0x208B2C] - [rc + 0x208B28]) * [rc + 0x205260]) + [rc + 0x208B28]) * [rc + 0x1C5D98])`
fn x87_2(race: &Race) {
    let t0 = ld(race.physics.camera.view_shift_b.bits());
    let t1 = ld(race.physics.camera.view_shift_a.bits());
    let t2 = t0 - t1;
    let t3 = ld(race.shared.tables.view_share.bits());
    let t4 = t2 * t3;
    let t5 = ld(race.physics.camera.view_shift_a.bits());
    let t6 = t4 + t5;
    race.shared.car.camera_zoom.set_bits(st(t6));
    let t7 = ld(race.physics.camera.view_shift_d.bits());
    let t8 = ld(race.physics.camera.view_shift_c.bits());
    let t9 = t7 - t8;
    let t10 = ld(race.shared.tables.view_share.bits());
    let t11 = t9 * t10;
    let t12 = ld(race.physics.camera.view_shift_c.bits());
    let t13 = t11 + t12;
    let t14 = ld(race.course.numbers.k65536.bits());
    let t15 = t13 * t14;
    let t16 = -t15;
    race.physics
        .camera
        .camera_offset_z
        .set_bits(to_int(t16) as u32);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) ".byte 0xD9, 0xE8", "fsub dword ptr [{rc} + 0x205260]", "fstp dword ptr [{rc} + 0x205260]");
    reference!(ref_1(rc) "fld dword ptr [{rc} + 0x208B24]", "fsub dword ptr [{rc} + 0x208B20]", "fmul dword ptr [{rc} + 0x205260]", "fadd dword ptr [{rc} + 0x208B20]", "fstp dword ptr [{rc} + 0x31A6A0]", "fld dword ptr [{rc} + 0x208B2C]", "fsub dword ptr [{rc} + 0x208B28]", "fmul dword ptr [{rc} + 0x205260]", "fadd dword ptr [{rc} + 0x208B28]", "fmul dword ptr [{rc} + 0x1C5D98]", ".byte 0xD9, 0xE0", "fistp dword ptr [{rc} + 0x208B38]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "ease_view_blend line 27",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x205260, In::F)],
                    writes: &[(0x205260, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |_| x87_1(&race().shared),
            },
            Block {
                name: "ease_view_blend line 34",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D98, In::F),
                        (0x205260, In::F),
                        (0x208B20, In::F),
                        (0x208B24, In::F),
                        (0x208B28, In::F),
                        (0x208B2C, In::F),
                    ],
                    writes: &[(0x208B38, Out::I), (0x31A6A0, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |_| x87_2(crate::race::owner::race()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
