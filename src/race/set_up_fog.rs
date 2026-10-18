//! `race::set_up_fog` (0x44FBD9): the frame's fog: `scene::restart_clock`, the fog colour
//! into `dword_624000` (`dword_6092CC` << 16 + `dword_6092D0` << 8 + `dword_6092D4`),
//! `race::fog_table`, then the colour with 0xFF000000 to the device (`sub_44E76B`:
//! `dword_5AB964`, then render state 0x22 through `sub_41AE25` and `d3d::set_render_state`;
//! a call into the game, section 15).
//!
//! In:  nothing
//! Out: eax the colour | 0xFF000000; the rest as `fog_table` leaves them (callers
//!      `sub_4557AB` 0x4557E0 and `sub_455F17` 0x455FEA go on with a `call` / `retn`)
//!
//! It begins with its call: private copy (0x40, its three calls relocated), manual patch.
//! Refuses (before anything) what `fog_table` would. Shadow mode runs the original alone
//! (it tells the device and moves the clock). Self-test: a copy whose clock and device
//! calls go to stand-ins (the clock gives the test's time, the device call notes eax),
//! against the Rust version on the same parts; the fog's inputs set; all put back.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_write, hook_asm, rva, shadow};
use crate::race::fog_table::fog_regions;
use crate::race::frame_draw::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::set_up_fog", rva(0x44FBD9), thunk, &ORIGINAL)
    .private_copy(0x40)
    .relocate_calls(&[0x00, 0x2B, 0x3A])
    .manual_patch()
    .self_test(self_test);

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
    match unsafe { set_up_fog(r, &mut fog_part) } {
        Ok(after) => {
            let esp = r.esp;
            *r = after;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

static CALLED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());
const TIME: u32 = 0x0012_3456;

fn stand_in(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((place, r.eax));
    if place == 1 {
        r.eax = TIME;
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(stand_in);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x44FBD9,
            len: 0x40,
            entry: &[0xE8, 0xA3, 0x07, 0x00, 0x00, 0xBB, 0x00, 0x00],
            calls: &[0x00, 0x2B, 0x3A],
            jumps_out: &[],
        },
        &|c| {
            FOG_CALLEES
                .iter()
                .find(|&&(_, k)| rva(k) == c)
                .map(|&(place, _)| crate::install::stand_ins::stand_in(place))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut regions: Vec<Region> = fog_regions();
    regions.push(at(FOG_COLOUR_RVA));
    let mut kept = regions.clone();
    for a in [0x6092CCusize, 0x6092D0, 0x6092D4, 0x63DC58, 0x60AC84] {
        kept.push(at(rva(a)));
    }
    let saved = capture(&kept);
    for (colour, level, length) in [
        ([0x40u32, 0x60, 0x80], 2u32, 0x20u32),
        ([0xFF, 0, 0x1FF], 4, 0x200),
        ([0, 0, 0], 0, 0x08),
    ] {
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x6092CC), colour[0]);
            game_write(rva(0x6092D0), colour[1]);
            game_write(rva(0x6092D4), colour[2]);
            game_write(rva(0x60AC84), level);
            game_write(rva(0x63DC58), length);
            CALLED.lock().unwrap().clear();
        };
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            Some(regs(&ro)),
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(&regions),
        );
        prepare();
        let mine = unsafe {
            set_up_fog(&Regs::SENTINEL, &mut |p, r| {
                stand_in(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "colour {colour:x?} level {level} length {length:#x}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    restore(&kept, &saved);
    t.finish()
}
