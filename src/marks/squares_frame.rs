//! `marks::squares_frame` (0x4BF60F): the squares' frame (as `marks::draw_plane`'s, with
//! the rate times `flt_8BC19E` and the lengths over `flt_8BC1A2` / `flt_8BC1A6`): the
//! rate, the place and two axes from the settings, made one long and square, scaled
//! (`marks::scale_axes`); nothing more unless the place's way faces the third axis away;
//! then `marks::run_pong`; unless the rate times `flt_8BC1AA` comes to 0 (`fistp`, as the
//! x87 is set) `marks::run_squares`, and with the settings' +0x2744 and `dword_7652A0`
//! `marks::squares_step`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C74DA` (0x4C7534). The x87 runs are the original's instructions over the
//! game's variables; the vector helpers, `scale_axes` and `squares_step` their Rust
//! versions. Refuses null settings and `squares_step`'s refusals.

use crate::errors::{BadInput, Refused};
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::squares_step::squares_step;
use crate::marks::store::{Slot, Where, game_addr, game_read};
use crate::math::x87;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;

const AXES: [Slot; 3] = [Slot::at(0x89CF1A), Slot::at(0x89CF26), Slot::at(0x89CF32)];
const RATE_RVA: Slot = Slot::at(0x89CA34); // flt_89CA34
const PLACE_RVA: Slot = Slot::at(0x8161A2); // flt_8161A2
const FACING_RVA: Slot = Slot::at(0x814718); // flt_814718
const SHARE_RVA: Slot = Slot::at(0x8BC1AA); // flt_8BC1AA
const RUNNING_RVA: Slot = Slot::at(0x7652A0); // dword_7652A0

fn addr(r: impl Where) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4BF615 .. 0x4BF6AB: the rate (`flt_89CA34`), the place and the first axis from the settings, its length over `flt_8BC1A2` into `flt_89CEEE`.
unsafe fn frame_axes(s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2740]",
            "fmul dword ptr [{mk} + 0x15725E]",
            "fstp dword ptr [{mk} + 0x137AF4]",
            "fild dword ptr [{s} + 0x2720]",
            "fild dword ptr [{s} + 0x2724]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FBE]",
            "fstp dword ptr [{mk} + 0x137FBA]",
            "fstp dword ptr [{mk} + 0x137FB6]",
            "fild dword ptr [{s} + 0x272C]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0x137FDA]",
            "fild dword ptr [{s} + 0x2730]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0x137FDE]",
            "fild dword ptr [{s} + 0x2734]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FE2]",
            "fld dword ptr [{mk} + 0x137FDA]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FE2]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{mk} + 0x157262]",
            "fstp dword ptr [{mk} + 0x137FAE]",
            mk = in(reg) crate::marks::store::base(),
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF6B8 .. 0x4BF718: the second axis, its length over `flt_8BC1A6` into `flt_89CEF2`.
unsafe fn frame_up(s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2708]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0x137FE6]",
            "fild dword ptr [{s} + 0x270C]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0x137FEA]",
            "fild dword ptr [{s} + 0x2710]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FEE]",
            "fld dword ptr [{mk} + 0x137FE6]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FEA]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FEE]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{mk} + 0x157266]",
            "fstp dword ptr [{mk} + 0x137FB2]",
            mk = in(reg) crate::marks::store::base(),
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF769 .. 0x4BF78D: the place as floats into `flt_8161A2` ...
unsafe fn frame_place(s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0xB1262]",
            "fild dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0xB1266]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0xB126A]",
            mk = in(reg) crate::marks::store::base(),
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF7A4 .. 0x4BF7BE: the place's way dotted with the third axis into `flt_814718`.
unsafe fn frame_facing() {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{mk} + 0xB1262]",
            "fmul dword ptr [{mk} + 0x137FF2]",
            "fld dword ptr [{mk} + 0xB1266]",
            "fmul dword ptr [{mk} + 0x137FF6]",
            "fld dword ptr [{mk} + 0xB126A]",
            "fmul dword ptr [{mk} + 0x137FFA]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0xAF7D8]",
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn squares_frame() -> Result<(), Refused> {
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        frame_axes(s);
        let [a0, a1, a2] = AXES.map(addr);
        normalise_vector3(a0);
        frame_up(s);
        normalise_vector3(a1);
        cross_product3(a1, a0, a2)?;
        normalise_vector3(a2);
        cross_product3(a0, a2, a1)?;
        crate::marks::scale_axes::scale_axes();
        frame_place(s);
        normalise_vector3(addr(PLACE_RVA));
        frame_facing();
        if game_read::<u32>(FACING_RVA) & 0x8000_0000 == 0 {
            return Ok(());
        }
        crate::marks::run_pong::run_pong()?;
        if x87::mul_to_int(game_read(RATE_RVA), game_read(SHARE_RVA)) == 0 {
            return Ok(());
        }
        crate::marks::run_squares::run_squares();
        let s: u32 = game_read(SETTINGS_RVA);
        if ((s + 0x2744) as *const u32).read_unaligned() == 0 {
            return Ok(());
        }
        if game_read::<u32>(RUNNING_RVA) == 0 {
            return Ok(());
        }
        squares_step()?;
    }
    Ok(())
}
