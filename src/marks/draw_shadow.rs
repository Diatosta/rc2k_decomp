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
    projected: unsafe fn(u32),
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
            mk = in(reg) crate::marks::store::base(),
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C838A .. 0x4C83FA: `flt_933EE0` .. turned by the matrix at m into `dword_933F4C` ...
unsafe fn second_turned(m: u32, t: u32) {
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
            mk = in(reg) crate::marks::store::base(),
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C8428 .. 0x4C86E6: the two sides scaled, the quad's four corners about `flt_933ED0` .., lifted along the up (`flt_934B78`), the way between two corners into `flt_933F8C` ...
unsafe fn corners() {
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
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C86F1 .. 0x4C87FF: that way times `flt_933F88`, taken off every corner.
unsafe fn corners_in() {
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
            mk = in(reg) crate::marks::store::base(),
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
            "fdiv dword ptr [{mk} + 0x1CF060]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF058]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fmul dword ptr [{mk} + 0x1CF05C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF06C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF064]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fmul dword ptr [{mk} + 0x1CF068]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF078]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF070]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fmul dword ptr [{mk} + 0x1CF074]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF084]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF07C]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fmul dword ptr [{mk} + 0x1CF080]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            b = in(reg) b,
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's (0x4C7680 ..): as `light_turned`, on the second shadow's variables.
unsafe fn light_turned_2(m: u32, t: u32) {
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
            mk = in(reg) crate::marks::store::base(),
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's: as `second_turned`.
unsafe fn second_turned_2(m: u32, t: u32) {
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
            mk = in(reg) crate::marks::store::base(),
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's: as `corners`.
unsafe fn corners_2() {
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
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The twin's: as `corners_in`.
unsafe fn corners_in_2() {
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
            mk = in(reg) crate::marks::store::base(),
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
            "fdiv dword ptr [{mk} + 0x1CE344]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE33C]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fmul dword ptr [{mk} + 0x1CE340]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE350]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE348]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fmul dword ptr [{mk} + 0x1CE34C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE35C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE354]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fmul dword ptr [{mk} + 0x1CE358]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE368]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE360]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fmul dword ptr [{mk} + 0x1CE364]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE364]",
            b = in(reg) b,
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
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
        (sh.projected)(addr(0));
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
