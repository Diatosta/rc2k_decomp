//! `effects::windscreen_drops_frame` (0x4BC0B1, with its chunk 0x4BB94D .. 0x4BBE62): the windscreen drops'
//! frame. With the drift on (`dword_7652A0`) `effects::windscreen_drops_drift`; `effects::windscreen_drops_setup`;
//! then each live item (counted in `dword_814710`): with the drift on, its motion eased
//! toward the drift at its cell and its place moved; it dies off the map (x or z past
//! 0x7FF, as `(x | z << 1) & 0x800`) or on `g_marksEdgeMask`; else its sprite: the place
//! turned, unless the depth is below 4096.0 the six corners (its kind's offsets)
//! projected into the buffer at `dword_816232` (0xC0 bytes each; at `unk_628A44` back to
//! `unk_625A44`, `dword_62423C` 0x300, `dword_625A40` 0x180 and the batch drawn by
//! `d3d::draw_marks_windscreen_drops`, the platform's). Then, with fewer live than the settings'
//! +0x273C and the drift on, `effects::windscreen_drops_spawn`.
//!
//! Works on `g_marksSettings`.
//!
//! The float steps run on the x87 model (`math::x87_float`), in the original's order and
//! rounding. Refuses, before anything, null settings and a live item of a kind past 7.

use crate::effects::expire_windscreen_drops::{ITEM_LEN, ITEMS};
use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner_frame::Frame;
/// `unk_628A44`, past the 0x40 records of 0xC0 bytes.
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

/// The original's 0x4BC107 .. 0x4BC12E: the drift at the item's cell (`g_marksWindscreenDropCurve`[k]) into `flt_814772`, `flt_814776`.
fn drift_cell(drops: &WindscreenDrops, k: u32) {
    drift_cell_x87(drops, k as usize);
}

/// The original's 0x4BC131 .. 0x4BC17B: the item's motion (+0x1C, +0x20) eased toward the drift times its kind's `flt_814BC2`[k] by `flt_81622A`, its place moved by it.
fn drift_move(drops: &WindscreenDrops, k: u32, s: u32) {
    drift_move_x87(drops, k as usize, s as usize);
}

/// The original's 0x4BC1DD .. 0x4BC1EF and 0x4BB94D .. 0x4BB9BE: the cell place scaled and turned into `flt_814CAA` .., the depth also in `flt_814734`.
fn sprite_depth(drops: &WindscreenDrops) {
    sprite_depth_x87(drops);
}

/// The original's 0x4BB9CE .. 0x4BBB1E: the six corners (the kind's offsets at `flt_814CFE`[c] ..) into `flt_814CB6` .., the depth's inverse (left on the x87 stack, so returned).
fn sprite_corners(drops: &WindscreenDrops, c: u32, scene: &crate::scene::owner::Scene) -> bh::X {
    sprite_corners_x87(drops, c as usize, scene)
}

/// The original's 0x4BBB24 .. 0x4BBE28: the centre and corners projected into the buffer at e (six vertices), drawn in by `flt_81619A` / `E`, their depth.
fn sprite_project(
    drops: &WindscreenDrops,
    e: u32,
    inverse_depth: bh::X,
    scene: &crate::scene::owner::Scene,
) {
    sprite_project_x87(drops, e as usize, inverse_depth, scene);
}

/// Why it refuses, if it does.
fn refusal(drops: &WindscreenDrops) -> Result<Option<&'static str>, Refused> {
    Ok({
        if crate::effects::setup::marks().is_err() {
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
pub fn draw_batch() -> Result<(), Refused> {
    crate::d3d::draw_marks_windscreen_drops::draw_marks_windscreen_drops();
    Ok(())
}

/// The Rust version (see the header).
pub fn windscreen_drops_frame(
    frame: &Frame,
    masks: &Masks,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    if let Some(why) = refusal(drops)? {
        return Err(BadInput(why).into());
    }
    {
        if setup.kind_floats.running.bits() != 0 {
            crate::effects::windscreen_drops_drift::windscreen_drops_drift(frame, drops, effects)?;
        }
        crate::effects::windscreen_drops_setup::windscreen_drops_setup(drops, scene);
        drops.state.live.set_bits(0u32);
        for n in 0..ITEMS {
            let e = n * ITEM_LEN;
            if get(drops, e) != 1 {
                continue;
            }
            drops.state.live.set(drops.state.live.get().wrapping_add(1));
            if setup.kind_floats.running.bits() != 0 {
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
            if (drops.work.cell[2].bits() as i32) < 0x4580_0000 {
                continue;
            }
            let inverse_depth = sprite_corners(drops, shape.wrapping_mul(0x48), scene);
            sprite_project(drops, drops.hex.buffer.get(), inverse_depth, scene);
            let next = drops.hex.buffer.get().wrapping_add(0xC0);
            drops.hex.buffer.set(next);
            if next as usize == BUFFER_BYTES {
                drops.hex.buffer.set(0);
                gfx.card.windscreen_drop_batch.index_count.set(0x300);
                gfx.card.windscreen_drop_batch.vertex_count.set(0x180);
                draw_batch()?;
            }
        }
        let marks = crate::effects::setup::marks()?;
        let live: u32 = drops.state.live.bits();
        if (live as i32) < marks.listener_switch_2.get() as i32
            && setup.kind_floats.running.bits() != 0
        {
            crate::effects::windscreen_drops_spawn::windscreen_drops_spawn(drops, effects)?;
        }
    }
    Ok(())
}

use crate::effects::owner::effects;
use crate::effects::plain::{ld_at, ld_cell, st_at};
use crate::math::x87_float as bh;

/// `drift_cell`: the drift at the item's cell (`curve[k / 4]`) into `drift`: x from the
/// shape tilt's point, y its second coordinate.
fn drift_cell_x87(drops: &WindscreenDrops, k: usize) {
    let x = -(bh::ld(drops.work.curve[k / 4].bits()) * bh::ld(drops.tilt.point[2].bits()))
        + bh::ld(drops.tilt.point[0].bits());
    drops.work.drift[0].set_bits(bh::st(x));
    let y = bh::ld(drops.tilt.point[1].bits());
    drops.work.drift[1].set_bits(bh::st(y));
}

/// `drift_move`: the item at byte `s` of the items has its motion eased toward the drift
/// (scaled by its kind's radius `radii[k / 4]`) and its place moved by the new motion.
fn drift_move_x87(drops: &WindscreenDrops, k: usize, s: usize) {
    let ld_item = |field: u32| ld_at(&drops.hex.items, s + field as usize);
    let st_item = |field: u32, x| st_at(&drops.hex.items, s + field as usize, x);
    for (motion, drift) in [(ITEM_MOTION_X, 0), (ITEM_MOTION_Z, 1)] {
        let toward =
            bh::ld(drops.work.drift[drift].bits()) * bh::ld(drops.work.radii[k / 4].bits());
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
/// depth, which is `cell[2]` too). Same order and rounding as the original's x87 block.
fn sprite_depth_x87(drops: &WindscreenDrops) {
    let work = &drops.work;
    let x = bh::int(work.whole[0].bits() as i32);
    let z = ld_cell(&drops.hex.frame_k) - bh::int(work.whole[1].bits() as i32);
    let z = z * bh::ld(work.shares[1].bits());
    let x = x * bh::ld(work.shares[0].bits());
    // Output i: x times the first axes row, z times the second, plus the view's place. The
    // last one adds the products the other way round (the original's order) and is also
    // the depth.
    for i in 0..3 {
        let from_x = x * bh::ld(work.axes[i].bits());
        let from_z = z * bh::ld(work.axes[3 + i].bits());
        let sum = if i < 2 {
            from_x + from_z
        } else {
            from_z + from_x
        };
        let out = sum + bh::ld(work.place[i].bits());
        if i == 2 {
            drops.state.depth.set_bits(bh::st(out));
        }
        work.cell[i].set_bits(bh::st(out));
    }
}

/// `sprite_corners`: the six corners of the kind's shape (its offsets from `scaled[c / 4]`)
/// placed around the item's `cell` into `corners`. Returns the inverse depth the projection
/// takes (`view_scale / cell[2]`).
fn sprite_corners_x87(
    drops: &WindscreenDrops,
    c: usize,
    scene: &crate::scene::owner::Scene,
) -> bh::X {
    let work = &drops.work;
    let inverse_depth = bh::ld(scene.drawing.scene.view_scale.bits()) / bh::ld(work.cell[2].bits());
    // Axis by axis (x, y, z), each of the six corners.
    for axis in 0..3 {
        for corner in 0..6 {
            let i = 3 * corner + axis;
            let sum = bh::ld(work.cell[axis].bits()) + bh::ld(work.scaled[c / 4 + i].bits());
            work.corners[i].set_bits(bh::st(sum));
        }
    }
    inverse_depth
}

/// `sprite_project`: the centre and the six corners projected to the screen into the six
/// vertices at `e` of the batch's records (0x20 bytes each; the depth at +8), `inverse_depth`
/// being `view_scale / depth`. Same order and rounding as the original's x87 block.
fn sprite_project_x87(
    drops: &WindscreenDrops,
    e: usize,
    inverse_depth: bh::X,
    scene: &crate::scene::owner::Scene,
) {
    let gfx = crate::gfx::owner::gfx();
    let work = &drops.work;
    let records = &gfx.card.windscreen_drop_batch.records;
    let centre = |axis: usize| bh::ld(scene.drawing.scene.view_centre[axis].bits());
    work.cell[2].set_bits(bh::st(inverse_depth));
    // The centre.
    let up = -(bh::ld(work.cell[1].bits()) * bh::ld(work.cell[2].bits()));
    let across = bh::ld(work.cell[0].bits()) * bh::ld(work.cell[2].bits()) + centre(0);
    work.cell[0].set_bits(bh::st(across));
    work.cell[1].set_bits(bh::st(up + centre(1)));
    // The corners: x and y written both at the vertex's +0x18 / +0x1C and at its +0 / +4.
    for v in 0..6 {
        let at = e + 0x20 * v;
        let x = bh::ld(work.corners[3 * v].bits()) * bh::ld(work.cell[2].bits());
        let y = -(bh::ld(work.corners[3 * v + 1].bits()) * bh::ld(work.cell[2].bits()));
        let y = y + centre(1);
        st_at(records, at + 0x1C, y);
        st_at(records, at + 0x4, y);
        let x = x + centre(0);
        st_at(records, at + 0x18, x);
        st_at(records, at, x);
    }
    // The vertices' screen places moved around the centre by the scales.
    let screen_x = bh::ld(work.cell[0].bits()) * ld_cell(&drops.hex.scale[0]);
    let screen_y = bh::ld(work.cell[1].bits()) * ld_cell(&drops.hex.scale[1]);
    work.screen[0].set_bits(bh::st(screen_x));
    work.screen[1].set_bits(bh::st(screen_y));
    for v in 0..6 {
        let at = e + 0x20 * v;
        let x = (ld_record(at + 0x18, gfx) - bh::ld(work.cell[0].bits()))
            * ld_cell(&drops.hex.scale[2])
            + bh::ld(work.screen[0].bits());
        st_at(records, at + 0x18, x);
        let y = (ld_record(at + 0x1C, gfx) - bh::ld(work.cell[1].bits()))
            * ld_cell(&drops.hex.scale[3])
            + bh::ld(work.screen[1].bits());
        st_at(records, at + 0x1C, y);
    }
    // Every vertex's depth.
    let depth = (bh::ld(drops.state.depth.bits())
        + bh::ld(effects().setup.kind_floats.kind_float.bits()))
        * bh::ld(work.inverse.bits());
    for v in 0..6 {
        st_at(records, e + 0x20 * v + 0x8, depth);
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn drift_cell_x87(mk: usize, k: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + {k} + 0xAF882]",
            "fmul dword ptr [{mk} + 0xACCA7]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{mk} + 0xACC9F]",
            "fstp dword ptr [{mk} + 0xAF832]",
            "fld dword ptr [{mk} + 0xACCA3]",
            "fstp dword ptr [{mk} + 0xAF836]",
            mk = in(reg) mk,
            k = in(reg) k,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn drift_move_x87(mk: usize, k: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0xAF832]",
            "fmul dword ptr [{mk} + {k} + 0xAFC82]",
            "fsub dword ptr [{s} + 0x1C]",
            "fmul dword ptr [{mk} + 0xB12EA]",
            "fadd dword ptr [{s} + 0x1C]",
            "fstp dword ptr [{s} + 0x1C]",
            "fld dword ptr [{mk} + 0xAF836]",
            "fmul dword ptr [{mk} + {k} + 0xAFC82]",
            "fsub dword ptr [{s} + 0x20]",
            "fmul dword ptr [{mk} + 0xB12EA]",
            "fadd dword ptr [{s} + 0x20]",
            "fstp dword ptr [{s} + 0x20]",
            "fld dword ptr [{s} + 0x14]",
            "fadd dword ptr [{s} + 0x1C]",
            "fld dword ptr [{s} + 0x18]",
            "fadd dword ptr [{s} + 0x20]",
            "fstp dword ptr [{s} + 0x18]",
            "fstp dword ptr [{s} + 0x14]",
            mk = in(reg) mk,
            k = in(reg) k,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn sprite_depth_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{mk} + 0xB0242]",
            "fld dword ptr [{mk} + 0xC9366]",
            "fisub dword ptr [{mk} + 0xB0246]",
            "fmul dword ptr [{mk} + 0xAFCCA]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0xAFCC6]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0xAFCDA]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0xAFCE6]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0xAFCCE]",
            "fstp dword ptr [{mk} + 0xAFD6A]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0xAFCDE]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0xAFCEA]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0xAFCD2]",
            "fstp dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xAFCEE]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0xAFCE2]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0xAFCD6]",
            "fst dword ptr [{mk} + 0xAF7F4]",
            "fstp dword ptr [{mk} + 0xAFD72]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn sprite_corners_x87(rc: usize, mk: usize, c: usize) -> f64 {
        let mut out = 0f64;
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0xAFD72]",
            "fdivr dword ptr [{rc} + 0x224088]",
            "fld dword ptr [{mk} + 0xAFD6A]",
            "fadd dword ptr [{mk} + {c} + 0xAFDBE]",
            "fstp dword ptr [{mk} + 0xAFD76]",
            "fld dword ptr [{mk} + 0xAFD6A]",
            "fadd dword ptr [{mk} + {c} + 0xAFDCA]",
            "fstp dword ptr [{mk} + 0xAFD82]",
            "fld dword ptr [{mk} + 0xAFD6A]",
            "fadd dword ptr [{mk} + {c} + 0xAFDD6]",
            "fstp dword ptr [{mk} + 0xAFD8E]",
            "fld dword ptr [{mk} + 0xAFD6A]",
            "fadd dword ptr [{mk} + {c} + 0xAFDE2]",
            "fstp dword ptr [{mk} + 0xAFD9A]",
            "fld dword ptr [{mk} + 0xAFD6A]",
            "fadd dword ptr [{mk} + {c} + 0xAFDEE]",
            "fstp dword ptr [{mk} + 0xAFDA6]",
            "fld dword ptr [{mk} + 0xAFD6A]",
            "fadd dword ptr [{mk} + {c} + 0xAFDFA]",
            "fstp dword ptr [{mk} + 0xAFDB2]",
            "fld dword ptr [{mk} + 0xAFD6E]",
            "fadd dword ptr [{mk} + {c} + 0xAFDC2]",
            "fstp dword ptr [{mk} + 0xAFD7A]",
            "fld dword ptr [{mk} + 0xAFD6E]",
            "fadd dword ptr [{mk} + {c} + 0xAFDCE]",
            "fstp dword ptr [{mk} + 0xAFD86]",
            "fld dword ptr [{mk} + 0xAFD6E]",
            "fadd dword ptr [{mk} + {c} + 0xAFDDA]",
            "fstp dword ptr [{mk} + 0xAFD92]",
            "fld dword ptr [{mk} + 0xAFD6E]",
            "fadd dword ptr [{mk} + {c} + 0xAFDE6]",
            "fstp dword ptr [{mk} + 0xAFD9E]",
            "fld dword ptr [{mk} + 0xAFD6E]",
            "fadd dword ptr [{mk} + {c} + 0xAFDF2]",
            "fstp dword ptr [{mk} + 0xAFDAA]",
            "fld dword ptr [{mk} + 0xAFD6E]",
            "fadd dword ptr [{mk} + {c} + 0xAFDFE]",
            "fstp dword ptr [{mk} + 0xAFDB6]",
            "fld dword ptr [{mk} + 0xAFD72]",
            "fadd dword ptr [{mk} + {c} + 0xAFDC6]",
            "fstp dword ptr [{mk} + 0xAFD7E]",
            "fld dword ptr [{mk} + 0xAFD72]",
            "fadd dword ptr [{mk} + {c} + 0xAFDD2]",
            "fstp dword ptr [{mk} + 0xAFD8A]",
            "fld dword ptr [{mk} + 0xAFD72]",
            "fadd dword ptr [{mk} + {c} + 0xAFDDE]",
            "fstp dword ptr [{mk} + 0xAFD96]",
            "fld dword ptr [{mk} + 0xAFD72]",
            "fadd dword ptr [{mk} + {c} + 0xAFDEA]",
            "fstp dword ptr [{mk} + 0xAFDA2]",
            "fld dword ptr [{mk} + 0xAFD72]",
            "fadd dword ptr [{mk} + {c} + 0xAFDF6]",
            "fstp dword ptr [{mk} + 0xAFDAE]",
            "fld dword ptr [{mk} + 0xAFD72]",
            "fadd dword ptr [{mk} + {c} + 0xAFE02]",
            "fstp dword ptr [{mk} + 0xAFDBA]",
            "fstp qword ptr [{out}]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            c = in(reg) c,
            out = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
        out
    }

    pub(super) fn sprite_project_x87(rc: usize, mk: usize, e: usize, pre0: u32) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{pre0}]",
            "fstp dword ptr [{mk} + 0xAFD72]",
            "fld dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            ".byte 0xD9, 0xE0",
            "fld dword ptr [{mk} + 0xAFD6A]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0xAFD6A]",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0xAFD6E]",
            "fld dword ptr [{mk} + 0xAFD76]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            "fld dword ptr [{mk} + 0xAFD7A]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fst dword ptr [{e} + 0x1C]",
            "fstp dword ptr [{e} + 0x4]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fst dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x0]",
            "fld dword ptr [{mk} + 0xAFD82]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            "fld dword ptr [{mk} + 0xAFD86]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fst dword ptr [{e} + 0x3C]",
            "fstp dword ptr [{e} + 0x24]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fst dword ptr [{e} + 0x38]",
            "fstp dword ptr [{e} + 0x20]",
            "fld dword ptr [{mk} + 0xAFD8E]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            "fld dword ptr [{mk} + 0xAFD92]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fst dword ptr [{e} + 0x5C]",
            "fstp dword ptr [{e} + 0x44]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fst dword ptr [{e} + 0x58]",
            "fstp dword ptr [{e} + 0x40]",
            "fld dword ptr [{mk} + 0xAFD9A]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            "fld dword ptr [{mk} + 0xAFD9E]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fst dword ptr [{e} + 0x7C]",
            "fstp dword ptr [{e} + 0x64]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fst dword ptr [{e} + 0x78]",
            "fstp dword ptr [{e} + 0x60]",
            "fld dword ptr [{mk} + 0xAFDA6]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            "fld dword ptr [{mk} + 0xAFDAA]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fst dword ptr [{e} + 0x9C]",
            "fstp dword ptr [{e} + 0x84]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fst dword ptr [{e} + 0x98]",
            "fstp dword ptr [{e} + 0x80]",
            "fld dword ptr [{mk} + 0xAFDB2]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            "fld dword ptr [{mk} + 0xAFDB6]",
            "fmul dword ptr [{mk} + 0xAFD72]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fst dword ptr [{e} + 0xBC]",
            "fstp dword ptr [{e} + 0xA4]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fst dword ptr [{e} + 0xB8]",
            "fstp dword ptr [{e} + 0xA0]",
            "fld dword ptr [{mk} + 0xAFD6A]",
            "fmul dword ptr [{mk} + 0xB1252]",
            "fld dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xB1256]",
            ".byte 0xD9, 0xC9",
            "fstp dword ptr [{mk} + 0xAFD5E]",
            "fstp dword ptr [{mk} + 0xAFD62]",
            "fld dword ptr [{e} + 0x18]",
            "fsub dword ptr [{mk} + 0xAFD6A]",
            "fmul dword ptr [{mk} + 0xB125A]",
            "fadd dword ptr [{mk} + 0xAFD5E]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x1C]",
            "fsub dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xB125E]",
            "fadd dword ptr [{mk} + 0xAFD62]",
            "fstp dword ptr [{e} + 0x1C]",
            "fld dword ptr [{e} + 0x38]",
            "fsub dword ptr [{mk} + 0xAFD6A]",
            "fmul dword ptr [{mk} + 0xB125A]",
            "fadd dword ptr [{mk} + 0xAFD5E]",
            "fstp dword ptr [{e} + 0x38]",
            "fld dword ptr [{e} + 0x3C]",
            "fsub dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xB125E]",
            "fadd dword ptr [{mk} + 0xAFD62]",
            "fstp dword ptr [{e} + 0x3C]",
            "fld dword ptr [{e} + 0x58]",
            "fsub dword ptr [{mk} + 0xAFD6A]",
            "fmul dword ptr [{mk} + 0xB125A]",
            "fadd dword ptr [{mk} + 0xAFD5E]",
            "fstp dword ptr [{e} + 0x58]",
            "fld dword ptr [{e} + 0x5C]",
            "fsub dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xB125E]",
            "fadd dword ptr [{mk} + 0xAFD62]",
            "fstp dword ptr [{e} + 0x5C]",
            "fld dword ptr [{e} + 0x78]",
            "fsub dword ptr [{mk} + 0xAFD6A]",
            "fmul dword ptr [{mk} + 0xB125A]",
            "fadd dword ptr [{mk} + 0xAFD5E]",
            "fstp dword ptr [{e} + 0x78]",
            "fld dword ptr [{e} + 0x7C]",
            "fsub dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xB125E]",
            "fadd dword ptr [{mk} + 0xAFD62]",
            "fstp dword ptr [{e} + 0x7C]",
            "fld dword ptr [{e} + 0x98]",
            "fsub dword ptr [{mk} + 0xAFD6A]",
            "fmul dword ptr [{mk} + 0xB125A]",
            "fadd dword ptr [{mk} + 0xAFD5E]",
            "fstp dword ptr [{e} + 0x98]",
            "fld dword ptr [{e} + 0x9C]",
            "fsub dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xB125E]",
            "fadd dword ptr [{mk} + 0xAFD62]",
            "fstp dword ptr [{e} + 0x9C]",
            "fld dword ptr [{e} + 0xB8]",
            "fsub dword ptr [{mk} + 0xAFD6A]",
            "fmul dword ptr [{mk} + 0xB125A]",
            "fadd dword ptr [{mk} + 0xAFD5E]",
            "fstp dword ptr [{e} + 0xB8]",
            "fld dword ptr [{e} + 0xBC]",
            "fsub dword ptr [{mk} + 0xAFD6E]",
            "fmul dword ptr [{mk} + 0xB125E]",
            "fadd dword ptr [{mk} + 0xAFD62]",
            "fstp dword ptr [{e} + 0xBC]",
            "fld dword ptr [{mk} + 0xAF7F4]",
            "fadd dword ptr [{mk} + 0x35C]",
            "fmul dword ptr [{mk} + 0xAFCC2]",
            "fst dword ptr [{e} + 0x8]",
            "fst dword ptr [{e} + 0x28]",
            "fst dword ptr [{e} + 0x48]",
            "fst dword ptr [{e} + 0x68]",
            "fst dword ptr [{e} + 0x88]",
            "fstp dword ptr [{e} + 0xA8]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            e = in(reg) e,
            pre0 = in(reg) &raw const pre0,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::owner::effects;
    use crate::effects::tilt_shape::ShapeTilt;
    use crate::effects::tuning::MarksKindFloats;
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_state::{WindscreenDropState, WindscreenDropWork};

    /// Runs `f` with the windscreen drops' statics holding the buffer's bytes (`typed_sync`).
    fn windscreen_drops<R>(mk: usize, f: impl FnOnce() -> R) -> R {
        with_overlays(
            mk,
            &[
                overlay(
                    &crate::effects::owner::EFFECTS.windscreen_drops.tilt,
                    ShapeTilt::START,
                ),
                overlay(
                    &crate::effects::owner::EFFECTS.windscreen_drops.state,
                    WindscreenDropState::START,
                ),
                overlay(
                    &crate::effects::owner::EFFECTS.windscreen_drops.work,
                    WindscreenDropWork::START,
                ),
            ],
            f,
        )
    }
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, canon, differential};
    use crate::effects::windscreen_drop_vars::WindscreenDropVars;

    /// Runs `f` with `HEX` holding the buffer's bytes (`typed_sync`).
    fn hex<R>(mk: usize, f: impl FnOnce() -> R) -> R {
        with_overlays(
            mk,
            &[overlay(
                &crate::effects::owner::EFFECTS.windscreen_drops.hex,
                WindscreenDropVars::START,
            )],
            f,
        )
    }

    /// The test buffer's `len` bytes at `at` run through `f` as `bytes` (a stand-in for the
    /// item or record the original found by address): `bytes` holds them from offset 0 and
    /// what `f` leaves goes back to the buffer, the stand-in as it was.
    fn staged<const N: usize, R>(
        at: usize,
        from: usize,
        len: usize,
        bytes: &crate::session::Bytes<N>,
        f: impl FnOnce() -> R,
    ) -> R {
        let host = crate::pointers::host_at(at + from);
        let saved: Vec<u8> = (from..from + len).map(|i| bytes.get(i)).collect();
        for i in 0..len {
            bytes.set(from + i, unsafe { host.add(i).read() });
        }
        let out = f();
        for (i, &b) in saved.iter().enumerate() {
            unsafe { host.add(i).write(bytes.get(from + i)) };
            bytes.set(from + i, b);
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
        differential(
            &SPEC,
            |b| reference::drift_cell_x87(b[0], b[1]),
            |b| {
                windscreen_drops(b[0], || {
                    super::drift_cell_x87(&crate::effects::owner::EFFECTS.windscreen_drops, b[1]);
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
        differential(
            &SPEC,
            |b| reference::drift_move_x87(b[0], b[1], b[2]),
            |b| {
                windscreen_drops(b[0], || {
                    hex(b[0], || {
                        staged(
                            b[2],
                            0x14,
                            0x10,
                            &crate::effects::owner::EFFECTS.windscreen_drops.hex.items,
                            || {
                                super::drift_move_x87(
                                    &crate::effects::owner::EFFECTS.windscreen_drops,
                                    b[1],
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
        differential(
            &SPEC,
            |b| reference::sprite_depth_x87(b[0]),
            |b| {
                windscreen_drops(b[0], || {
                    hex(b[0], || {
                        super::sprite_depth_x87(&crate::effects::owner::EFFECTS.windscreen_drops);
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
        differential(
            &SPEC,
            |b| canon(reference::sprite_corners_x87(b[0], b[1], b[2])),
            |b| {
                canon(
                    windscreen_drops(b[1], || {
                        super::sprite_corners_x87(
                            &crate::effects::owner::EFFECTS.windscreen_drops,
                            b[2],
                            crate::scene::owner::scene(),
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
        differential(
            &SPEC,
            |b| reference::sprite_project_x87(b[0], b[1], b[2], b[3] as u32),
            |b| {
                windscreen_drops(b[1], || {
                    with_overlays(
                        b[1],
                        &[overlay(
                            &effects().setup.kind_floats,
                            MarksKindFloats::START,
                        )],
                        || {
                            hex(b[1], || {
                                staged(
                                    b[2],
                                    0,
                                    0xC0,
                                    &crate::gfx::owner::gfx().card.windscreen_drop_batch.records,
                                    || {
                                        super::sprite_project_x87(
                                            &crate::effects::owner::EFFECTS.windscreen_drops,
                                            0,
                                            super::bh::ld(b[3] as u32),
                                            crate::scene::owner::scene(),
                                        );
                                    },
                                );
                            });
                        },
                    );
                });
            },
        );
    }
}
