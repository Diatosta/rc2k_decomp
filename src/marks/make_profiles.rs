//! `marks::make_profiles` (0x4C3B01): `init_marks`' step 3 (through `sub_4C4E1C`), the
//! rings, the profile tables once, the corners from the course, the links and the
//! settings blocks.
//!
//! In:  nothing
//! Out: as `marks::reset_settings_blocks` leaves them; edi as `marks::scale_corners`,
//!      edx kept
//!
//! See `marks/profiles.rs`. One caller, `sub_4C4E1C`. Refuses a mark neither 0 nor
//! `loc_4C3EA6` (the original jumps to it). Shadow mode runs the original alone (it takes
//! memory). Self-test: a redirected copy whose two calls go to stand-ins, the course's
//! corners and the tables' parameters of the test's (the game's own and others), the
//! mark 0 and set (to the copy's own place), the game's variables put back afterwards.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::marks::blocks::{game_part, give, test_give};
use crate::marks::profiles::*;
use crate::race::mud::{copy, run_both};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_profiles", rva(0x4C3B01), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let resume = game_addr(RESUME_RVA) as u32;
    match unsafe { make_profiles(r, resume, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The tables' first four dwords as RAL.EXE holds them, in table order (0x8E506D ..).
const PARAMETERS: [[u32; 4]; 13] = [
    [0, 0, 0, 0],
    [0xC000_0000, 0, 0x469C_4000, 0x4843_5000],
    [0, 0x3F00_0000, 0xC87D_E800, 0],
    [0xFF, 0, 0, 0],
    [0x7F, 0, 0, 0],
    [0, 0, 0x466A_6000, 0x3BC9_0FDB],
    [0, 0, 0x4821_2200, 0x3BC9_0FDB],
    [0, 0, 0x4851_F600, 0x3BC9_0FDB],
    [0, 0, 0x4881_6500, 0x3BC9_0FDB],
    [0, 0, 0x4899_CF00, 0x3BC9_0FDB],
    [0, 0, 0x48B2_3900, 0x3BC9_0FDB],
    [0, 0, 0x48CA_A300, 0x3BC9_0FDB],
    [0, 0, 0x48E3_0D00, 0x3BC9_0FDB],
];

/// Other parameters: ramps of other ends, sines of other angles and steps, curves of
/// other spans (floats as bits: 0x3F80_0000 1.0, 0x4120_0000 10.0, 0x3E80_0000 0.25).
const OTHERS: [[u32; 4]; 13] = [
    [0x10, 0x200, 0, 0],
    [0x3F80_0000, 0x4120_0000, 0x4120_0000, 0x4700_0000],
    [0xC120_0000, 0x3E80_0000, 0x447A_0000, 0x4000_0000],
    [-0x40i32 as u32, 0x7FFF, 0, 0],
    [0x1234, -0x1234i32 as u32, 0, 0],
    [0x3F80_0000, 0x4120_0000, 0x466A_6000, 0x3E80_0000],
    [0x4049_0FDB, 0, 0x4821_2200, 0x3BC9_0FDB],
    [0, 0x447A_0000, 0x4851_F600, 0x3C00_0000],
    [0xBF80_0000, 0, 0x4881_6500, 0x3BC9_0FDB],
    [0, 0, 0xC899_CF00, 0x3BC9_0FDB],
    [0x4120_0000, 0, 0x48B2_3900, 0xBBC9_0FDB],
    [0, 0, 0x48CA_A300, 0x3F80_0000],
    [0, 0x3F80_0000, 0x48E3_0D00, 0x3BC9_0FDB],
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry = [0xC7, 0x05, 0x61, 0xD8, 0x8D, 0x00, 0x00, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x4C3B01,
        0x3E2,
        &entry,
        &[0x3D7, 0x3DC],
        &[(0x4C5398, 2), (0x4C9A6D, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(give);
    let regions = profile_regions();
    let inputs = [at(COURSE_INFO_RVA)];
    let (saved, saved_inputs) = (capture(&regions), capture(&inputs));
    let mut course = vec![0u8; (COURSE_CORNERS as usize) + CORNERS_LEN];
    for (n, b) in course[COURSE_CORNERS as usize..].iter_mut().enumerate() {
        *b = (n * 7 + n / 0x80) as u8;
    }
    unsafe { game_write(COURSE_INFO_RVA, course.as_ptr() as u32) };
    let in_copy = copy.addr() as u32 + (0x4C3EA6 - 0x4C3B01);
    let in_game = game_addr(RESUME_RVA) as u32;
    for (what, parameters, done, resume) in [
        ("the game's parameters", PARAMETERS, 0, in_game),
        ("others", OTHERS, 0, in_game),
        ("done before", PARAMETERS, in_copy, in_copy),
    ] {
        run_both(
            &mut t,
            what,
            &copy,
            &regions,
            &|| unsafe {
                for &(a, n) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0xAA, n);
                }
                for (k, p) in parameters.iter().enumerate() {
                    game_write(TABLES_RVA + 0x800 * k, *p);
                }
                game_write(DONE_RVA, done);
            },
            &|r| unsafe { make_profiles(r, resume, &mut test_give) },
        );
    }
    unsafe { game_write(DONE_RVA, 0x1234u32) };
    t.check(
        unsafe { make_profiles(&Regs::SENTINEL, in_game, &mut test_give) }.is_err(),
        || "a mark of 0x1234: the Rust version went on".into(),
    );
    restore(&inputs, &saved_inputs);
    restore(&regions, &saved);
    drop(course);
    t.finish()
}
