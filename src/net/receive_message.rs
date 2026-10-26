//! `net::receive_message` (0x419722): one message taken from DirectPlay: its size
//! (`dword_5AB280`) set to 0x100, `Receive` (`IDirectPlay4A` +0x64: the sender into
//! `dword_5AB278`, the addressee into `unk_5AB27C`, flags 1 DPRECEIVE_ALL, the bytes into
//! `unk_52A968`) through `g_dplay`; the size made 0 if it failed (no message, or one too
//! big for the buffer).
//!
//! In:  nothing
//! Out: eax = the sender (0: a system message), ebx = the bytes (`unk_52A968`), ecx = the
//!      size (0: none); edx kept (pushed and popped), esi, edi, ebp kept (the method's);
//!      the flags of `test eax, eax` on the method's result
//! Caller: `net::take_messages`. `Receive` from Rust (`net::dplay`, section 15). Refuses
//! nothing (no check of `g_dplay`: its caller makes it). Shadow mode runs the original
//! alone (a message is taken). Self-test: a copy whose variables are the test's and whose
//! DirectPlay is a test object writing a message, against the Rust version: taken, none.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{RECEIVE, method};
use crate::net::lobby::test_eax;
use crate::net::message_vars::{MessageVars, get, set};
use crate::net::{Part, net_entry};

/// The buffer's size.
pub const ROOM: u32 = 0x100;
/// DPRECEIVE_ALL.
const RECEIVE_ALL: u32 = 1;

/// The Rust version on the variables at `v`.
pub fn receive_message_with(r: &mut Regs, v: &MessageVars) {
    set(v.size, ROOM);
    let result = unsafe {
        method(
            get(v.dplay),
            RECEIVE,
            &[v.from, v.to, RECEIVE_ALL, v.data, v.size],
        )
    };
    if result != 0 {
        set(v.size, 0);
    }
    r.eax = result;
    test_eax(r);
    r.eax = get(v.from);
    r.ebx = v.data;
    r.ecx = get(v.size);
}

/// The Rust version.
pub fn receive_message(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    receive_message_with(r, &MessageVars::game());
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, receive_message)
}
