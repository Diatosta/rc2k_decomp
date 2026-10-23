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
//! Out: the registers as the path leaves them
//!
//! Caller: the race's frame (0x456330). The long x87 runs are the original's instructions
//! over the game's variables; the vector helpers their Rust versions; the device calls
//! through `part` (1 the depth buffer, 2 the depth test, 3 a render state, 4 `draw_fan`),
//! in the game their Rust versions. Refuses, before anything, a corner past
//! `g_marksCorners` and an angle `fsincos` cannot take. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose device calls go to stand-ins (the vector helpers
//! real), against the Rust version, on the game's variables with sides and settings of the
//! test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::setup::SETTINGS_RVA;
use crate::math::{x87, x87_run};
use crate::scene::cross_product3::cross_product3;
use crate::scene::matrix3::{Axis, extended, rounded, turn_matrix3, turnable};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::normalise_vector3;

const VIEW_RVA: usize = rva(0x5EA0CC); // unk_5EA0CC
const MARK_CORNERS_RVA: usize = rva(0x8EB971); // g_marksCorners
const UV_K_RVA: usize = rva(0x8EB969); // flt_8EB969
const TINT_RVA: usize = rva(0x939BA6); // dword_939BA6
const NEAR_RVA: usize = rva(0x5C5E18); // flt_5C5E18
const Z_K_RVA: usize = rva(0x8EB96D); // flt_8EB96D
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

/// A shadow's variables (the second, `sub_4C74DA`'s, 0xD1C lower, its constants 0xD08 lower)
/// and its x87 blocks.
pub struct Shadow {
    corner: usize,
    sides: [usize; 2],
    side: [usize; 3],
    matrix: usize,
    split: usize,
    /// (factor, base) below the split, (factor, base) above it.
    lines: [(usize, usize); 2],
    way: usize,
    corners: usize,
    lifts: usize,
    /// The settings' field the angle comes from.
    field: u32,
    light_turned: unsafe fn(u32, u32, u32),
    second_turned: unsafe fn(u32, u32, u32),
    corners_block: unsafe fn(u32),
    corners_in: unsafe fn(u32),
    projected: unsafe fn(u32),
}

/// `draw_shadow`'s.
pub const FIRST: Shadow = Shadow {
    corner: rva(0x933F64),
    sides: [rva(0x933F1C), rva(0x933F28)],
    side: [rva(0x933F40), rva(0x933F4C), rva(0x933F58)],
    matrix: rva(0x933EEC),
    split: rva(0x933F78),
    lines: [
        (rva(0x933F74), rva(0x933F70)),
        (rva(0x933F84), rva(0x933F80)),
    ],
    way: rva(0x933F8C),
    corners: rva(0x933F98),
    lifts: rva(0x934BC0),
    field: 0x26A4,
    light_turned,
    second_turned,
    corners_block: corners,
    corners_in,
    projected,
};

/// `sub_4C74DA`'s.
pub const SECOND: Shadow = Shadow {
    corner: rva(0x933248),
    sides: [rva(0x933200), rva(0x93320C)],
    side: [rva(0x933224), rva(0x933230), rva(0x93323C)],
    matrix: rva(0x9331D0),
    split: rva(0x93325C),
    lines: [
        (rva(0x933258), rva(0x933254)),
        (rva(0x933268), rva(0x933264)),
    ],
    way: rva(0x933270),
    corners: rva(0x93327C),
    lifts: rva(0x933EC0),
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

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn addr(r: usize) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4C82B4 .. 0x4C8324: `flt_934BB4` .. turned by the matrix at m (`[esp]` temporaries at t) into `flt_933EE0` ...
unsafe fn light_turned(b: u32, m: u32, t: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x534BB4]",
            "fld dword ptr [{b} + 0x534BB8]",
            "fld dword ptr [{b} + 0x534BBC]",
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
            "fstp dword ptr [{b} + 0x533EE0]",
            "fstp dword ptr [{b} + 0x533EE4]",
            "fstp dword ptr [{b} + 0x533EE8]",
            b = in(reg) b,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C838A .. 0x4C83FA: `flt_933EE0` .. turned by the matrix at m into `dword_933F4C` ...
unsafe fn second_turned(b: u32, m: u32, t: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x533EE8]",
            "fld dword ptr [{b} + 0x533EE4]",
            "fld dword ptr [{b} + 0x533EE0]",
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
            "fstp dword ptr [{b} + 0x533F4C]",
            "fstp dword ptr [{b} + 0x533F50]",
            "fstp dword ptr [{b} + 0x533F54]",
            b = in(reg) b,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C8428 .. 0x4C86E6: the two sides scaled, the quad's four corners about `flt_933ED0` .., lifted along the up (`flt_934B78`), the way between two corners into `flt_933F8C` ...
unsafe fn corners(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x533F40]",
            "fmul dword ptr [{b} + 0x533F68]",
            "fstp dword ptr [{b} + 0x533F40]",
            "fld dword ptr [{b} + 0x533F44]",
            "fmul dword ptr [{b} + 0x533F68]",
            "fstp dword ptr [{b} + 0x533F44]",
            "fld dword ptr [{b} + 0x533F48]",
            "fmul dword ptr [{b} + 0x533F68]",
            "fstp dword ptr [{b} + 0x533F48]",
            "fld dword ptr [{b} + 0x533F4C]",
            "fmul dword ptr [{b} + 0x533F6C]",
            "fstp dword ptr [{b} + 0x533F4C]",
            "fld dword ptr [{b} + 0x533F50]",
            "fmul dword ptr [{b} + 0x533F6C]",
            "fstp dword ptr [{b} + 0x533F50]",
            "fld dword ptr [{b} + 0x533F54]",
            "fmul dword ptr [{b} + 0x533F6C]",
            "fstp dword ptr [{b} + 0x533F54]",
            "fld dword ptr [{b} + 0x533ED0]",
            "fsub dword ptr [{b} + 0x533F40]",
            "fstp dword ptr [{b} + 0x533F98]",
            "fld dword ptr [{b} + 0x533ED4]",
            "fsub dword ptr [{b} + 0x533F44]",
            "fstp dword ptr [{b} + 0x533F9C]",
            "fld dword ptr [{b} + 0x533ED8]",
            "fsub dword ptr [{b} + 0x533F48]",
            "fstp dword ptr [{b} + 0x533FA0]",
            "fld dword ptr [{b} + 0x533ED0]",
            "fadd dword ptr [{b} + 0x533F40]",
            "fstp dword ptr [{b} + 0x533FA4]",
            "fld dword ptr [{b} + 0x533ED4]",
            "fadd dword ptr [{b} + 0x533F44]",
            "fstp dword ptr [{b} + 0x533FA8]",
            "fld dword ptr [{b} + 0x533ED8]",
            "fadd dword ptr [{b} + 0x533F48]",
            "fstp dword ptr [{b} + 0x533FAC]",
            "fld dword ptr [{b} + 0x533ED0]",
            "fadd dword ptr [{b} + 0x533F40]",
            "fadd dword ptr [{b} + 0x533F4C]",
            "fstp dword ptr [{b} + 0x533FB0]",
            "fld dword ptr [{b} + 0x533ED4]",
            "fadd dword ptr [{b} + 0x533F44]",
            "fadd dword ptr [{b} + 0x533F50]",
            "fstp dword ptr [{b} + 0x533FB4]",
            "fld dword ptr [{b} + 0x533ED8]",
            "fadd dword ptr [{b} + 0x533F48]",
            "fadd dword ptr [{b} + 0x533F54]",
            "fstp dword ptr [{b} + 0x533FB8]",
            "fld dword ptr [{b} + 0x533ED0]",
            "fsub dword ptr [{b} + 0x533F40]",
            "fadd dword ptr [{b} + 0x533F4C]",
            "fstp dword ptr [{b} + 0x533FBC]",
            "fld dword ptr [{b} + 0x533ED4]",
            "fsub dword ptr [{b} + 0x533F44]",
            "fadd dword ptr [{b} + 0x533F50]",
            "fstp dword ptr [{b} + 0x533FC0]",
            "fld dword ptr [{b} + 0x533ED8]",
            "fsub dword ptr [{b} + 0x533F48]",
            "fadd dword ptr [{b} + 0x533F54]",
            "fstp dword ptr [{b} + 0x533FC4]",
            "fld dword ptr [{b} + 0x533EF0]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533F98]",
            "fstp dword ptr [{b} + 0x533F98]",
            "fld dword ptr [{b} + 0x533EFC]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533F9C]",
            "fstp dword ptr [{b} + 0x533F9C]",
            "fld dword ptr [{b} + 0x533F08]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FA0]",
            "fstp dword ptr [{b} + 0x533FA0]",
            "fld dword ptr [{b} + 0x533EF0]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FA4]",
            "fstp dword ptr [{b} + 0x533FA4]",
            "fld dword ptr [{b} + 0x533EFC]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FA8]",
            "fstp dword ptr [{b} + 0x533FA8]",
            "fld dword ptr [{b} + 0x533F08]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FAC]",
            "fstp dword ptr [{b} + 0x533FAC]",
            "fld dword ptr [{b} + 0x533EF0]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FB0]",
            "fstp dword ptr [{b} + 0x533FB0]",
            "fld dword ptr [{b} + 0x533EFC]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FB4]",
            "fstp dword ptr [{b} + 0x533FB4]",
            "fld dword ptr [{b} + 0x533F08]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FB8]",
            "fstp dword ptr [{b} + 0x533FB8]",
            "fld dword ptr [{b} + 0x533EF0]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FBC]",
            "fstp dword ptr [{b} + 0x533FBC]",
            "fld dword ptr [{b} + 0x533EFC]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FC0]",
            "fstp dword ptr [{b} + 0x533FC0]",
            "fld dword ptr [{b} + 0x533F08]",
            "fmul dword ptr [{b} + 0x534B78]",
            "fadd dword ptr [{b} + 0x533FC4]",
            "fstp dword ptr [{b} + 0x533FC4]",
            "fld dword ptr [{b} + 0x533FB0]",
            "fsub dword ptr [{b} + 0x533FA4]",
            "fstp dword ptr [{b} + 0x533F8C]",
            "fld dword ptr [{b} + 0x533FB4]",
            "fsub dword ptr [{b} + 0x533FA8]",
            "fstp dword ptr [{b} + 0x533F90]",
            "fld dword ptr [{b} + 0x533FB8]",
            "fsub dword ptr [{b} + 0x533FAC]",
            "fstp dword ptr [{b} + 0x533F94]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C86F1 .. 0x4C87FF: that way times `flt_933F88`, taken off every corner.
unsafe fn corners_in(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x533F8C]",
            "fmul dword ptr [{b} + 0x533F88]",
            "fstp dword ptr [{b} + 0x533F8C]",
            "fld dword ptr [{b} + 0x533F90]",
            "fmul dword ptr [{b} + 0x533F88]",
            "fstp dword ptr [{b} + 0x533F90]",
            "fld dword ptr [{b} + 0x533F94]",
            "fmul dword ptr [{b} + 0x533F88]",
            "fstp dword ptr [{b} + 0x533F94]",
            "fld dword ptr [{b} + 0x533F98]",
            "fsub dword ptr [{b} + 0x533F8C]",
            "fstp dword ptr [{b} + 0x533F98]",
            "fld dword ptr [{b} + 0x533F9C]",
            "fsub dword ptr [{b} + 0x533F90]",
            "fstp dword ptr [{b} + 0x533F9C]",
            "fld dword ptr [{b} + 0x533FA0]",
            "fsub dword ptr [{b} + 0x533F94]",
            "fstp dword ptr [{b} + 0x533FA0]",
            "fld dword ptr [{b} + 0x533FA4]",
            "fsub dword ptr [{b} + 0x533F8C]",
            "fstp dword ptr [{b} + 0x533FA4]",
            "fld dword ptr [{b} + 0x533FA8]",
            "fsub dword ptr [{b} + 0x533F90]",
            "fstp dword ptr [{b} + 0x533FA8]",
            "fld dword ptr [{b} + 0x533FAC]",
            "fsub dword ptr [{b} + 0x533F94]",
            "fstp dword ptr [{b} + 0x533FAC]",
            "fld dword ptr [{b} + 0x533FB0]",
            "fsub dword ptr [{b} + 0x533F8C]",
            "fstp dword ptr [{b} + 0x533FB0]",
            "fld dword ptr [{b} + 0x533FB4]",
            "fsub dword ptr [{b} + 0x533F90]",
            "fstp dword ptr [{b} + 0x533FB4]",
            "fld dword ptr [{b} + 0x533FB8]",
            "fsub dword ptr [{b} + 0x533F94]",
            "fstp dword ptr [{b} + 0x533FB8]",
            "fld dword ptr [{b} + 0x533FBC]",
            "fsub dword ptr [{b} + 0x533F8C]",
            "fstp dword ptr [{b} + 0x533FBC]",
            "fld dword ptr [{b} + 0x533FC0]",
            "fsub dword ptr [{b} + 0x533F90]",
            "fstp dword ptr [{b} + 0x533FC0]",
            "fld dword ptr [{b} + 0x533FC4]",
            "fsub dword ptr [{b} + 0x533F94]",
            "fstp dword ptr [{b} + 0x533FC4]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C8A4A .. 0x4C8B1A: the four corners projected.
unsafe fn projected(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x533FA0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x533F98]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x533F98]",
            "fmul dword ptr [{b} + 0x533F9C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x533F9C]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x533FAC]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x533FA4]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x533FA4]",
            "fmul dword ptr [{b} + 0x533FA8]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x533FA8]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x533FB8]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x533FB0]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x533FB0]",
            "fmul dword ptr [{b} + 0x533FB4]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x533FB4]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x533FC4]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x533FBC]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x533FBC]",
            "fmul dword ptr [{b} + 0x533FC0]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x533FC0]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's (0x4C7680 ..): as `light_turned`, on the second shadow's variables.
unsafe fn light_turned_2(b: u32, m: u32, t: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x533EB4]",
            "fld dword ptr [{b} + 0x533EB8]",
            "fld dword ptr [{b} + 0x533EBC]",
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
            "fstp dword ptr [{b} + 0x5331C4]",
            "fstp dword ptr [{b} + 0x5331C8]",
            "fstp dword ptr [{b} + 0x5331CC]",
            b = in(reg) b,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's: as `second_turned`.
unsafe fn second_turned_2(b: u32, m: u32, t: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x5331CC]",
            "fld dword ptr [{b} + 0x5331C8]",
            "fld dword ptr [{b} + 0x5331C4]",
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
            "fstp dword ptr [{b} + 0x533230]",
            "fstp dword ptr [{b} + 0x533234]",
            "fstp dword ptr [{b} + 0x533238]",
            b = in(reg) b,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's: as `corners`.
unsafe fn corners_2(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x533224]",
            "fmul dword ptr [{b} + 0x53324C]",
            "fstp dword ptr [{b} + 0x533224]",
            "fld dword ptr [{b} + 0x533228]",
            "fmul dword ptr [{b} + 0x53324C]",
            "fstp dword ptr [{b} + 0x533228]",
            "fld dword ptr [{b} + 0x53322C]",
            "fmul dword ptr [{b} + 0x53324C]",
            "fstp dword ptr [{b} + 0x53322C]",
            "fld dword ptr [{b} + 0x533230]",
            "fmul dword ptr [{b} + 0x533250]",
            "fstp dword ptr [{b} + 0x533230]",
            "fld dword ptr [{b} + 0x533234]",
            "fmul dword ptr [{b} + 0x533250]",
            "fstp dword ptr [{b} + 0x533234]",
            "fld dword ptr [{b} + 0x533238]",
            "fmul dword ptr [{b} + 0x533250]",
            "fstp dword ptr [{b} + 0x533238]",
            "fld dword ptr [{b} + 0x5331B4]",
            "fsub dword ptr [{b} + 0x533224]",
            "fstp dword ptr [{b} + 0x53327C]",
            "fld dword ptr [{b} + 0x5331B8]",
            "fsub dword ptr [{b} + 0x533228]",
            "fstp dword ptr [{b} + 0x533280]",
            "fld dword ptr [{b} + 0x5331BC]",
            "fsub dword ptr [{b} + 0x53322C]",
            "fstp dword ptr [{b} + 0x533284]",
            "fld dword ptr [{b} + 0x5331B4]",
            "fadd dword ptr [{b} + 0x533224]",
            "fstp dword ptr [{b} + 0x533288]",
            "fld dword ptr [{b} + 0x5331B8]",
            "fadd dword ptr [{b} + 0x533228]",
            "fstp dword ptr [{b} + 0x53328C]",
            "fld dword ptr [{b} + 0x5331BC]",
            "fadd dword ptr [{b} + 0x53322C]",
            "fstp dword ptr [{b} + 0x533290]",
            "fld dword ptr [{b} + 0x5331B4]",
            "fadd dword ptr [{b} + 0x533224]",
            "fadd dword ptr [{b} + 0x533230]",
            "fstp dword ptr [{b} + 0x533294]",
            "fld dword ptr [{b} + 0x5331B8]",
            "fadd dword ptr [{b} + 0x533228]",
            "fadd dword ptr [{b} + 0x533234]",
            "fstp dword ptr [{b} + 0x533298]",
            "fld dword ptr [{b} + 0x5331BC]",
            "fadd dword ptr [{b} + 0x53322C]",
            "fadd dword ptr [{b} + 0x533238]",
            "fstp dword ptr [{b} + 0x53329C]",
            "fld dword ptr [{b} + 0x5331B4]",
            "fsub dword ptr [{b} + 0x533224]",
            "fadd dword ptr [{b} + 0x533230]",
            "fstp dword ptr [{b} + 0x5332A0]",
            "fld dword ptr [{b} + 0x5331B8]",
            "fsub dword ptr [{b} + 0x533228]",
            "fadd dword ptr [{b} + 0x533234]",
            "fstp dword ptr [{b} + 0x5332A4]",
            "fld dword ptr [{b} + 0x5331BC]",
            "fsub dword ptr [{b} + 0x53322C]",
            "fadd dword ptr [{b} + 0x533238]",
            "fstp dword ptr [{b} + 0x5332A8]",
            "fld dword ptr [{b} + 0x5331D4]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x53327C]",
            "fstp dword ptr [{b} + 0x53327C]",
            "fld dword ptr [{b} + 0x5331E0]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x533280]",
            "fstp dword ptr [{b} + 0x533280]",
            "fld dword ptr [{b} + 0x5331EC]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x533284]",
            "fstp dword ptr [{b} + 0x533284]",
            "fld dword ptr [{b} + 0x5331D4]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x533288]",
            "fstp dword ptr [{b} + 0x533288]",
            "fld dword ptr [{b} + 0x5331E0]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x53328C]",
            "fstp dword ptr [{b} + 0x53328C]",
            "fld dword ptr [{b} + 0x5331EC]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x533290]",
            "fstp dword ptr [{b} + 0x533290]",
            "fld dword ptr [{b} + 0x5331D4]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x533294]",
            "fstp dword ptr [{b} + 0x533294]",
            "fld dword ptr [{b} + 0x5331E0]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x533298]",
            "fstp dword ptr [{b} + 0x533298]",
            "fld dword ptr [{b} + 0x5331EC]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x53329C]",
            "fstp dword ptr [{b} + 0x53329C]",
            "fld dword ptr [{b} + 0x5331D4]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x5332A0]",
            "fstp dword ptr [{b} + 0x5332A0]",
            "fld dword ptr [{b} + 0x5331E0]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x5332A4]",
            "fstp dword ptr [{b} + 0x5332A4]",
            "fld dword ptr [{b} + 0x5331EC]",
            "fmul dword ptr [{b} + 0x533E70]",
            "fadd dword ptr [{b} + 0x5332A8]",
            "fstp dword ptr [{b} + 0x5332A8]",
            "fld dword ptr [{b} + 0x533294]",
            "fsub dword ptr [{b} + 0x533288]",
            "fstp dword ptr [{b} + 0x533270]",
            "fld dword ptr [{b} + 0x533298]",
            "fsub dword ptr [{b} + 0x53328C]",
            "fstp dword ptr [{b} + 0x533274]",
            "fld dword ptr [{b} + 0x53329C]",
            "fsub dword ptr [{b} + 0x533290]",
            "fstp dword ptr [{b} + 0x533278]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's: as `corners_in`.
unsafe fn corners_in_2(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x533270]",
            "fmul dword ptr [{b} + 0x53326C]",
            "fstp dword ptr [{b} + 0x533270]",
            "fld dword ptr [{b} + 0x533274]",
            "fmul dword ptr [{b} + 0x53326C]",
            "fstp dword ptr [{b} + 0x533274]",
            "fld dword ptr [{b} + 0x533278]",
            "fmul dword ptr [{b} + 0x53326C]",
            "fstp dword ptr [{b} + 0x533278]",
            "fld dword ptr [{b} + 0x53327C]",
            "fsub dword ptr [{b} + 0x533270]",
            "fstp dword ptr [{b} + 0x53327C]",
            "fld dword ptr [{b} + 0x533280]",
            "fsub dword ptr [{b} + 0x533274]",
            "fstp dword ptr [{b} + 0x533280]",
            "fld dword ptr [{b} + 0x533284]",
            "fsub dword ptr [{b} + 0x533278]",
            "fstp dword ptr [{b} + 0x533284]",
            "fld dword ptr [{b} + 0x533288]",
            "fsub dword ptr [{b} + 0x533270]",
            "fstp dword ptr [{b} + 0x533288]",
            "fld dword ptr [{b} + 0x53328C]",
            "fsub dword ptr [{b} + 0x533274]",
            "fstp dword ptr [{b} + 0x53328C]",
            "fld dword ptr [{b} + 0x533290]",
            "fsub dword ptr [{b} + 0x533278]",
            "fstp dword ptr [{b} + 0x533290]",
            "fld dword ptr [{b} + 0x533294]",
            "fsub dword ptr [{b} + 0x533270]",
            "fstp dword ptr [{b} + 0x533294]",
            "fld dword ptr [{b} + 0x533298]",
            "fsub dword ptr [{b} + 0x533274]",
            "fstp dword ptr [{b} + 0x533298]",
            "fld dword ptr [{b} + 0x53329C]",
            "fsub dword ptr [{b} + 0x533278]",
            "fstp dword ptr [{b} + 0x53329C]",
            "fld dword ptr [{b} + 0x5332A0]",
            "fsub dword ptr [{b} + 0x533270]",
            "fstp dword ptr [{b} + 0x5332A0]",
            "fld dword ptr [{b} + 0x5332A4]",
            "fsub dword ptr [{b} + 0x533274]",
            "fstp dword ptr [{b} + 0x5332A4]",
            "fld dword ptr [{b} + 0x5332A8]",
            "fsub dword ptr [{b} + 0x533278]",
            "fstp dword ptr [{b} + 0x5332A8]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's: as `projected`.
unsafe fn projected_2(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x533284]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x53327C]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x53327C]",
            "fmul dword ptr [{b} + 0x533280]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x533280]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x533290]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x533288]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x533288]",
            "fmul dword ptr [{b} + 0x53328C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x53328C]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x53329C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x533294]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x533294]",
            "fmul dword ptr [{b} + 0x533298]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x533298]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x5332A8]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x5332A0]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{b} + 0x5332A0]",
            "fmul dword ptr [{b} + 0x5332A4]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{b} + 0x5332A4]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The z turn's angle (see the header), 80 bits.
unsafe fn angle(g: &mut Regs, sh: &Shadow) -> [u8; 10] {
    unsafe {
        g.esi = game_read(SETTINGS_RVA);
        g.eax = game_read(sh.split);
        let v = ((g.esi + sh.field) as *const u32).read_unaligned();
        if (g.eax as i32) <= v as i32 {
            let (k, c) = sh.lines[1];
            x87::less_times_plus_negated_80(v, g.eax, game_read(k), game_read(c))
        } else {
            let (k, c) = sh.lines[0];
            x87::times_plus_negated_80(v, game_read(k), game_read(c))
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_shadow(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { draw_shadow_with(entry, &FIRST, true, part) }
}

impl Shadow {
    /// The corner's variable (-1: no shadow).
    pub fn corner(&self) -> usize {
        self.corner
    }

    /// The angle's refusal, if any (for a caller that checks before drawing).
    pub fn angle_refuses(&self) -> Option<Refused> {
        unsafe { turnable(&angle(&mut Regs::default(), self)) }.err()
    }
}

/// A shadow (see the header) on `sh`'s variables; `depth` the two depth calls first.
pub unsafe fn draw_shadow_with(
    entry: &Regs,
    sh: &Shadow,
    depth: bool,
    part: Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let corner: u32 = game_read(sh.corner);
        if corner == 0xFFFF_FFFF {
            return Ok(g);
        }
        if corner >= 0x100 {
            return Err(BadInput("a shadow corner past g_marksCorners").into());
        }
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        turnable(&angle(&mut g.clone(), sh))?;
        let b = addr(0);
        let mut temps = [0u32; 2];
        let t = temps.as_mut_ptr() as u32;
        if depth {
            part(1, &mut g)?;
            part(2, &mut g)?;
        }
        for (value, state) in STATES {
            [g.eax, g.ebx] = [value, state];
            part(3, &mut g)?;
        }
        for (n, from) in sh.sides.into_iter().enumerate() {
            let v: [u32; 3] = game_read(from);
            [g.eax, g.ebx, g.ecx] = v;
            for (k, x) in v.into_iter().enumerate() {
                game_write(sh.side[n] + 4 * k, x);
            }
        }
        let [s0, s1, s2] = sh.side.map(addr);
        g.esi = s0;
        normalise_vector3(s0);
        g.esi = s1;
        normalise_vector3(s1);
        [g.esi, g.edi, g.ebp] = [s0, s1, s2];
        cross_product3(s0, s1, s2)?;
        [g.esi, g.edi, g.ebp] = [s2, s0, s1];
        cross_product3(s2, s0, s1)?;
        g.esi = s1;
        normalise_vector3(s1);
        g.esi = s2;
        normalise_vector3(s2);
        let a = angle(&mut g, sh);
        g.esi = addr(sh.matrix);
        turn_matrix3(Axis::Z, g.esi, a)?;
        (sh.light_turned)(b, g.esi, t);
        let m = g.esi;
        for (n, side) in sh.side.into_iter().enumerate() {
            let v: [u32; 3] = game_read(side);
            [g.eax, g.ebx, g.ecx] = v;
            for (k, x) in v.into_iter().enumerate() {
                ((m + 4 * n as u32 + 0xC * k as u32) as *mut u32).write_unaligned(x);
            }
        }
        (sh.second_turned)(b, m, t);
        [g.esi, g.edi, g.ebp] = [s2, s1, s0];
        cross_product3(s2, s1, s0)?;
        [g.esi, g.edi, g.ebp] = [s2, s0, s1];
        cross_product3(s2, s0, s1)?;
        (sh.corners_block)(b);
        g.esi = addr(sh.way);
        normalise_vector3(g.esi);
        (sh.corners_in)(b);
        g.esi = addr(VIEW_RVA);
        for n in 0..4 {
            let c = sh.corners + 12 * n;
            let p: [u32; 3] = game_read(c);
            let turned = rotate_point(g.esi, p.map(extended));
            for (k, v) in turned.into_iter().enumerate() {
                game_write(c + 4 * k, rounded(v));
            }
        }
        g.eax = addr(MARK_CORNERS_RVA) + corner * 0x80;
        let word = |off: u32| ((g.eax + off) as *const u16).read_unaligned() as u32;
        let uv_k: u32 = game_read(UV_K_RVA);
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex + 0x18, x87::int_times(word(u) as i32, uv_k));
            game_write(vertex + 0x1C, x87::int_times(word(v) as i32, uv_k));
        }
        [g.ebp, g.edi] = [word(8), word(0xA)];
        let mut c = game_read::<u32>(TINT_RVA) | 0xFF00_0000;
        let half = |shift: u32| ((!(c >> shift) & 0xFF) >> 1) << shift;
        let (r, gr, bl) = (half(16), half(8), half(0));
        c = c.wrapping_add(r).wrapping_add(gr).wrapping_add(bl);
        g.ecx = c;
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, c);
        }
        for n in 0..4 {
            g.ecx = game_read(sh.corners + 12 * n + 8);
            if (g.ecx as i32) <= game_read::<i32>(NEAR_RVA) {
                return Ok(g);
            }
        }
        (sh.projected)(b);
        let z_k: u32 = game_read(Z_K_RVA);
        for n in 0..4 {
            let c = sh.corners + 12 * n;
            let vertex = VERTICES_RVA + 0x20 * n;
            [g.eax, g.ebx] = [game_read(c), game_read(c + 4)];
            game_write(vertex, g.eax);
            game_write(vertex + 4, g.ebx);
            let depth = x87_run::sub_mul(game_read(c + 8), game_read(sh.lifts + 4 * n), z_k);
            game_write(vertex + 8, depth);
        }
        part(4, &mut g)?;
    }
    Ok(g)
}

/// In the game: the device calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::depth_buffer_on::depth_buffer(r, true),
            2 => crate::scene::depth_test_on::depth_test_on(r),
            3 => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, value, state);
            }
            _ => crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part),
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_shadow(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
