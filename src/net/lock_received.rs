//! `net::lock_received` (0x418474): the received messages' mutex (`g_netMutex`) taken:
//! `system::wait_object` (0x4110A6: `WaitForSingleObject(eax, INFINITE)`, the flags of
//! `cmp result + 1, 1`) inside `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: every register kept; the flags `wait_object` left (zero: taken, carry: failed; no
//!      caller reads them)
//! Six callers (`net::copy_received`, `net::start_link`, `net::lobby_frame`,
//! `net::send_chat`, `net::chat_line_locked`, `net::take_messages`). The wait through
//! `part` (in the game `WaitForSingleObject` from Rust, `net::game_part`). Refuses nothing.
//! Shadow mode runs the original alone (the mutex). Self-test: a copy whose mutex variable
//! is the test's and whose wait goes to a noting stand-in (answering each flag setting),
//! against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG};
use crate::net::lobby::{FLAGS, answer, run_both};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, set};
use crate::net::{Part, WAIT_OBJECT, net_entry};

/// The Rust version on the variables at `v`.
pub fn lock_received_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        eax: get(v.mutex),
        ..*r
    };
    part(WAIT_OBJECT, &mut c)?;
    r.eflags = c.eflags;
    Ok(())
}

/// The Rust version.
pub fn lock_received(r: &mut Regs, part: Part) -> Result<(), Refused> {
    lock_received_with(r, &MessageVars::game(), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, lock_received)
}

/// The two wrappers' test: a copy (`len` bytes, its first bytes `entry`, the call at +6
/// to `place`) against `rust`, each flag setting the callee may leave.
pub fn test_wrapper(
    hook: &RawHook,
    address: usize,
    entry: &[u8],
    place: u32,
    rust: fn(&mut Regs, &MessageVars, Part) -> Result<(), Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let tv = TestVars::new(0);
    let copy = match tv.copy(address, 0xD, entry, &[6], &[place], 1) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    for flags in [0, ZERO_FLAG, CARRY_FLAG] {
        run_both(
            &mut t,
            &format!("the callee's flags {flags:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &[tv.region()],
            FLAGS,
            &[],
            &|| {
                set(v.mutex, 0x0BAD_0E5E);
                answer(Box::new(move |_, r| {
                    // What the callee may change it keeps; a register changed would show.
                    r.eflags |= flags;
                }));
            },
            &as_rust(|r, part| rust(r, &v, part)),
        );
    }
    t.finish()
}
