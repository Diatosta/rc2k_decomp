//! `net::send_message` (0x41736D): a message to every player: the sequence byte
//! (`byte_52A915` + 1, top bit set, kept) and the ecx bytes at ebx into `byte_52AA68`,
//! sent (DirectPlay's Send, `sub_4196FF`: eax to whom, `dword_5AB274`; ebx the bytes; ecx
//! their count, one more than given; edx 0 the flags).
//!
//! In:  ebx = the bytes, ecx = how many
//! Out: eax `dword_5AB274`, ebx `byte_52AA68`, ecx the count + 1, edx 0 (as the send
//!      keeps them), esi and edi kept; the carry the send's (set if it failed; not read)
//!
//! See `net/mod.rs`. One caller, `net::send_race_message`. Its call through `part` (in
//! the game DirectPlay's, through the game). Refuses (before anything) more than 0xFF
//! bytes: the buffer is 0x100 (the original writes on into `dword_52AB68`). Shadow mode
//! runs the original alone. Self-test: a copy whose three variables are the test's and
//! whose call goes to a noting stand-in, against the Rust version: counts 0, 1, 0x40,
//! 0xFF, sequence bytes either side of the wrap.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{Part, SEND, a, dispatch_net, net_entry};
use crate::system::api_stand_ins::copy_with;
use rc2k_core::net::inbox;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::send_message", rva(0x41736D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SEQUENCE: usize = 0x52A915; // byte_52A915
const BUFFER: usize = 0x52AA68; // byte_52AA68: 0x100 bytes
const TO: usize = 0x5AB274; // dword_5AB274: whom to send to

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub sequence: u32,
    pub buffer: u32,
    pub to: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            sequence: a(SEQUENCE),
            buffer: a(BUFFER),
            to: a(TO),
        }
    }
}

/// The Rust version on the variables in `v`.
pub fn send_message_with(r: &mut Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    let count = r.ecx;
    // Converted: `rc2k_core::net::inbox` (`next_sequence`, `outgoing`: a message too long
    // refused before anything is written).
    unsafe {
        let seq = v.sequence as *mut u8;
        let byte = inbox::next_sequence(seq.read_volatile());
        let bytes = std::slice::from_raw_parts(r.ebx as *const u8, count as usize);
        let message = inbox::outgoing(byte, bytes)?;
        seq.write_volatile(byte);
        std::ptr::copy(message.as_ptr(), v.buffer as *mut u8, message.len());
        r.eax = (v.to as *const u32).read_volatile();
    }
    let (esi, edi) = (r.esi, r.edi);
    // As `rep movsb` leaves them at the send.
    r.esi = r.ebx.wrapping_add(count);
    r.edi = v.buffer + 1 + count;
    r.ebx = v.buffer;
    r.ecx = count + 1;
    r.edx = 0;
    part(SEND, r)?;
    r.esi = esi;
    r.edi = edi;
    Ok(())
}

/// The Rust version.
pub fn send_message(r: &mut Regs, part: Part) -> Result<(), Refused> {
    send_message_with(r, &Vars::game(), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, send_message);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, send_message)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = vec![0u8; 0x110];
    let base = mem.as_mut_ptr() as u32;
    let v = Vars {
        sequence: base,
        to: base + 4,
        buffer: base + 0x10,
    };
    // push esi; push edi; mov al, byte_52A915
    let entry = [0x56, 0x57, 0xA0, 0x15, 0xA9, 0x52, 0x00];
    let copy = match copy_with(
        0x41736D,
        0x33,
        &entry,
        &[0x2B],
        &[SEND],
        &[],
        &[
            (SEQUENCE, 1, v.sequence),
            (BUFFER, 0x100, v.buffer),
            (TO, 4, v.to),
        ],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let source: Vec<u8> = (0..0x100u32).map(|n| (n * 7 + 3) as u8).collect();
    let all = [(base, 0x110usize)];
    for count in [0u32, 1, 0x40, 0xFF] {
        for seq in [0u8, 0x7E, 0x7F, 0xFE, 0xFF] {
            let fill = || unsafe {
                let m = base as *mut u8;
                for n in 0..0x110 {
                    m.add(n).write_volatile(0xAA);
                }
                m.write_volatile(seq);
                (v.to as *mut u32).write_volatile(0xFFFF_FFF0 + count);
            };
            let entry = Regs {
                ebx: source.as_ptr() as u32,
                ecx: count,
                ..Regs::SENTINEL
            };
            compare(
                &mut t,
                &format!("count {count:#x}, sequence {seq:#x}"),
                &copy,
                entry,
                &fill,
                &all,
                &|r, part| send_message_with(r, &v, part),
                false,
            );
        }
    }
    let mut r = Regs {
        ebx: source.as_ptr() as u32,
        ecx: 0x100,
        ..Regs::SENTINEL
    };
    t.check(
        send_message_with(&mut r, &v, &mut crate::sound::turn_parts::test_part).is_err(),
        || "0x100 bytes were not refused".into(),
    );
    let _ = &mut mem;
    t.finish()
}
