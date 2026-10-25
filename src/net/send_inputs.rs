//! `net::send_inputs` (0x464194): "send j", the player and the step logged, and the
//! message 'JGSM' (0x4D53474A) with the car's four input dwords (the current car's block
//! +0xFD0 .. +0xFDC) built in `unk_71C4D0` and sent (`net::send_race_message`, which is
//! `call net::send_message; retn`: `net::send_message` here).
//!
//! In:  nothing
//! Out: nothing (the original's registers, as the send leaves them, are read by no caller:
//!      `lq -s 0x464194`, esi only, kept)
//!
//! See `net/mod.rs`; `send_position` and `send_extra` are the same with other dwords (this
//! file has their shared body). One caller, `net::send_turn` (every fourth step).

use crate::errors::Refused;
use crate::net::send_message::send_message;
use crate::net::{CAR, MESSAGE, a, log_send};
use rc2k_core::net::race::Kind;

/// What one of the three sends: its log text  and its kind (the
/// tag and the car block's dwords: `rc2k_core::net::race::Kind`).
pub struct Message {
    pub text: &'static std::ffi::CStr,
    pub kind: Kind,
}

pub const INPUTS: Message = Message {
    text: c"send j", // "send j"
    kind: Kind::Inputs,
};

/// The shared body: the car block `net::CAR`, the message built at `net::MESSAGE`.
pub fn send_fields(m: &Message) -> Result<(), Refused> {
    let (car, message) = (a(CAR), a(MESSAGE));
    log_send(m.text);
    // Converted: `rc2k_core::net::race::Kind::pack`.
    let dwords = m
        .kind
        .pack(|field| unsafe { ((car + field) as *const u32).read_unaligned() });
    let out = message as *mut u32;
    for (n, &v) in dwords.iter().enumerate() {
        unsafe { out.add(n).write_unaligned(v) };
    }
    send_message(message as *const u8, 4 * dwords.len() as u32)
}

/// The Rust version.
pub fn send_inputs() -> Result<(), Refused> {
    send_fields(&INPUTS)
}
