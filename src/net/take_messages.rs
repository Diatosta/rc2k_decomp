//! `net::take_messages` (0x4194CC): the receiving thread's turn: while there is a DirectPlay
//! object (`g_dplay`) and `net::receive_message` gives a message (ecx not 0), the messages
//! taken (`dword_52A91C` counted) and handled:
//! - sender 0: DirectPlay's system message (`net::system_message`);
//! - a sender not among the players (`g_netReceived` +0x20, `g_netPlayers`; the first
//!   player compared even with none): dropped;
//! - first byte 0x22: a chat line (`net::chat_line_locked`, eax the player, ebx the text
//!   after the byte; a NUL put at the end first, at the bytes + ecx, past the buffer for a
//!   message of 0x100 bytes: on `g_netSendBuffer`'s first byte, section 16);
//! - 0x40 / 0x50 (two bytes, the second the round `g_netSync`; on a machine not the host,
//!   `g_netOwnRecord` bit 0x10 clear): answered 0x41 / 0x51 with the round
//!   (`word_52A916`, `net::dplay_send` to the sender, flags 0) and `g_netAllRaced` /
//!   `g_netRaceOver` set;
//! - 0x41 / 0x51 (two bytes, the round; on the host): the player's byte of `g_netSynced` 1;
//! - bit 0x80 (a race message), 1 .. 0xFF bytes: the bytes after the first, with the time
//!   (`system::tick_count`), put into the player's block (`net::store_received`) under the
//!   lock (`net::lock_received` .. `net::unlock_received`);
//! - others dropped.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of the end's compare (zero set,
//!      carry and sign clear)
//! Caller: `net::receive_thread` (with a network session, when its event is signalled).
//! The registers go on from call to call as in the original. Its calls through `part` (in
//! the game the Rust versions). Refuses (before anything) more than 16 players (the
//! blocks' room). A loop target in its first five bytes: private copy (the whole function,
//! its nine calls relocated), manual patch. Shadow mode runs the original alone (messages
//! are taken). Self-test: a copy whose variables are the test's and whose calls go to
//! noting stand-ins (the receive one giving a script of messages), against the Rust
//! version: no object, no message, each kind of message, on the host and not.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{FLAGS, answer, run_both, set_sign};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, get8, set, set8};
use crate::net::{
    CHAT_LINE, LOCK, Part, RECEIVE, RECEIVED_LEN, SEND, STORE, SYSTEM_MESSAGE, TICK, UNLOCK,
    dispatch_net, net_entry,
};
use rc2k_core::net::messages::{self, Kind};
use std::sync::atomic::AtomicUsize;

/// The calls' offsets (receive, system message, chat line, send twice, tick, lock, store,
/// unlock).
const CALLS: [usize; 9] = [0x0E, 0x26, 0x5A, 0x9A, 0x129, 0x18D, 0x195, 0x19A, 0x19F];

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("net::take_messages", rva(0x4194CC), thunk, &ORIGINAL)
    .private_copy(0x1AB)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The players the blocks have room for.
pub const MOST_PLAYERS: u32 = 16;

/// The Rust version on the variables at `v`.
pub fn take_messages_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    if get(v.players) > MOST_PLAYERS {
        return Err(BadInput("take_messages: more than 16 players").into());
    }
    let mut c = *r;
    while get(v.dplay) != 0 {
        part(RECEIVE, &mut c)?;
        if c.ecx == 0 {
            break;
        }
        set(v.taken, get(v.taken).wrapping_add(1));
        if c.eax == 0 {
            part(SYSTEM_MESSAGE, &mut c)?;
            continue;
        }
        c.edi = v.received;
        c.edx = 0;
        let found = loop {
            if c.eax == get(c.edi + 0x20) {
                break true;
            }
            c.edi = c.edi.wrapping_add(RECEIVED_LEN);
            c.edx += 1;
            if c.edx >= get(v.players) {
                break false;
            }
        };
        if !found {
            continue;
        }
        c.eax = get8(c.ebx) as u32;
        // Converted: `rc2k_core::net::messages` (`Kind`, `this_round`, `answers_call`,
        // `notes_answer`, `answer`, `race_fits`).
        let own_state = get8(v.own);
        let round = |c: &mut Regs| {
            messages::this_round(c.ecx, || {
                c.eax = get8(c.ebx + 1) as u32;
                (c.eax as u8, get8(v.sync))
            })
        };
        match Kind::of(c.eax as u8) {
            Kind::Chat => {
                c.eax = c.edx;
                set8(c.ecx.wrapping_add(c.ebx), 0);
                c.ebx = c.ebx.wrapping_add(1);
                part(CHAT_LINE, &mut c)?;
            }
            Kind::Call { reply, ready } => {
                let same = round(&mut c);
                if !messages::answers_call(same, own_state) {
                    continue;
                }
                c.eax = get(c.edi + 0x20);
                c.ebx = v.answer;
                c.ecx = (c.ecx & 0xFFFF_0000) | messages::answer(reply, get8(v.sync)) as u32;
                unsafe { (c.ebx as *mut u16).write_unaligned(c.ecx as u16) };
                c.ecx = 2;
                c.edx = 0;
                part(SEND, &mut c)?;
                set(if ready { v.all_raced } else { v.race_over }, 1);
            }
            Kind::Answer => {
                let same = round(&mut c);
                if !messages::notes_answer(same, own_state) {
                    continue;
                }
                set8(v.synced.wrapping_add(c.edx), 1);
            }
            Kind::Race => {
                c.ebx = c.ebx.wrapping_add(1);
                c.ecx = c.ecx.wrapping_sub(1);
                if !messages::race_fits(c.ecx) {
                    continue;
                }
                let first = c.eax;
                part(TICK, &mut c)?;
                c.edx = c.eax;
                c.eax = first;
                part(LOCK, &mut c)?;
                part(STORE, &mut c)?;
                part(UNLOCK, &mut c)?;
            }
            Kind::Other => {}
        }
    }
    // popa; the flags of `cmp g_dplay, 0` or `test ecx, ecx` on 0.
    r.set_carry(false);
    r.set_zero(true);
    set_sign(r, 0);
    Ok(())
}

/// The Rust version.
pub fn take_messages(r: &mut Regs, part: Part) -> Result<(), Refused> {
    take_messages_with(r, &MessageVars::game(), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, take_messages);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, take_messages)
}

/// For the Rust receiving thread: the Rust version (counted as a call of the hook), a
/// refusal handed on as the hook's.
pub fn from_rust(r: &mut Regs) {
    HOOK.count_call();
    let mut g = *r;
    match take_messages(&mut g, &mut crate::net::game_part) {
        Ok(()) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const PLAYERS: u32 = 3;
    let mut blocks = vec![0u8; (PLAYERS * RECEIVED_LEN) as usize];
    let received = blocks.as_mut_ptr() as u32;
    let tv = TestVars::new(received);
    // pusha; cmp g_dplay, 0
    let entry = [0x60, 0x83, 0x3D, 0x88, 0xB2, 0x5A, 0x00, 0x00];
    let copy = match tv.copy(
        0x4194CC,
        0x1AB,
        &entry,
        &CALLS,
        &[
            RECEIVE,
            SYSTEM_MESSAGE,
            CHAT_LINE,
            SEND,
            TICK,
            LOCK,
            STORE,
            UNLOCK,
        ],
        20,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    const SLOT: u32 = 0x110;
    const MESSAGES: usize = 24;
    let mut buffer = vec![0u8; SLOT as usize * MESSAGES];
    let buf = buffer.as_mut_ptr() as u32;
    let ids = [0x0000_7001u32, 0x0000_7002, 0x0000_7003];
    const ROUND: u8 = 0x37;
    // (the sender, the bytes; the size given is their length unless `sizes` says)
    type Message = (u32, Vec<u8>, Option<u32>);
    let race: Vec<u8> = (0..0x20u8).map(|k| k ^ 0x93).collect();
    let mut chat = b"\x22hello".to_vec();
    chat.resize(6, 0);
    let mut long_race = vec![0x85u8; 0x100];
    long_race[1] = 0x11;
    let mixed: Vec<Message> = vec![
        (0, vec![3, 0, 0, 0], Some(0x10)),
        (0x0000_7999, vec![0x85, 1, 2], None),
        (ids[1], chat.clone(), None),
        (ids[2], vec![0x40, ROUND], None),
        (ids[0], vec![0x40, ROUND ^ 1], None),
        (ids[0], vec![0x40, ROUND, 0], None),
        (ids[1], vec![0x41, ROUND], None),
        (ids[2], vec![0x41, ROUND], None),
        (ids[0], vec![0x50, ROUND], None),
        (ids[2], vec![0x51, ROUND], None),
        (ids[1], vec![0x51, ROUND ^ 2], None),
        (ids[1], race.clone(), None),
        (ids[2], vec![0x81], None),
        (ids[0], long_race.clone(), None),
        (ids[0], long_race[..0xFF].to_vec(), None),
        (ids[2], vec![0x10, 1, 2, 3], None),
        (ids[1], vec![0xFF; 0x100], Some(0x100)),
        (
            ids[1],
            {
                let mut c = vec![0x22u8; 0x100];
                c[1] = b'x';
                c
            },
            None,
        ),
    ];
    let cases: Vec<(&str, u32, u32, u8, Vec<Message>)> = vec![
        ("no object", 0, PLAYERS, 0, mixed.clone()),
        ("no message", 1, PLAYERS, 0, Vec::new()),
        ("not the host", 1, PLAYERS, 0x01, mixed.clone()),
        ("the host", 1, PLAYERS, 0x11, mixed.clone()),
        ("no players", 1, 0, 0x00, mixed.clone()),
        ("one player", 1, 1, 0x00, mixed.clone()),
    ];
    for (what, object, players, own, script) in cases {
        let script = std::sync::Arc::new(script);
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &[tv.region(), (buf, SLOT as usize * MESSAGES)],
            FLAGS,
            &[],
            &|| {
                set(v.dplay, object);
                set(v.taken, 0x0000_0100);
                set(v.players, players);
                set8(v.sync, ROUND);
                set8(v.own, own);
                set(v.answer, 0xAAAA_AAAA);
                set(v.all_raced, 0);
                set(v.race_over, 0);
                set(v.synced, 0);
                set(v.synced + 4, 0);
                for (k, id) in ids.iter().enumerate() {
                    set(received + k as u32 * RECEIVED_LEN + 0x20, *id);
                }
                for k in 0..SLOT * MESSAGES as u32 {
                    set8(buf + k, (k as u8).wrapping_mul(7));
                }
                let script = script.clone();
                let mut next = 0usize;
                let mut clock = 0x00C0_0000u32;
                answer(Box::new(move |place, r| match place {
                    RECEIVE => match script.get(next) {
                        Some((from, bytes, size)) => {
                            let at = buf + next as u32 * SLOT;
                            for (k, &b) in bytes.iter().enumerate() {
                                set8(at + k as u32, b);
                            }
                            r.eax = *from;
                            r.ebx = at;
                            r.ecx = size.unwrap_or(bytes.len() as u32);
                            next += 1;
                        }
                        None => {
                            r.eax = 0x0000_7777;
                            r.ebx = buf;
                            r.ecx = 0;
                        }
                    },
                    TICK => {
                        clock += 0x11;
                        r.eax = clock;
                    }
                    _ => {}
                }));
            },
            &as_rust(|r, part| take_messages_with(r, &v, part)),
        );
    }
    drop(blocks);
    drop(buffer);
    t.finish()
}
