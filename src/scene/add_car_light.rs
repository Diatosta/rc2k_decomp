//! `scene::add_car_light` (0x44F29F): what one car's lamps add to the light on an
//! object.
//!
//! In:  esi = the car's block
//! Out: nothing changed if the car's lamps are off; else the original leaves eax, ebx,
//!      ecx and edx changed; both its callers keep every register, and the Rust
//!      version changes none
//!
//! Nothing unless the block's dword +0x84 is set. The place being lit
//! (`dword_636588` ..) less the car's (+0x134 ..), through the car's matrix (+0x9CC;
//! `scene::rotate_point`), as whole numbers in `dword_6365B8` ..: the place as the
//! car sees it, x across, y up, z ahead. Then three amounts, each held to 0 .. 255:
//! - 0x100 less a quarter of ((z - 0x200000) squared + (2x) squared) times
//!   `flt_5C5DA0` twice: a cone ahead of the car;
//! - one the original works out and then overwrites (((z - 0x100000) squared + x
//!   squared) times `flt_5C5DA0`, whose rounding is left in `dword_62400C`);
//! - (0x100000 less y without its sign) over 0x1000: how near the car's height.
//!
//! The third times twice the first over 256, times the block's dword +0xABC over 256,
//! held to 255, picks a colour from the table `dword_635D18`, which is added to
//! `g_lightColour` byte by byte without overflow. x87 with the original's
//! instructions.
//! Callers: `sub_44F1FE`, twice.
//! Refuses when there is no block.
//! Shadow mode: the Rust version, its writes put back, the original, compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};
use crate::scene::add_point_lights::{LIGHT_COLOUR_RVA, LIT_PLACE_RVA, MMX_RVA};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::dword;
use crate::scene::vertex_light::plus;

// ---- game data ----

const SEEN_RVA: usize = rva(0x6365B8); // dword_6365B8, dword_6365BC, dword_6365C0
const FACTOR_RVA: usize = rva(0x5C5DA0); // flt_5C5DA0
const TABLE_RVA: usize = rva(0x635D18); // dword_635D18
/// In a car's block.
const LAMPS_ON: usize = 0x84;
const PLACE: usize = 0x134;
const MATRIX: usize = 0x9CC;
const STRENGTH: usize = 0xABC;

// ---- Rust version ----

/// Three whole numbers through a matrix and back to whole numbers: `fild` three times,
/// `scene::rotate_point`, `fistp` three times.
unsafe fn turned(from: [u32; 3], matrix: *const u32) -> [u32; 3] {
    let mut w = [from[0], from[1], from[2], 0, 0, 0];
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{w} + 8]",
            "fild dword ptr [{w} + 4]",
            "fild dword ptr [{w}]",
            "fld dword ptr [{m}]",
            "fmul st(0), st(1)",
            "fld dword ptr [{m} + 0x10]",
            "fmul st(0), st(3)",
            "fld dword ptr [{m} + 0x20]",
            "fmul st(0), st(5)",
            "faddp st(1), st(0)",
            "faddp st(1), st(0)",
            "fadd dword ptr [{m} + 0x30]",
            "fxch st(3)",
            "fld dword ptr [{m} + 4]",
            "fmul st(0), st(2)",
            "fld dword ptr [{m} + 0x14]",
            "fmul st(0), st(4)",
            "fld dword ptr [{m} + 0x24]",
            "fmul st(0), st(3)",
            "faddp st(1), st(0)",
            "faddp st(1), st(0)",
            "fadd dword ptr [{m} + 0x34]",
            "fxch st(3)",
            "fld dword ptr [{m} + 8]",
            "fmulp st(3), st(0)",
            "fld dword ptr [{m} + 0x18]",
            "fmulp st(1), st(0)",
            "fld dword ptr [{m} + 0x28]",
            "fmulp st(2), st(0)",
            "faddp st(1), st(0)",
            "faddp st(1), st(0)",
            "fadd dword ptr [{m} + 0x38]",
            "fxch st(2)",
            "fistp dword ptr [{w} + 12]",
            "fistp dword ptr [{w} + 16]",
            "fistp dword ptr [{w} + 20]",
            w = in(reg) w.as_mut_ptr(),
            m = in(reg) matrix,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    [w[3], w[4], w[5]]
}

/// `fild a; fmul st, st; fild b; fmul st, st; faddp; fmul k` (once or twice) `; fistp`.
fn squares(a: u32, b: u32, factor: u32, twice: bool) -> u32 {
    let (v, mut out) = ([a, b, factor], 0u32);
    unsafe {
        if twice {
            core::arch::asm!(
                "fild dword ptr [{v}]",
                ".byte 0xD8, 0xC8", // fmul st, st
                "fild dword ptr [{v} + 4]",
                ".byte 0xD8, 0xC8",
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fmul dword ptr [{v} + 8]",
                "fmul dword ptr [{v} + 8]",
                "fistp dword ptr [{o}]",
                v = in(reg) v.as_ptr(),
                o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        } else {
            core::arch::asm!(
                "fild dword ptr [{v}]",
                ".byte 0xD8, 0xC8",
                "fild dword ptr [{v} + 4]",
                ".byte 0xD8, 0xC8",
                ".byte 0xDE, 0xC1",
                "fmul dword ptr [{v} + 8]",
                "fistp dword ptr [{o}]",
                v = in(reg) v.as_ptr(),
                o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }
    out
}

/// `cmp eax, 0; jns; eax = 0` and `cmp eax, 0FFh; js; eax = 0FFh`.
fn held(value: i32) -> u32 {
    let value = value.max(0);
    if value.wrapping_sub(0xFF) < 0 {
        value as u32
    } else {
        0xFF
    }
}

pub unsafe fn add_car_light(car: usize) -> Result<(), BadInput> {
    unsafe {
        if car == 0 {
            return Err(BadInput("no car"));
        }
        if dword(car + LAMPS_ON) == 0 {
            return Ok(());
        }
        let place: [u32; 3] = game_read(LIT_PLACE_RVA);
        let from = [0, 1, 2].map(|k| place[k].wrapping_sub(dword(car + PLACE + k * 4)));
        let seen = turned(from, (car + MATRIX) as *const u32);
        game_write(SEEN_RVA, seen);
        let [x, y, z] = seen;
        let factor: u32 = game_read(FACTOR_RVA);
        let ahead = squares(z.wrapping_sub(0x20_0000), x.wrapping_add(x), factor, true);
        let cone = held(0x100i32.wrapping_sub((ahead >> 2) as i32));
        // Worked out and not used; its rounding is what `dword_62400C` is left with.
        game_write(
            CHECKED_RVA,
            squares(z.wrapping_sub(0x10_0000), x, factor, false),
        );
        let height = if (y as i32) < 0 { y.wrapping_neg() } else { y };
        let level = held((0x10_0000u32.wrapping_sub(height) as i32) >> 12);
        let amount = (level.wrapping_mul(cone * 2) >> 8).wrapping_mul(dword(car + STRENGTH)) >> 8;
        let colour: u32 = game_read(TABLE_RVA + held(amount as i32) as usize * 4);
        let mmx = game_read::<u32>(MMX_RVA) != 0;
        game_write(
            LIGHT_COLOUR_RVA,
            plus(colour, game_read(LIGHT_COLOUR_RVA), mmx),
        );
        Ok(())
    }
}
