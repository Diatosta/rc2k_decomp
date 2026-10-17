//! `marks::run_shapes` (0x4BDF72): the mask shapes' frame: `marks::pick_shapes`,
//! `marks::turn_all_shapes`; then when `g_sceneClock` reaches `dword_844E30`, the next
//! time (the clock plus a random's low byte, `dword_89C994`) and whether the shapes move
//! (`dword_844E2C`: another random, turned right 16, plus the clock, `and`
//! `dword_844E34[settings +0x2748]`, not 0); while they move, the shape index
//! (`dword_84460C`) from a running count (`dword_82E38A` plus `g_marksFrameTicks` by
//! `flt_89C9A0`, or by `flt_89C99C` when the settings' +0x274C is 1 and the index >> 3 is
//! not 0), its bits 16 .. 23, and `pick_shapes` again.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the path's last step leaves them
//!
//! Caller: `sub_4C1148`. Refuses null settings, a +0x2748 of 9 or more when it is read
//! (`dword_844E34` holds 9), and its steps' refusals; on a refusal nothing stays written
//! (all it may write is put back). Self-test: the game's settings (skipped without them),
//! clocks, randoms and switches by lot; compared on all it may write.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::emitters::next_c5;
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::pick_shapes::pick_shapes;
use crate::marks::shape_points::turn_all_shapes;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::run_shapes", rva(0x4BDF72), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
const NEXT_RVA: usize = rva(0x844E30); // dword_844E30
const TIME_RANDOM_RVA: usize = rva(0x89C994); // dword_89C994
const MOVE_RANDOM_RVA: usize = rva(0x89C998); // dword_89C998
const MASKS_RVA: usize = rva(0x844E34); // dword_844E34: 9 masks
const MASKS: u32 = 9;
const MOVING_RVA: usize = rva(0x844E2C); // dword_844E2C
const INDEX_RVA: usize = rva(0x84460C); // dword_84460C
const COUNT_RVA: usize = rva(0x82E38A); // dword_82E38A
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const RATE_RVA: [usize; 2] = [rva(0x89C99C), rva(0x89C9A0)];

fn own() -> Vec<Region> {
    [
        NEXT_RVA,
        TIME_RANDOM_RVA,
        MOVE_RANDOM_RVA,
        MOVING_RVA,
        INDEX_RVA,
        COUNT_RVA,
    ]
    .map(at)
    .to_vec()
}

pub fn regions() -> Vec<Region> {
    let mut r = own();
    r.extend(crate::marks::pick_shapes::regions());
    r.extend(crate::marks::turn_all_shapes::regions());
    r
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_shapes(entry: &Regs) -> Result<Regs, Refused> {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let written = regions();
    let saved = capture(&written);
    let done = unsafe { run(entry, settings) };
    if done.is_err() {
        restore(&written, &saved);
    }
    done
}

unsafe fn run(entry: &Regs, settings: u32) -> Result<Regs, Refused> {
    let get = |o: u32| unsafe { ((settings + o) as *const u32).read_unaligned() };
    let mut g = Regs {
        esi: settings,
        ..*entry
    };
    unsafe {
        g = pick_shapes(&g)?;
        g = turn_all_shapes(&g)?;
        g.eax = game_read(CLOCK_RVA);
        if g.eax as i32 >= game_read::<i32>(NEXT_RVA) {
            let r = next_c5(TIME_RANDOM_RVA);
            g.ebx = (r & 0xFF).wrapping_add(game_read(CLOCK_RVA));
            game_write(NEXT_RVA, g.ebx);
            g.esi = settings;
            let mask = get(0x2748);
            if mask >= MASKS {
                return Err(BadInput("a mask number past dword_844E34's 9").into());
            }
            g.ebx = next_c5(MOVE_RANDOM_RVA)
                .rotate_right(16)
                .wrapping_add(g.eax)
                & game_read::<u32>(MASKS_RVA + 4 * mask as usize);
            g.eax = mask;
            game_write(MOVING_RVA, 1u32);
            if g.ebx == 0 {
                game_write(MOVING_RVA, 0u32);
                return Ok(g);
            }
        }
        if game_read::<u32>(MOVING_RVA) == 0 {
            return Ok(g);
        }
        g.esi = settings;
        let rate = if get(0x274C) == 1 {
            g.eax = game_read::<u32>(INDEX_RVA) >> 3;
            (g.eax != 0).then_some(RATE_RVA[0])
        } else {
            Some(RATE_RVA[1])
        };
        if let Some(rate) = rate {
            let count = x87::int_times_plus_int_to_int(
                game_read(TICKS_RVA),
                game_read(rate),
                game_read(COUNT_RVA),
            );
            game_write(COUNT_RVA, count);
            game_write(INDEX_RVA, (count as u32 >> 16) & 0xFF);
        }
        g = pick_shapes(&g)?;
    }
    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { run_shapes(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if settings == 0 {
        return t.skip("no marks settings");
    }
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([CLOCK_RVA, TICKS_RVA].map(at));
    vars.push((settings + 0x2748, 8));
    let saved = capture(&vars);
    for case in 0..8u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                let k = case.wrapping_mul(0x9E37_79B9);
                game_write(CLOCK_RVA, 1000 + case * 7);
                game_write(NEXT_RVA, if case % 3 == 0 { 2000 } else { 900 });
                game_write(TIME_RANDOM_RVA, k);
                game_write(MOVE_RANDOM_RVA, k ^ 0x5555);
                game_write(MOVING_RVA, case & 1);
                game_write(INDEX_RVA, (k >> 8) & 0xFF);
                game_write(COUNT_RVA, k >> 4);
                game_write(TICKS_RVA, 1 + case % 3);
                ((settings + 0x2748) as *mut u32).write_unaligned(case % MASKS);
                ((settings + 0x274C) as *mut u32).write_unaligned(case % 2);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
