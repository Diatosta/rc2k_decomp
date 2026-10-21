//! `system::wait_or_key` (0x407F69): waits up to eax milliseconds for a key: every 10 ms
//! (`system::sleep`) a key taken (`input::take_key`), the clock (`system::tick_count`)
//! read until it reaches the start plus eax (a signed difference).
//!
//! In:  eax the milliseconds
//! Out: eax the key (0 when the time ran out), zero = no key, carry clear; ebx kept (its
//!      `push` / `pop`); the rest as the calls leave them
//!
//! Caller: `sub_47F314` (the welcome's speech). The calls through `part`, in the game
//! their Rust versions. A private copy (a call at +3). Ungated (it waits). Shadow mode
//! runs the original alone (it takes keys). Self-test:
//! `menu::scripted::test_loop`, keys and clocks by script.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::scripted::{Answer, test_loop};
use rc2k_core::system::time::{time_up, wait_end};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("system::wait_or_key", rva(0x407F69), thunk, &ORIGINAL)
    .private_copy(0x2C)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CALLS: [usize; 4] = [0x3, 0xF, 0x16, 0x1D];

/// A call by place: 1 `system::tick_count`, 2 `system::sleep` (eax), 3 `input::take_key`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub fn wait_or_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    g.ebx = g.eax;
    part(1, &mut g)?;
    // Converted: `rc2k_core::system::time::{wait_end, time_up}`.
    g.ebx = wait_end(g.eax, g.ebx);
    loop {
        g.eax = 10;
        part(2, &mut g)?;
        g.eax = 0;
        part(3, &mut g)?;
        if g.eflags & 0x40 == 0 {
            break;
        }
        part(1, &mut g)?;
        if time_up(g.eax, g.ebx) {
            g.eax = 0;
            break;
        }
    }
    g.ebx = entry.ebx;
    g.set_zero(g.eax == 0);
    g.set_carry(false);
    Ok(g)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => r.eax = crate::system::tick_count::tick_count(),
        2 => crate::system::sleep::sleep(r.eax),
        _ => {
            let key = crate::input::take_key::take_key();
            r.eax = (r.eax & !0xFF) | u32::from(key);
            r.set_zero(key == 0);
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::framework::shadow() {
        unsafe { call_original(r) };
        return;
    }
    // The game's parts never fail.
    if let Ok(left) = wait_or_key(r, &mut game_part) {
        *r = Regs { esp: r.esp, ..left };
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let clock = |from: u32, n: u32, step: u32| {
        (0..n)
            .map(|k| Answer::eax(from.wrapping_add(k * step)))
            .collect::<Vec<_>>()
    };
    let scripts = vec![
        // the time runs out
        vec![
            (1, clock(1000, 50, 10), Answer::eax(0x7FFF_FFFF)),
            (3, vec![], Answer::key(0)),
        ],
        // a key after a few polls
        vec![
            (1, clock(5, 50, 7), Answer::eax(0x7FFF_FFFF)),
            (3, vec![Answer::key(0); 4], Answer::key(0x1C)),
        ],
        // a key at once
        vec![
            (1, vec![Answer::eax(0xFFFF_FFF0)], Answer::eax(0x10)),
            (3, vec![], Answer::key(1)),
        ],
        // the clock wraps past 0
        vec![
            (1, clock(0xFFFF_FF00, 80, 9), Answer::eax(0x7FFF_FFFF)),
            (3, vec![], Answer::key(0)),
        ],
    ];
    test_loop(
        t,
        0x407F69,
        0x2C,
        &[0x53, 0x8B, 0xD8, 0xE8, 0xB3, 0x90, 0x00, 0x00],
        &CALLS,
        &[(0x411024, 1), (0x41100C, 2), (0x407F95, 3)],
        &[],
        &|| {},
        &scripts,
        Regs {
            eax: 3000,
            ..Regs::SENTINEL
        },
        &|r| {
            wait_or_key(r, &mut |p, r| {
                crate::menu::scripted::handler(p, r);
                Ok(())
            })
            .unwrap_or(*r)
        },
    )
}
