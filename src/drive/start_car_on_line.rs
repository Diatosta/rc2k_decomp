//! `drive::start_car_on_line` (0x437613): the work car put at a start on the line:
//! `dword_71AA80` = `dword_605034[dword_6051A0]` (the start's number's entry); the grades'
//! mode `dword_71A234` 2; the checkpoint (`dword_70F484`, the lead car's, + ebx) into
//! `dword_71A070`, `6C`, `74`, `dword_6365E4` and `dword_6365EC`; a share `flt_6051AC` /
//! `flt_71A650` = lo + ((m - 1) - n) / (m - 1) (hi - lo) (n `dword_6051A0`, m
//! `dword_6051B4`, lo / hi `flt_6051A4` / `flt_6051A8`); `race::place_car_on_line`,
//! `race::reset_car_motion`; then the motion: 0, 0 and the speed `flt_6051B0` turned by
//! `g_crashMatrix` (`scene::rotate_point`) into 0x71A170 .. 0x71A178 (`g_carSpeed` the
//! speed). By the look of it a computer car placed at the start of a run (a reading).
//!
//! In:  ebx the checkpoints on from the lead's
//! Out: as `race::reset_car_motion` leaves them; no flags read (`drive::store_car` next)
//!
//! Caller: `drive::start_cars_behind` (0x43703E; four calls).
//! Refuses (before anything) a start number (`dword_6051A0`) above 0x400 (not worked
//! out), and what the two calls refuse.
//! Self-test: a copy whose two calls go to stream A's stand-ins (`race::car_starts`),
//! against the Rust version with the same, the calls and their registers compared, the
//! work car and the start's variables compared (the share exact at double precision,
//! within `FLOAT` at extended); all put back.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::x87;
use crate::race::car_starts::{Part, copy_of, game_part, take_called, test_part};
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::service::floats::{Agreement, FLOAT, Floats, at_test_precisions, compare_memory};
use crate::service::harness::{Region, capture, restore};
use rc2k_core::drive::start_places;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::start_car_on_line", rva(0x437613), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

/// The Rust core, the two calls through `part` (places 6 and 9).
// Converted: `rc2k_core::drive::start_places` (the share in plain floats; the motion's
// turn stays x87).
pub unsafe fn start_car_on_line(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        let start = g(0x6051A0);
        start_places::check_start(start)?;
        let mut r = *entry;
        p(0x71AA80, g(0x605034 + 4 * start));
        p(0x71A234, start_places::STARTED_MODE);
        r.eax = start_places::start_checkpoint(g(0x70F484), r.ebx);
        for v in [0x71A070, 0x71A06C, 0x71A074, 0x6365E4, 0x6365EC] {
            p(v, r.eax);
        }
        let f = |x: u32| f32::from_bits(g(x));
        let share = start_places::start_share(
            start as i32,
            g(0x6051B4) as i32,
            f(0x5C5D38),
            f(0x6051A8),
            f(0x6051A4),
        )
        .to_bits();
        p(0x6051AC, share);
        p(0x71A650, share);
        part(6, &mut r)?;
        part(9, &mut r)?;
        p(0x71A17C, 0);
        p(0x71A180, 0);
        p(0x71A184, x87::loaded(g(0x6051B0)));
        let m = game_addr(rva(0x71A08C)) as u32;
        let out = rotate_point(
            m,
            [
                extended(g(0x71A17C)),
                extended(g(0x71A180)),
                extended(g(0x71A184)),
            ],
        );
        p(0x71A170, rounded(out[0]));
        p(0x71A174, rounded(out[1]));
        p(0x71A178, rounded(out[2]));
        Ok(r)
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // `reset_car_motion` runs its original alone in shadow mode (its calls into the
    // game): this one does the same.
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { start_car_on_line(r, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// What a case sets and the function writes beyond stream A's regions.
fn extra() -> Vec<Region> {
    [
        0x6051A0u32,
        0x6051A4,
        0x6051A8,
        0x6051AC,
        0x6051B0,
        0x6051B4,
    ]
    .iter()
    .map(|&v| (game_addr(rva(v as usize)) as u32, 4))
    .collect()
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x437613,
        0xD0,
        &[0x60, 0x8B, 0x1D, 0xA0, 0x51, 0x60, 0x00],
        &[0x74, 0x79, 0xB7],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = crate::race::car_starts::regions();
    all.extend(extra());
    all.push((game_addr(rva(0x6365E4)) as u32, 0xC));
    let saved = capture(&all);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let share = |x: u32| ((game_addr(rva(x as usize)) as u32, 4), FLOAT);
    let tolerant = Floats {
        memory: &[share(0x6051AC), share(0x71A650)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for seed in 1..=8u32 {
            let fill = || {
                crate::race::car_starts::prepare(
                    &crate::race::car_starts::Case {
                        mode: 2,
                        no_place: 0,
                        side: 0,
                        cars_2: 1,
                        current: seed % 4,
                        cars: 4,
                    },
                    seed.wrapping_mul(0x2026_1017),
                );
                p(0x6051A0, seed % 6);
                p(0x6051B4, 2 + seed % 5);
                p(0x6051A4, (seed as f32 * 0.25).to_bits());
                p(0x6051A8, (seed as f32 * 1.5 + 2.0).to_bits());
                p(0x6051B0, (seed as f32 * 3.0).to_bits());
                // A turning matrix the rotation can use.
                for (k, v) in [
                    1.0f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 5.0, 6.0, 7.0,
                    0.0,
                ]
                .into_iter()
                .enumerate()
                {
                    p(0x71A08C + 4 * k as u32, v.to_bits());
                }
            };
            let before = Regs {
                ebx: seed * 3,
                ..Regs::SENTINEL
            };
            fill();
            take_called();
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = (Some(regs(&ro)), take_called(), capture(&all));
            fill();
            let mine = unsafe { start_car_on_line(&before, &mut test_part) };
            let mine = (mine.as_ref().ok().map(regs), take_called(), capture(&all));
            let memory = compare_memory(&all, &theirs.2, &mine.2, floats);
            t.check(
            (theirs.0, &theirs.1) == (mine.0, &mine.1) && memory != Agreement::Differ,
            || {
                let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
                format!(
                    "control word {word:#x}, seed {seed}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
                    theirs.0, theirs.1, mine.0, mine.1
                )
            },
        );
        }
    });
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
