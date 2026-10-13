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

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::scene::add_point_lights::{LIGHT_COLOUR_RVA, LIT_PLACE_RVA, MMX_RVA};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::dword;
use crate::scene::vertex_light::plus;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::add_car_light", rva(0x44F29F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

// ---- hook ----

fn regions() -> [Region; 3] {
    [
        (game_addr(SEEN_RVA) as u32, 12),
        at(CHECKED_RVA),
        at(LIGHT_COLOUR_RVA),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let car = r.esi as usize;
    if !shadow() {
        if let Err(e) = unsafe { add_car_light(car) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let regions = regions();
    let before = capture(&regions);
    let mine = unsafe { add_car_light(car) }.map(|()| capture(&regions));
    restore(&regions, &before);
    unsafe { call_original(r) };
    let Ok(mine) = mine else {
        return;
    };
    let theirs = capture(&regions);
    if mine != theirs {
        HOOK.mismatch(|| format!("before {before:x?}: original {theirs:x?}; rust {mine:x?}"));
    }
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let inputs = [
        (game_addr(LIT_PLACE_RVA) as u32, 12),
        at(MMX_RVA),
        (game_addr(TABLE_RVA) as u32, 0x100 * 4),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let table: Vec<u32> = (0..0x100u32)
        .map(|n| n.wrapping_mul(0x0102_0304) ^ 0x00A5_5A0F)
        .collect();
    let f = |value: f32| value.to_bits();
    let matrices: [[f32; 16]; 3] = [
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        [
            0.36, 0.48, -0.8, 0.0, -0.8, 0.6, 0.0, 0.0, 0.48, 0.64, 0.6, 0.0, 1000.5, -70000.0,
            250000.25, 0.0,
        ],
        [
            0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
    ];
    let places: [[i32; 3]; 8] = [
        [0, 0, 0x20_0000],
        [0x1_0000, 0x2000, 0x1F_0000],
        [-0x4_0000, -0x8_0000, 0x18_0000],
        [0, 0x10_0000, 0x20_0000],
        [0x100, 0x100, 0x100],
        [0x7FFF_FFFF, i32::MIN, 0x12_3456],
        [-0x1_8000, 0x7_FFFF, 0x22_0000],
        [0, 0xF_F000, 0x20_0800],
    ];
    let mut cases = 0u32;
    for matrix in matrices {
        for place in places {
            for (on, strength) in [
                (1u32, 0x100u32),
                (0, 0x100),
                (7, 0x80),
                (1, 0x1_0000),
                (1, 0),
            ] {
                cases += 1;
                let mut car = vec![0u32; 0xAC0 / 4];
                car[LAMPS_ON / 4] = on;
                car[PLACE / 4..PLACE / 4 + 3].copy_from_slice(&[0x1000, 0xFFFF_F000, 0x800]);
                car[MATRIX / 4..MATRIX / 4 + 16].copy_from_slice(&matrix.map(f));
                car[STRENGTH / 4] = strength;
                let car_at = car.as_ptr() as usize;
                let run = |rust: bool| unsafe {
                    restore(&regions, &vec![0xAA; saved.0.len()]);
                    game_write(LIT_PLACE_RVA, place);
                    game_write(MMX_RVA, cases % 2);
                    game_write(
                        LIGHT_COLOUR_RVA,
                        [0u32, 0x8010_F0FE][(cases / 2 % 2) as usize],
                    );
                    std::ptr::copy_nonoverlapping(
                        table.as_ptr(),
                        game_addr(TABLE_RVA) as *mut u32,
                        table.len(),
                    );
                    let done = if rust {
                        add_car_light(car_at).is_ok()
                    } else {
                        let mut ro = Regs {
                            esi: car_at as u32,
                            ..Regs::default()
                        };
                        call_original(&mut ro);
                        true
                    };
                    (done, capture(&regions))
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine, || {
                    format!(
                        "case {cases}: place {place:x?}, on {on}, strength {strength:#x}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
