//! `net::copy_received` (0x4173A0): under the mutex of the received messages
//! (`net::lock_received` / `net::unlock_received`, `dword_52AB6C`), each player's
//! messages as the receiving thread left them (its record in `byte_52AD9C`, 0x804C bytes:
//! from +0x34, the count and the records, 0x1003 dwords) copied to +0x4040 for
//! `net::apply_received`, the count made 0 and the byte at +0x4039 copied to +0x4038.
//! The loop runs at least once (player 0 with no players).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `net/mod.rs`. Two callers: `net::apply_received` and `sub_4171B5`. Its calls through
//! `part` (in the game `WaitForSingleObject` and `ReleaseMutex`). A call in its first
//! five bytes: private copy (0x47 bytes, both calls relocated), manual patch. Refuses
//! (before anything) more than 16 players (the records' room, up to `dword_5AB270`).
//! Shadow mode runs the original alone (the mutex). Self-test: a copy whose records and
//! count are the test's and whose calls go to noting stand-ins, against the Rust version:
//! 0 .. 3 players.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::types::NetPlayerBlock;
use crate::net::{LOCK, PLAYERS, Part, RECEIVED, RECEIVED_LEN, UNLOCK, a, dispatch_net, net_entry};
use crate::system::api_stand_ins::copy_with;
use rc2k_core::net::inbox;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("net::copy_received", rva(0x4173A0), thunk, &ORIGINAL)
    .private_copy(0x47)
    .relocate_calls(&[1, 0x40])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The records' room: 16 players.
pub const MOST_PLAYERS: u32 = 16;

/// The Rust version on the records at `received` and the count at `players`.
pub fn copy_received_with(
    r: &mut Regs,
    received: u32,
    players: u32,
    part: Part,
) -> Result<(), Refused> {
    let count = || unsafe { (players as *const u32).read_volatile() };
    if count() > MOST_PLAYERS {
        return Err(BadInput("more than the records' 16 players").into());
    }
    let mut g = *r;
    part(LOCK, &mut g)?;
    let mut player = 0u32;
    loop {
        let record = received + RECEIVED_LEN * player;
        // Converted: `rc2k_core::net::inbox::hand_over`, on the inbox and its copy.
        let both = unsafe {
            std::slice::from_raw_parts_mut(
                (record + field!(NetPlayerBlock, count)) as *mut u8,
                2 * inbox::COPIED,
            )
        };
        let byte = inbox::hand_over(both);
        // The registers as the loop leaves them (the unlock's call sees them).
        g.eax = g.eax & !0xFF | byte as u32;
        g.ecx = 0;
        g.esi = record + field!(NetPlayerBlock, copied_count);
        g.edi = record + RECEIVED_LEN;
        player += 1;
        g.ebx = received + RECEIVED_LEN * player;
        g.edx = player;
        if player >= count() {
            break;
        }
    }
    part(UNLOCK, &mut g)
}

/// The Rust version.
pub fn copy_received(r: &mut Regs, part: Part) -> Result<(), Refused> {
    copy_received_with(r, a(RECEIVED), a(PLAYERS), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, copy_received);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, copy_received)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let len = 3 * RECEIVED_LEN as usize + 0x10;
    let mut mem = vec![0u8; len];
    let mut players = 0u32;
    let (base, count) = (mem.as_mut_ptr() as u32, &raw mut players as u32);
    // pusha; call sub_418474
    let entry = [0x60, 0xE8, 0xCE, 0x10, 0x00, 0x00];
    let copy = match copy_with(
        0x4173A0,
        0x47,
        &entry,
        &[1, 0x40],
        &[LOCK, UNLOCK],
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
    for n in 0..4u32 {
        let fill = || unsafe {
            let m = base as *mut u8;
            for k in 0..len {
                m.add(k)
                    .write_volatile((k as u32).wrapping_mul(0x3B).wrapping_add(n) as u8);
            }
            (count as *mut u32).write_volatile(n);
        };
        compare(
            &mut t,
            &format!("{n} players"),
            &copy,
            Regs::SENTINEL,
            &fill,
            &all,
            &|r, part| copy_received_with(r, base, count, part),
            false,
        );
    }
    unsafe { (count as *mut u32).write_volatile(17) };
    let mut r = Regs::SENTINEL;
    t.check(
        copy_received_with(
            &mut r,
            base,
            count,
            &mut crate::sound::turn_parts::test_part,
        )
        .is_err(),
        || "17 players were not refused".into(),
    );
    crate::sound::turn_parts::take_called();
    let _ = (&mut mem, &mut players);
    t.finish()
}
