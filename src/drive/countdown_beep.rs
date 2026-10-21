//! `drive::countdown_beep` (0x440EA3): one car's count down (the block at esi, its figure
//! at edi): for a car not finished (+0x1C 0) and running (+0x25C), the time since the
//! count down's start (+0x14 less `g_countdownStart`; nothing before it) held at 1500,
//! in seconds (/ 100), -1 from 6 on, into [edi]; on a whole second not -1 the second's
//! sound (`dword_6D1818[s]`, into `dword_614114`) cued (`drive::countdown_cue`).
//!
//! In:  esi the car's block, edi where its second goes
//! Out: eax the second, edx the remainder, ebx 100 or the sound; the rest kept; no flags
//!      read
//!
//! Caller: `drive::countdown_beeps`.
//! Refuses what `countdown_cue` refuses.
//! Self-test: the original and the Rust version on a test block and the game's variables
//! (saved and put back), the car with no sound slot: before, during, after.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use rc2k_core::drive::countdown::{self, Beep};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::countdown_beep", rva(0x440EA3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the block at `block`, the second into `to`.
// Converted: `rc2k_core::drive::countdown::countdown_beep`.
pub unsafe fn countdown_beep(block: u32, to: u32) -> Result<Left, Refused> {
    let rd = |o: u32| unsafe { ((block + o) as *const u32).read_unaligned() };
    let beep =
        countdown::countdown_beep(rd(0x1C), rd(0x25C), || rd(0x14).wrapping_sub(g(0x6D1814)));
    let (second, remainder, cue) = match beep {
        Beep::Idle => return Ok(Left::nothing()),
        Beep::Before(t) => return Ok(Left::nothing().with(EAX, t)),
        Beep::At {
            second,
            remainder,
            cue,
        } => (second, remainder, cue),
    };
    unsafe { (to as *mut u32).write_unaligned(second) };
    let mut left = Left::nothing()
        .with(EAX, second)
        .with(EDX, remainder)
        .with(EBX, 100);
    if cue {
        let sound = g(0x6D1818 + 4 * second as usize);
        p(0x614114, sound);
        left = left
            .with(EBX, sound)
            .with(EAX, crate::drive::countdown_cue::countdown_cue(sound)?);
    }
    Ok(left)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { countdown_beep(r.esi, r.edi) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(r.edi, 4), (a(0x614114), 4)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [
        (a(0x614114), 4),
        (a(0x6D1814), 4),
        (a(0x71B3F8), 4),
        (a(0x641680), 0x20),
    ];
    let saved = capture(&all);
    let mut block = vec![0u32; 0x100];
    let mut out = [0xAAAA_AAAAu32; 1];
    let (bl, to) = (block.as_mut_ptr() as u32, out.as_mut_ptr() as u32);
    for case in 0..12u32 {
        let set = || {
            p(0x71B3F8, 3);
            p(0x641680 + 12, u32::MAX);
            p(0x6D1814, 1000);
            unsafe {
                ((bl + 0x1C) as *mut u32).write((case == 11) as u32);
                ((bl + 0x25C) as *mut u32).write((case != 10) as u32);
                ((bl + 0x14) as *mut u32).write(
                    [900u32, 1000, 1150, 1200, 1300, 1550, 1600, 2600, 1499, 1001]
                        [(case % 10) as usize],
                );
                (to as *mut u32).write(0xAAAA_AAAA);
            }
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs {
                esi: bl,
                edi: to,
                ..Regs::SENTINEL
            },
            &set,
            &[(to, 4), (a(0x614114), 4)],
            &rust,
        );
    }
    restore(&all, &saved);
    let _ = &mut block;
    t.finish()
}
