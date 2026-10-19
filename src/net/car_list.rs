//! `net::car_list` (0x4AAAF7): the car list the network game's car screen shows: with any
//! of the cars won (`g_secretCars`, ended by -1) in the secret cars' list (`off_75A042`,
//! ended by -1), the list with them (`off_759E52`), else the one without (`off_759DCA`).
//!
//! In:  nothing
//! Out: esi = the list; the rest kept (`pusha` .. `popa`); the flags of the last compare
//!      (no caller reads them)
//! Each won car is compared with the secret list's first entry before its end is looked
//! for (so the -1 of an empty secret list's end can match a -1 entry, as in the original).
//! Lists without their end in 0x100 entries are refused. Caller: `net::choose_car`.
//! Shadow mode: both run, esi compared. Self-test: a copy whose two lists are the test's,
//! against the Rust version: no car won, one in the list, none of them in it.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{Part, a, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::car_list", rva(0x4AAAF7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SECRET: usize = 0x75A042; // off_75A042: the secret cars
const WON: usize = 0x759F4A; // g_secretCars: the cars won
const WITH: usize = 0x759E52; // off_759E52
const WITHOUT: usize = 0x759DCA; // off_759DCA
const MOST: u32 = 0x100;

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Whether one of the cars at `won` is in the list at `secret`.
pub fn any_won(secret: u32, won: u32) -> Result<bool, Refused> {
    let mut esi = won;
    for _ in 0..MOST {
        let car = get(esi);
        let mut edi = secret;
        let mut ended = false;
        for _ in 0..MOST {
            if get(edi) == car {
                return Ok(true);
            }
            edi += 4;
            if get(edi) == u32::MAX {
                ended = true;
                break;
            }
        }
        if !ended {
            return Err(BadInput("a secret cars' list with no end").into());
        }
        esi += 4;
        if get(esi) == u32::MAX {
            return Ok(false);
        }
    }
    Err(BadInput("a won cars' list with no end").into())
}

/// The Rust version on the lists at `secret` and `won`.
pub fn car_list_with(entry: &Regs, secret: u32, won: u32) -> Result<Regs, Refused> {
    let found = any_won(secret, won)?;
    Ok(Regs {
        esi: a(if found { WITH } else { WITHOUT }),
        ..*entry
    })
}

/// The Rust version: the registers it leaves.
pub fn car_list(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    car_list_with(entry, a(SECRET), a(WON))
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &[], 0, &|r| {
        car_list(r, &mut crate::net::lobby::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut lists = vec![u32::MAX; 16];
    let at = lists.as_mut_ptr() as u32;
    // pusha; lea edi, off_75A042
    let copy = match crate::net::lobby::copy_vars(
        0x4AAAF7,
        0x3B,
        &[0x60, 0x8D, 0x3D, 0x42, 0xA0, 0x75, 0x00],
        &[],
        &[],
        &[(SECRET, 4, at), (WON, 4, at + 32)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let secret = [0x10u32, 0x20, 0x30, u32::MAX];
    let cases: [(&str, [u32; 4]); 4] = [
        ("no car won", [u32::MAX; 4]),
        ("the second won", [0x55, 0x20, u32::MAX, u32::MAX]),
        ("none in the list", [0x55, 0x66, u32::MAX, u32::MAX]),
        ("the first won", [0x10, u32::MAX, u32::MAX, u32::MAX]),
    ];
    for (what, won) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &[(at, 64)],
            0,
            &[],
            &|| unsafe {
                (at as *mut [u32; 4]).write(secret);
                ((at + 32) as *mut [u32; 4]).write(won);
            },
            &|r, _| car_list_with(r, at, at + 32),
        );
    }
    let _ = &mut lists;
    t.finish()
}
