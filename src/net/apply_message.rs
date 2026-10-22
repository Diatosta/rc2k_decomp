//! `net::apply_message` (0x4644EC): one message received from player ebp put into that
//! player's car block (`g_carBlocks[ebp]`), after logging the own player, the step, the
//! message's first byte, "rcvd fm plyr" and ebp. Players 8 and up: nothing more.
//! - 'JGSM': the four input dwords into +0xFD0 .. +0xFDC;
//! - 'XGSM': nine dwords into +0x64 .. +0x84;
//! - 'PGSM': the place into +0xFC0 .. +0xFC8 (where the car is to be), the other 37
//!   dwords where `net::send_position` took them from. Then, after the car's first 1000
//!   steps (+8; `js` on +8 - 1000) and when the place received is 10 or more away from the
//!   car's (+0x134 .. +0x13C, whole numbers: `fild` / `fisub`, `fsqrt`, `flt_5C5D54`) the
//!   car is put there; else +0xFC0 .. +0xFC8 become a 32nd of the way (`sar 5`) and
//!   +0xFCC 32, for `net::follow_remote` to glide it;
//! - others: nothing.
//!
//! In:  ebx = the message (its first dword the tag), ebp = the player
//! Out: eax the first byte (players 8 and up), the tag (others), or the last dword moved
//!      (or the last 32nd); ecx past the dwords read; edi the car block; the rest kept
//!
//! See `net/mod.rs`. One caller, `net::apply_received` (inside `pusha`). Its calls through
//! `part` (the log's). The distance with the original's x87 instructions. Shadow mode runs
//! the original alone (it logs). Self-test: a copy whose block table is the test's and
//! whose calls go to noting stand-ins, against the Rust version: each tag and another,
//! players 0, 7, 8, steps either side of 1000, places near and far.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{
    LOG_INT, LOG_INT_NL, LOG_TEXT, PLAYER, Part, STEP, a, dispatch_net, g, net_entry,
};
use crate::system::api_stand_ins::copy_with;
use rc2k_core::net::race::{self, Arrival, Kind};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::apply_message", rva(0x4644EC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// g_carBlocks: the eight cars' blocks.
pub const BLOCKS: usize = 0x71B3B8;
const TEXT: usize = 0x4DE0EE; // "rcvd fm plyr"
const NEAR: usize = 0x5C5D54; // flt_5C5D54: 10.0

/// The Rust version on the block table at `blocks`.
pub fn apply_message_with(r: &mut Regs, blocks: u32, part: Part) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_unaligned() };
    let wr = |at: u32, v: u32| unsafe { (at as *mut u32).write_unaligned(v) };
    r.eax = unsafe { (r.ebx as *const u8).read() } as u32;
    for (place, eax) in [
        (LOG_INT, g(PLAYER)),
        (LOG_INT, g(STEP)),
        (LOG_INT, r.eax),
        (LOG_TEXT, a(TEXT)),
        (LOG_INT_NL, r.ebp),
    ] {
        let mut l = Regs { eax, ..*r };
        part(place, &mut l)?;
    }
    if r.ebp >= 8 {
        return Ok(());
    }
    let block = rd(blocks + 4 * r.ebp);
    r.edi = block;
    r.eax = rd(r.ebx);
    // Converted: `rc2k_core::net::race` (`Kind`, `arrival`; the distance's test in `f64`,
    // exact for the decision).
    let Some(kind) = Kind::of(r.eax) else {
        return Ok(());
    };
    r.ecx = 4;
    for n in 0..kind.fields().len() {
        r.eax = rd(r.ebx + r.ecx);
        wr(block + kind.destination(n), r.eax);
        r.ecx += 4;
    }
    if kind != Kind::Position {
        return Ok(());
    }
    let three = |at: u32| [rd(block + at), rd(block + at + 4), rd(block + at + 8)];
    let (wanted, now) = (three(race::WANTED), three(race::PLACE));
    let steps = rd(block + race::STEPS);
    let near = f32::from_bits(unsafe { game_read(rva(NEAR)) });
    match race::arrival(wanted, now, steps, near) {
        Arrival::Jump => {
            for (k, v) in (0..).zip(wanted) {
                wr(block + race::PLACE + 4 * k, v);
            }
        }
        Arrival::Glide(step) => {
            for (k, v) in (0..).zip(step) {
                r.eax = v;
                wr(block + race::WANTED + 4 * k, v);
            }
            wr(block + race::GLIDE_STEPS, race::GLIDE);
        }
    }
    Ok(())
}

/// The Rust version.
pub fn apply_message(r: &mut Regs, part: Part) -> Result<(), Refused> {
    apply_message_with(r, a(BLOCKS), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, apply_message);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, apply_message)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut table = [0u32; 8];
    let mut block = vec![0u32; 0x1000 / 4];
    let mut message = vec![0u32; 0x40];
    let (tb, bb, mb) = (
        table.as_mut_ptr() as u32,
        block.as_mut_ptr() as u32,
        message.as_mut_ptr() as u32,
    );
    // mov eax, 0
    let entry = [0xB8, 0x00, 0x00, 0x00, 0x00];
    let copy = match copy_with(
        0x4644EC,
        0x397,
        &entry,
        &[13, 25, 32, 45, 54],
        &[LOG_INT, LOG_TEXT, LOG_INT_NL],
        &[],
        &[(BLOCKS, 0x20, tb)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(bb, 0x1000usize), (mb, 0x100)];
    let mut case = 0u32;
    for tag in [0x4D53_4750u32, 0x4D53_474A, 0x4D53_4758, 0x4D53_4751] {
        for player in [0u32, 7, 8] {
            for (steps, dx) in [
                (999u32, 3i32),
                (1000, 3),
                (1000, 40),
                (5000, -9),
                (0x8000_0000, 100),
            ] {
                case += 1;
                let fill = || unsafe {
                    for n in 0..8 {
                        ((tb + 4 * n) as *mut u32).write_volatile(bb);
                    }
                    for n in 0..0x400u32 {
                        ((bb + 4 * n) as *mut u32)
                            .write_volatile(n.wrapping_mul(0x9E37_79B9) ^ case);
                    }
                    ((bb + 8) as *mut u32).write_volatile(steps);
                    for (k, v) in [1000i32, -2000, 30].iter().enumerate() {
                        ((bb + 0x134 + 4 * k as u32) as *mut i32).write_volatile(*v);
                    }
                    for n in 0..0x40u32 {
                        ((mb + 4 * n) as *mut u32)
                            .write_volatile(n.wrapping_mul(0x01F3_5A01) ^ case);
                    }
                    (mb as *mut u32).write_volatile(tag);
                    for (k, v) in [1000 + dx, -2000 - dx / 2, 30 + dx / 3].iter().enumerate() {
                        ((mb + 4 + 4 * k as u32) as *mut i32).write_volatile(*v);
                    }
                };
                let entry = Regs {
                    ebx: mb,
                    ebp: player,
                    ..Regs::SENTINEL
                };
                compare(
                    &mut t,
                    &format!("tag {tag:#x}, player {player}, steps {steps}, dx {dx}"),
                    &copy,
                    entry,
                    &fill,
                    &all,
                    &|r, part| apply_message_with(r, tb, part),
                    false,
                );
            }
        }
    }
    let _ = (&mut table, &mut block, &mut message);
    t.finish()
}
