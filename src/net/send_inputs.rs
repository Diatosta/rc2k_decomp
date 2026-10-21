//! `net::send_inputs` (0x464194): "send j", the player and the step logged, and the
//! message 'JGSM' (0x4D53474A) with the car's four input dwords (the current car's block
//! +0xFD0 .. +0xFDC) built in `unk_71C4D0` and sent (`net::send_race_message`).
//!
//! In:  nothing
//! Out: as `send_race_message` leaves them (in: eax the last dword, ebx the message, ecx
//!      its bytes, the rest kept)
//!
//! See `net/mod.rs`; `send_position` and `send_extra` are the same with other dwords (this
//! file has their shared body). One caller, `net::send_turn` (every fourth step). Its
//! calls through `part` (the log's and the send). Shadow mode runs the original alone.
//! Self-test: a copy whose car block and message are the test's and whose calls go to
//! noting stand-ins, against the Rust version (also the other two's tests).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{
    CAR, LOG_INT, LOG_INT_NL, LOG_TEXT, MESSAGE, Part, SEND_RACE, a, dispatch_net, log_send,
};
use crate::system::api_stand_ins::copy_with;
use rc2k_core::net::race::Kind;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::send_inputs", rva(0x464194), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// What one of the three sends: its log text (an RVA as IDA shows it) and its kind (the
/// tag and the car block's dwords: `rc2k_core::net::race::Kind`).
pub struct Message {
    pub text: usize,
    pub kind: Kind,
}

pub const INPUTS: Message = Message {
    text: 0x4DE0D9, // "send j"
    kind: Kind::Inputs,
};

/// The shared body on a car block and a message buffer at the addresses given.
pub fn send_fields_with(
    r: &mut Regs,
    m: &Message,
    car: u32,
    message: u32,
    part: Part,
) -> Result<(), Refused> {
    log_send(m.text, r, part)?;
    // Converted: `rc2k_core::net::race::Kind::pack`.
    let dwords = m
        .kind
        .pack(|field| unsafe { ((car + field) as *const u32).read_unaligned() });
    let out = message as *mut u32;
    for (n, &v) in dwords.iter().enumerate() {
        unsafe { out.add(n).write_unaligned(v) };
    }
    r.eax = *dwords.last().unwrap_or(&0);
    r.ebx = message;
    r.ecx = 4 * dwords.len() as u32;
    part(SEND_RACE, r)
}

/// In the game.
pub fn send_fields(r: &mut Regs, m: &Message, part: Part) -> Result<(), Refused> {
    send_fields_with(r, m, a(CAR), a(MESSAGE), part)
}

/// The Rust version.
pub fn send_inputs(r: &mut Regs, part: Part) -> Result<(), Refused> {
    send_fields(r, &INPUTS, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, send_inputs);
}

/// The test of the three: a copy of the original at `address` (`len` bytes, its first
/// bytes `entry`: `pusha; lea eax, aSendX`; the send's call at `send`).
pub fn test_send(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    send: usize,
    m: &'static Message,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut block = vec![0u32; 0x1010 / 4];
    let mut message = vec![0u32; 0x100 / 4];
    let (car, out) = (block.as_mut_ptr() as u32, message.as_mut_ptr() as u32);
    let copy = match copy_with(
        address,
        len,
        entry,
        &[7, 19, 31, send],
        &[LOG_TEXT, LOG_INT, LOG_INT_NL, SEND_RACE],
        &[],
        &[(CAR, 0x1008, car), (MESSAGE, 0x100, out)],
        m.kind.fields().len() + 1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(car, 0x1010usize), (out, 0x100)];
    for seed in [1u32, 0x9E37_79B9] {
        let fill = || unsafe {
            for n in 0..0x1010 / 4 {
                ((car + 4 * n) as *mut u32).write_volatile(n.wrapping_mul(seed) ^ 0x5A5A_0000);
            }
            for n in 0..0x100 / 4 {
                ((out + 4 * n) as *mut u32).write_volatile(0xAAAA_AAAA);
            }
        };
        compare(
            &mut t,
            &format!("seed {seed:#x}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            &all,
            &|r, part| send_fields_with(r, m, car, out, part),
            false,
        );
    }
    let _ = (&mut block, &mut message);
    t.finish()
}

fn self_test() -> (usize, usize) {
    test_send(
        &HOOK,
        0x464194,
        0x6D,
        &[0x60, 0x8D, 0x05, 0xD9, 0xE0, 0x4D, 0x00],
        0x67,
        &INPUTS,
    )
}
