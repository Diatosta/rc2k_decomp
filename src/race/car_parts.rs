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

/// The five axle values (`rc2k_core::race::car_parts::axles`) from the model's record
/// (`dword_5C9098` .. of its 0x134-byte record) and the lift.
fn axles(model: u32) -> Option<[u32; 5]> {
    let record = [0u32, 4, 8, 0xC, 0x10].map(|o| k(0x5C9098 + model.wrapping_mul(0x134) + o));
    rc2k_core::race::car_parts::axles(record, k(0x613FC4))
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
    // Converted: `rc2k_core::race::car_parts::{axles, mark_switches, marks_bytes,
    // steering}` (the size's and the speed's floats the x87's).
    use crate::math::x87::loaded;
    use crate::math::x87_load::below_or_unordered;
    use rc2k_core::race::car_parts::{
        FIRST_SWITCH, PartState, mark_switches, marks_bytes, steering,
    };
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
        let s = mark_switches(&PartState {
            on,
            marks_mask: k(0x62DAD0),
            lamps_mask: k(0x62DACC),
            inside: f(0x63C),
            mode: f(0x684),
            reversing: f(0x1D4),
            broken: [0x410, 0x414, 0x418, 0x41C, 0x420, 0x424].map(f),
            flags: f(0x298),
        });
        set(0xAB4, s.marks_on);
        set(0xAB8, s.mode_on);
        std::ptr::copy_nonoverlapping(
            s.bytes.as_ptr(),
            (car + FIRST_SWITCH) as *mut u8,
            s.bytes.len(),
        );
        let slow = below_or_unordered(f(0x310), k(0x5C5D14));
        set(0xABC, marks_bytes(s.count, slow, on));
        set(0xAAC, steering(f(0x650)));
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
/// `car_part_matrices` (esi the block), then the six matrices kept in the block
/// (+0x740 .. +0x8BF) and the body's turn times the car's (`flt_5EA3CC`) at +0x8C0. Refuses
/// (before anything) no race cars (the original's loop would run away), more than eight, or
/// one without its block.
pub unsafe fn ready_car_parts() -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::car_parts::{check_part_cars, parts_shown,
        // KEPT_MATRICES}`.
        use rc2k_core::race::car_parts::{
            BODY_AT, KEPT_AT, KEPT_MATRICES, check_part_cars, parts_shown,
        };
        let count = k(0x63DB90);
        check_part_cars(count, |n| k(0x71B3B8 + 4 * n))?;
        // The loop's test is at its end: no cars still looks at car 0.
        let mut c = 0u32;
        loop {
            let car = k(0x71B3B8 + 4 * c);
            if parts_shown(((car + 0x25C) as *const u32).read_unaligned()) {
                car_part_matrices(car)?;
                for (i, from) in [
                    0x6417E0u32,
                    0x641820,
                    0x641860,
                    0x6418A0,
                    0x5EA08C,
                    0x641760,
                ]
                .into_iter()
                .take(KEPT_MATRICES as usize)
                .enumerate()
                {
                    std::ptr::copy(
                        ga(from) as *const u8,
                        (car + KEPT_AT + 0x40 * i as u32) as *mut u8,
                        0x40,
                    );
                }
                multiply_matrices(ga(0x5EA08C), ga(0x641760), ga(0x5EA3CC));
                std::ptr::copy(ga(0x5EA3CC) as *const u8, (car + BODY_AT) as *mut u8, 0x40);
            }
            c += 1;
            if c == count {
                break;
            }
        }
        Ok(())
    }
}
