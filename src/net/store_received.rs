//! `net::store_received` (0x419677): a race message from another player put into its
//! block of `g_netReceived` (from +0x34: a count, 0x40 records of 0x100 bytes from +0x38:
//! +0 the sequence byte doubled, +1 the length, +2 the bytes; +0x4038 the oldest sequence
//! still wanted (`net::copy_received` sets it from +0x4039), +0x4039 the newest seen,
//! +0x403C the time of the last message), kept in sequence order.
//!
//! The sequence (al) is doubled (`add al, al`: the top bit, which marks a race message,
//! goes). The time is stored. Older than +0x4038 (the difference negative as a signed
//! byte): nothing more. Not older than +0x4039: the new newest. Then: no records, it is
//! record 0; 0x40 records: the first one dropped (the other 0x3F moved down) and the new one
//! written at record 0x40, past the end, over +0x4038 .. (the count not changed; the game's
//! slip, section 16); else the records newer than it moved up one and it goes in after the
//! last one not newer (or first), the count up one. The record: the sequence, cl, and ecx
//! bytes from ebx (`rep movsb`: all of ecx); but once a record was moved up, ecx is 0
//! (the move's `rep movsd`): a message stored between others keeps no length or bytes
//! (the game's slip, section 16).
//!
//! In:  al = the sequence byte, edi = the player's block, edx = the time, ecx = the length,
//!      ebx = the bytes
//! Out: every register kept (`pusha` .. `popa`); the flags not modelled (the one caller
//!      calls `net::unlock_received` next, which sets its own)
//! Caller: `net::take_messages` (under the lock). Refuses (before anything) a count above
//! 0x40 and a length above 0xFE (the original writes past the block or the record; its
//! caller sends neither). Shadow mode runs the original alone (the caller's lock).
//! Self-test: a copy against the Rust version on a test block: empty, appended, inserted,
//! first, too old, full, the sequence wrapping, lengths 0 and 0xFE; the refusals.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{movsb, run_both};
use crate::net::message_vars::{get, get8, set, set8};
use crate::net::{Part, dispatch_net};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::store_received", rva(0x419677), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The records' room.
pub const RECORDS: u32 = 0x40;
const RECORD: u32 = 0x100;
/// The longest message a record holds.
pub const LONGEST: u32 = 0xFE;

/// True if `a` is older than `b` (`cmp a, b; js`).
fn older(a: u8, b: u8) -> bool {
    (a.wrapping_sub(b) as i8) < 0
}

/// The Rust version.
pub fn store_received(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    let sequence = (r.eax as u8).wrapping_add(r.eax as u8);
    let base = r.edi.wrapping_add(0x34);
    let count = get(base);
    if count > RECORDS {
        return Err(BadInput("store_received: more than 0x40 records").into());
    }
    if r.ecx > LONGEST {
        return Err(BadInput("store_received: a message longer than 0xFE bytes").into());
    }
    set(base + 0x4008, r.edx);
    if older(sequence, get8(base + 0x4004)) {
        return Ok(());
    }
    if !older(sequence, get8(base + 0x4005)) {
        set8(base + 0x4005, sequence);
    }
    let mut length = r.ecx;
    let at = if count == 0 {
        set(base, 1);
        0
    } else if count == RECORDS {
        // rep movsd forwards: the records 1 .. 0x3F to 0 .. 0x3E.
        movsb(base + 4, base + 4 + RECORD, (RECORDS - 1) * RECORD);
        RECORDS * RECORD
    } else {
        let mut at = count * RECORD;
        loop {
            if !older(sequence, get8(base + at - 0xFC)) {
                break;
            }
            movsb(base + at + 4, base + at - 0xFC, RECORD);
            // `mov ecx, 40h; rep movsd` leaves ecx 0: the record written below gets
            // length 0 and none of its bytes (the game's slip, section 16).
            length = 0;
            at -= RECORD;
            if at == 0 {
                break;
            }
        }
        set(base, count + 1);
        at
    };
    set8(base + at + 4, sequence);
    set8(base + at + 5, length as u8);
    movsb(base + at + 6, r.ebx, length);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, store_received);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; add al, al; lea ebp, [edi+34h]
    let copy = match crate::net::lobby::copy(
        0x419677,
        0x88,
        &[0x60, 0x02, 0xC0, 0x8D, 0x6F, 0x34],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    const BLOCK_LEN: usize = 0x4200;
    const DATA_LEN: usize = 0x100;
    let mut block = vec![0u8; BLOCK_LEN];
    let mut bytes = vec![0u8; DATA_LEN];
    let (b, data) = (block.as_mut_ptr() as u32, bytes.as_mut_ptr() as u32);
    let base = b + 0x34;
    let regions = [(b, BLOCK_LEN), (data, DATA_LEN)];
    let fill = |count: u32, low: u8, high: u8, sequences: &[u8]| {
        for k in 0..BLOCK_LEN as u32 {
            set8(b + k, (k.wrapping_mul(0x9E37) >> 3) as u8);
        }
        for k in 0..DATA_LEN as u32 {
            set8(data + k, k as u8 ^ 0xC3);
        }
        set(base, count);
        set8(base + 0x4004, low);
        set8(base + 0x4005, high);
        for (k, &s) in sequences.iter().enumerate() {
            set8(base + 4 + k as u32 * RECORD, s);
        }
    };
    let full: Vec<u8> = (0..RECORDS as u8).map(|k| 2 * k).collect();
    // (what, the count, low, high, the records' sequences, al, the length)
    type Case<'a> = (&'a str, u32, u8, u8, &'a [u8], u32, u32);
    let cases: [Case; 11] = [
        ("empty", 0, 0, 0, &[], 0x81, 5),
        ("appended", 3, 0x10, 0x30, &[0x10, 0x20, 0x30], 0x20, 7),
        ("inserted", 3, 0x10, 0x30, &[0x10, 0x20, 0x30], 0x12, 9),
        (
            "same as the last",
            3,
            0x10,
            0x30,
            &[0x10, 0x20, 0x30],
            0x18,
            3,
        ),
        ("too old", 3, 0x10, 0x30, &[0x10, 0x20, 0x30], 0x04, 4),
        ("first", 3, 0x00, 0x30, &[0x10, 0x20, 0x30], 0x01, 6),
        ("full", RECORDS, 0x00, 0x10, &full, 0x3F, 0x0C),
        ("full, older", RECORDS, 0x00, 0x7E, &full, 0x05, 0x0D),
        ("wrapping", 2, 0xF0, 0xF8, &[0xF0, 0xF8], 0x82, 8),
        ("length 0", 1, 0x00, 0x02, &[0x02], 0x8A, 0),
        ("length 0xFE", 1, 0x00, 0x02, &[0x02], 0x8B, LONGEST),
    ];
    for (what, count, low, high, sequences, al, len) in cases {
        let entry = Regs {
            eax: 0xEEEE_EE00 | al,
            ebx: data,
            ecx: len,
            edx: 0x0012_3456,
            edi: b,
            ..Regs::SENTINEL
        };
        run_both(
            &mut t,
            what,
            copy.addr(),
            entry,
            &regions,
            0,
            &[],
            &|| fill(count, low, high, sequences),
            &|e, part| {
                let mut r = *e;
                store_received(&mut r, part)?;
                Ok(r)
            },
        );
    }
    // The refusals: before anything is written.
    for (count, len) in [(RECORDS + 1, 4), (1, LONGEST + 1)] {
        let saved = crate::service::harness::capture(&regions);
        let mut r = Regs {
            eax: 0x10,
            ebx: data,
            ecx: len,
            edx: 1,
            edi: b,
            ..Regs::SENTINEL
        };
        set(base, count);
        let before = crate::service::harness::capture(&regions);
        let refused = matches!(
            store_received(&mut r, &mut crate::net::lobby::test_part),
            Err(Refused::BadInput(_))
        );
        let after = crate::service::harness::capture(&regions);
        crate::service::harness::restore(&regions, &saved);
        t.check(refused && before == after, || {
            format!("count {count:#x}, length {len:#x}: not refused, or written")
        });
    }
    drop(block);
    drop(bytes);
    t.finish()
}
