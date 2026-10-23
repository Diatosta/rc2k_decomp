//! `marks::squares_frame` (0x4BF60F): the squares' frame (as `marks::draw_plane`'s, with
//! the rate times `flt_8BC19E` and the lengths over `flt_8BC1A2` / `flt_8BC1A6`): the
//! rate, the place and two axes from the settings, made one long and square, scaled
//! (`marks::scale_axes`); nothing more unless the place's way faces the third axis away;
//! then `marks::run_pong`; unless the rate times `flt_8BC1AA` comes to 0 (`fistp`, as the
//! x87 is set) `marks::run_squares`, and with the settings' +0x2744 and `dword_7652A0`
//! `marks::squares_step`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the path leaves them
//!
//! Caller: `sub_4C74DA` (0x4C7534). The x87 runs are the original's instructions over the
//! game's variables; the vector helpers, `scale_axes` and `squares_step` their Rust
//! versions; pong and the squares through `part` (1 `run_pong`, 2 `run_squares`), in the
//! game their Rust versions. Refuses null settings and `squares_step`'s refusals. Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose pong and squares go
//! to stand-ins, against the Rust version, on settings of the test's; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::squares_step::squares_step;
use crate::math::x87;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;

const AXES: [usize; 3] = [rva(0x89CF1A), rva(0x89CF26), rva(0x89CF32)];
const RATE_RVA: usize = rva(0x89CA34); // flt_89CA34
const PLACE_RVA: usize = rva(0x8161A2); // flt_8161A2
const FACING_RVA: usize = rva(0x814718); // flt_814718
const SHARE_RVA: usize = rva(0x8BC1AA); // flt_8BC1AA
const RUNNING_RVA: usize = rva(0x7652A0); // dword_7652A0

fn addr(r: usize) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4BF615 .. 0x4BF6AB: the rate (`flt_89CA34`), the place and the first axis from the settings, its length over `flt_8BC1A2` into `flt_89CEEE`.
unsafe fn frame_axes(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2740]",
            "fmul dword ptr [{b} + 0x4BC19E]",
            "fstp dword ptr [{b} + 0x49CA34]",
            "fild dword ptr [{s} + 0x2720]",
            "fild dword ptr [{s} + 0x2724]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CEFE]",
            "fstp dword ptr [{b} + 0x49CEFA]",
            "fstp dword ptr [{b} + 0x49CEF6]",
            "fild dword ptr [{s} + 0x272C]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x49CF1A]",
            "fild dword ptr [{s} + 0x2730]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x49CF1E]",
            "fild dword ptr [{s} + 0x2734]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CF22]",
            "fld dword ptr [{b} + 0x49CF1A]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF1E]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF22]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{b} + 0x4BC1A2]",
            "fstp dword ptr [{b} + 0x49CEEE]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF6B8 .. 0x4BF718: the second axis, its length over `flt_8BC1A6` into `flt_89CEF2`.
unsafe fn frame_up(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2708]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x49CF26]",
            "fild dword ptr [{s} + 0x270C]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x49CF2A]",
            "fild dword ptr [{s} + 0x2710]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x49CF2E]",
            "fld dword ptr [{b} + 0x49CF26]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF2A]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x49CF2E]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{b} + 0x4BC1A6]",
            "fstp dword ptr [{b} + 0x49CEF2]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF769 .. 0x4BF78D: the place as floats into `flt_8161A2` ...
unsafe fn frame_place(b: u32, s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{b} + 0x4161A2]",
            "fild dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{b} + 0x4161A6]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{b} + 0x4161AA]",
            b = in(reg) b,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF7A4 .. 0x4BF7BE: the place's way dotted with the third axis into `flt_814718`.
unsafe fn frame_facing(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x4161A2]",
            "fmul dword ptr [{b} + 0x49CF32]",
            "fld dword ptr [{b} + 0x4161A6]",
            "fmul dword ptr [{b} + 0x49CF36]",
            "fld dword ptr [{b} + 0x4161AA]",
            "fmul dword ptr [{b} + 0x49CF3A]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{b} + 0x414718]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn squares_frame(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let b = addr(0);
        g.esi = s;
        frame_axes(b, s);
        let [a0, a1, a2] = AXES.map(addr);
        normalise_vector3(a0);
        frame_up(b, s);
        normalise_vector3(a1);
        [g.esi, g.edi, g.ebp] = [a1, a0, a2];
        cross_product3(a1, a0, a2)?;
        normalise_vector3(a2);
        [g.esi, g.edi, g.ebp] = [a0, a2, a1];
        cross_product3(a0, a2, a1)?;
        g.esi = s;
        g = crate::marks::scale_axes::scale_axes(&g);
        g.esi = s;
        frame_place(b, s);
        normalise_vector3(addr(PLACE_RVA));
        [g.esi, g.edi] = [addr(PLACE_RVA), a2];
        frame_facing(b);
        if game_read::<u32>(FACING_RVA) & 0x8000_0000 == 0 {
            return Ok(g);
        }
        crate::marks::squares_frame::game_part(1, &mut g)?;
        g.eax = x87::mul_to_int(game_read(RATE_RVA), game_read(SHARE_RVA)) as u32;
        if g.eax == 0 {
            return Ok(g);
        }
        crate::marks::squares_frame::game_part(2, &mut g)?;
        g.esi = game_read(SETTINGS_RVA);
        if ((g.esi + 0x2744) as *const u32).read_unaligned() == 0 {
            return Ok(g);
        }
        if game_read::<u32>(RUNNING_RVA) == 0 {
            return Ok(g);
        }
        g = squares_step(&g)?;
    }
    Ok(g)
}

/// In the game: pong's and the squares' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::{run_pong, run_squares};
    unsafe {
        *r = match place {
            1 => run_pong::run_pong(r)?,
            _ => run_squares::run_squares(r, &mut run_squares::game_part)?,
        };
    }
    Ok(())
}
