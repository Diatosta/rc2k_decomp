//! `effects::windscreen_drops_frame` (0x4BC0B1, with its chunk 0x4BB94D .. 0x4BBE62): the windscreen drops'
//! frame. With the drift on (`dword_7652A0`) `effects::windscreen_drops_drift`; `effects::windscreen_drops_setup`;
//! then each live item (counted in `dword_814710`): with the drift on, its motion eased
//! toward the drift at its cell and its place moved; it dies off the map (x or z past
//! 0x7FF, as `(x | z << 1) & 0x800`) or on `g_marksEdgeMask`; else its sprite: the place
//! turned, unless the depth is below 4096.0 the six corners (its kind's offsets)
//! projected into the buffer at `dword_816232` (0xC0 bytes each; at `unk_628A44` back to
//! `unk_625A44`, `dword_62423C` 0x300, `dword_625A40` 0x180 and the batch drawn by
//! `sub_41ADB9`, the platform's). Then, with fewer live than the settings' +0x273C and the
//! drift on, `effects::windscreen_drops_spawn`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BAC87` (0x4BAF08). The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! items and the game's variables. Refuses, before anything, null settings and a live item
//! of a kind past 7.

use crate::effects::edges::EDGE_MASK;
use crate::effects::expire_windscreen_drops::{ITEM_LEN, ITEMS};
use crate::effects::tilt_shape::SHAPE_TILT;
use crate::effects::tuning::MARKS_KIND_FLOATS;
use crate::effects::windscreen_drop_state::{WINDSCREEN_DROP_STATE, WINDSCREEN_DROP_WORK};
use crate::effects::windscreen_drop_vars::HEX;
use crate::effects::work_store::MARKS_WORK;
use crate::errors::{BadInput, Refused};
use crate::gfx::windscreen_drop_batch::WINDSCREEN_DROP_BATCH;
use crate::math::x87;
use crate::pointers::GamePtr;
/// `unk_628A44`, past the 0x40 records of 0xC0 bytes.
const BUFFER_BYTES: usize = 0x3000;
const KINDS: u32 = 8;

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a game field")
}

fn put(a: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(a).write(v, "a game field")
}

/// The original's 0x4BC107 .. 0x4BC12E: the drift at the item's cell (`g_marksWindscreenDropCurve`[k]) into `flt_814772`, `flt_814776`.
fn drift_cell(k: u32) {
    drift_cell_x87(k as usize)
}

/// The original's 0x4BC131 .. 0x4BC17B: the item's motion (+0x1C, +0x20) eased toward the drift times its kind's `flt_814BC2`[k] by `flt_81622A`, its place moved by it.
fn drift_move(k: u32, s: u32) {
    drift_move_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        k as usize,
        s as usize,
    )
}

/// The original's 0x4BC1DD .. 0x4BC1EF and 0x4BB94D .. 0x4BB9BE: the cell place scaled and turned into `flt_814CAA` .., the depth also in `flt_814734`.
fn sprite_depth() {
    sprite_depth_x87(crate::effects::windscreen_drop_vars::WindscreenDropVars::base())
}

/// The original's 0x4BB9CE .. 0x4BBB1E: the six corners (the kind's offsets at `flt_814CFE`[c] ..) into `flt_814CB6` .., the depth's inverse (left on the x87 stack, so returned).
fn sprite_corners(c: u32) -> bh::X {
    sprite_corners_x87(c as usize)
}

/// The original's 0x4BBB24 .. 0x4BBE28: the centre and corners projected into the buffer at e (six vertices), drawn in by `flt_81619A` / `E`, their depth.
fn sprite_project(e: u32, inverse_depth: bh::X) {
    sprite_project_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        e as usize,
        inverse_depth,
    )
}

/// Why it refuses, if it does.
fn refusal() -> Result<Option<&'static str>, Refused> {
    Ok({
        if SETTINGS.pointer.get() == 0 {
            return Ok(Some("no marks settings"));
        }
        let base = HEX.items.addr();
        let mut bad = false;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            if get(e)? == 1 && get(e + 4)? >= KINDS {
                bad = true;
                break;
            }
        }
        bad.then_some("a live windscreen drop of a kind past 7")
    })
}

/// The batch's draw (`d3d::draw_marks_windscreen_drops`; its result unread).
pub fn draw_batch() -> Result<(), Refused> {
    crate::d3d::draw_marks_windscreen_drops::draw_marks_windscreen_drops();
    Ok(())
}

/// The Rust version (see the header).
pub fn windscreen_drops_frame() -> Result<(), Refused> {
    if let Some(why) = refusal()? {
        return Err(BadInput(why).into());
    }
    {
        if MARKS_KIND_FLOATS.running.bits() != 0 {
            crate::effects::windscreen_drops_drift::windscreen_drops_drift()?;
        }
        crate::effects::windscreen_drops_setup::windscreen_drops_setup();
        WINDSCREEN_DROP_STATE.live.set_bits(0u32);
        let base = HEX.items.addr();
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            if get(e)? != 1 {
                continue;
            }
            WINDSCREEN_DROP_STATE
                .live
                .set(WINDSCREEN_DROP_STATE.live.get().wrapping_add(1));
            if MARKS_KIND_FLOATS.running.bits() != 0 {
                let x = x87::to_int(get(e + 0x14)?);
                MARKS_WORK.scratch.set(x as u32);
                drift_cell(4 * (((x as u32) >> 3) & 0xFF));
                drift_move(4 * get(e + 4)?, e);
            }
            let x = x87::to_int(get(e + 0x14)?);
            let z = x87::to_int(get(e + 0x18)?);
            WINDSCREEN_DROP_WORK.whole[0].set(x as u32);
            WINDSCREEN_DROP_WORK.whole[1].set(z as u32);
            if (x as u32 | (z as u32).wrapping_add(z as u32)) & 0x800 != 0 {
                put(e, 0)?;
                continue;
            }
            let shape = get(e + 4)?;
            let cell = ((((z as u32) >> 3) & 0x7F) << 8) + (((x as u32) >> 3) & 0xFF);
            if EDGE_MASK.mask.get(cell as usize) != 0 {
                put(e, 0)?;
                continue;
            }
            sprite_depth();
            if (WINDSCREEN_DROP_WORK.cell[2].bits() as i32) < 0x4580_0000 {
                continue;
            }
            let inverse_depth = sprite_corners(shape.wrapping_mul(0x48));
            sprite_project(HEX.buffer.get(), inverse_depth);
            let next = HEX.buffer.get().wrapping_add(0xC0);
            HEX.buffer.set(next);
            if next as usize
                == (WINDSCREEN_DROP_BATCH.records.addr() + BUFFER_BYTES as u32) as usize
            {
                HEX.buffer.set(WINDSCREEN_DROP_BATCH.records.addr());
                WINDSCREEN_DROP_BATCH.index_count.set(0x300);
                WINDSCREEN_DROP_BATCH.vertex_count.set(0x180);
                draw_batch()?;
            }
        }
        let settings: u32 = SETTINGS.pointer.get();
        let live: u32 = WINDSCREEN_DROP_STATE.live.bits();
        if (live as i32) < get(settings + 0x273C)? as i32 && MARKS_KIND_FLOATS.running.bits() != 0 {
            crate::effects::windscreen_drops_spawn::windscreen_drops_spawn()?;
        }
    }
    Ok(())
}

use crate::effects::plain::{ld, st};
use crate::effects::setup::SETTINGS;
use crate::math::x87_float as bh;
use crate::scene::draw_store::DRAW;

/// `drift_cell`'s x87 block as plain Rust on the x87 model (7 instructions), same order and memory places.
fn drift_cell_x87(k: usize) {
    {
        let t0 = bh::ld(WINDSCREEN_DROP_WORK.curve[k / 4].bits());
        let t1 = t0 * bh::ld(SHAPE_TILT.point[2].bits());
        let t2 = -t1;
        let t3 = t2 + bh::ld(SHAPE_TILT.point[0].bits());
        WINDSCREEN_DROP_WORK.drift[0].set_bits(bh::st(t3));
        let t4 = bh::ld(SHAPE_TILT.point[1].bits());
        WINDSCREEN_DROP_WORK.drift[1].set_bits(bh::st(t4));
    }
}

/// `drift_move`'s x87 block as plain Rust on the x87 model (18 instructions), same order and memory places.
fn drift_move_x87(hx: usize, k: usize, s: usize) {
    let t0 = bh::ld(WINDSCREEN_DROP_WORK.drift[0].bits());
    let t1 = t0 * bh::ld(WINDSCREEN_DROP_WORK.radii[k / 4].bits());
    let t2 = t1 - ld(s, 0x1C);
    let t3 = t2 * ld(hx, 0xB12EA);
    let t4 = t3 + ld(s, 0x1C);
    st(s, 0x1C, t4);
    let t5 = bh::ld(WINDSCREEN_DROP_WORK.drift[1].bits());
    let t6 = t5 * bh::ld(WINDSCREEN_DROP_WORK.radii[k / 4].bits());
    let t7 = t6 - ld(s, 0x20);
    let t8 = t7 * ld(hx, 0xB12EA);
    let t9 = t8 + ld(s, 0x20);
    st(s, 0x20, t9);
    let t10 = ld(s, 0x14);
    let t11 = t10 + ld(s, 0x1C);
    let t12 = ld(s, 0x18);
    let t13 = t12 + ld(s, 0x20);
    st(s, 0x18, t13);
    st(s, 0x14, t11);
}

/// `sprite_depth`'s x87 block as plain Rust on the x87 model (28 instructions), same order and memory places.
fn sprite_depth_x87(hx: usize) {
    let t0 = bh::int(WINDSCREEN_DROP_WORK.whole[0].bits() as i32);
    let t1 = ld(hx, 0xC9366);
    let t2 = t1 - bh::int(WINDSCREEN_DROP_WORK.whole[1].bits() as i32);
    let t3 = t2 * bh::ld(WINDSCREEN_DROP_WORK.shares[1].bits());
    let t4 = t0 * bh::ld(WINDSCREEN_DROP_WORK.shares[0].bits());
    let t5 = t4 * bh::ld(WINDSCREEN_DROP_WORK.axes[0].bits());
    let t6 = t3 * bh::ld(WINDSCREEN_DROP_WORK.axes[3].bits());
    let t7 = t5 + t6;
    let t8 = t7 + bh::ld(WINDSCREEN_DROP_WORK.place[0].bits());
    WINDSCREEN_DROP_WORK.cell[0].set_bits(bh::st(t8));
    let t9 = t4 * bh::ld(WINDSCREEN_DROP_WORK.axes[1].bits());
    let t10 = t3 * bh::ld(WINDSCREEN_DROP_WORK.axes[4].bits());
    let t11 = t9 + t10;
    let t12 = t11 + bh::ld(WINDSCREEN_DROP_WORK.place[1].bits());
    WINDSCREEN_DROP_WORK.cell[1].set_bits(bh::st(t12));
    let t13 = t3 * bh::ld(WINDSCREEN_DROP_WORK.axes[5].bits());
    let t14 = t4 * bh::ld(WINDSCREEN_DROP_WORK.axes[2].bits());
    let t15 = t13 + t14;
    let t16 = t15 + bh::ld(WINDSCREEN_DROP_WORK.place[2].bits());
    WINDSCREEN_DROP_STATE.depth.set_bits(bh::st(t16));
    WINDSCREEN_DROP_WORK.cell[2].set_bits(bh::st(t16));
}

/// `sprite_corners`'s x87 block as plain Rust on the x87 model (56 instructions), same order and memory places.
fn sprite_corners_x87(c: usize) -> bh::X {
    let t0 = bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t1 = crate::math::x87_float::ld(DRAW.view_scale.bits()) / t0;
    let t2 = bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t3 = t2 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4].bits());
    WINDSCREEN_DROP_WORK.corners[0].set_bits(bh::st(t3));
    let t4 = bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t5 = t4 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 3].bits());
    WINDSCREEN_DROP_WORK.corners[3].set_bits(bh::st(t5));
    let t6 = bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t7 = t6 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 6].bits());
    WINDSCREEN_DROP_WORK.corners[6].set_bits(bh::st(t7));
    let t8 = bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t9 = t8 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 9].bits());
    WINDSCREEN_DROP_WORK.corners[9].set_bits(bh::st(t9));
    let t10 = bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t11 = t10 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 12].bits());
    WINDSCREEN_DROP_WORK.corners[12].set_bits(bh::st(t11));
    let t12 = bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t13 = t12 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 15].bits());
    WINDSCREEN_DROP_WORK.corners[15].set_bits(bh::st(t13));
    let t14 = bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t15 = t14 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 1].bits());
    WINDSCREEN_DROP_WORK.corners[1].set_bits(bh::st(t15));
    let t16 = bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t17 = t16 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 4].bits());
    WINDSCREEN_DROP_WORK.corners[4].set_bits(bh::st(t17));
    let t18 = bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t19 = t18 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 7].bits());
    WINDSCREEN_DROP_WORK.corners[7].set_bits(bh::st(t19));
    let t20 = bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t21 = t20 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 10].bits());
    WINDSCREEN_DROP_WORK.corners[10].set_bits(bh::st(t21));
    let t22 = bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t23 = t22 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 13].bits());
    WINDSCREEN_DROP_WORK.corners[13].set_bits(bh::st(t23));
    let t24 = bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t25 = t24 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 16].bits());
    WINDSCREEN_DROP_WORK.corners[16].set_bits(bh::st(t25));
    let t26 = bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t27 = t26 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 2].bits());
    WINDSCREEN_DROP_WORK.corners[2].set_bits(bh::st(t27));
    let t28 = bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t29 = t28 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 5].bits());
    WINDSCREEN_DROP_WORK.corners[5].set_bits(bh::st(t29));
    let t30 = bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t31 = t30 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 8].bits());
    WINDSCREEN_DROP_WORK.corners[8].set_bits(bh::st(t31));
    let t32 = bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t33 = t32 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 11].bits());
    WINDSCREEN_DROP_WORK.corners[11].set_bits(bh::st(t33));
    let t34 = bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t35 = t34 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 14].bits());
    WINDSCREEN_DROP_WORK.corners[14].set_bits(bh::st(t35));
    let t36 = bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t37 = t36 + bh::ld(WINDSCREEN_DROP_WORK.scaled[c / 4 + 17].bits());
    WINDSCREEN_DROP_WORK.corners[17].set_bits(bh::st(t37));
    t1
}

/// `sprite_project`'s x87 block as plain Rust on the x87 model (152 instructions), same order and memory places.
fn sprite_project_x87(hx: usize, e: usize, pre0: bh::X) {
    WINDSCREEN_DROP_WORK.cell[2].set_bits(bh::st(pre0));
    let t0 = bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t1 = t0 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t2 = -t1;
    let t3 = bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t4 = t3 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t5 = t4 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    WINDSCREEN_DROP_WORK.cell[0].set_bits(bh::st(t5));
    let t6 = t2 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    WINDSCREEN_DROP_WORK.cell[1].set_bits(bh::st(t6));
    let t7 = bh::ld(WINDSCREEN_DROP_WORK.corners[0].bits());
    let t8 = t7 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t9 = bh::ld(WINDSCREEN_DROP_WORK.corners[1].bits());
    let t10 = t9 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t11 = -t10;
    let t12 = t11 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    st(e, 0x1C, t12);
    st(e, 0x4, t12);
    let t13 = t8 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    st(e, 0x18, t13);
    st(e, 0, t13);
    let t14 = bh::ld(WINDSCREEN_DROP_WORK.corners[3].bits());
    let t15 = t14 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t16 = bh::ld(WINDSCREEN_DROP_WORK.corners[4].bits());
    let t17 = t16 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t18 = -t17;
    let t19 = t18 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    st(e, 0x3C, t19);
    st(e, 0x24, t19);
    let t20 = t15 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    st(e, 0x38, t20);
    st(e, 0x20, t20);
    let t21 = bh::ld(WINDSCREEN_DROP_WORK.corners[6].bits());
    let t22 = t21 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t23 = bh::ld(WINDSCREEN_DROP_WORK.corners[7].bits());
    let t24 = t23 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t25 = -t24;
    let t26 = t25 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    st(e, 0x5C, t26);
    st(e, 0x44, t26);
    let t27 = t22 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    st(e, 0x58, t27);
    st(e, 0x40, t27);
    let t28 = bh::ld(WINDSCREEN_DROP_WORK.corners[9].bits());
    let t29 = t28 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t30 = bh::ld(WINDSCREEN_DROP_WORK.corners[10].bits());
    let t31 = t30 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t32 = -t31;
    let t33 = t32 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    st(e, 0x7C, t33);
    st(e, 0x64, t33);
    let t34 = t29 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    st(e, 0x78, t34);
    st(e, 0x60, t34);
    let t35 = bh::ld(WINDSCREEN_DROP_WORK.corners[12].bits());
    let t36 = t35 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t37 = bh::ld(WINDSCREEN_DROP_WORK.corners[13].bits());
    let t38 = t37 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t39 = -t38;
    let t40 = t39 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    st(e, 0x9C, t40);
    st(e, 0x84, t40);
    let t41 = t36 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    st(e, 0x98, t41);
    st(e, 0x80, t41);
    let t42 = bh::ld(WINDSCREEN_DROP_WORK.corners[15].bits());
    let t43 = t42 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t44 = bh::ld(WINDSCREEN_DROP_WORK.corners[16].bits());
    let t45 = t44 * bh::ld(WINDSCREEN_DROP_WORK.cell[2].bits());
    let t46 = -t45;
    let t47 = t46 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    st(e, 0xBC, t47);
    st(e, 0xA4, t47);
    let t48 = t43 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    st(e, 0xB8, t48);
    st(e, 0xA0, t48);
    let t49 = bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t50 = t49 * ld(hx, 0xB1252);
    let t51 = bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t52 = t51 * ld(hx, 0xB1256);
    WINDSCREEN_DROP_WORK.screen[0].set_bits(bh::st(t50));
    WINDSCREEN_DROP_WORK.screen[1].set_bits(bh::st(t52));
    let t53 = ld(e, 0x18);
    let t54 = t53 - bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t55 = t54 * ld(hx, 0xB125A);
    let t56 = t55 + bh::ld(WINDSCREEN_DROP_WORK.screen[0].bits());
    st(e, 0x18, t56);
    let t57 = ld(e, 0x1C);
    let t58 = t57 - bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t59 = t58 * ld(hx, 0xB125E);
    let t60 = t59 + bh::ld(WINDSCREEN_DROP_WORK.screen[1].bits());
    st(e, 0x1C, t60);
    let t61 = ld(e, 0x38);
    let t62 = t61 - bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t63 = t62 * ld(hx, 0xB125A);
    let t64 = t63 + bh::ld(WINDSCREEN_DROP_WORK.screen[0].bits());
    st(e, 0x38, t64);
    let t65 = ld(e, 0x3C);
    let t66 = t65 - bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t67 = t66 * ld(hx, 0xB125E);
    let t68 = t67 + bh::ld(WINDSCREEN_DROP_WORK.screen[1].bits());
    st(e, 0x3C, t68);
    let t69 = ld(e, 0x58);
    let t70 = t69 - bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t71 = t70 * ld(hx, 0xB125A);
    let t72 = t71 + bh::ld(WINDSCREEN_DROP_WORK.screen[0].bits());
    st(e, 0x58, t72);
    let t73 = ld(e, 0x5C);
    let t74 = t73 - bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t75 = t74 * ld(hx, 0xB125E);
    let t76 = t75 + bh::ld(WINDSCREEN_DROP_WORK.screen[1].bits());
    st(e, 0x5C, t76);
    let t77 = ld(e, 0x78);
    let t78 = t77 - bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t79 = t78 * ld(hx, 0xB125A);
    let t80 = t79 + bh::ld(WINDSCREEN_DROP_WORK.screen[0].bits());
    st(e, 0x78, t80);
    let t81 = ld(e, 0x7C);
    let t82 = t81 - bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t83 = t82 * ld(hx, 0xB125E);
    let t84 = t83 + bh::ld(WINDSCREEN_DROP_WORK.screen[1].bits());
    st(e, 0x7C, t84);
    let t85 = ld(e, 0x98);
    let t86 = t85 - bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t87 = t86 * ld(hx, 0xB125A);
    let t88 = t87 + bh::ld(WINDSCREEN_DROP_WORK.screen[0].bits());
    st(e, 0x98, t88);
    let t89 = ld(e, 0x9C);
    let t90 = t89 - bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t91 = t90 * ld(hx, 0xB125E);
    let t92 = t91 + bh::ld(WINDSCREEN_DROP_WORK.screen[1].bits());
    st(e, 0x9C, t92);
    let t93 = ld(e, 0xB8);
    let t94 = t93 - bh::ld(WINDSCREEN_DROP_WORK.cell[0].bits());
    let t95 = t94 * ld(hx, 0xB125A);
    let t96 = t95 + bh::ld(WINDSCREEN_DROP_WORK.screen[0].bits());
    st(e, 0xB8, t96);
    let t97 = ld(e, 0xBC);
    let t98 = t97 - bh::ld(WINDSCREEN_DROP_WORK.cell[1].bits());
    let t99 = t98 * ld(hx, 0xB125E);
    let t100 = t99 + bh::ld(WINDSCREEN_DROP_WORK.screen[1].bits());
    st(e, 0xBC, t100);
    let t101 = bh::ld(WINDSCREEN_DROP_STATE.depth.bits());
    let t102 = t101 + bh::ld(MARKS_KIND_FLOATS.kind_float.bits());
    let t103 = t102 * bh::ld(WINDSCREEN_DROP_WORK.inverse.bits());
    st(e, 0x8, t103);
    st(e, 0x28, t103);
    st(e, 0x48, t103);
    st(e, 0x68, t103);
    st(e, 0x88, t103);
    st(e, 0xA8, t103);
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
            )
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
            )
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
            )
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
            out = in(reg) &mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
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
            pre0 = in(reg) &pre0,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::tilt_shape::{SHAPE_TILT, ShapeTilt};
    use crate::effects::tuning::{MARKS_KIND_FLOATS, MarksKindFloats};
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_state::{
        WINDSCREEN_DROP_STATE, WINDSCREEN_DROP_WORK, WindscreenDropState, WindscreenDropWork,
    };

    /// Runs `f` with the windscreen drops' statics holding the buffer's bytes (`typed_sync`).
    fn windscreen_drops<R>(mk: usize, f: impl FnOnce() -> R) -> R {
        with_overlays(
            mk,
            &[
                overlay(&SHAPE_TILT, ShapeTilt::START),
                overlay(&WINDSCREEN_DROP_STATE, WindscreenDropState::START),
                overlay(&WINDSCREEN_DROP_WORK, WindscreenDropWork::START),
            ],
            f,
        )
    }
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, canon, differential};

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
            |b| windscreen_drops(b[0], || super::drift_cell_x87(b[1])),
        );
    }

    #[test]
    fn drift_move_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xACC83,
                    hi: 0xB12EE,
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
            |b| windscreen_drops(b[0], || super::drift_move_x87(b[0], b[1], b[2])),
        );
    }

    #[test]
    fn sprite_depth_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xACC83,
                hi: 0xC936A,
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
            |b| windscreen_drops(b[0], || super::sprite_depth_x87(b[0])),
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
            |b| canon(windscreen_drops(b[1], || super::sprite_corners_x87(b[2])).0),
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
                    hi: 0xB1262,
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
                        &[overlay(&MARKS_KIND_FLOATS, MarksKindFloats::START)],
                        || super::sprite_project_x87(b[1], b[2], super::bh::ld(b[3] as u32)),
                    )
                })
            },
        );
    }
}
