//! `marks::draw_shadow` (0x4C8168): the car's ground shadow (a reading), unless
//! `dword_933F64` (its corner) is -1: `scene::depth_buffer_on`, `scene::depth_test_on`,
//! render states 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1; two sides
//! (`dword_933F1C` .., `dword_933F28` ..) made one long and square (`scene::cross_product3`,
//! `scene::normalise_vector3`); the matrix at `dword_933EEC` turned about z by an angle
//! from the settings' +0x26A4 (two lines, by `dword_933F78`); the light's way turned by it,
//! the sides made its columns, the way turned again and the sides crossed once more; the
//! quad's corners about `flt_933ED0` (`flt_933F98` ..), drawn in a little, turned to the
//! view (`unk_5EA0CC`, `scene::rotate_point`); texture places from corner `dword_933F64`
//! of `g_marksCorners`; the colour `dword_939BA6` | 0xFF000000 brought halfway to white;
//! unless a corner's depth is at or below `flt_5C5E18` (signed compares of the bits): the
//! corners projected, their depths less `flt_934BC0` .. times `flt_8EB96D`, and
//! `scene::draw_fan`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: the race's frame (0x456330). The long x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables; the vector helpers their Rust versions. Refuses, before
//! anything, a corner past `g_marksCorners` and an angle `fsincos` cannot take.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::store::{Slot, Where, game_addr, game_read, game_write};
use crate::math::{x87, x87_run};
use crate::scene::cross_product3::cross_product3;
use crate::scene::matrix3::{Axis, extended, rounded, turn_matrix3, turnable};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::normalise_vector3;

const VIEW_RVA: usize = rva(0x5EA0CC); // unk_5EA0CC
const MARK_CORNERS_RVA: Slot = Slot::at(0x8EB971); // g_marksCorners
const UV_K_RVA: Slot = Slot::at(0x8EB969); // flt_8EB969
const TINT_RVA: Slot = Slot::at(0x939BA6); // dword_939BA6
const NEAR_RVA: usize = rva(0x5C5E18); // flt_5C5E18
const Z_K_RVA: Slot = Slot::at(0x8EB96D); // flt_8EB96D
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

/// A shadow's variables (the second, `sub_4C74DA`'s, 0xD1C lower, its constants 0xD08 lower)
/// and its x87 blocks.
pub struct Shadow {
    corner: Slot,
    sides: [Slot; 2],
    side: [Slot; 3],
    matrix: Slot,
    split: Slot,
    /// (factor, base) below the split, (factor, base) above it.
    lines: [(Slot, Slot); 2],
    way: Slot,
    corners: Slot,
    lifts: Slot,
    /// The settings' field the angle comes from.
    field: u32,
    light_turned: unsafe fn(u32, u32),
    second_turned: unsafe fn(u32, u32),
    corners_block: unsafe fn(),
    corners_in: unsafe fn(),
    projected: unsafe fn(),
}

/// `draw_shadow`'s.
pub const FIRST: Shadow = Shadow {
    corner: Slot::at(0x933F64),
    sides: [Slot::at(0x933F1C), Slot::at(0x933F28)],
    side: [Slot::at(0x933F40), Slot::at(0x933F4C), Slot::at(0x933F58)],
    matrix: Slot::at(0x933EEC),
    split: Slot::at(0x933F78),
    lines: [
        (Slot::at(0x933F74), Slot::at(0x933F70)),
        (Slot::at(0x933F84), Slot::at(0x933F80)),
    ],
    way: Slot::at(0x933F8C),
    corners: Slot::at(0x933F98),
    lifts: Slot::at(0x934BC0),
    field: 0x26A4,
    light_turned,
    second_turned,
    corners_block: corners,
    corners_in,
    projected,
};

/// `sub_4C74DA`'s.
pub const SECOND: Shadow = Shadow {
    corner: Slot::at(0x933248),
    sides: [Slot::at(0x933200), Slot::at(0x93320C)],
    side: [Slot::at(0x933224), Slot::at(0x933230), Slot::at(0x93323C)],
    matrix: Slot::at(0x9331D0),
    split: Slot::at(0x93325C),
    lines: [
        (Slot::at(0x933258), Slot::at(0x933254)),
        (Slot::at(0x933268), Slot::at(0x933264)),
    ],
    way: Slot::at(0x933270),
    corners: Slot::at(0x93327C),
    lifts: Slot::at(0x933EC0),
    field: 0x26A0,
    light_turned: light_turned_2,
    second_turned: second_turned_2,
    corners_block: corners_2,
    corners_in: corners_in_2,
    projected: projected_2,
};

/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

fn addr(r: impl Where) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4C82B4 .. 0x4C8324: `flt_934BB4` .. turned by the matrix at m (`[esp]` temporaries at t) into `flt_933EE0` ...
unsafe fn light_turned(m: u32, t: u32) {
    unsafe { light_turned_x87(crate::marks::store::base() as usize, m as usize, t as usize) }
}

/// The original's 0x4C838A .. 0x4C83FA: `flt_933EE0` .. turned by the matrix at m into `dword_933F4C` ...
unsafe fn second_turned(m: u32, t: u32) {
    unsafe { second_turned_x87(crate::marks::store::base() as usize, m as usize, t as usize) }
}

/// The original's 0x4C8428 .. 0x4C86E6: the two sides scaled, the quad's four corners about `flt_933ED0` .., lifted along the up (`flt_934B78`), the way between two corners into `flt_933F8C` ...
unsafe fn corners() {
    unsafe { corners_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4C86F1 .. 0x4C87FF: that way times `flt_933F88`, taken off every corner.
unsafe fn corners_in() {
    unsafe { corners_in_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4C8A4A .. 0x4C8B1A: the four corners projected.
unsafe fn projected() {
    unsafe {
        projected_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
        )
    }
}

/// The twin's (0x4C7680 ..): as `light_turned`, on the second shadow's variables.
unsafe fn light_turned_2(m: u32, t: u32) {
    unsafe { light_turned_2_x87(crate::marks::store::base() as usize, m as usize, t as usize) }
}

/// The twin's: as `second_turned`.
unsafe fn second_turned_2(m: u32, t: u32) {
    unsafe { second_turned_2_x87(crate::marks::store::base() as usize, m as usize, t as usize) }
}

/// The twin's: as `corners`.
unsafe fn corners_2() {
    unsafe { corners_2_x87(crate::marks::store::base() as usize) }
}

/// The twin's: as `corners_in`.
unsafe fn corners_in_2() {
    unsafe { corners_in_2_x87(crate::marks::store::base() as usize) }
}

/// The twin's: as `projected`.
unsafe fn projected_2() {
    unsafe {
        projected_2_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
        )
    }
}

/// The z turn's angle (see the header), 80 bits.
unsafe fn angle(sh: &Shadow) -> [u8; 10] {
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        let split: u32 = game_read(sh.split);
        let v = ((settings + sh.field) as *const u32).read_unaligned();
        if (split as i32) <= v as i32 {
            let (k, c) = sh.lines[1];
            x87::less_times_plus_negated_80(v, split, game_read(k), game_read(c))
        } else {
            let (k, c) = sh.lines[0];
            x87::times_plus_negated_80(v, game_read(k), game_read(c))
        }
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_shadow() -> Result<(), Refused> {
    unsafe { draw_shadow_with(&FIRST, true) }
}

impl Shadow {
    /// The corner's variable (-1: no shadow).
    pub fn corner(&self) -> Slot {
        self.corner
    }

    /// The angle's refusal, if any (for a caller that checks before drawing).
    pub fn angle_refuses(&self) -> Option<Refused> {
        unsafe { turnable(&angle(self)) }.err()
    }
}

/// A shadow (see the header) on `sh`'s variables; `depth` the two depth calls first.
pub unsafe fn draw_shadow_with(sh: &Shadow, depth: bool) -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        let corner: u32 = game_read(sh.corner);
        if corner == 0xFFFF_FFFF {
            return Ok(());
        }
        if corner >= 0x100 {
            return Err(BadInput("a shadow corner past g_marksCorners").into());
        }
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        turnable(&angle(sh))?;
        let mut temps = [0u32; 2];
        let t = temps.as_mut_ptr() as u32;
        if depth {
            crate::scene::depth_buffer_on::depth_buffer(true);
            crate::scene::depth_test_on::depth_test_on();
        }
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        for (n, from) in sh.sides.into_iter().enumerate() {
            let v: [u32; 3] = game_read(from);
            for (k, x) in v.into_iter().enumerate() {
                game_write(sh.side[n] + 4 * k, x);
            }
        }
        let [s0, s1, s2] = sh.side.map(addr);
        normalise_vector3(s0);
        normalise_vector3(s1);
        cross_product3(s0, s1, s2)?;
        cross_product3(s2, s0, s1)?;
        normalise_vector3(s1);
        normalise_vector3(s2);
        let a = angle(sh);
        let m = addr(sh.matrix);
        turn_matrix3(Axis::Z, m, a)?;
        (sh.light_turned)(m, t);
        for (n, side) in sh.side.into_iter().enumerate() {
            let v: [u32; 3] = game_read(side);
            for (k, x) in v.into_iter().enumerate() {
                ((m + 4 * n as u32 + 0xC * k as u32) as *mut u32).write_unaligned(x);
            }
        }
        (sh.second_turned)(m, t);
        cross_product3(s2, s1, s0)?;
        cross_product3(s2, s0, s1)?;
        (sh.corners_block)();
        normalise_vector3(addr(sh.way));
        (sh.corners_in)();
        let view = addr(VIEW_RVA);
        for n in 0..4 {
            let c = sh.corners + 12 * n;
            let p: [u32; 3] = game_read(c);
            let turned = rotate_point(view, p.map(extended));
            for (k, v) in turned.into_iter().enumerate() {
                game_write(c + 4 * k, rounded(v));
            }
        }
        let at = addr(MARK_CORNERS_RVA) + corner * 0x80;
        let word = |off: u32| ((at + off) as *const u16).read_unaligned() as u32;
        let uv_k: u32 = game_read(UV_K_RVA);
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex + 0x18, x87::int_times(word(u) as i32, uv_k));
            game_write(vertex + 0x1C, x87::int_times(word(v) as i32, uv_k));
        }
        let mut c = game_read::<u32>(TINT_RVA) | 0xFF00_0000;
        let half = |shift: u32| ((!(c >> shift) & 0xFF) >> 1) << shift;
        let (r, gr, bl) = (half(16), half(8), half(0));
        c = c.wrapping_add(r).wrapping_add(gr).wrapping_add(bl);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, c);
        }
        for n in 0..4 {
            if game_read::<i32>(sh.corners + 12 * n + 8) <= game_read::<i32>(NEAR_RVA) {
                return Ok(());
            }
        }
        (sh.projected)();
        let z_k: u32 = game_read(Z_K_RVA);
        for n in 0..4 {
            let c = sh.corners + 12 * n;
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex, game_read::<u32>(c));
            game_write(vertex + 4, game_read::<u32>(c + 4));
            let depth = x87_run::sub_mul(game_read(c + 8), game_read(sh.lifts + 4 * n), z_k);
            game_write(vertex + 8, depth);
        }
        crate::marks::clip_quad::draw_fan();
    }
    Ok(())
}

use crate::marks::plain::{ld, st};

/// `light_turned`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
unsafe fn light_turned_x87(mk: usize, m: usize, t: usize) {
    unsafe {
        let t0 = ld(mk, 0x1CFC74);
        let t1 = ld(mk, 0x1CFC78);
        let t2 = ld(mk, 0x1CFC7C);
        let t3 = t2 * ld(m, 0);
        let t4 = t1 * ld(m, 0x4);
        let t5 = t0 * ld(m, 0x8);
        let t6 = t4 + t5;
        let t7 = t3 + t6;
        st(t, 0, t7);
        let t8 = t2 * ld(m, 0xC);
        let t9 = t1 * ld(m, 0x10);
        let t10 = t0 * ld(m, 0x14);
        let t11 = t9 + t10;
        let t12 = t8 + t11;
        st(t, 0x4, t12);
        let t13 = t2 * ld(m, 0x18);
        let t14 = t1 * ld(m, 0x1C);
        let t15 = t0 * ld(m, 0x20);
        let t16 = t13 + t15;
        let t17 = t14 + t16;
        let t18 = ld(t, 0x4);
        let t19 = ld(t, 0);
        st(mk, 0x1CEFA0, t19);
        st(mk, 0x1CEFA4, t18);
        st(mk, 0x1CEFA8, t17);
    }
}

/// `second_turned`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
unsafe fn second_turned_x87(mk: usize, m: usize, t: usize) {
    unsafe {
        let t0 = ld(mk, 0x1CEFA8);
        let t1 = ld(mk, 0x1CEFA4);
        let t2 = ld(mk, 0x1CEFA0);
        let t3 = t2 * ld(m, 0);
        let t4 = t1 * ld(m, 0x4);
        let t5 = t0 * ld(m, 0x8);
        let t6 = t4 + t5;
        let t7 = t3 + t6;
        st(t, 0, t7);
        let t8 = t2 * ld(m, 0xC);
        let t9 = t1 * ld(m, 0x10);
        let t10 = t0 * ld(m, 0x14);
        let t11 = t9 + t10;
        let t12 = t8 + t11;
        st(t, 0x4, t12);
        let t13 = t2 * ld(m, 0x18);
        let t14 = t1 * ld(m, 0x1C);
        let t15 = t0 * ld(m, 0x20);
        let t16 = t13 + t15;
        let t17 = t14 + t16;
        let t18 = ld(t, 0x4);
        let t19 = ld(t, 0);
        st(mk, 0x1CF00C, t19);
        st(mk, 0x1CF010, t18);
        st(mk, 0x1CF014, t17);
    }
}

/// `corners`'s x87 block as plain Rust on the x87 model (117 instructions), same order and memory places.
unsafe fn corners_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x1CF000);
        let t1 = t0 * ld(mk, 0x1CF028);
        st(mk, 0x1CF000, t1);
        let t2 = ld(mk, 0x1CF004);
        let t3 = t2 * ld(mk, 0x1CF028);
        st(mk, 0x1CF004, t3);
        let t4 = ld(mk, 0x1CF008);
        let t5 = t4 * ld(mk, 0x1CF028);
        st(mk, 0x1CF008, t5);
        let t6 = ld(mk, 0x1CF00C);
        let t7 = t6 * ld(mk, 0x1CF02C);
        st(mk, 0x1CF00C, t7);
        let t8 = ld(mk, 0x1CF010);
        let t9 = t8 * ld(mk, 0x1CF02C);
        st(mk, 0x1CF010, t9);
        let t10 = ld(mk, 0x1CF014);
        let t11 = t10 * ld(mk, 0x1CF02C);
        st(mk, 0x1CF014, t11);
        let t12 = ld(mk, 0x1CEF90);
        let t13 = t12 - ld(mk, 0x1CF000);
        st(mk, 0x1CF058, t13);
        let t14 = ld(mk, 0x1CEF94);
        let t15 = t14 - ld(mk, 0x1CF004);
        st(mk, 0x1CF05C, t15);
        let t16 = ld(mk, 0x1CEF98);
        let t17 = t16 - ld(mk, 0x1CF008);
        st(mk, 0x1CF060, t17);
        let t18 = ld(mk, 0x1CEF90);
        let t19 = t18 + ld(mk, 0x1CF000);
        st(mk, 0x1CF064, t19);
        let t20 = ld(mk, 0x1CEF94);
        let t21 = t20 + ld(mk, 0x1CF004);
        st(mk, 0x1CF068, t21);
        let t22 = ld(mk, 0x1CEF98);
        let t23 = t22 + ld(mk, 0x1CF008);
        st(mk, 0x1CF06C, t23);
        let t24 = ld(mk, 0x1CEF90);
        let t25 = t24 + ld(mk, 0x1CF000);
        let t26 = t25 + ld(mk, 0x1CF00C);
        st(mk, 0x1CF070, t26);
        let t27 = ld(mk, 0x1CEF94);
        let t28 = t27 + ld(mk, 0x1CF004);
        let t29 = t28 + ld(mk, 0x1CF010);
        st(mk, 0x1CF074, t29);
        let t30 = ld(mk, 0x1CEF98);
        let t31 = t30 + ld(mk, 0x1CF008);
        let t32 = t31 + ld(mk, 0x1CF014);
        st(mk, 0x1CF078, t32);
        let t33 = ld(mk, 0x1CEF90);
        let t34 = t33 - ld(mk, 0x1CF000);
        let t35 = t34 + ld(mk, 0x1CF00C);
        st(mk, 0x1CF07C, t35);
        let t36 = ld(mk, 0x1CEF94);
        let t37 = t36 - ld(mk, 0x1CF004);
        let t38 = t37 + ld(mk, 0x1CF010);
        st(mk, 0x1CF080, t38);
        let t39 = ld(mk, 0x1CEF98);
        let t40 = t39 - ld(mk, 0x1CF008);
        let t41 = t40 + ld(mk, 0x1CF014);
        st(mk, 0x1CF084, t41);
        let t42 = ld(mk, 0x1CEFB0);
        let t43 = t42 * ld(mk, 0x1CFC38);
        let t44 = t43 + ld(mk, 0x1CF058);
        st(mk, 0x1CF058, t44);
        let t45 = ld(mk, 0x1CEFBC);
        let t46 = t45 * ld(mk, 0x1CFC38);
        let t47 = t46 + ld(mk, 0x1CF05C);
        st(mk, 0x1CF05C, t47);
        let t48 = ld(mk, 0x1CEFC8);
        let t49 = t48 * ld(mk, 0x1CFC38);
        let t50 = t49 + ld(mk, 0x1CF060);
        st(mk, 0x1CF060, t50);
        let t51 = ld(mk, 0x1CEFB0);
        let t52 = t51 * ld(mk, 0x1CFC38);
        let t53 = t52 + ld(mk, 0x1CF064);
        st(mk, 0x1CF064, t53);
        let t54 = ld(mk, 0x1CEFBC);
        let t55 = t54 * ld(mk, 0x1CFC38);
        let t56 = t55 + ld(mk, 0x1CF068);
        st(mk, 0x1CF068, t56);
        let t57 = ld(mk, 0x1CEFC8);
        let t58 = t57 * ld(mk, 0x1CFC38);
        let t59 = t58 + ld(mk, 0x1CF06C);
        st(mk, 0x1CF06C, t59);
        let t60 = ld(mk, 0x1CEFB0);
        let t61 = t60 * ld(mk, 0x1CFC38);
        let t62 = t61 + ld(mk, 0x1CF070);
        st(mk, 0x1CF070, t62);
        let t63 = ld(mk, 0x1CEFBC);
        let t64 = t63 * ld(mk, 0x1CFC38);
        let t65 = t64 + ld(mk, 0x1CF074);
        st(mk, 0x1CF074, t65);
        let t66 = ld(mk, 0x1CEFC8);
        let t67 = t66 * ld(mk, 0x1CFC38);
        let t68 = t67 + ld(mk, 0x1CF078);
        st(mk, 0x1CF078, t68);
        let t69 = ld(mk, 0x1CEFB0);
        let t70 = t69 * ld(mk, 0x1CFC38);
        let t71 = t70 + ld(mk, 0x1CF07C);
        st(mk, 0x1CF07C, t71);
        let t72 = ld(mk, 0x1CEFBC);
        let t73 = t72 * ld(mk, 0x1CFC38);
        let t74 = t73 + ld(mk, 0x1CF080);
        st(mk, 0x1CF080, t74);
        let t75 = ld(mk, 0x1CEFC8);
        let t76 = t75 * ld(mk, 0x1CFC38);
        let t77 = t76 + ld(mk, 0x1CF084);
        st(mk, 0x1CF084, t77);
        let t78 = ld(mk, 0x1CF070);
        let t79 = t78 - ld(mk, 0x1CF064);
        st(mk, 0x1CF04C, t79);
        let t80 = ld(mk, 0x1CF074);
        let t81 = t80 - ld(mk, 0x1CF068);
        st(mk, 0x1CF050, t81);
        let t82 = ld(mk, 0x1CF078);
        let t83 = t82 - ld(mk, 0x1CF06C);
        st(mk, 0x1CF054, t83);
    }
}

/// `corners_in`'s x87 block as plain Rust on the x87 model (45 instructions), same order and memory places.
unsafe fn corners_in_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x1CF04C);
        let t1 = t0 * ld(mk, 0x1CF048);
        st(mk, 0x1CF04C, t1);
        let t2 = ld(mk, 0x1CF050);
        let t3 = t2 * ld(mk, 0x1CF048);
        st(mk, 0x1CF050, t3);
        let t4 = ld(mk, 0x1CF054);
        let t5 = t4 * ld(mk, 0x1CF048);
        st(mk, 0x1CF054, t5);
        let t6 = ld(mk, 0x1CF058);
        let t7 = t6 - ld(mk, 0x1CF04C);
        st(mk, 0x1CF058, t7);
        let t8 = ld(mk, 0x1CF05C);
        let t9 = t8 - ld(mk, 0x1CF050);
        st(mk, 0x1CF05C, t9);
        let t10 = ld(mk, 0x1CF060);
        let t11 = t10 - ld(mk, 0x1CF054);
        st(mk, 0x1CF060, t11);
        let t12 = ld(mk, 0x1CF064);
        let t13 = t12 - ld(mk, 0x1CF04C);
        st(mk, 0x1CF064, t13);
        let t14 = ld(mk, 0x1CF068);
        let t15 = t14 - ld(mk, 0x1CF050);
        st(mk, 0x1CF068, t15);
        let t16 = ld(mk, 0x1CF06C);
        let t17 = t16 - ld(mk, 0x1CF054);
        st(mk, 0x1CF06C, t17);
        let t18 = ld(mk, 0x1CF070);
        let t19 = t18 - ld(mk, 0x1CF04C);
        st(mk, 0x1CF070, t19);
        let t20 = ld(mk, 0x1CF074);
        let t21 = t20 - ld(mk, 0x1CF050);
        st(mk, 0x1CF074, t21);
        let t22 = ld(mk, 0x1CF078);
        let t23 = t22 - ld(mk, 0x1CF054);
        st(mk, 0x1CF078, t23);
        let t24 = ld(mk, 0x1CF07C);
        let t25 = t24 - ld(mk, 0x1CF04C);
        st(mk, 0x1CF07C, t25);
        let t26 = ld(mk, 0x1CF080);
        let t27 = t26 - ld(mk, 0x1CF050);
        st(mk, 0x1CF080, t27);
        let t28 = ld(mk, 0x1CF084);
        let t29 = t28 - ld(mk, 0x1CF054);
        st(mk, 0x1CF084, t29);
    }
}

/// `projected`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places.
unsafe fn projected_x87(rc: usize, mk: usize) {
    unsafe {
        let t0 = ld(rc, 0x224088);
        let t1 = t0 / ld(mk, 0x1CF060);
        let t2 = t1 * ld(mk, 0x1CF058);
        let t3 = t2 + ld(rc, 0x22409C);
        st(mk, 0x1CF058, t3);
        let t4 = t1 * ld(mk, 0x1CF05C);
        let t5 = -t4;
        let t6 = t5 + ld(rc, 0x2240A0);
        st(mk, 0x1CF05C, t6);
        let t7 = ld(rc, 0x224088);
        let t8 = t7 / ld(mk, 0x1CF06C);
        let t9 = t8 * ld(mk, 0x1CF064);
        let t10 = t9 + ld(rc, 0x22409C);
        st(mk, 0x1CF064, t10);
        let t11 = t8 * ld(mk, 0x1CF068);
        let t12 = -t11;
        let t13 = t12 + ld(rc, 0x2240A0);
        st(mk, 0x1CF068, t13);
        let t14 = ld(rc, 0x224088);
        let t15 = t14 / ld(mk, 0x1CF078);
        let t16 = t15 * ld(mk, 0x1CF070);
        let t17 = t16 + ld(rc, 0x22409C);
        st(mk, 0x1CF070, t17);
        let t18 = t15 * ld(mk, 0x1CF074);
        let t19 = -t18;
        let t20 = t19 + ld(rc, 0x2240A0);
        st(mk, 0x1CF074, t20);
        let t21 = ld(rc, 0x224088);
        let t22 = t21 / ld(mk, 0x1CF084);
        let t23 = t22 * ld(mk, 0x1CF07C);
        let t24 = t23 + ld(rc, 0x22409C);
        st(mk, 0x1CF07C, t24);
        let t25 = t22 * ld(mk, 0x1CF080);
        let t26 = -t25;
        let t27 = t26 + ld(rc, 0x2240A0);
        st(mk, 0x1CF080, t27);
    }
}

/// `light_turned_2`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
unsafe fn light_turned_2_x87(mk: usize, m: usize, t: usize) {
    unsafe {
        let t0 = ld(mk, 0x1CEF74);
        let t1 = ld(mk, 0x1CEF78);
        let t2 = ld(mk, 0x1CEF7C);
        let t3 = t2 * ld(m, 0);
        let t4 = t1 * ld(m, 0x4);
        let t5 = t0 * ld(m, 0x8);
        let t6 = t4 + t5;
        let t7 = t3 + t6;
        st(t, 0, t7);
        let t8 = t2 * ld(m, 0xC);
        let t9 = t1 * ld(m, 0x10);
        let t10 = t0 * ld(m, 0x14);
        let t11 = t9 + t10;
        let t12 = t8 + t11;
        st(t, 0x4, t12);
        let t13 = t2 * ld(m, 0x18);
        let t14 = t1 * ld(m, 0x1C);
        let t15 = t0 * ld(m, 0x20);
        let t16 = t13 + t15;
        let t17 = t14 + t16;
        let t18 = ld(t, 0x4);
        let t19 = ld(t, 0);
        st(mk, 0x1CE284, t19);
        st(mk, 0x1CE288, t18);
        st(mk, 0x1CE28C, t17);
    }
}

/// `second_turned_2`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
unsafe fn second_turned_2_x87(mk: usize, m: usize, t: usize) {
    unsafe {
        let t0 = ld(mk, 0x1CE28C);
        let t1 = ld(mk, 0x1CE288);
        let t2 = ld(mk, 0x1CE284);
        let t3 = t2 * ld(m, 0);
        let t4 = t1 * ld(m, 0x4);
        let t5 = t0 * ld(m, 0x8);
        let t6 = t4 + t5;
        let t7 = t3 + t6;
        st(t, 0, t7);
        let t8 = t2 * ld(m, 0xC);
        let t9 = t1 * ld(m, 0x10);
        let t10 = t0 * ld(m, 0x14);
        let t11 = t9 + t10;
        let t12 = t8 + t11;
        st(t, 0x4, t12);
        let t13 = t2 * ld(m, 0x18);
        let t14 = t1 * ld(m, 0x1C);
        let t15 = t0 * ld(m, 0x20);
        let t16 = t13 + t15;
        let t17 = t14 + t16;
        let t18 = ld(t, 0x4);
        let t19 = ld(t, 0);
        st(mk, 0x1CE2F0, t19);
        st(mk, 0x1CE2F4, t18);
        st(mk, 0x1CE2F8, t17);
    }
}

/// `corners_2`'s x87 block as plain Rust on the x87 model (117 instructions), same order and memory places.
unsafe fn corners_2_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x1CE2E4);
        let t1 = t0 * ld(mk, 0x1CE30C);
        st(mk, 0x1CE2E4, t1);
        let t2 = ld(mk, 0x1CE2E8);
        let t3 = t2 * ld(mk, 0x1CE30C);
        st(mk, 0x1CE2E8, t3);
        let t4 = ld(mk, 0x1CE2EC);
        let t5 = t4 * ld(mk, 0x1CE30C);
        st(mk, 0x1CE2EC, t5);
        let t6 = ld(mk, 0x1CE2F0);
        let t7 = t6 * ld(mk, 0x1CE310);
        st(mk, 0x1CE2F0, t7);
        let t8 = ld(mk, 0x1CE2F4);
        let t9 = t8 * ld(mk, 0x1CE310);
        st(mk, 0x1CE2F4, t9);
        let t10 = ld(mk, 0x1CE2F8);
        let t11 = t10 * ld(mk, 0x1CE310);
        st(mk, 0x1CE2F8, t11);
        let t12 = ld(mk, 0x1CE274);
        let t13 = t12 - ld(mk, 0x1CE2E4);
        st(mk, 0x1CE33C, t13);
        let t14 = ld(mk, 0x1CE278);
        let t15 = t14 - ld(mk, 0x1CE2E8);
        st(mk, 0x1CE340, t15);
        let t16 = ld(mk, 0x1CE27C);
        let t17 = t16 - ld(mk, 0x1CE2EC);
        st(mk, 0x1CE344, t17);
        let t18 = ld(mk, 0x1CE274);
        let t19 = t18 + ld(mk, 0x1CE2E4);
        st(mk, 0x1CE348, t19);
        let t20 = ld(mk, 0x1CE278);
        let t21 = t20 + ld(mk, 0x1CE2E8);
        st(mk, 0x1CE34C, t21);
        let t22 = ld(mk, 0x1CE27C);
        let t23 = t22 + ld(mk, 0x1CE2EC);
        st(mk, 0x1CE350, t23);
        let t24 = ld(mk, 0x1CE274);
        let t25 = t24 + ld(mk, 0x1CE2E4);
        let t26 = t25 + ld(mk, 0x1CE2F0);
        st(mk, 0x1CE354, t26);
        let t27 = ld(mk, 0x1CE278);
        let t28 = t27 + ld(mk, 0x1CE2E8);
        let t29 = t28 + ld(mk, 0x1CE2F4);
        st(mk, 0x1CE358, t29);
        let t30 = ld(mk, 0x1CE27C);
        let t31 = t30 + ld(mk, 0x1CE2EC);
        let t32 = t31 + ld(mk, 0x1CE2F8);
        st(mk, 0x1CE35C, t32);
        let t33 = ld(mk, 0x1CE274);
        let t34 = t33 - ld(mk, 0x1CE2E4);
        let t35 = t34 + ld(mk, 0x1CE2F0);
        st(mk, 0x1CE360, t35);
        let t36 = ld(mk, 0x1CE278);
        let t37 = t36 - ld(mk, 0x1CE2E8);
        let t38 = t37 + ld(mk, 0x1CE2F4);
        st(mk, 0x1CE364, t38);
        let t39 = ld(mk, 0x1CE27C);
        let t40 = t39 - ld(mk, 0x1CE2EC);
        let t41 = t40 + ld(mk, 0x1CE2F8);
        st(mk, 0x1CE368, t41);
        let t42 = ld(mk, 0x1CE294);
        let t43 = t42 * ld(mk, 0x1CEF30);
        let t44 = t43 + ld(mk, 0x1CE33C);
        st(mk, 0x1CE33C, t44);
        let t45 = ld(mk, 0x1CE2A0);
        let t46 = t45 * ld(mk, 0x1CEF30);
        let t47 = t46 + ld(mk, 0x1CE340);
        st(mk, 0x1CE340, t47);
        let t48 = ld(mk, 0x1CE2AC);
        let t49 = t48 * ld(mk, 0x1CEF30);
        let t50 = t49 + ld(mk, 0x1CE344);
        st(mk, 0x1CE344, t50);
        let t51 = ld(mk, 0x1CE294);
        let t52 = t51 * ld(mk, 0x1CEF30);
        let t53 = t52 + ld(mk, 0x1CE348);
        st(mk, 0x1CE348, t53);
        let t54 = ld(mk, 0x1CE2A0);
        let t55 = t54 * ld(mk, 0x1CEF30);
        let t56 = t55 + ld(mk, 0x1CE34C);
        st(mk, 0x1CE34C, t56);
        let t57 = ld(mk, 0x1CE2AC);
        let t58 = t57 * ld(mk, 0x1CEF30);
        let t59 = t58 + ld(mk, 0x1CE350);
        st(mk, 0x1CE350, t59);
        let t60 = ld(mk, 0x1CE294);
        let t61 = t60 * ld(mk, 0x1CEF30);
        let t62 = t61 + ld(mk, 0x1CE354);
        st(mk, 0x1CE354, t62);
        let t63 = ld(mk, 0x1CE2A0);
        let t64 = t63 * ld(mk, 0x1CEF30);
        let t65 = t64 + ld(mk, 0x1CE358);
        st(mk, 0x1CE358, t65);
        let t66 = ld(mk, 0x1CE2AC);
        let t67 = t66 * ld(mk, 0x1CEF30);
        let t68 = t67 + ld(mk, 0x1CE35C);
        st(mk, 0x1CE35C, t68);
        let t69 = ld(mk, 0x1CE294);
        let t70 = t69 * ld(mk, 0x1CEF30);
        let t71 = t70 + ld(mk, 0x1CE360);
        st(mk, 0x1CE360, t71);
        let t72 = ld(mk, 0x1CE2A0);
        let t73 = t72 * ld(mk, 0x1CEF30);
        let t74 = t73 + ld(mk, 0x1CE364);
        st(mk, 0x1CE364, t74);
        let t75 = ld(mk, 0x1CE2AC);
        let t76 = t75 * ld(mk, 0x1CEF30);
        let t77 = t76 + ld(mk, 0x1CE368);
        st(mk, 0x1CE368, t77);
        let t78 = ld(mk, 0x1CE354);
        let t79 = t78 - ld(mk, 0x1CE348);
        st(mk, 0x1CE330, t79);
        let t80 = ld(mk, 0x1CE358);
        let t81 = t80 - ld(mk, 0x1CE34C);
        st(mk, 0x1CE334, t81);
        let t82 = ld(mk, 0x1CE35C);
        let t83 = t82 - ld(mk, 0x1CE350);
        st(mk, 0x1CE338, t83);
    }
}

/// `corners_in_2`'s x87 block as plain Rust on the x87 model (45 instructions), same order and memory places.
unsafe fn corners_in_2_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x1CE330);
        let t1 = t0 * ld(mk, 0x1CE32C);
        st(mk, 0x1CE330, t1);
        let t2 = ld(mk, 0x1CE334);
        let t3 = t2 * ld(mk, 0x1CE32C);
        st(mk, 0x1CE334, t3);
        let t4 = ld(mk, 0x1CE338);
        let t5 = t4 * ld(mk, 0x1CE32C);
        st(mk, 0x1CE338, t5);
        let t6 = ld(mk, 0x1CE33C);
        let t7 = t6 - ld(mk, 0x1CE330);
        st(mk, 0x1CE33C, t7);
        let t8 = ld(mk, 0x1CE340);
        let t9 = t8 - ld(mk, 0x1CE334);
        st(mk, 0x1CE340, t9);
        let t10 = ld(mk, 0x1CE344);
        let t11 = t10 - ld(mk, 0x1CE338);
        st(mk, 0x1CE344, t11);
        let t12 = ld(mk, 0x1CE348);
        let t13 = t12 - ld(mk, 0x1CE330);
        st(mk, 0x1CE348, t13);
        let t14 = ld(mk, 0x1CE34C);
        let t15 = t14 - ld(mk, 0x1CE334);
        st(mk, 0x1CE34C, t15);
        let t16 = ld(mk, 0x1CE350);
        let t17 = t16 - ld(mk, 0x1CE338);
        st(mk, 0x1CE350, t17);
        let t18 = ld(mk, 0x1CE354);
        let t19 = t18 - ld(mk, 0x1CE330);
        st(mk, 0x1CE354, t19);
        let t20 = ld(mk, 0x1CE358);
        let t21 = t20 - ld(mk, 0x1CE334);
        st(mk, 0x1CE358, t21);
        let t22 = ld(mk, 0x1CE35C);
        let t23 = t22 - ld(mk, 0x1CE338);
        st(mk, 0x1CE35C, t23);
        let t24 = ld(mk, 0x1CE360);
        let t25 = t24 - ld(mk, 0x1CE330);
        st(mk, 0x1CE360, t25);
        let t26 = ld(mk, 0x1CE364);
        let t27 = t26 - ld(mk, 0x1CE334);
        st(mk, 0x1CE364, t27);
        let t28 = ld(mk, 0x1CE368);
        let t29 = t28 - ld(mk, 0x1CE338);
        st(mk, 0x1CE368, t29);
    }
}

/// `projected_2`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places.
unsafe fn projected_2_x87(rc: usize, mk: usize) {
    unsafe {
        let t0 = ld(rc, 0x224088);
        let t1 = t0 / ld(mk, 0x1CE344);
        let t2 = t1 * ld(mk, 0x1CE33C);
        let t3 = t2 + ld(rc, 0x22409C);
        st(mk, 0x1CE33C, t3);
        let t4 = t1 * ld(mk, 0x1CE340);
        let t5 = -t4;
        let t6 = t5 + ld(rc, 0x2240A0);
        st(mk, 0x1CE340, t6);
        let t7 = ld(rc, 0x224088);
        let t8 = t7 / ld(mk, 0x1CE350);
        let t9 = t8 * ld(mk, 0x1CE348);
        let t10 = t9 + ld(rc, 0x22409C);
        st(mk, 0x1CE348, t10);
        let t11 = t8 * ld(mk, 0x1CE34C);
        let t12 = -t11;
        let t13 = t12 + ld(rc, 0x2240A0);
        st(mk, 0x1CE34C, t13);
        let t14 = ld(rc, 0x224088);
        let t15 = t14 / ld(mk, 0x1CE35C);
        let t16 = t15 * ld(mk, 0x1CE354);
        let t17 = t16 + ld(rc, 0x22409C);
        st(mk, 0x1CE354, t17);
        let t18 = t15 * ld(mk, 0x1CE358);
        let t19 = -t18;
        let t20 = t19 + ld(rc, 0x2240A0);
        st(mk, 0x1CE358, t20);
        let t21 = ld(rc, 0x224088);
        let t22 = t21 / ld(mk, 0x1CE368);
        let t23 = t22 * ld(mk, 0x1CE360);
        let t24 = t23 + ld(rc, 0x22409C);
        st(mk, 0x1CE360, t24);
        let t25 = t22 * ld(mk, 0x1CE364);
        let t26 = -t25;
        let t27 = t26 + ld(rc, 0x2240A0);
        st(mk, 0x1CE364, t27);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn light_turned_x87(mk: usize, m: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CFC74]",
            "fld dword ptr [{mk} + 0x1CFC78]",
            "fld dword ptr [{mk} + 0x1CFC7C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp dword ptr [{mk} + 0x1CEFA0]",
            "fstp dword ptr [{mk} + 0x1CEFA4]",
            "fstp dword ptr [{mk} + 0x1CEFA8]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn second_turned_x87(mk: usize, m: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CEFA8]",
            "fld dword ptr [{mk} + 0x1CEFA4]",
            "fld dword ptr [{mk} + 0x1CEFA0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp dword ptr [{mk} + 0x1CF00C]",
            "fstp dword ptr [{mk} + 0x1CF010]",
            "fstp dword ptr [{mk} + 0x1CF014]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn corners_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CF000]",
            "fmul dword ptr [{mk} + 0x1CF028]",
            "fstp dword ptr [{mk} + 0x1CF000]",
            "fld dword ptr [{mk} + 0x1CF004]",
            "fmul dword ptr [{mk} + 0x1CF028]",
            "fstp dword ptr [{mk} + 0x1CF004]",
            "fld dword ptr [{mk} + 0x1CF008]",
            "fmul dword ptr [{mk} + 0x1CF028]",
            "fstp dword ptr [{mk} + 0x1CF008]",
            "fld dword ptr [{mk} + 0x1CF00C]",
            "fmul dword ptr [{mk} + 0x1CF02C]",
            "fstp dword ptr [{mk} + 0x1CF00C]",
            "fld dword ptr [{mk} + 0x1CF010]",
            "fmul dword ptr [{mk} + 0x1CF02C]",
            "fstp dword ptr [{mk} + 0x1CF010]",
            "fld dword ptr [{mk} + 0x1CF014]",
            "fmul dword ptr [{mk} + 0x1CF02C]",
            "fstp dword ptr [{mk} + 0x1CF014]",
            "fld dword ptr [{mk} + 0x1CEF90]",
            "fsub dword ptr [{mk} + 0x1CF000]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fld dword ptr [{mk} + 0x1CEF94]",
            "fsub dword ptr [{mk} + 0x1CF004]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{mk} + 0x1CEF98]",
            "fsub dword ptr [{mk} + 0x1CF008]",
            "fstp dword ptr [{mk} + 0x1CF060]",
            "fld dword ptr [{mk} + 0x1CEF90]",
            "fadd dword ptr [{mk} + 0x1CF000]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fld dword ptr [{mk} + 0x1CEF94]",
            "fadd dword ptr [{mk} + 0x1CF004]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{mk} + 0x1CEF98]",
            "fadd dword ptr [{mk} + 0x1CF008]",
            "fstp dword ptr [{mk} + 0x1CF06C]",
            "fld dword ptr [{mk} + 0x1CEF90]",
            "fadd dword ptr [{mk} + 0x1CF000]",
            "fadd dword ptr [{mk} + 0x1CF00C]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fld dword ptr [{mk} + 0x1CEF94]",
            "fadd dword ptr [{mk} + 0x1CF004]",
            "fadd dword ptr [{mk} + 0x1CF010]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{mk} + 0x1CEF98]",
            "fadd dword ptr [{mk} + 0x1CF008]",
            "fadd dword ptr [{mk} + 0x1CF014]",
            "fstp dword ptr [{mk} + 0x1CF078]",
            "fld dword ptr [{mk} + 0x1CEF90]",
            "fsub dword ptr [{mk} + 0x1CF000]",
            "fadd dword ptr [{mk} + 0x1CF00C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fld dword ptr [{mk} + 0x1CEF94]",
            "fsub dword ptr [{mk} + 0x1CF004]",
            "fadd dword ptr [{mk} + 0x1CF010]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            "fld dword ptr [{mk} + 0x1CEF98]",
            "fsub dword ptr [{mk} + 0x1CF008]",
            "fadd dword ptr [{mk} + 0x1CF014]",
            "fstp dword ptr [{mk} + 0x1CF084]",
            "fld dword ptr [{mk} + 0x1CEFB0]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF058]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fld dword ptr [{mk} + 0x1CEFBC]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF05C]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{mk} + 0x1CEFC8]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF060]",
            "fstp dword ptr [{mk} + 0x1CF060]",
            "fld dword ptr [{mk} + 0x1CEFB0]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF064]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fld dword ptr [{mk} + 0x1CEFBC]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF068]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{mk} + 0x1CEFC8]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF06C]",
            "fstp dword ptr [{mk} + 0x1CF06C]",
            "fld dword ptr [{mk} + 0x1CEFB0]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF070]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fld dword ptr [{mk} + 0x1CEFBC]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF074]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{mk} + 0x1CEFC8]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF078]",
            "fstp dword ptr [{mk} + 0x1CF078]",
            "fld dword ptr [{mk} + 0x1CEFB0]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF07C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fld dword ptr [{mk} + 0x1CEFBC]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF080]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            "fld dword ptr [{mk} + 0x1CEFC8]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF084]",
            "fstp dword ptr [{mk} + 0x1CF084]",
            "fld dword ptr [{mk} + 0x1CF070]",
            "fsub dword ptr [{mk} + 0x1CF064]",
            "fstp dword ptr [{mk} + 0x1CF04C]",
            "fld dword ptr [{mk} + 0x1CF074]",
            "fsub dword ptr [{mk} + 0x1CF068]",
            "fstp dword ptr [{mk} + 0x1CF050]",
            "fld dword ptr [{mk} + 0x1CF078]",
            "fsub dword ptr [{mk} + 0x1CF06C]",
            "fstp dword ptr [{mk} + 0x1CF054]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn corners_in_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CF04C]",
            "fmul dword ptr [{mk} + 0x1CF048]",
            "fstp dword ptr [{mk} + 0x1CF04C]",
            "fld dword ptr [{mk} + 0x1CF050]",
            "fmul dword ptr [{mk} + 0x1CF048]",
            "fstp dword ptr [{mk} + 0x1CF050]",
            "fld dword ptr [{mk} + 0x1CF054]",
            "fmul dword ptr [{mk} + 0x1CF048]",
            "fstp dword ptr [{mk} + 0x1CF054]",
            "fld dword ptr [{mk} + 0x1CF058]",
            "fsub dword ptr [{mk} + 0x1CF04C]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fld dword ptr [{mk} + 0x1CF05C]",
            "fsub dword ptr [{mk} + 0x1CF050]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{mk} + 0x1CF060]",
            "fsub dword ptr [{mk} + 0x1CF054]",
            "fstp dword ptr [{mk} + 0x1CF060]",
            "fld dword ptr [{mk} + 0x1CF064]",
            "fsub dword ptr [{mk} + 0x1CF04C]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fld dword ptr [{mk} + 0x1CF068]",
            "fsub dword ptr [{mk} + 0x1CF050]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{mk} + 0x1CF06C]",
            "fsub dword ptr [{mk} + 0x1CF054]",
            "fstp dword ptr [{mk} + 0x1CF06C]",
            "fld dword ptr [{mk} + 0x1CF070]",
            "fsub dword ptr [{mk} + 0x1CF04C]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fld dword ptr [{mk} + 0x1CF074]",
            "fsub dword ptr [{mk} + 0x1CF050]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{mk} + 0x1CF078]",
            "fsub dword ptr [{mk} + 0x1CF054]",
            "fstp dword ptr [{mk} + 0x1CF078]",
            "fld dword ptr [{mk} + 0x1CF07C]",
            "fsub dword ptr [{mk} + 0x1CF04C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fld dword ptr [{mk} + 0x1CF080]",
            "fsub dword ptr [{mk} + 0x1CF050]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            "fld dword ptr [{mk} + 0x1CF084]",
            "fsub dword ptr [{mk} + 0x1CF054]",
            "fstp dword ptr [{mk} + 0x1CF084]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn projected_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF060]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF058]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fmul dword ptr [{mk} + 0x1CF05C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF06C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF064]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fmul dword ptr [{mk} + 0x1CF068]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF078]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF070]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fmul dword ptr [{mk} + 0x1CF074]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF084]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF07C]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fmul dword ptr [{mk} + 0x1CF080]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn light_turned_2_x87(mk: usize, m: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CEF74]",
            "fld dword ptr [{mk} + 0x1CEF78]",
            "fld dword ptr [{mk} + 0x1CEF7C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp dword ptr [{mk} + 0x1CE284]",
            "fstp dword ptr [{mk} + 0x1CE288]",
            "fstp dword ptr [{mk} + 0x1CE28C]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn second_turned_2_x87(mk: usize, m: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CE28C]",
            "fld dword ptr [{mk} + 0x1CE288]",
            "fld dword ptr [{mk} + 0x1CE284]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp dword ptr [{mk} + 0x1CE2F0]",
            "fstp dword ptr [{mk} + 0x1CE2F4]",
            "fstp dword ptr [{mk} + 0x1CE2F8]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn corners_2_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CE2E4]",
            "fmul dword ptr [{mk} + 0x1CE30C]",
            "fstp dword ptr [{mk} + 0x1CE2E4]",
            "fld dword ptr [{mk} + 0x1CE2E8]",
            "fmul dword ptr [{mk} + 0x1CE30C]",
            "fstp dword ptr [{mk} + 0x1CE2E8]",
            "fld dword ptr [{mk} + 0x1CE2EC]",
            "fmul dword ptr [{mk} + 0x1CE30C]",
            "fstp dword ptr [{mk} + 0x1CE2EC]",
            "fld dword ptr [{mk} + 0x1CE2F0]",
            "fmul dword ptr [{mk} + 0x1CE310]",
            "fstp dword ptr [{mk} + 0x1CE2F0]",
            "fld dword ptr [{mk} + 0x1CE2F4]",
            "fmul dword ptr [{mk} + 0x1CE310]",
            "fstp dword ptr [{mk} + 0x1CE2F4]",
            "fld dword ptr [{mk} + 0x1CE2F8]",
            "fmul dword ptr [{mk} + 0x1CE310]",
            "fstp dword ptr [{mk} + 0x1CE2F8]",
            "fld dword ptr [{mk} + 0x1CE274]",
            "fsub dword ptr [{mk} + 0x1CE2E4]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fld dword ptr [{mk} + 0x1CE278]",
            "fsub dword ptr [{mk} + 0x1CE2E8]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{mk} + 0x1CE27C]",
            "fsub dword ptr [{mk} + 0x1CE2EC]",
            "fstp dword ptr [{mk} + 0x1CE344]",
            "fld dword ptr [{mk} + 0x1CE274]",
            "fadd dword ptr [{mk} + 0x1CE2E4]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fld dword ptr [{mk} + 0x1CE278]",
            "fadd dword ptr [{mk} + 0x1CE2E8]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{mk} + 0x1CE27C]",
            "fadd dword ptr [{mk} + 0x1CE2EC]",
            "fstp dword ptr [{mk} + 0x1CE350]",
            "fld dword ptr [{mk} + 0x1CE274]",
            "fadd dword ptr [{mk} + 0x1CE2E4]",
            "fadd dword ptr [{mk} + 0x1CE2F0]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fld dword ptr [{mk} + 0x1CE278]",
            "fadd dword ptr [{mk} + 0x1CE2E8]",
            "fadd dword ptr [{mk} + 0x1CE2F4]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{mk} + 0x1CE27C]",
            "fadd dword ptr [{mk} + 0x1CE2EC]",
            "fadd dword ptr [{mk} + 0x1CE2F8]",
            "fstp dword ptr [{mk} + 0x1CE35C]",
            "fld dword ptr [{mk} + 0x1CE274]",
            "fsub dword ptr [{mk} + 0x1CE2E4]",
            "fadd dword ptr [{mk} + 0x1CE2F0]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fld dword ptr [{mk} + 0x1CE278]",
            "fsub dword ptr [{mk} + 0x1CE2E8]",
            "fadd dword ptr [{mk} + 0x1CE2F4]",
            "fstp dword ptr [{mk} + 0x1CE364]",
            "fld dword ptr [{mk} + 0x1CE27C]",
            "fsub dword ptr [{mk} + 0x1CE2EC]",
            "fadd dword ptr [{mk} + 0x1CE2F8]",
            "fstp dword ptr [{mk} + 0x1CE368]",
            "fld dword ptr [{mk} + 0x1CE294]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE33C]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fld dword ptr [{mk} + 0x1CE2A0]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE340]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{mk} + 0x1CE2AC]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE344]",
            "fstp dword ptr [{mk} + 0x1CE344]",
            "fld dword ptr [{mk} + 0x1CE294]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE348]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fld dword ptr [{mk} + 0x1CE2A0]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE34C]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{mk} + 0x1CE2AC]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE350]",
            "fstp dword ptr [{mk} + 0x1CE350]",
            "fld dword ptr [{mk} + 0x1CE294]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE354]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fld dword ptr [{mk} + 0x1CE2A0]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE358]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{mk} + 0x1CE2AC]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE35C]",
            "fstp dword ptr [{mk} + 0x1CE35C]",
            "fld dword ptr [{mk} + 0x1CE294]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE360]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fld dword ptr [{mk} + 0x1CE2A0]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE364]",
            "fstp dword ptr [{mk} + 0x1CE364]",
            "fld dword ptr [{mk} + 0x1CE2AC]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE368]",
            "fstp dword ptr [{mk} + 0x1CE368]",
            "fld dword ptr [{mk} + 0x1CE354]",
            "fsub dword ptr [{mk} + 0x1CE348]",
            "fstp dword ptr [{mk} + 0x1CE330]",
            "fld dword ptr [{mk} + 0x1CE358]",
            "fsub dword ptr [{mk} + 0x1CE34C]",
            "fstp dword ptr [{mk} + 0x1CE334]",
            "fld dword ptr [{mk} + 0x1CE35C]",
            "fsub dword ptr [{mk} + 0x1CE350]",
            "fstp dword ptr [{mk} + 0x1CE338]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn corners_in_2_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CE330]",
            "fmul dword ptr [{mk} + 0x1CE32C]",
            "fstp dword ptr [{mk} + 0x1CE330]",
            "fld dword ptr [{mk} + 0x1CE334]",
            "fmul dword ptr [{mk} + 0x1CE32C]",
            "fstp dword ptr [{mk} + 0x1CE334]",
            "fld dword ptr [{mk} + 0x1CE338]",
            "fmul dword ptr [{mk} + 0x1CE32C]",
            "fstp dword ptr [{mk} + 0x1CE338]",
            "fld dword ptr [{mk} + 0x1CE33C]",
            "fsub dword ptr [{mk} + 0x1CE330]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fld dword ptr [{mk} + 0x1CE340]",
            "fsub dword ptr [{mk} + 0x1CE334]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{mk} + 0x1CE344]",
            "fsub dword ptr [{mk} + 0x1CE338]",
            "fstp dword ptr [{mk} + 0x1CE344]",
            "fld dword ptr [{mk} + 0x1CE348]",
            "fsub dword ptr [{mk} + 0x1CE330]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fld dword ptr [{mk} + 0x1CE34C]",
            "fsub dword ptr [{mk} + 0x1CE334]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{mk} + 0x1CE350]",
            "fsub dword ptr [{mk} + 0x1CE338]",
            "fstp dword ptr [{mk} + 0x1CE350]",
            "fld dword ptr [{mk} + 0x1CE354]",
            "fsub dword ptr [{mk} + 0x1CE330]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fld dword ptr [{mk} + 0x1CE358]",
            "fsub dword ptr [{mk} + 0x1CE334]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{mk} + 0x1CE35C]",
            "fsub dword ptr [{mk} + 0x1CE338]",
            "fstp dword ptr [{mk} + 0x1CE35C]",
            "fld dword ptr [{mk} + 0x1CE360]",
            "fsub dword ptr [{mk} + 0x1CE330]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fld dword ptr [{mk} + 0x1CE364]",
            "fsub dword ptr [{mk} + 0x1CE334]",
            "fstp dword ptr [{mk} + 0x1CE364]",
            "fld dword ptr [{mk} + 0x1CE368]",
            "fsub dword ptr [{mk} + 0x1CE338]",
            "fstp dword ptr [{mk} + 0x1CE368]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn projected_2_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE344]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE33C]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fmul dword ptr [{mk} + 0x1CE340]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE350]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE348]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fmul dword ptr [{mk} + 0x1CE34C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE35C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE354]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fmul dword ptr [{mk} + 0x1CE358]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE368]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE360]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fmul dword ptr [{mk} + 0x1CE364]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE364]",
            rc = in(reg) rc,
            mk = in(reg) mk,
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
    use crate::marks::asm_test::{Op, Spec, differential};

    #[test]
    fn light_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1CEFA0,
                    hi: 0x1CFC80,
                    reads: &[(0x1CFC74, F32), (0x1CFC78, F32), (0x1CFC7C, F32)],
                    writes: &[(0x1CEFA0, F32), (0x1CEFA4, F32), (0x1CEFA8, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::light_turned_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::light_turned_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn second_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1CEFA0,
                    hi: 0x1CF018,
                    reads: &[(0x1CEFA0, F32), (0x1CEFA4, F32), (0x1CEFA8, F32)],
                    writes: &[(0x1CF00C, F32), (0x1CF010, F32), (0x1CF014, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::second_turned_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::second_turned_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1CEF90,
                hi: 0x1CFC3C,
                reads: &[
                    (0x1CEF90, F32),
                    (0x1CEF94, F32),
                    (0x1CEF98, F32),
                    (0x1CEFB0, F32),
                    (0x1CEFBC, F32),
                    (0x1CEFC8, F32),
                    (0x1CF000, F32),
                    (0x1CF004, F32),
                    (0x1CF008, F32),
                    (0x1CF00C, F32),
                    (0x1CF010, F32),
                    (0x1CF014, F32),
                    (0x1CF028, F32),
                    (0x1CF02C, F32),
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                    (0x1CFC38, F32),
                ],
                writes: &[
                    (0x1CF000, F32),
                    (0x1CF004, F32),
                    (0x1CF008, F32),
                    (0x1CF00C, F32),
                    (0x1CF010, F32),
                    (0x1CF014, F32),
                    (0x1CF04C, F32),
                    (0x1CF050, F32),
                    (0x1CF054, F32),
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                ],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::corners_x87(b[0]) },
            |b| unsafe { super::corners_x87(b[0]) },
        );
    }

    #[test]
    fn corners_in_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1CF048,
                hi: 0x1CF088,
                reads: &[
                    (0x1CF048, F32),
                    (0x1CF04C, F32),
                    (0x1CF050, F32),
                    (0x1CF054, F32),
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                ],
                writes: &[
                    (0x1CF04C, F32),
                    (0x1CF050, F32),
                    (0x1CF054, F32),
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                ],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::corners_in_x87(b[0]) },
            |b| unsafe { super::corners_in_x87(b[0]) },
        );
    }

    #[test]
    fn projected_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x2240A4,
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x1CF058,
                    hi: 0x1CF088,
                    reads: &[
                        (0x1CF058, F32),
                        (0x1CF05C, F32),
                        (0x1CF060, F32),
                        (0x1CF064, F32),
                        (0x1CF068, F32),
                        (0x1CF06C, F32),
                        (0x1CF070, F32),
                        (0x1CF074, F32),
                        (0x1CF078, F32),
                        (0x1CF07C, F32),
                        (0x1CF080, F32),
                        (0x1CF084, F32),
                    ],
                    writes: &[
                        (0x1CF058, F32),
                        (0x1CF05C, F32),
                        (0x1CF064, F32),
                        (0x1CF068, F32),
                        (0x1CF070, F32),
                        (0x1CF074, F32),
                        (0x1CF07C, F32),
                        (0x1CF080, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::projected_x87(b[0], b[1]) },
            |b| unsafe { super::projected_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn light_turned_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1CE284,
                    hi: 0x1CEF80,
                    reads: &[(0x1CEF74, F32), (0x1CEF78, F32), (0x1CEF7C, F32)],
                    writes: &[(0x1CE284, F32), (0x1CE288, F32), (0x1CE28C, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::light_turned_2_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::light_turned_2_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn second_turned_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1CE284,
                    hi: 0x1CE2FC,
                    reads: &[(0x1CE284, F32), (0x1CE288, F32), (0x1CE28C, F32)],
                    writes: &[(0x1CE2F0, F32), (0x1CE2F4, F32), (0x1CE2F8, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::second_turned_2_x87(b[0], b[1], b[2]) },
            |b| unsafe { super::second_turned_2_x87(b[0], b[1], b[2]) },
        );
    }

    #[test]
    fn corners_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1CE274,
                hi: 0x1CEF34,
                reads: &[
                    (0x1CE274, F32),
                    (0x1CE278, F32),
                    (0x1CE27C, F32),
                    (0x1CE294, F32),
                    (0x1CE2A0, F32),
                    (0x1CE2AC, F32),
                    (0x1CE2E4, F32),
                    (0x1CE2E8, F32),
                    (0x1CE2EC, F32),
                    (0x1CE2F0, F32),
                    (0x1CE2F4, F32),
                    (0x1CE2F8, F32),
                    (0x1CE30C, F32),
                    (0x1CE310, F32),
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                    (0x1CEF30, F32),
                ],
                writes: &[
                    (0x1CE2E4, F32),
                    (0x1CE2E8, F32),
                    (0x1CE2EC, F32),
                    (0x1CE2F0, F32),
                    (0x1CE2F4, F32),
                    (0x1CE2F8, F32),
                    (0x1CE330, F32),
                    (0x1CE334, F32),
                    (0x1CE338, F32),
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                ],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::corners_2_x87(b[0]) },
            |b| unsafe { super::corners_2_x87(b[0]) },
        );
    }

    #[test]
    fn corners_in_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1CE32C,
                hi: 0x1CE36C,
                reads: &[
                    (0x1CE32C, F32),
                    (0x1CE330, F32),
                    (0x1CE334, F32),
                    (0x1CE338, F32),
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                ],
                writes: &[
                    (0x1CE330, F32),
                    (0x1CE334, F32),
                    (0x1CE338, F32),
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                ],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::corners_in_2_x87(b[0]) },
            |b| unsafe { super::corners_in_2_x87(b[0]) },
        );
    }

    #[test]
    fn projected_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x2240A4,
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x1CE33C,
                    hi: 0x1CE36C,
                    reads: &[
                        (0x1CE33C, F32),
                        (0x1CE340, F32),
                        (0x1CE344, F32),
                        (0x1CE348, F32),
                        (0x1CE34C, F32),
                        (0x1CE350, F32),
                        (0x1CE354, F32),
                        (0x1CE358, F32),
                        (0x1CE35C, F32),
                        (0x1CE360, F32),
                        (0x1CE364, F32),
                        (0x1CE368, F32),
                    ],
                    writes: &[
                        (0x1CE33C, F32),
                        (0x1CE340, F32),
                        (0x1CE348, F32),
                        (0x1CE34C, F32),
                        (0x1CE354, F32),
                        (0x1CE358, F32),
                        (0x1CE360, F32),
                        (0x1CE364, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::projected_2_x87(b[0], b[1]) },
            |b| unsafe { super::projected_2_x87(b[0], b[1]) },
        );
    }
}
