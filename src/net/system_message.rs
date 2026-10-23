//! `net::system_message` (0x41976F): a DirectPlay system message (sender 0) handled, by its
//! first dword: 3 (DPSYS_CREATEPLAYERORGROUP) the sync table to be cleared again
//! (`dword_52AD8C` 1); 5 (DPSYS_DESTROYPLAYERORGROUP) the same and, for a player (+4 is 1),
//! that player (+8) taken out of the group (`DeletePlayerFromGroup`, `IDirectPlay4A`
//! +0x1C, `g_netSendTo`); 0x31 (DPSYS_SESSIONLOST) `net::destroy_connection`; 0x101
//! (DPSYS_HOST) this machine the host (`g_netOwnRecord` bit 0x10 set, bit 1 ready cleared),
//! `dword_52AD8C` 1 and the stage item open (`dword_52AC2C` 1). Others: nothing.
//!
//! In:  ebx = the message
//! Out: every register kept (`pusha` .. `popa`); the flags of its last compare or `and`,
//!      `destroy_connection`'s for 0x31, after `DeletePlayerFromGroup` the method's (not
//!      modelled: the Rust version leaves the compare's); the one caller does not read them
//! Caller: `net::take_messages`. `destroy_connection` through `part`,
//! `DeletePlayerFromGroup` from Rust (`net::dplay`, section 15). Refuses nothing. Shadow
//! mode runs the original alone. Self-test: a copy whose variables are the test's, whose
//! call goes to a noting stand-in and whose DirectPlay is a test object, against the Rust
//! version: each kind.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{DELETE_PLAYER_FROM_GROUP, method};
use crate::net::lobby::{cmp_flags, set_sign};
use crate::net::message_vars::{MessageVars, get, get8, set, set8};
use crate::net::{DESTROY, Part, net_entry};
use rc2k_core::net::players::{A_PLAYER, System, made_host};

pub const CREATE_PLAYER_OR_GROUP: u32 = 3;
pub const DESTROY_PLAYER_OR_GROUP: u32 = 5;
pub const SESSION_LOST: u32 = 0x31;
pub const HOST: u32 = 0x101;

/// The Rust version on the variables at `v`.
pub fn system_message_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    let kind = get(r.ebx);
    // Converted: `rc2k_core::net::players` (`System`, `made_host`).
    match System::of(kind) {
        System::Created => {
            cmp_flags(r, kind, CREATE_PLAYER_OR_GROUP);
            set(v.sync_first, 1);
        }
        System::Destroyed => {
            set(v.sync_first, 1);
            let what = get(r.ebx + 4);
            cmp_flags(r, what, A_PLAYER);
            if what == A_PLAYER {
                unsafe {
                    method(
                        get(v.dplay),
                        DELETE_PLAYER_FROM_GROUP,
                        &[get(v.send_to), get(r.ebx + 8)],
                    )
                };
            }
        }
        System::SessionLost => {
            let mut c = Regs { eax: kind, ..*r };
            part(DESTROY, &mut c)?;
            r.eflags = c.eflags;
        }
        System::Host => {
            let own = made_host(get8(v.own));
            set8(v.own, own);
            r.set_carry(false);
            r.set_zero(own == 0);
            set_sign(r, own as i8 as i32);
            set(v.sync_first, 1);
            set(v.stage_item, 1);
        }
        System::Other => cmp_flags(r, kind, HOST),
    }
    Ok(())
}

/// The Rust version.
pub fn system_message(r: &mut Regs, part: Part) -> Result<(), Refused> {
    system_message_with(r, &MessageVars::game(), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, system_message)
}
