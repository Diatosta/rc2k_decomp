//! `net::host_session` (0x418B13): "NET: Host" and the machine's name logged; with
//! DirectPlay's object there, the session kept (`unk_52AD00`: no GUID yet, the name
//! `g_machineName`, the most players `dword_52ABD0`, `unk_52ABD4` as its user data) and
//! described (`dword_5AB2AC`: zeroed, size 0x50, flags 0x44, the game's GUID, the most
//! players, the name, the user data) and made (`Open` with DPOPEN_CREATE, or with
//! `g_netPosting` 1 posted: the code at 0x418C5A); "NET: Open" and its result logged, the
//! window restored (`system::restore_window`); then `dword_52ACFC` 1, the player made
//! (`net::create_player`), the group made (`net::create_group`), "NET: Host complete".
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if something failed, else the
//!      flags of `test eax, eax`
//! Caller: `net::hosting`. Its calls through `part`; `Open` from Rust (`net::dplay`; posted,
//! `posted_open_host`). Shadow mode runs the original alone. Self-test: a copy whose calls go
//! to stand-ins and whose DirectPlay is a test object, against the Rust version: no object,
//! posting, each failure, done.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::Regs;
use crate::net::dplay::{OPEN, call};
use crate::net::find_sessions::zeroed_description;
use crate::net::lobby::{
    APP_GUID, DPLAY, JOINED, JOINED_MOST, JOINED_NAME, JOINED_PASSWORD, LOG_HEX_L2_NL, LOG_L2,
    LOG_L2_NL, MACHINE_NAME, MOST_PLAYERS, POST_CALL, POSTING, Part, RESTORE_WINDOW, SESSION_DESC,
    SESSION_OPEN, SESSION_PASSWORD, ZERO_MEMORY, a, cmp_flags, g, game, lobby_entry, log_text,
    movsb, p, test_eax,
};
use crate::net::types::DpSessionDesc;

const HOST: usize = 0x5C2EDA; // "NET: Host"
pub const OPENED: usize = 0x5C2EE4; // "NET: Open"
const COMPLETE: usize = 0x5C2EEE; // "NET: Host complete"
pub const POSTED_CODE: u32 = 0x418C5A; // the piece posted: `Open` to create
const CREATE_PLAYER: u32 = 0x4190FB; // net::create_player
const CREATE_GROUP: u32 = 0x4192EA; // net::create_group
const OPEN_CREATE: u32 = 2; // DPOPEN_CREATE
pub const OPEN_JOIN: u32 = 1; // DPOPEN_JOIN

/// `Open(dword_5AB2AC, flags)`.
pub unsafe fn open(flags: u32) -> u32 {
    unsafe { call(DPLAY, OPEN, &[a(SESSION_DESC), flags]) }
}

/// The code at 0x418C5A as the receiving thread calls it when it is posted.
pub extern "C" fn posted_open_host() -> u32 {
    unsafe { open(OPEN_CREATE) }
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn host_session(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2, a(HOST), &r, part)?;
    log_text(LOG_L2_NL, a(MACHINE_NAME), &r, part)?;
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    let mut c = Regs {
        ebx: a(JOINED),
        ecx: 0x10,
        ..r
    };
    part(ZERO_MEMORY, &mut c)?;
    movsb(a(JOINED_NAME), a(MACHINE_NAME), 0x40);
    r.eax = g(MOST_PLAYERS); // mov eax, g_netMostPlayers
    p(JOINED_MOST, r.eax);
    movsb(a(JOINED_PASSWORD), a(SESSION_PASSWORD), 0x10);
    zeroed_description(&r, part)?;
    p(SESSION_DESC + 4, 0x44);
    movsb(a(SESSION_DESC) + 0x18, a(APP_GUID), 0x10);
    p(SESSION_DESC + 0x28, g(MOST_PLAYERS));
    p(SESSION_DESC + 0x30, a(MACHINE_NAME));
    movsb(
        a(SESSION_DESC) + field!(DpSessionDesc, user),
        a(SESSION_PASSWORD),
        0x10,
    );
    finish(
        &mut r,
        OPEN_CREATE,
        POSTED_CODE,
        CREATE_GROUP,
        COMPLETE,
        part,
    )?;
    Ok(left(&r))
}

/// The session opened (directly, or posted as the code at `posted`), "NET: Open" and its
/// result logged, the window restored; `dword_52ACFC` 1, the player made, then `then` (the
/// group made or joined) and `done` logged. Shared with `net::join_session`.
pub fn finish(
    r: &mut Regs,
    flags: u32,
    posted: u32,
    then: u32,
    done: usize,
    part: Part,
) -> Result<(), Refused> {
    if g(POSTING) == 1 {
        r.eax = game(posted);
        part(POST_CALL, r)?;
    } else {
        r.eax = unsafe { open(flags) };
    }
    log_text(LOG_L2, a(OPENED), r, part)?;
    let mut l = *r;
    part(LOG_HEX_L2_NL, &mut l)?;
    part(RESTORE_WINDOW, r)?;
    test_eax(r);
    if r.eax != 0 {
        r.set_carry(true);
        return Ok(());
    }
    p(SESSION_OPEN, 1);
    part(CREATE_PLAYER, r)?;
    if r.carry() {
        return Ok(());
    }
    part(then, r)?;
    if r.carry() {
        return Ok(());
    }
    log_text(LOG_L2_NL, a(done), r, part)?;
    test_eax(r);
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, host_session)
}
