//! `race::set_up_car_points` (0x426B67): a car's collision points into its block.
//!
//! In:  esi = the car's slot, edi = its block
//! Out: eax the last moved z, ebx the slot's first point + the count, ecx 3, edx the
//!      count (with no points ecx 0); the rest preserved
//!
//! See `race/objects.rs`. One caller, `race::set_object_kinds` (inside `pusha`). Refuses
//! more than 0x40 points, and one (the original returns through its saved edi). Shadow mode runs the original alone (it logs). Self-test:
//! a copy of the original whose five variables are the test's and whose log calls go to
//! stand-ins (the sorts stay on their hooks), so that it runs paused and the frames drawn
//! meanwhile are not touched (the game's `g_objectPoints` changed for the test made a menu
//! object's geometry fly apart, 2026-10-16); points and a block of the test's.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::objects::*;
use crate::service::harness::{Region, capture};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_up_car_points", rva(0x426B67), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { set_up_car_points(r, &CarPointVars::game(), &mut game_part) } {
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_up_car_points(r, &CarPointVars::game(), &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

/// The original's calls a copy sends to stand-ins: the log's, by place.
const LOGS: &[(usize, u32)] = &[(0x401427, 4), (0x401472, 5), (0x401476, 14)];
/// Offsets of its calls: the sorts' (left on their hooks; the ones inside the sorts'
/// bodies, which the copy carries along, are never run) and the log's.
const CALLS: &[usize] = &[
    0x74, 0xFA, 0x107, 0x130, 0x1B6, 0x1C3, 0x1EC, 0x26D, 0x27A, 0x2A4, 0x325, 0x332, 0x444, 0x44E,
    0x45B, 0x467,
];

/// The Rust side in the test: the sorts run, the log noted as the stand-ins note it.
fn test_part(place: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if place < 4 {
        game_part(place, r)
    } else {
        crate::race::level_init::note(place, r);
        Ok(())
    }
}

const SLOT: u32 = 0x50;

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    use crate::race::level_init::take_called;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The test's variables, so that the frames the game draws meanwhile are not touched.
    let mut counts = vec![0u32; SLOT as usize + 1];
    let mut firsts = vec![0u32; SLOT as usize + 1];
    let mut words = [0u32; 3]; // the points' pointer, the count, the x87's way in
    let mut block = vec![0u8; 0x1000];
    let mut points = vec![0u32; 0x50 * 3];
    let v = CarPointVars {
        counts: counts.as_mut_ptr() as u32,
        firsts: firsts.as_mut_ptr() as u32,
        points: words.as_mut_ptr() as u32,
        car_points: words.as_mut_ptr() as u32 + 4,
        scratch: words.as_mut_ptr() as u32 + 8,
    };
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x426B67,
            len: 0x47A,
            entry: &[0x8B, 0x0C, 0xB5, 0x9C, 0x23, 0x63, 0x00],
            calls: CALLS,
            jumps_out: &[],
        },
        &|c| {
            LOGS.iter()
                .find(|&&(k, _)| rva(k) == c)
                .map(|&(_, p)| crate::install::stand_ins::stand_in(p))
        },
        &[
            (0x63239C, v.counts),
            (0x631F9C, v.firsts),
            (0x5C6FA4, v.points),
            (0x6D31BC, v.car_points),
            (0x62400C, v.scratch),
        ],
        33,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let b = block.as_mut_ptr() as u32;
    let p = points.as_mut_ptr() as u32;
    let compared: [Region; 3] = [(b, 0x1000), (p, 0x50 * 12), (v.points, 12)];
    let mut s = 0x0BAD_F00Du32;
    for count in [0u32, 2, 3, 4, 9, 0x3D, 0x40] {
        let start: Vec<u32> = (0..0x50 * 3)
            .map(|_| {
                s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((s >> 4) % 0x40_0000).wrapping_sub(0x20_0000)
            })
            .collect();
        let prepare = || unsafe {
            for (n, byte) in std::slice::from_raw_parts_mut(b as *mut u8, 0x1000)
                .iter_mut()
                .enumerate()
            {
                *byte = n as u8 ^ 0x5A;
            }
            std::ptr::copy_nonoverlapping(start.as_ptr(), p as *mut u32, start.len());
            ((v.counts + 4 * SLOT) as *mut u32).write_volatile(count);
            ((v.firsts + 4 * SLOT) as *mut u32).write_volatile(3);
            (v.points as *mut [u32; 3]).write_volatile([p, 0x77, 0xAAAA_AAAA]);
        };
        let before = Regs {
            esi: SLOT,
            edi: b,
            ..Regs::SENTINEL
        };
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        prepare();
        take_called();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&compared));
        prepare();
        let mine = unsafe { set_up_car_points(&before, &v, &mut test_part) };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&compared),
        );
        t.check(theirs == mine, || {
            let at = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "{count} points: original {:x?} {:x?}; rust {:x?} {:x?}; first difference at {at:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    drop((counts, firsts, words, block, points));
    t.finish()
}
