//! `drive::next_camera_view` (0x43AA38): unless the block's (esi) view is held (+0x694),
//! the next view: +0x684 + 1, but views 2 and 3 become 4 when the car has none
//! (`dword_5C5AC0[car]` 0, or the car number +0x24 past 0x21) or with two players
//! (`dword_63DB94` 2); 7 and above wrap to 0; into +0x688, then
//! `drive::set_camera_view`. A reading: the change-view key.
//!
//! In:  esi the block
//! Out: eax the step count (when a view was taken), else kept; no flags read
//!
//! Callers: `sub_43A35A` (0x43A454), `sub_43A649` (0x43A66C).
//! Self-test: a copy of the original (`dword_63DB94` the test's, its call to a copy of
//! `set_camera_view`) on a test block: views 0 .. 8, held or not, cars 0 .. 0x23, one or
//! two players.

use crate::drive::set_camera_view::{VIEW_LEN, fill_block, set_view_at};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::next_camera_view", rva(0x43AA38), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLAYERS_RVA: usize = rva(0x63DB94); // dword_63DB94
const HAS_VIEWS_RVA: usize = rva(0x5C5AC0); // dword_5C5AC0, by car (0x22)

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core: block `s`, the players at `players`, the step count at `step`.
pub unsafe fn next_view_at(s: u32, players: u32, step: u32) -> Left {
    unsafe {
        if at(s + 0x694) != 0 {
            return Left::nothing();
        }
        let mut view = at(s + 0x684).wrapping_add(1);
        if (2..=3).contains(&view) {
            let car = at(s + 0x24);
            let has = car < 0x22 && game_read::<u32>(HAS_VIEWS_RVA + 4 * car as usize) != 0;
            if !has || at(players) == 2 {
                view = 4;
            }
        }
        if view >= 7 {
            view = 0;
        }
        (s.wrapping_add(0x688) as *mut u32).write_unaligned(view);
        set_view_at(s, step)
    }
}

/// The Rust version.
pub unsafe fn next_camera_view(s: u32) -> Left {
    unsafe {
        next_view_at(
            s,
            game_addr(PLAYERS_RVA) as u32,
            game_addr(crate::drive::set_camera_view::STEP_RVA) as u32,
        )
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { next_camera_view(r.esi) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.esi + 0x684, 0x1C)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; VIEW_LEN as usize / 4 + 2];
    let s = cells.as_mut_ptr() as u32;
    let (step, players) = (s + VIEW_LEN, s + VIEW_LEN + 4);
    let inner = match crate::drive::set_camera_view::test_copy(step) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43AA38,
            len: 0x59,
            entry: &[0x83, 0xBE, 0x94, 0x06, 0x00, 0x00, 0x00],
            calls: &[0x53],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43A9BA)).then_some(target),
        &[(0x63DB94, players)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(s, VIEW_LEN as usize + 8)];
    let mut case = 0u32;
    for view in 0..9u32 {
        for (held, car, two) in [
            (0u32, 0u32, 1u32),
            (0, 5, 2),
            (0, 0x23, 1),
            (1, 3, 1),
            (0, 0x21, 1),
        ] {
            case += 1;
            let fill = || unsafe {
                fill_block(s, case);
                let p = s as *mut u32;
                p.add(0x684 / 4).write(view.wrapping_sub(1));
                p.add(0x694 / 4).write(held);
                p.add(0x24 / 4).write(car);
                (step as *mut u32).write(case * 77);
                (players as *mut u32).write(two);
            };
            fill();
            let before = Regs {
                esi: s,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill();
            let left = unsafe { next_view_at(s, players, step) };
            let mine = capture(&all);
            let regs_ok = left.matches(&before, &ro);
            t.check(regs_ok && theirs == mine, || {
                format!("view {view}, held {held}, car {car:#x}, players {two}: regs_ok={regs_ok}; the block differs")
            });
        }
    }
    drop((copy, inner, cells));
    t.finish()
}
