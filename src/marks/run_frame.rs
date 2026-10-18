//! `marks::run_frame` (0x4D1549): the marks' frame drawn: `marks::swap_mask`,
//! `marks::tick_clock`; render states 0x15 = 4, 0x1C = 0, 0x13 = 5, 0x14 = 6, 0x1B = 1,
//! 3 = 1; `marks::run_rain`, `marks::run_snow`; `marks::draw_frame` (through the stub at
//! 0x4C4E28); `marks::ring_frame`; render state 0x14 = 2; `marks::flare_frame`; render
//! state 3 = 1; `marks::draw_mark_slots`, `marks::run_particles`; `marks::stage_record`,
//! `marks::draw_glow`, `scene::depth_buffer_on`, `scene::depth_test_on`. Three shares of
//! the frame's time (`rdtsc` over `dword_765114`, times `flt_93C8C4` / `C8` / `CC`) into
//! `flt_765134` (the draw), `flt_76511C` (the ring) and `flt_765120` (the whole).
//!
//! In:  nothing
//! Out: the registers as `depth_test_on` leaves them
//!
//! Caller: the race's frame (0x456464 ..). Its calls through `part` (places 1 .. 14, in the
//! order above); in the game their Rust versions. Shadow mode runs the original alone (it
//! draws). It begins with a call: private copy (its 21 calls relocated), manual patch.
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, the
//! calls and their registers compared (the three times are not: `rdtsc`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::math::x87;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::run_frame", rva(0x4D1549), thunk, &ORIGINAL)
    .private_copy(0x15A)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLS: [usize; 23] = [
    0x0, 0x5, 0x20, 0x2F, 0x3E, 0x4D, 0x5C, 0x6B, 0x70, 0x75, 0xAC, 0xB1, 0xC2, 0xF7, 0xFC, 0x10B,
    0x110, 0x115, 0x140, 0x145, 0x14A, 0x14F, 0x154,
];
const FRAME_CYCLES_RVA: usize = rva(0x765114); // dword_765114
/// (where the share goes, its factor): the draw, the ring, the whole.
const SHARES: [(usize, usize); 3] = [
    (rva(0x765134), rva(0x93C8C4)),
    (rva(0x76511C), rva(0x93C8C8)),
    (rva(0x765120), rva(0x93C8CC)),
];
const STATES: [(u32, u32); 6] = [
    (4, 0x15),
    (0, 0x1C),
    (5, 0x13),
    (6, 0x14),
    (1, 0x1B),
    (1, 3),
];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn rdtsc() -> u32 {
    unsafe { core::arch::x86::_rdtsc() as u32 }
}

unsafe fn share(at: usize, start: u32) {
    unsafe {
        let (to, k) = SHARES[at];
        let took = rdtsc().wrapping_sub(start) as i32;
        game_write(
            to,
            x87::int_over_int_times(took, game_read(FRAME_CYCLES_RVA), game_read(k)),
        );
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        part(1, &mut g)?;
        part(2, &mut g)?;
        let whole = rdtsc();
        for (value, state) in STATES {
            [g.eax, g.ebx] = [value, state];
            part(3, &mut g)?;
        }
        part(4, &mut g)?;
        part(5, &mut g)?;
        let start = rdtsc();
        share(0, start);
        part(6, &mut g)?;
        let start = rdtsc();
        part(7, &mut g)?;
        share(1, start);
        [g.eax, g.ebx] = [2, 0x14];
        part(3, &mut g)?;
        part(8, &mut g)?;
        [g.eax, g.ebx] = [1, 3];
        part(3, &mut g)?;
        part(9, &mut g)?;
        part(10, &mut g)?;
        share(2, whole);
        for place in 11..=14 {
            part(place, &mut g)?;
        }
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::*;
    unsafe {
        let left = match place {
            1 => swap_mask::swap_mask(r, &mut swap_mask::game_part)?,
            2 => setup::tick_clock(r),
            4 => run_rain::run_rain(r, &mut |p, g, v| run_rain::game_part(p, g, v))?,
            5 => run_snow::run_snow(r, &mut run_snow::game_part)?,
            6 => draw_frame::draw_frame(r, &mut draw_frame::game_part)?,
            7 => ring_frame::ring_frame(r, &mut ring_frame::game_part)?,
            8 => flare_frame::flare_frame(r, &mut flare_frame::game_part)?,
            9 => draw_mark_slots::draw_mark_slots(r, &mut draw_mark_slots::game_part)?,
            10 => {
                run_particles::run_particles(r, &mut |p, g, v| run_particles::game_part(p, g, v))?
            }
            11 => stage_record::stage_record(r)?,
            12 => draw_glow::draw_glow(r, &mut |p, g, v| draw_glow::game_part(p, g, v))?,
            _ => {
                match place {
                    3 => {
                        let (value, state) = (r.eax, r.ebx);
                        crate::scene::set_render_state::set_render_state(r, value, state);
                    }
                    13 => crate::scene::depth_buffer_on::depth_buffer(r, true),
                    _ => crate::scene::depth_test_on::depth_test_on(r),
                }
                return Ok(());
            }
        };
        *r = Regs { esp: r.esp, ..left };
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call marks::swap_mask
    let copy = match copy(
        0x4D1549,
        0x15A,
        &[0xE8, 0xA7, 0x80, 0xFF, 0xFF],
        &CALLS,
        &[
            (0x4C95F5, 1),
            (0x4D2BDC, 2),
            (0x44EB72, 3),
            (0x4C16F9, 4),
            (0x4C25F6, 5),
            (0x4C4E28, 6),
            (0x4CB5B9, 7),
            (0x4C4EA2, 8),
            (0x4C969A, 9),
            (0x4CFB49, 10),
            (0x4D2DE0, 11),
            (0x4D2DED, 12),
            (0x44E41F, 13),
            (0x44E3E1, 14),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let times: Vec<_> = SHARES
        .iter()
        .map(|&(r, _)| crate::service::harness::at(r))
        .collect();
    let saved = crate::service::harness::capture(&times);
    run_both(&mut t, "frame", &copy, &[], &|| {}, &|r| unsafe {
        run_frame(r, &mut test_part)
    });
    crate::service::harness::restore(&times, &saved);
    t.finish()
}
