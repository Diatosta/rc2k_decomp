//! `marks::hexagons_frame` (0x4BC0B1, with its chunk 0x4BB94D .. 0x4BBE62): the hexagons'
//! frame. With the drift on (`dword_7652A0`) `marks::hexagons_drift`; `marks::hexagons_setup`;
//! then each live item (counted in `dword_814710`): with the drift on, its motion eased
//! toward the drift at its cell and its place moved; it dies off the map (x or z past
//! 0x7FF, as `(x | z << 1) & 0x800`) or on `g_marksEdgeMask`; else its sprite: the place
//! turned, unless the depth is below 4096.0 the six corners (its kind's offsets)
//! projected into the buffer at `dword_816232` (0xC0 bytes each; at `unk_628A44` back to
//! `unk_625A44`, `dword_62423C` 0x300, `dword_625A40` 0x180 and the batch drawn by
//! `sub_41ADB9`, the platform's). Then, with fewer live than the settings' +0x273C and the
//! drift on, `marks::hexagons_spawn`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BAC87` (0x4BAF08). The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! items and the game's variables. Refuses, before anything, null settings and a live item
//! of a kind past 7.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::gfx::hexagon_batch::Slot as HexSlot;
use crate::marks::expire_hexagons::{ITEM_LEN, ITEMS, ITEMS_RVA};
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

const DRIFTING_RVA: Slot = Slot::at(0x7652A0); // dword_7652A0
const LIVE_RVA: Slot = Slot::at(0x814710); // dword_814710
const CELL_X_RVA: usize = rva(0x71C880); // dword_71C880
const WHOLE_RVA: [Slot; 2] = [Slot::at(0x815182), Slot::at(0x815186)]; // dword_815182 / 6
const EDGE_RVA: Slot = Slot::at(0x809BC3); // g_marksEdgeMask
const DEPTH_RVA: Slot = Slot::at(0x814CB2); // flt_814CB2
const BUFFER_RVA: Slot = Slot::at(0x816232); // dword_816232: where the next sprite goes
const BUFFER_START: HexSlot = HexSlot::at(0x625A44); // unk_625A44
/// `unk_628A44`, past the 0x40 records of 0xC0 bytes.
const BUFFER_BYTES: usize = 0x3000;
const BATCH: [(HexSlot, u32); 2] = [
    (HexSlot::at(0x62423C), 0x300),
    (HexSlot::at(0x625A40), 0x180),
];
const KINDS: u32 = 8;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The original's 0x4BC107 .. 0x4BC12E: the drift at the item's cell (`g_marksHexagonCurve`[k]) into `flt_814772`, `flt_814776`.
unsafe fn drift_cell(k: u32) {
    unsafe { drift_cell_x87(crate::marks::store::base() as usize, k as usize) }
}

/// The original's 0x4BC131 .. 0x4BC17B: the item's motion (+0x1C, +0x20) eased toward the drift times its kind's `flt_814BC2`[k] by `flt_81622A`, its place moved by it.
unsafe fn drift_move(k: u32, s: u32) {
    unsafe { drift_move_x87(crate::marks::store::base() as usize, k as usize, s as usize) }
}

/// The original's 0x4BC1DD .. 0x4BC1EF and 0x4BB94D .. 0x4BB9BE: the cell place scaled and turned into `flt_814CAA` .., the depth also in `flt_814734`.
unsafe fn sprite_depth() {
    unsafe { sprite_depth_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4BB9CE .. 0x4BBB1E: the six corners (the kind's offsets at `flt_814CFE`[c] ..) into `flt_814CB6` .., the depth's inverse (left on the x87 stack, so returned).
unsafe fn sprite_corners(c: u32) -> bh::X {
    unsafe {
        sprite_corners_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
            c as usize,
        )
    }
}

/// The original's 0x4BBB24 .. 0x4BBE28: the centre and corners projected into the buffer at e (six vertices), drawn in by `flt_81619A` / `E`, their depth.
unsafe fn sprite_project(e: u32, inverse_depth: bh::X) {
    unsafe {
        sprite_project_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
            e as usize,
            inverse_depth,
        )
    }
}

/// Why it refuses, if it does.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return Some("no marks settings");
        }
        let base = game_addr(ITEMS_RVA) as u32;
        let bad = (0..ITEMS).any(|n| {
            let e = base + n * ITEM_LEN;
            get(e) == 1 && get(e + 4) >= KINDS
        });
        bad.then_some("a live hexagon of a kind past 7")
    }
}

/// The batch's draw (`d3d::draw_marks_hexagons`; its result unread).
pub fn draw_batch() -> Result<(), Refused> {
    crate::d3d::draw_marks_hexagons::draw_marks_hexagons();
    Ok(())
}

/// The Rust version (see the header).
pub unsafe fn hexagons_frame() -> Result<(), Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    unsafe {
        if game_read::<u32>(DRIFTING_RVA) != 0 {
            crate::marks::hexagons_drift::hexagons_drift()?;
        }
        crate::marks::hexagons_setup::hexagons_setup();
        game_write(LIVE_RVA, 0u32);
        let base = game_addr(ITEMS_RVA) as u32;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            if get(e) != 1 {
                continue;
            }
            game_write(LIVE_RVA, game_read::<u32>(LIVE_RVA).wrapping_add(1));
            if game_read::<u32>(DRIFTING_RVA) != 0 {
                let x = x87::to_int(get(e + 0x14));
                game_write(CELL_X_RVA, x);
                drift_cell(4 * (((x as u32) >> 3) & 0xFF));
                drift_move(4 * get(e + 4), e);
            }
            let x = x87::to_int(get(e + 0x14));
            let z = x87::to_int(get(e + 0x18));
            game_write(WHOLE_RVA[0], x);
            game_write(WHOLE_RVA[1], z);
            if (x as u32 | (z as u32).wrapping_add(z as u32)) & 0x800 != 0 {
                put(e, 0);
                continue;
            }
            let shape = get(e + 4);
            let cell = ((((z as u32) >> 3) & 0x7F) << 8) + (((x as u32) >> 3) & 0xFF);
            if game_read::<u8>(EDGE_RVA + cell as usize) != 0 {
                put(e, 0);
                continue;
            }
            sprite_depth();
            if game_read::<i32>(DEPTH_RVA) < 0x4580_0000 {
                continue;
            }
            let inverse_depth = sprite_corners(shape.wrapping_mul(0x48));
            sprite_project(game_read(BUFFER_RVA), inverse_depth);
            let next = game_read::<u32>(BUFFER_RVA).wrapping_add(0xC0);
            game_write(BUFFER_RVA, next);
            if next as usize == game_addr(BUFFER_START) + BUFFER_BYTES {
                game_write(BUFFER_RVA, game_addr(BUFFER_START) as u32);
                for (r, v) in BATCH {
                    game_write(r, v);
                }
                draw_batch()?;
            }
        }
        let settings: u32 = game_read(SETTINGS_RVA);
        let live: u32 = game_read(LIVE_RVA);
        if (live as i32) < get(settings + 0x273C) as i32 && game_read::<u32>(DRIFTING_RVA) != 0 {
            crate::marks::hexagons_spawn::hexagons_spawn();
        }
    }
    Ok(())
}

use crate::marks::plain::{ld, ldi, st};
use crate::math::x87_float as bh;

/// `drift_cell`'s x87 block as plain Rust on the x87 model (7 instructions), same order and memory places.
unsafe fn drift_cell_x87(mk: usize, k: usize) {
    unsafe {
        let t0 = ld(mk, k.wrapping_add(0xAF882));
        let t1 = t0 * ld(mk, 0xACCA7);
        let t2 = -t1;
        let t3 = t2 + ld(mk, 0xACC9F);
        st(mk, 0xAF832, t3);
        let t4 = ld(mk, 0xACCA3);
        st(mk, 0xAF836, t4);
    }
}

/// `drift_move`'s x87 block as plain Rust on the x87 model (18 instructions), same order and memory places.
unsafe fn drift_move_x87(mk: usize, k: usize, s: usize) {
    unsafe {
        let t0 = ld(mk, 0xAF832);
        let t1 = t0 * ld(mk, k.wrapping_add(0xAFC82));
        let t2 = t1 - ld(s, 0x1C);
        let t3 = t2 * ld(mk, 0xB12EA);
        let t4 = t3 + ld(s, 0x1C);
        st(s, 0x1C, t4);
        let t5 = ld(mk, 0xAF836);
        let t6 = t5 * ld(mk, k.wrapping_add(0xAFC82));
        let t7 = t6 - ld(s, 0x20);
        let t8 = t7 * ld(mk, 0xB12EA);
        let t9 = t8 + ld(s, 0x20);
        st(s, 0x20, t9);
        let t10 = ld(s, 0x14);
        let t11 = t10 + ld(s, 0x1C);
        let t12 = ld(s, 0x18);
        let t13 = t12 + ld(s, 0x20);
        st(s, 0x18, t13);
        st(s, 0x14, t11);
    }
}

/// `sprite_depth`'s x87 block as plain Rust on the x87 model (28 instructions), same order and memory places.
unsafe fn sprite_depth_x87(mk: usize) {
    unsafe {
        let t0 = ldi(mk, 0xB0242);
        let t1 = ld(mk, 0xC9366);
        let t2 = t1 - ldi(mk, 0xB0246);
        let t3 = t2 * ld(mk, 0xAFCCA);
        let t4 = t0 * ld(mk, 0xAFCC6);
        let t5 = t4 * ld(mk, 0xAFCDA);
        let t6 = t3 * ld(mk, 0xAFCE6);
        let t7 = t5 + t6;
        let t8 = t7 + ld(mk, 0xAFCCE);
        st(mk, 0xAFD6A, t8);
        let t9 = t4 * ld(mk, 0xAFCDE);
        let t10 = t3 * ld(mk, 0xAFCEA);
        let t11 = t9 + t10;
        let t12 = t11 + ld(mk, 0xAFCD2);
        st(mk, 0xAFD6E, t12);
        let t13 = t3 * ld(mk, 0xAFCEE);
        let t14 = t4 * ld(mk, 0xAFCE2);
        let t15 = t13 + t14;
        let t16 = t15 + ld(mk, 0xAFCD6);
        st(mk, 0xAF7F4, t16);
        st(mk, 0xAFD72, t16);
    }
}

/// `sprite_corners`'s x87 block as plain Rust on the x87 model (56 instructions), same order and memory places.
unsafe fn sprite_corners_x87(rc: usize, mk: usize, c: usize) -> bh::X {
    unsafe {
        let t0 = ld(mk, 0xAFD72);
        let t1 = ld(rc, 0x224088) / t0;
        let t2 = ld(mk, 0xAFD6A);
        let t3 = t2 + ld(mk, c.wrapping_add(0xAFDBE));
        st(mk, 0xAFD76, t3);
        let t4 = ld(mk, 0xAFD6A);
        let t5 = t4 + ld(mk, c.wrapping_add(0xAFDCA));
        st(mk, 0xAFD82, t5);
        let t6 = ld(mk, 0xAFD6A);
        let t7 = t6 + ld(mk, c.wrapping_add(0xAFDD6));
        st(mk, 0xAFD8E, t7);
        let t8 = ld(mk, 0xAFD6A);
        let t9 = t8 + ld(mk, c.wrapping_add(0xAFDE2));
        st(mk, 0xAFD9A, t9);
        let t10 = ld(mk, 0xAFD6A);
        let t11 = t10 + ld(mk, c.wrapping_add(0xAFDEE));
        st(mk, 0xAFDA6, t11);
        let t12 = ld(mk, 0xAFD6A);
        let t13 = t12 + ld(mk, c.wrapping_add(0xAFDFA));
        st(mk, 0xAFDB2, t13);
        let t14 = ld(mk, 0xAFD6E);
        let t15 = t14 + ld(mk, c.wrapping_add(0xAFDC2));
        st(mk, 0xAFD7A, t15);
        let t16 = ld(mk, 0xAFD6E);
        let t17 = t16 + ld(mk, c.wrapping_add(0xAFDCE));
        st(mk, 0xAFD86, t17);
        let t18 = ld(mk, 0xAFD6E);
        let t19 = t18 + ld(mk, c.wrapping_add(0xAFDDA));
        st(mk, 0xAFD92, t19);
        let t20 = ld(mk, 0xAFD6E);
        let t21 = t20 + ld(mk, c.wrapping_add(0xAFDE6));
        st(mk, 0xAFD9E, t21);
        let t22 = ld(mk, 0xAFD6E);
        let t23 = t22 + ld(mk, c.wrapping_add(0xAFDF2));
        st(mk, 0xAFDAA, t23);
        let t24 = ld(mk, 0xAFD6E);
        let t25 = t24 + ld(mk, c.wrapping_add(0xAFDFE));
        st(mk, 0xAFDB6, t25);
        let t26 = ld(mk, 0xAFD72);
        let t27 = t26 + ld(mk, c.wrapping_add(0xAFDC6));
        st(mk, 0xAFD7E, t27);
        let t28 = ld(mk, 0xAFD72);
        let t29 = t28 + ld(mk, c.wrapping_add(0xAFDD2));
        st(mk, 0xAFD8A, t29);
        let t30 = ld(mk, 0xAFD72);
        let t31 = t30 + ld(mk, c.wrapping_add(0xAFDDE));
        st(mk, 0xAFD96, t31);
        let t32 = ld(mk, 0xAFD72);
        let t33 = t32 + ld(mk, c.wrapping_add(0xAFDEA));
        st(mk, 0xAFDA2, t33);
        let t34 = ld(mk, 0xAFD72);
        let t35 = t34 + ld(mk, c.wrapping_add(0xAFDF6));
        st(mk, 0xAFDAE, t35);
        let t36 = ld(mk, 0xAFD72);
        let t37 = t36 + ld(mk, c.wrapping_add(0xAFE02));
        st(mk, 0xAFDBA, t37);
        t1
    }
}

/// `sprite_project`'s x87 block as plain Rust on the x87 model (152 instructions), same order and memory places.
unsafe fn sprite_project_x87(rc: usize, mk: usize, e: usize, pre0: bh::X) {
    unsafe {
        st(mk, 0xAFD72, pre0);
        let t0 = ld(mk, 0xAFD6E);
        let t1 = t0 * ld(mk, 0xAFD72);
        let t2 = -t1;
        let t3 = ld(mk, 0xAFD6A);
        let t4 = t3 * ld(mk, 0xAFD72);
        let t5 = t4 + ld(rc, 0x22409C);
        st(mk, 0xAFD6A, t5);
        let t6 = t2 + ld(rc, 0x2240A0);
        st(mk, 0xAFD6E, t6);
        let t7 = ld(mk, 0xAFD76);
        let t8 = t7 * ld(mk, 0xAFD72);
        let t9 = ld(mk, 0xAFD7A);
        let t10 = t9 * ld(mk, 0xAFD72);
        let t11 = -t10;
        let t12 = t11 + ld(rc, 0x2240A0);
        st(e, 0x1C, t12);
        st(e, 0x4, t12);
        let t13 = t8 + ld(rc, 0x22409C);
        st(e, 0x18, t13);
        st(e, 0, t13);
        let t14 = ld(mk, 0xAFD82);
        let t15 = t14 * ld(mk, 0xAFD72);
        let t16 = ld(mk, 0xAFD86);
        let t17 = t16 * ld(mk, 0xAFD72);
        let t18 = -t17;
        let t19 = t18 + ld(rc, 0x2240A0);
        st(e, 0x3C, t19);
        st(e, 0x24, t19);
        let t20 = t15 + ld(rc, 0x22409C);
        st(e, 0x38, t20);
        st(e, 0x20, t20);
        let t21 = ld(mk, 0xAFD8E);
        let t22 = t21 * ld(mk, 0xAFD72);
        let t23 = ld(mk, 0xAFD92);
        let t24 = t23 * ld(mk, 0xAFD72);
        let t25 = -t24;
        let t26 = t25 + ld(rc, 0x2240A0);
        st(e, 0x5C, t26);
        st(e, 0x44, t26);
        let t27 = t22 + ld(rc, 0x22409C);
        st(e, 0x58, t27);
        st(e, 0x40, t27);
        let t28 = ld(mk, 0xAFD9A);
        let t29 = t28 * ld(mk, 0xAFD72);
        let t30 = ld(mk, 0xAFD9E);
        let t31 = t30 * ld(mk, 0xAFD72);
        let t32 = -t31;
        let t33 = t32 + ld(rc, 0x2240A0);
        st(e, 0x7C, t33);
        st(e, 0x64, t33);
        let t34 = t29 + ld(rc, 0x22409C);
        st(e, 0x78, t34);
        st(e, 0x60, t34);
        let t35 = ld(mk, 0xAFDA6);
        let t36 = t35 * ld(mk, 0xAFD72);
        let t37 = ld(mk, 0xAFDAA);
        let t38 = t37 * ld(mk, 0xAFD72);
        let t39 = -t38;
        let t40 = t39 + ld(rc, 0x2240A0);
        st(e, 0x9C, t40);
        st(e, 0x84, t40);
        let t41 = t36 + ld(rc, 0x22409C);
        st(e, 0x98, t41);
        st(e, 0x80, t41);
        let t42 = ld(mk, 0xAFDB2);
        let t43 = t42 * ld(mk, 0xAFD72);
        let t44 = ld(mk, 0xAFDB6);
        let t45 = t44 * ld(mk, 0xAFD72);
        let t46 = -t45;
        let t47 = t46 + ld(rc, 0x2240A0);
        st(e, 0xBC, t47);
        st(e, 0xA4, t47);
        let t48 = t43 + ld(rc, 0x22409C);
        st(e, 0xB8, t48);
        st(e, 0xA0, t48);
        let t49 = ld(mk, 0xAFD6A);
        let t50 = t49 * ld(mk, 0xB1252);
        let t51 = ld(mk, 0xAFD6E);
        let t52 = t51 * ld(mk, 0xB1256);
        st(mk, 0xAFD5E, t50);
        st(mk, 0xAFD62, t52);
        let t53 = ld(e, 0x18);
        let t54 = t53 - ld(mk, 0xAFD6A);
        let t55 = t54 * ld(mk, 0xB125A);
        let t56 = t55 + ld(mk, 0xAFD5E);
        st(e, 0x18, t56);
        let t57 = ld(e, 0x1C);
        let t58 = t57 - ld(mk, 0xAFD6E);
        let t59 = t58 * ld(mk, 0xB125E);
        let t60 = t59 + ld(mk, 0xAFD62);
        st(e, 0x1C, t60);
        let t61 = ld(e, 0x38);
        let t62 = t61 - ld(mk, 0xAFD6A);
        let t63 = t62 * ld(mk, 0xB125A);
        let t64 = t63 + ld(mk, 0xAFD5E);
        st(e, 0x38, t64);
        let t65 = ld(e, 0x3C);
        let t66 = t65 - ld(mk, 0xAFD6E);
        let t67 = t66 * ld(mk, 0xB125E);
        let t68 = t67 + ld(mk, 0xAFD62);
        st(e, 0x3C, t68);
        let t69 = ld(e, 0x58);
        let t70 = t69 - ld(mk, 0xAFD6A);
        let t71 = t70 * ld(mk, 0xB125A);
        let t72 = t71 + ld(mk, 0xAFD5E);
        st(e, 0x58, t72);
        let t73 = ld(e, 0x5C);
        let t74 = t73 - ld(mk, 0xAFD6E);
        let t75 = t74 * ld(mk, 0xB125E);
        let t76 = t75 + ld(mk, 0xAFD62);
        st(e, 0x5C, t76);
        let t77 = ld(e, 0x78);
        let t78 = t77 - ld(mk, 0xAFD6A);
        let t79 = t78 * ld(mk, 0xB125A);
        let t80 = t79 + ld(mk, 0xAFD5E);
        st(e, 0x78, t80);
        let t81 = ld(e, 0x7C);
        let t82 = t81 - ld(mk, 0xAFD6E);
        let t83 = t82 * ld(mk, 0xB125E);
        let t84 = t83 + ld(mk, 0xAFD62);
        st(e, 0x7C, t84);
        let t85 = ld(e, 0x98);
        let t86 = t85 - ld(mk, 0xAFD6A);
        let t87 = t86 * ld(mk, 0xB125A);
        let t88 = t87 + ld(mk, 0xAFD5E);
        st(e, 0x98, t88);
        let t89 = ld(e, 0x9C);
        let t90 = t89 - ld(mk, 0xAFD6E);
        let t91 = t90 * ld(mk, 0xB125E);
        let t92 = t91 + ld(mk, 0xAFD62);
        st(e, 0x9C, t92);
        let t93 = ld(e, 0xB8);
        let t94 = t93 - ld(mk, 0xAFD6A);
        let t95 = t94 * ld(mk, 0xB125A);
        let t96 = t95 + ld(mk, 0xAFD5E);
        st(e, 0xB8, t96);
        let t97 = ld(e, 0xBC);
        let t98 = t97 - ld(mk, 0xAFD6E);
        let t99 = t98 * ld(mk, 0xB125E);
        let t100 = t99 + ld(mk, 0xAFD62);
        st(e, 0xBC, t100);
        let t101 = ld(mk, 0xAF7F4);
        let t102 = t101 + ld(mk, 0x35C);
        let t103 = t102 * ld(mk, 0xAFCC2);
        st(e, 0x8, t103);
        st(e, 0x28, t103);
        st(e, 0x48, t103);
        st(e, 0x68, t103);
        st(e, 0x88, t103);
        st(e, 0xA8, t103);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn drift_cell_x87(mk: usize, k: usize) {
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

    pub(super) unsafe fn drift_move_x87(mk: usize, k: usize, s: usize) {
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

    pub(super) unsafe fn sprite_depth_x87(mk: usize) {
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

    pub(super) unsafe fn sprite_corners_x87(rc: usize, mk: usize, c: usize) -> f64 {
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

    pub(super) unsafe fn sprite_project_x87(rc: usize, mk: usize, e: usize, pre0: u32) {
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

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, canon, differential};

    #[test]
    fn drift_cell_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xACC9F,
                    hi: 0xAFC82,
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
            |b| unsafe { reference::drift_cell_x87(b[0], b[1]) },
            |b| unsafe { super::drift_cell_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn drift_move_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xAF832,
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
                Op::Index(&[0, 4, 12, 100, 512, 1020]),
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
            |b| unsafe { reference::drift_move_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::drift_move_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn sprite_depth_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xAF7F4,
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
            |b| unsafe { reference::sprite_depth_x87(b[0]) },
            |b| unsafe { super::sprite_depth_x87(b[0]) },
        );
    }

    #[test]
    fn sprite_corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x22408C,
                    reads: &[(0x224088, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0xAFD6A,
                    hi: 0xAFFFE,
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
            |b| canon(unsafe { reference::sprite_corners_x87(b[0], b[1], b[2]) }),
            |b| canon(unsafe { super::sprite_corners_x87(b[0], b[1], b[2]) }.0),
        );
    }

    #[test]
    fn sprite_project_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x22409C,
                    hi: 0x2240A4,
                    reads: &[(0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x35C,
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
            |b| unsafe { reference::sprite_project_x87(b[0], b[1], b[2], b[3] as u32) },
            |b| unsafe { super::sprite_project_x87(b[0], b[1], b[2], super::bh::ld(b[3] as u32)) },
        );
    }
}
