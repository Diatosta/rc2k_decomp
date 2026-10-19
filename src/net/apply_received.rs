//! `net::apply_received` (0x4644A4): the messages received since the last step
//! (`net::copy_received`), each player's in turn (`byte_52AD9C` + player * 0x804C +
//! 0x4040: a count, then records of 0x100 bytes, +1 the length, +2 the message) applied
//! (`net::apply_message`, inside `pusha`: ebx the message, ecx its length, ebp the player).
//! The players' loop ends on `jnz` against `dword_52AD98`: with no players it would not end.
//! The receiving thread keeps a count to 0x40 (`sub_419677`), the room there is.
//!
//! In:  nothing
//! Out: eax kept; ebx, ecx, edx, esi as the last player's loop leaves them; ebp the
//!      players; edi kept
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Its calls through `part`. A call at its
//! start: private copy (0x48 bytes, both calls relocated), manual patch. Refuses (before
//! anything) no players and more than 16. Shadow mode runs the original alone. Self-test:
//! a copy whose records and count are the test's and whose calls go to noting stand-ins,
//! against the Rust version: 1 .. 3 players, 0 .. 3 messages each.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::copy_received::MOST_PLAYERS;
use crate::net::types::NetPlayerBlock;
use crate::net::{APPLY, COPY_RECEIVED, PLAYERS, Part, RECEIVED, RECEIVED_LEN, a, dispatch_net};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("net::apply_received", rva(0x4644A4), thunk, &ORIGINAL)
    .private_copy(0x48)
    .relocate_calls(&[0, 0x2E])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the records at `received` and the count at `players`.
pub fn apply_received_with(
    r: &mut Regs,
    received: u32,
    players: u32,
    part: Part,
) -> Result<(), Refused> {
    let count = || unsafe { (players as *const u32).read_volatile() };
    if count() == 0 || count() > MOST_PLAYERS {
        return Err(BadInput("no players, or more than the records' 16").into());
    }
    part(COPY_RECEIVED, r)?;
    r.ebp = 0;
    loop {
        r.esi = received + RECEIVED_LEN * r.ebp + field!(NetPlayerBlock, copied_count);
        r.edx = unsafe { (r.esi as *const u32).read_unaligned() };
        if r.edx != 0 {
            r.esi += 4;
            loop {
                r.ecx = unsafe { ((r.esi + 1) as *const u8).read() } as u32;
                r.ebx = r.esi + 2;
                let mut inner = *r;
                part(APPLY, &mut inner)?;
                r.esi += 0x100;
                r.edx = r.edx.wrapping_sub(1);
                if r.edx == 0 {
                    break;
                }
            }
        }
        r.ebp += 1;
        if r.ebp == count() {
            return Ok(());
        }
    }
}

/// The Rust version.
pub fn apply_received(r: &mut Regs, part: Part) -> Result<(), Refused> {
    apply_received_with(r, a(RECEIVED), a(PLAYERS), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, apply_received);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let len = 3 * RECEIVED_LEN as usize;
    let mut mem = vec![0u8; len];
    let mut players = 0u32;
    let (base, count) = (mem.as_mut_ptr() as u32, &raw mut players as u32);
    // call sub_4173A0
    let entry = [0xE8, 0xF7, 0x2E, 0xFB, 0xFF];
    let copy = match copy_with(
        0x4644A4,
        0x48,
        &entry,
        &[0, 0x2E],
        &[COPY_RECEIVED, APPLY],
        &[],
        &[
            (RECEIVED, (RECEIVED_LEN * MOST_PLAYERS) as usize, base),
            (PLAYERS, 4, count),
        ],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, len), (count, 4)];
    for n in 1..4u32 {
        for messages in 0..4u32 {
            let fill = || unsafe {
                let m = base as *mut u8;
                for k in 0..len {
                    m.add(k).write_volatile((k as u32).wrapping_mul(0x2F) as u8);
                }
                for p in 0..n {
                    let at = base + RECEIVED_LEN * p + field!(NetPlayerBlock, copied_count);
                    ((at) as *mut u32).write_volatile((messages + p) % 4);
                }
                (count as *mut u32).write_volatile(n);
            };
            compare(
                &mut t,
                &format!("{n} players, {messages} messages"),
                &copy,
                Regs::SENTINEL,
                &fill,
                &all,
                &|r, part| apply_received_with(r, base, count, part),
                false,
            );
        }
    }
    for refused in [0u32, 17] {
        unsafe { (count as *mut u32).write_volatile(refused) };
        let mut r = Regs::SENTINEL;
        t.check(
            apply_received_with(
                &mut r,
                base,
                count,
                &mut crate::sound::turn_parts::test_part,
            )
            .is_err(),
            || format!("{refused} players were not refused"),
        );
    }
    crate::sound::turn_parts::take_called();
    let _ = (&mut mem, &mut players);
    t.finish()
}
