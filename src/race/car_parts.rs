//! The cars' parts made ready to draw (`sub_460D5C` -> `sub_460E7C`, part R3, IDA
//! 2026-10-18): each car's marks' switches (+0xAB4 .. +0xAD0) from its state, and its four
//! parts' matrices (the wheels: turned about their axles' places) into `flt_6417E0` .., then
//! kept in the block (+0x740 .. +0x8FF) for `race::draw_car`.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::scene::matrix::{multiply_matrices, set_place, set_turn_x, set_turn_y};

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

fn ga(a: u32) -> usize {
    game_addr(rva(a as usize))
}

const PLACED: u32 = 0x5EA04C; // g_placedMatrix
const TURN_X: u32 = 0x5E9F8C; // g_turnXMatrix
const TURN_Y: u32 = 0x5E9FCC; // g_turnYMatrix
const TURNED: u32 = 0x5EA10C; // g_turnedMatrix
const LIT: u32 = 0x5EA14C; // g_litMatrix

/// The model's axle places (`dword_5C9098` .. of its 0x134-byte record) x 0x10000 / 1000 as
/// `imul` / `idiv`: none where the `idiv` would fault.
fn per_thousand(v: i64) -> Option<u32> {
    let q = v * 0x1_0000 / 1000;
    (i32::MIN as i64..=i32::MAX as i64)
        .contains(&q)
        .then_some(q as u32)
}

/// The five axle values: (front x, rear x, height, front z, rear z) as `sub_460E7C` keeps
/// them in `dword_608C50`, `dword_608C54`, `dword_608C58`, `dword_608C5C`, `dword_608C48`.
fn axles(model: u32) -> Option<[u32; 5]> {
    let rec = |o: u32| k(0x5C9098 + model.wrapping_mul(0x134) + o) as i32 as i64;
    let lift = k(0x613FC4);
    Some([
        per_thousand((rec(8) as i32).wrapping_add(0xC3) as i64)?,
        per_thousand(rec(0xC))?.wrapping_sub(lift),
        per_thousand(rec(0))?,
        per_thousand((rec(4) as i32).wrapping_neg() as i64)?,
        per_thousand(rec(0x10))?.wrapping_sub(lift),
    ])
}

unsafe fn byte(p: u32) -> u8 {
    unsafe { (p as *const u8).read() }
}

unsafe fn set_byte(p: u32, v: u8) {
    unsafe { (p as *mut u8).write(v) }
}

/// `race::car_part_matrices` (0x460E7C): car esi's marks' switches and its parts' matrices
/// (see the header): +0x718 the size from +0x1D8 (not below `flt_6140F8`); the switch bytes
/// from +0x84, `dword_62DAD0` / `dword_62DACC`, +0x63C, +0x684, +0x1D4, each cut by its
/// broken part (+0x410 .. +0x424; the original cuts them twice); +0xABC the marks' count x
/// 0x20 (0 when +0x310 is not below `flt_5C5D14` or +0x84 is 0); +0xAAC the steering; the
/// axles from the model's record; the light and place; the car's turn (+0xB4) and place
/// into `flt_5EA08C` / `flt_641760`; each wheel placed at minus its axle, turned about x by
/// its spin (+0x1F8 front, +0x1FC rear) and about y by the steering (front only), placed at
/// its axle with its height (+0x910 ..), into `flt_6417E0` .. `flt_6418A0`. Refuses (before
/// anything) axle values whose scaling would fault.
pub unsafe fn car_part_matrices(car: u32) -> Result<(), Refused> {
    use crate::math::x87::loaded;
    use crate::math::x87_load::below_or_unordered;
    unsafe {
        let f = |o: u32| ((car + o) as *const u32).read_unaligned();
        let set = |o: u32, v: u32| ((car + o) as *mut u32).write_unaligned(v);
        let Some(ax) = axles(f(0x24)) else {
            return Err(BadInput("a car model whose axle scaling would fault").into());
        };
        set(0x718, loaded(f(0x1D8)));
        if below_or_unordered(f(0x718), k(0x6140F8)) {
            set(0x718, loaded(k(0x6140F8)));
        }
        let on = f(0x84);
        set(0xAB4, k(0x62DAD0) & on);
        let b = (k(0x62DACC) & on) as u8;
        set_byte(car + 0xAC8, b);
        set_byte(car + 0xAC9, b);
        for o in 0xAC0..=0xAC3 {
            set_byte(car + o, on as u8);
        }
        let b = (on & k(0x62DAD0)) as u8;
        for o in 0xACA..=0xACF {
            set_byte(car + o, b);
        }
        let b = u8::from(f(0x63C) != 0);
        for o in [0xAC4u32, 0xAC5, 0xAD0] {
            set_byte(car + o, b);
        }
        set(0xAB8, u32::from(matches!(f(0x684), 2 | 3)));
        let b = u8::from(f(0x1D4) == u32::MAX);
        set_byte(car + 0xAC6, b);
        set_byte(car + 0xAC7, b);
        let cut = |o: u32, by: u32| set_byte(car + o, byte(car + o) & (f(by) ^ 1) as u8);
        cut(0xAC0, 0x410);
        cut(0xAC1, 0x414);
        for _ in 0..2 {
            for o in 0xACA..=0xACF {
                cut(o, 0x420);
            }
            for o in [0xAC8u32, 0xAC2, 0xAC4, 0xAC6] {
                cut(o, 0x418);
            }
            for o in [0xAC9u32, 0xAC3, 0xAC5, 0xAC7] {
                cut(o, 0x41C);
            }
            cut(0xAD0, 0x424);
        }
        let mut count = byte(car + 0xAC0).wrapping_add(byte(car + 0xAC1));
        count = count.wrapping_add(count);
        count = count.wrapping_add(count);
        if f(0x298) & (1 << 9) != 0 {
            for o in 0xACA..=0xACF {
                count = count.wrapping_add(byte(car + o));
            }
        }
        set(0xABC, count as u32 * 0x20);
        if !below_or_unordered(f(0x310), k(0x5C5D14)) {
            set(0xABC, 0);
        }
        if on == 0 {
            set(0xABC, 0);
        }
        set(0xAAC, f(0x650) >> 16);
        let [front, rear_lift, height, front_z, _] = ax;
        w(0x608C50, front);
        w(0x608C4C, front.wrapping_neg());
        w(0x608C54, rear_lift);
        w(0x608C58, height);
        w(0x608C5C, front_z);
        w(0x608C48, ax[4]);
        w(0x635038, 0x10);
        w(0x635050, f(0xAAC));
        w(0x635048, f(0x1F8));
        w(0x63504C, f(0x1FC));
        for (i, o) in [0x134u32, 0x138, 0x13C].into_iter().enumerate() {
            w(0x63501C + 4 * i as u32, f(o));
        }
        std::ptr::copy((car + 0xB4) as *const u8, ga(0x5EA08C) as *mut u8, 0x40);
        for i in 0..4 {
            w(0x635074 + 4 * i, f(0x910 + 4 * i));
        }
        set_place([k(0x63501C), k(0x635020), k(0x635024)]);
        std::ptr::copy(ga(PLACED) as *const u8, ga(0x641760) as *mut u8, 0x40);
        // (x, z, the x turn, the height, the matrix): the four wheels.
        for (n, (x, z, spin, lift, out)) in [
            (
                0x608C4Cu32,
                0x608C58u32,
                0x635048u32,
                0x635074u32,
                0x6417E0u32,
            ),
            (0x608C50, 0x608C58, 0x635048, 0x635078, 0x641820),
            (0x608C4C, 0x608C5C, 0x63504C, 0x63507C, 0x641860),
            (0x608C50, 0x608C5C, 0x63504C, 0x635080, 0x6418A0),
        ]
        .into_iter()
        .enumerate()
        {
            if n == 2 {
                w(0x635050, 0);
            }
            set_place([
                k(x).wrapping_neg(),
                k(0x608C54).wrapping_neg(),
                k(z).wrapping_neg(),
            ]);
            set_turn_x(k(spin));
            multiply_matrices(ga(PLACED), ga(TURN_X), ga(TURNED));
            set_turn_y(k(0x635050));
            multiply_matrices(ga(TURNED), ga(TURN_Y), ga(LIT));
            set_place([k(x), k(lift), k(z)]);
            multiply_matrices(ga(LIT), ga(PLACED), ga(out));
        }
        Ok(())
    }
}

/// `race::ready_car_parts` (0x460D5C): for each race car with its parts shown (+0x25C):
/// `car_part_matrices` (part 1, esi the block), then the six matrices kept in the block
/// (+0x740 .. +0x8BF) and the body's turn times the car's (`flt_5EA3CC`) at +0x8C0. Refuses
/// (before anything) no race cars (the original's loop would run away), more than eight, or
/// one without its block.
pub unsafe fn ready_car_parts(
    r: &mut crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    unsafe {
        let count = k(0x63DB90);
        if count == 0 || count > 8 || (0..count).any(|n| k(0x71B3B8 + 4 * n) == 0) {
            // With none the original's loop (its test at the end) runs on past the eight.
            return Err(BadInput("no race cars, more than eight, or one without its block").into());
        }
        // The loop's test is at its end: no cars still looks at car 0.
        let mut c = 0u32;
        loop {
            let car = k(0x71B3B8 + 4 * c);
            r.ecx = c;
            r.esi = car;
            if ((car + 0x25C) as *const u32).read_unaligned() != 0 {
                let mut p = *r;
                part(1, &mut p)?;
                for (i, from) in [
                    0x6417E0u32,
                    0x641820,
                    0x641860,
                    0x6418A0,
                    0x5EA08C,
                    0x641760,
                ]
                .into_iter()
                .enumerate()
                {
                    std::ptr::copy(
                        ga(from) as *const u8,
                        (car + 0x740 + 0x40 * i as u32) as *mut u8,
                        0x40,
                    );
                }
                multiply_matrices(ga(0x5EA08C), ga(0x641760), ga(0x5EA3CC));
                std::ptr::copy(ga(0x5EA3CC) as *const u8, (car + 0x8C0) as *mut u8, 0x40);
            }
            c += 1;
            if c == count {
                break;
            }
        }
        Ok(())
    }
}

/// `ready_car_parts`' part in the game.
pub fn parts_part(_: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    unsafe { car_part_matrices(r.esi) }
}
