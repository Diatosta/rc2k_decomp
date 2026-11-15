//! `effects::windscreen_drops_frame`: the windscreen drops' frame: the drift and set-up first, then
//! each live item (with the drift on, its motion eased toward the drift at its cell and its place
//! moved). An item dies off the map (x or z past 0x7FF) or on the edge mask; else its place is
//! turned and its six corners projected into the vertex ring (0xC0 bytes each; the ring wraps back
//! to its start, and the batch is drawn by `d3d::draw_marks_windscreen_drops`, the platform's).
//! Then, with fewer live than the settings' +0x273C and the drift on, new drops are spawned.
//!
//! Works on the effects settings.
//!
//! The float steps run on the x87 model (`math::x87_float`), rounding as the control word says.
//! Refuses, before anything, null settings and a live item of a kind past 7.

use crate::effects::cell_x87::CellX87;
use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_drops::{ITEM_LEN, ITEMS};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner_frame::Frame;
/// The vertex batch's bytes, past the 0x40 records of 0xC0 bytes.
const BUFFER_BYTES: usize = 0x3000;
const KINDS: u32 = 8;

/// An item's fields (byte offsets in the items buffer): live flag at 0, then the shape kind,
/// the place (x, z) and the motion eased toward the drift (x, z).
const ITEM_KIND: u32 = 4;
const ITEM_X: u32 = 0x14;
const ITEM_Z: u32 = 0x18;
const ITEM_MOTION_X: u32 = 0x1C;
const ITEM_MOTION_Z: u32 = 0x20;

/// The dword at byte `e` of the items.
fn get(drops: &WindscreenDrops, e: u32) -> u32 {
    drops.hex.items.get_u32(e as usize)
}

fn put(drops: &WindscreenDrops, e: u32, v: u32) {
    drops.hex.items.set_u32(e as usize, v);
}

/// The drift at the item's cell (the drift curve at k) into `WindscreenDropWork.drift[0]`,
/// `WindscreenDropWork.drift[1]`.
fn drift_cell(drops: &WindscreenDrops, k: u32) {
    drift_cell_x87(drops, k as usize);
}

/// The item's motion (+0x1C, +0x20) eased toward the drift times its kind's
/// `WindscreenDropWork.radii[0]`[k] by `WindscreenDropVars.ease`, its place moved by it.
fn drift_move(drops: &WindscreenDrops, k: u32, s: u32) {
    drift_move_x87(drops, k as usize, s as usize);
}

/// The cell place scaled and turned into `WindscreenDropWork.cell[0]` .., the depth also in
/// `WindscreenDropState.depth`.
fn sprite_depth(drops: &WindscreenDrops) {
    sprite_depth_x87(drops);
}

/// The six corners (the kind's offsets at `WindscreenDropWork.scaled[0]`[c] ..) into
/// `WindscreenDropWork.corners[0]` .., the depth's inverse (left on the x87 stack, so returned).
fn sprite_corners(drops: &WindscreenDrops, c: u32) -> bh::X {
    sprite_corners_x87(drops, c as usize)
}

/// The centre and corners projected into the buffer at e (six vertices), drawn in by
/// `WindscreenDropVars.scale[2]` / `E`, their depth.
fn sprite_project(
    drops: &WindscreenDrops,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    e: u32,
    inverse_depth: bh::X,
) {
    sprite_project_x87(drops, gfx, effects, e as usize, inverse_depth);
}

/// Why it refuses, if it does.
fn refusal(
    effects: &crate::effects::owner::Effects,
    drops: &WindscreenDrops,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if effects.settings().is_err() {
            return Ok(Some("no marks settings"));
        }
        let items = &drops.hex.items;
        let bad = (0..ITEMS as usize).any(|n| {
            let e = n * ITEM_LEN as usize;
            items.get_u32(e) == 1 && items.get_u32(e + ITEM_KIND as usize) >= KINDS
        });
        bad.then_some("a live windscreen drop of a kind past 7")
    })
}

/// The batch's draw (`d3d::draw_marks_windscreen_drops`; its result unread).
pub fn draw_batch(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    crate::d3d::draw_marks_windscreen_drops::draw_marks_windscreen_drops(
        crate::workers::game_scratch(),
        gfx,
    );
    Ok(())
}

pub fn windscreen_drops_frame(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    frame: &Frame,
    masks: &Masks,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    if let Some(why) = refusal(effects, drops)? {
        return Err(BadInput(why).into());
    }
    {
        if setup.kind_floats.running.get() != 0 {
            crate::effects::windscreen_drops_drift::windscreen_drops_drift(frame, drops, effects)?;
        }
        crate::effects::windscreen_drops_setup::windscreen_drops_setup(buffer, drops, scene);
        drops.state.live.set(0);
        for n in 0..ITEMS {
            let e = n * ITEM_LEN;
            if get(drops, e) != 1 {
                continue;
            }
            drops.state.live.set(drops.state.live.get().wrapping_add(1));
            if setup.kind_floats.running.get() != 0 {
                let x = x87::to_int(get(drops, e + ITEM_X));
                setup.work.scratch.set(x as u32);
                drift_cell(drops, 4 * (((x as u32) >> 3) & 0xFF));
                drift_move(drops, 4 * get(drops, e + ITEM_KIND), e);
            }
            let x = x87::to_int(get(drops, e + ITEM_X));
            let z = x87::to_int(get(drops, e + ITEM_Z));
            drops.work.whole[0].set(x as u32);
            drops.work.whole[1].set(z as u32);
            if (x as u32 | (z as u32).wrapping_add(z as u32)) & 0x800 != 0 {
                put(drops, e, 0);
                continue;
            }
            let shape = get(drops, e + ITEM_KIND);
            let cell = ((((z as u32) >> 3) & 0x7F) << 8) + (((x as u32) >> 3) & 0xFF);
            if masks.edge_mask.mask.get(cell as usize) != 0 {
                put(drops, e, 0);
                continue;
            }
            sprite_depth(drops);
            // The depth's bits below 4096.0's.
            if !crate::math::x87::bits_at_or_below(4096.0, drops.work.cell[2].get()) {
                continue;
            }
            let inverse_depth = sprite_corners(drops, shape.wrapping_mul(0x48));
            sprite_project(drops, gfx, effects, drops.hex.buffer.get(), inverse_depth);
            let next = drops.hex.buffer.get().wrapping_add(0xC0);
            drops.hex.buffer.set(next);
            if next as usize == BUFFER_BYTES {
                drops.hex.buffer.set(0);
                gfx.card.windscreen_drop_batch.index_count.set(0x300);
                gfx.card.windscreen_drop_batch.vertex_count.set(0x180);
                draw_batch(gfx)?;
            }
        }
        let marks = effects.settings()?;
        let live: u32 = drops.state.live.get();
        if (live as i32) < marks.windscreen_drops_wanted.get() as i32
            && setup.kind_floats.running.get() != 0
        {
            crate::effects::windscreen_drops_spawn::windscreen_drops_spawn(drops, effects)?;
        }
    }
    Ok(())
}

use crate::effects::plain::{ld_at, ld_cell, st_at};
use crate::math::x87_float as bh;

/// `drift_cell`: the drift at the item's cell (`curve[k / 4]`) into `drift`: x from the
/// shape tilt's point, y its second coordinate.
fn drift_cell_x87(drops: &WindscreenDrops, k: usize) {
    let x = -(drops.work.curve[k / 4].ld() * drops.tilt.point[2].ld()) + drops.tilt.point[0].ld();
    drops.work.drift[0].st(x);
    let y = drops.tilt.point[1].ld();
    drops.work.drift[1].st(y);
}

/// `drift_move`: the item at byte `s` of the items has its motion eased toward the drift
/// (scaled by its kind's radius `radii[k / 4]`) and its place moved by the new motion.
fn drift_move_x87(drops: &WindscreenDrops, k: usize, s: usize) {
    let ld_item = |field: u32| ld_at(&drops.hex.items, s + field as usize);
    let st_item = |field: u32, x| st_at(&drops.hex.items, s + field as usize, x);
    for (motion, drift) in [(ITEM_MOTION_X, 0), (ITEM_MOTION_Z, 1)] {
        let toward = drops.work.drift[drift].ld() * drops.work.radii[k / 4].ld();
        let eased = (toward - ld_item(motion)) * ld_cell(&drops.hex.ease) + ld_item(motion);
        st_item(motion, eased);
    }
    let x = ld_item(ITEM_X) + ld_item(ITEM_MOTION_X);
    let z = ld_item(ITEM_Z) + ld_item(ITEM_MOTION_Z);
    st_item(ITEM_Z, z);
    st_item(ITEM_X, x);
}

/// The dword at `offset` of the record batch, as the x87 loads it.
fn ld_record(offset: usize, gfx: &crate::gfx::owner::Gfx) -> bh::X {
    ld_at(&gfx.card.windscreen_drop_batch.records, offset)
}

/// `sprite_depth`: the item's whole-number place turned by the view into `cell` (and the
/// depth, which is `cell[2]` too). Same order and rounding as the x87.
fn sprite_depth_x87(drops: &WindscreenDrops) {
    let work = &drops.work;
    let x = bh::int(work.whole[0].get() as i32);
    let z = ld_cell(&drops.hex.frame_k) - bh::int(work.whole[1].get() as i32);
    let z = z * work.shares[1].ld();
    let x = x * work.shares[0].ld();
    // Output i: x times the first axes row, z times the second, plus the view's place. The
    // last one adds the products the other way round and is also
    // the depth.
    for i in 0..3 {
        let from_x = x * work.axes[i].ld();
        let from_z = z * work.axes[3 + i].ld();
        let sum = if i < 2 {
            from_x + from_z
        } else {
            from_z + from_x
        };
        let out = sum + work.place[i].ld();
        if i == 2 {
            drops.state.depth.st(out);
        }
        work.cell[i].st(out);
    }
}

/// `sprite_corners`: the six corners of the kind's shape (its offsets from `scaled[c / 4]`)
/// placed around the item's `cell` into `corners`. Returns the inverse depth the projection
/// takes (`view_scale / cell[2]`).
fn sprite_corners_x87(drops: &WindscreenDrops, c: usize) -> bh::X {
    let work = &drops.work;
    let inverse_depth = bh::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    ) / work.cell[2].ld();
    // Axis by axis (x, y, z), each of the six corners.
    for axis in 0..3 {
        for corner in 0..6 {
            let i = 3 * corner + axis;
            let sum = work.cell[axis].ld() + work.scaled[c / 4 + i].ld();
            work.corners[i].st(sum);
        }
    }
    inverse_depth
}

/// `sprite_project`: the centre and the six corners projected to the screen into the six
/// vertices at `e` of the batch's records (0x20 bytes each; the depth at +8), `inverse_depth`
/// being `view_scale / depth`. Same order and rounding as the x87.
fn sprite_project_x87(
    drops: &WindscreenDrops,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    e: usize,
    inverse_depth: bh::X,
) {
    let work = &drops.work;
    let records = &gfx.card.windscreen_drop_batch.records;
    let centre =
        |axis: usize| crate::scene::polygon_work::front_end().window.view_centre[axis].ld();
    work.cell[2].st(inverse_depth);
    // The centre.
    let up = -(work.cell[1].ld() * work.cell[2].ld());
    let across = work.cell[0].ld() * work.cell[2].ld() + centre(0);
    work.cell[0].st(across);
    work.cell[1].st(up + centre(1));
    // The corners: x and y written both at the vertex's +0x18 / +0x1C and at its +0 / +4.
    for v in 0..6 {
        let at = e + 0x20 * v;
        let x = work.corners[3 * v].ld() * work.cell[2].ld();
        let y = -(work.corners[3 * v + 1].ld() * work.cell[2].ld());
        let y = y + centre(1);
        st_at(records, at + 0x1C, y);
        st_at(records, at + 0x4, y);
        let x = x + centre(0);
        st_at(records, at + 0x18, x);
        st_at(records, at, x);
    }
    // The vertices' screen places moved around the centre by the scales.
    let screen_x = work.cell[0].ld() * ld_cell(&drops.hex.scale[0]);
    let screen_y = work.cell[1].ld() * ld_cell(&drops.hex.scale[1]);
    work.screen[0].st(screen_x);
    work.screen[1].st(screen_y);
    for v in 0..6 {
        let at = e + 0x20 * v;
        let x = (ld_record(at + 0x18, gfx) - work.cell[0].ld()) * ld_cell(&drops.hex.scale[2])
            + work.screen[0].ld();
        st_at(records, at + 0x18, x);
        let y = (ld_record(at + 0x1C, gfx) - work.cell[1].ld()) * ld_cell(&drops.hex.scale[3])
            + work.screen[1].ld();
        st_at(records, at + 0x1C, y);
    }
    // Every vertex's depth.
    let depth =
        (drops.state.depth.ld() + effects.setup.kind_floats.kind_float.ld()) * work.inverse.ld();
    for v in 0..6 {
        st_at(records, e + 0x20 * v + 0x8, depth);
    }
}

#[cfg(test)]
mod reference_vectors {
    use crate::test_support::own::effects;

    use crate::effects::typed_sync::{overlay_laid, with_overlays};

    /// Runs `f` with the windscreen drops' statics holding the buffer's bytes (`typed_sync`).
    fn windscreen_drops<R>(mk: crate::effects::asm_test::Operand, f: impl FnOnce() -> R) -> R {
        with_overlays(
            mk,
            &[
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.tilt),
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.state),
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.work),
            ],
            f,
        )
    }
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, canon, recorded_writes};

    /// Runs `f` with `HEX` holding the buffer's bytes (`typed_sync`).
    fn hex<R>(mk: crate::effects::asm_test::Operand, f: impl FnOnce() -> R) -> R {
        with_overlays(
            mk,
            &[overlay_laid(
                &crate::test_support::own::effects().windscreen_drops.hex,
            )],
            f,
        )
    }

    /// The test buffer's `len` bytes at `at` run through `f` as `bytes` (a stand-in for the
    /// item or record found by address): `bytes` holds them from offset 0 and
    /// what `f` leaves goes back to the buffer, the stand-in as it was.
    fn staged<R>(
        at: crate::effects::asm_test::Operand,
        from: usize,
        len: usize,
        bytes: &impl crate::session::ByteBuffer,
        f: impl FnOnce() -> R,
    ) -> R {
        let host = (at + from).at(0, len);
        let saved: Vec<u8> = (from..from + len).map(|i| bytes.get_byte(i)).collect();
        for i in 0..len {
            // SAFETY: byte `i < len` of the `len` that `at` checked lie in the operand's test
            // buffer, which the recorded run keeps alive and nothing else reaches meanwhile.
            let b = unsafe { host.add(i).read() };
            bytes.write_bytes_at(from + i, &[b], "a test byte").unwrap();
        }
        let out = f();
        for (i, &b) in saved.iter().enumerate() {
            // SAFETY: byte `i < len` of the `len` that `at` checked lie in the operand's test
            // buffer, which the recorded run keeps alive and nothing else reaches meanwhile.
            unsafe { host.add(i).write(bytes.get_byte(from + i)) };
            bytes.write_bytes_at(from + i, &[b], "a test byte").unwrap();
        }
        out
    }

    #[test]
    fn drift_cell_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xACC83,
                    hi: 0xB0250,
                    reads: &[
                        (0xACC9F, F32),
                        (0xACCA3, F32),
                        (0xACCA7, F32),
                        (0xAF882, F32),
                        (0xAF886, F32),
                        (0xAF88E, F32),
                        (0xAF8E6, F32),
                        (0xAFA82, F32),
                        (0xAFC7E, F32),
                    ],
                    writes: &[(0xAF832, F32), (0xAF836, F32)],
                },
                Op::Index(&[0, 4, 12, 100, 512, 1020]),
            ],
            sincos: false,
        };
        recorded_writes(
            "effects.windscreen_drops_frame.drift_cell_x87",
            &SPEC,
            |b| {
                windscreen_drops(b[0], || {
                    super::drift_cell_x87(
                        &crate::test_support::own::effects().windscreen_drops,
                        b[1].number(),
                    );
                });
            },
        );
    }

    #[test]
    fn drift_move_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xACC83,
                    hi: 0xC9402,
                    reads: &[
                        (0xAF832, F32),
                        (0xAF836, F32),
                        (0xAFC82, F32),
                        (0xAFC86, F32),
                        (0xAFC8E, F32),
                        (0xAFCE6, F32),
                        (0xAFE82, F32),
                        (0xB007E, F32),
                        (0xB12EA, F32),
                    ],
                    writes: &[],
                },
                Op::Index(&[0, 4, 12, 28]),
                Op::Base {
                    lo: 0x14,
                    hi: 0x24,
                    reads: &[(0x14, F32), (0x18, F32), (0x1C, F32), (0x20, F32)],
                    writes: &[(0x14, F32), (0x18, F32), (0x1C, F32), (0x20, F32)],
                },
            ],
            sincos: false,
        };
        recorded_writes(
            "effects.windscreen_drops_frame.drift_move_x87",
            &SPEC,
            |b| {
                windscreen_drops(b[0], || {
                    hex(b[0], || {
                        staged(
                            b[2],
                            0x14,
                            0x10,
                            &crate::test_support::own::effects()
                                .windscreen_drops
                                .hex
                                .items,
                            || {
                                super::drift_move_x87(
                                    &crate::test_support::own::effects().windscreen_drops,
                                    b[1].number(),
                                    0,
                                );
                            },
                        );
                    });
                });
            },
        );
    }

    #[test]
    fn sprite_depth_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xACC83,
                hi: 0xC9402,
                reads: &[
                    (0xAFCC6, F32),
                    (0xAFCCA, F32),
                    (0xAFCCE, F32),
                    (0xAFCD2, F32),
                    (0xAFCD6, F32),
                    (0xAFCDA, F32),
                    (0xAFCDE, F32),
                    (0xAFCE2, F32),
                    (0xAFCE6, F32),
                    (0xAFCEA, F32),
                    (0xAFCEE, F32),
                    (0xB0242, I32),
                    (0xB0246, I32),
                    (0xC9366, F32),
                ],
                writes: &[
                    (0xAF7F4, F32),
                    (0xAFD6A, F32),
                    (0xAFD6E, F32),
                    (0xAFD72, F32),
                ],
            }],
            sincos: false,
        };
        recorded_writes(
            "effects.windscreen_drops_frame.sprite_depth_x87",
            &SPEC,
            |b| {
                windscreen_drops(b[0], || {
                    hex(b[0], || {
                        super::sprite_depth_x87(
                            &crate::test_support::own::effects().windscreen_drops,
                        );
                    });
                });
            },
        );
    }

    #[test]
    fn sprite_corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0xACC83,
                    hi: 0xB0250,
                    reads: &[
                        (0xAFD6A, F32),
                        (0xAFD6E, F32),
                        (0xAFD72, F32),
                        (0xAFDBE, F32),
                        (0xAFDC2, F32),
                        (0xAFDC6, F32),
                        (0xAFDCA, F32),
                        (0xAFDCE, F32),
                        (0xAFDD2, F32),
                        (0xAFDD6, F32),
                        (0xAFDDA, F32),
                        (0xAFDDE, F32),
                        (0xAFDE2, F32),
                        (0xAFDE6, F32),
                        (0xAFDEA, F32),
                        (0xAFDEE, F32),
                        (0xAFDF2, F32),
                        (0xAFDF6, F32),
                        (0xAFDFA, F32),
                        (0xAFDFE, F32),
                        (0xAFE02, F32),
                        (0xAFE06, F32),
                        (0xAFE0A, F32),
                        (0xAFE0E, F32),
                        (0xAFE12, F32),
                        (0xAFE16, F32),
                        (0xAFE1A, F32),
                        (0xAFE1E, F32),
                        (0xAFE22, F32),
                        (0xAFE26, F32),
                        (0xAFE2A, F32),
                        (0xAFE2E, F32),
                        (0xAFE32, F32),
                        (0xAFE36, F32),
                        (0xAFE3A, F32),
                        (0xAFE3E, F32),
                        (0xAFE42, F32),
                        (0xAFE46, F32),
                        (0xAFE4A, F32),
                        (0xAFE4E, F32),
                        (0xAFE52, F32),
                        (0xAFE56, F32),
                        (0xAFE5A, F32),
                        (0xAFE5E, F32),
                        (0xAFE62, F32),
                        (0xAFE66, F32),
                        (0xAFE6A, F32),
                        (0xAFE6E, F32),
                        (0xAFE72, F32),
                        (0xAFE76, F32),
                        (0xAFE7A, F32),
                        (0xAFE7E, F32),
                        (0xAFE82, F32),
                        (0xAFE86, F32),
                        (0xAFE8A, F32),
                        (0xAFE8E, F32),
                        (0xAFE92, F32),
                        (0xAFFB6, F32),
                        (0xAFFBA, F32),
                        (0xAFFBE, F32),
                        (0xAFFC2, F32),
                        (0xAFFC6, F32),
                        (0xAFFCA, F32),
                        (0xAFFCE, F32),
                        (0xAFFD2, F32),
                        (0xAFFD6, F32),
                        (0xAFFDA, F32),
                        (0xAFFDE, F32),
                        (0xAFFE2, F32),
                        (0xAFFE6, F32),
                        (0xAFFEA, F32),
                        (0xAFFEE, F32),
                        (0xAFFF2, F32),
                        (0xAFFF6, F32),
                        (0xAFFFA, F32),
                    ],
                    writes: &[
                        (0xAFD76, F32),
                        (0xAFD7A, F32),
                        (0xAFD7E, F32),
                        (0xAFD82, F32),
                        (0xAFD86, F32),
                        (0xAFD8A, F32),
                        (0xAFD8E, F32),
                        (0xAFD92, F32),
                        (0xAFD96, F32),
                        (0xAFD9A, F32),
                        (0xAFD9E, F32),
                        (0xAFDA2, F32),
                        (0xAFDA6, F32),
                        (0xAFDAA, F32),
                        (0xAFDAE, F32),
                        (0xAFDB2, F32),
                        (0xAFDB6, F32),
                        (0xAFDBA, F32),
                    ],
                },
                Op::Index(&[0, 72, 144, 504]),
            ],
            sincos: false,
        };
        recorded_writes(
            "effects.windscreen_drops_frame.sprite_corners_x87",
            &SPEC,
            |b| {
                canon(
                    windscreen_drops(b[1], || {
                        super::sprite_corners_x87(
                            &crate::test_support::own::effects().windscreen_drops,
                            b[2].number(),
                        )
                    })
                    .0,
                )
            },
        );
    }

    #[test]
    fn sprite_project_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x2F8,
                    hi: 0xC9402,
                    reads: &[
                        (0x35C, F32),
                        (0xAF7F4, F32),
                        (0xAFCC2, F32),
                        (0xAFD5E, F32),
                        (0xAFD62, F32),
                        (0xAFD6A, F32),
                        (0xAFD6E, F32),
                        (0xAFD72, F32),
                        (0xAFD76, F32),
                        (0xAFD7A, F32),
                        (0xAFD82, F32),
                        (0xAFD86, F32),
                        (0xAFD8E, F32),
                        (0xAFD92, F32),
                        (0xAFD9A, F32),
                        (0xAFD9E, F32),
                        (0xAFDA6, F32),
                        (0xAFDAA, F32),
                        (0xAFDB2, F32),
                        (0xAFDB6, F32),
                        (0xB1252, F32),
                        (0xB1256, F32),
                        (0xB125A, F32),
                        (0xB125E, F32),
                    ],
                    writes: &[
                        (0xAFD5E, F32),
                        (0xAFD62, F32),
                        (0xAFD6A, F32),
                        (0xAFD6E, F32),
                        (0xAFD72, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xC0,
                    reads: &[
                        (0x18, F32),
                        (0x1C, F32),
                        (0x38, F32),
                        (0x3C, F32),
                        (0x58, F32),
                        (0x5C, F32),
                        (0x78, F32),
                        (0x7C, F32),
                        (0x98, F32),
                        (0x9C, F32),
                        (0xB8, F32),
                        (0xBC, F32),
                    ],
                    writes: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x28, F32),
                        (0x38, F32),
                        (0x3C, F32),
                        (0x40, F32),
                        (0x44, F32),
                        (0x48, F32),
                        (0x58, F32),
                        (0x5C, F32),
                        (0x60, F32),
                        (0x64, F32),
                        (0x68, F32),
                        (0x78, F32),
                        (0x7C, F32),
                        (0x80, F32),
                        (0x84, F32),
                        (0x88, F32),
                        (0x98, F32),
                        (0x9C, F32),
                        (0xA0, F32),
                        (0xA4, F32),
                        (0xA8, F32),
                        (0xB8, F32),
                        (0xBC, F32),
                    ],
                },
                Op::Float,
            ],
            sincos: false,
        };
        recorded_writes(
            "effects.windscreen_drops_frame.sprite_project_x87",
            &SPEC,
            |b| {
                windscreen_drops(b[1], || {
                    with_overlays(b[1], &[overlay_laid(&effects().setup.kind_floats)], || {
                        hex(b[1], || {
                            staged(
                                b[2],
                                0,
                                0xC0,
                                &crate::gfx::owner::gfx().card.windscreen_drop_batch.records,
                                || {
                                    super::sprite_project_x87(
                                        &crate::test_support::own::effects().windscreen_drops,
                                        crate::gfx::owner::gfx(),
                                        crate::test_support::own::effects(),
                                        0,
                                        super::bh::ld(b[3].number() as u32),
                                    );
                                },
                            );
                        });
                    });
                });
            },
        );
    }
}
