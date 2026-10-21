//! `net::setting_key` (0x416C76): a key for the setting being edited (`net::edit_key`); when
//! that ends the editing: the player's name (item 1) left empty gets its old text back
//! (`byte_95EAF8`, 64 bytes); the most players (item 2) read from its text
//! (`settings::parse_number`), held to 2 .. 8 (signed), and to 4 with `g_noticeKind` 2, into
//! `dword_52ABD0`.
//!
//! In:  al = the key
//! Out: as `net::edit_key` leaves them while editing goes on; after it, eax = the item (or
//!      the number), the copy's esi, edi, ecx; no `pusha`; the flags of the last test (no
//!      caller reads them)
//! Caller: `net::settings_loop`. Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins (the editing ended or not, the
//! number given), against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva};
use crate::net::lobby::{
    CHOICE, EDIT_SAVED, MACHINE_NAME, MOST_PLAYERS, NOTICE_KIND, NUMBER_TEXT, PARSE_NUMBER, Part,
    Saved, a, answer, dispatch_lobby, dword, fill, g, gb, movsb, p, pb, region, run_both,
};
use rc2k_core::net::chat;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::setting_key", rva(0x416C76), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EDIT_KEY: u32 = 0x416E43; // net::edit_key

/// The Rust version: the registers it leaves.
pub fn setting_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(EDIT_KEY, &mut r)?;
    if r.eflags & ZERO_FLAG == 0 {
        return Ok(r);
    }
    r.eax = g(CHOICE);
    // Converted: `rc2k_core::net::chat` (`NAME`, `MOST_PLAYERS`, `most_players`).
    match r.eax {
        chat::NAME => {
            r.esi = a(EDIT_SAVED);
            r.edi = a(MACHINE_NAME);
            if gb(MACHINE_NAME) == 0 {
                movsb(r.edi, r.esi, 0x40);
                r.esi += 0x40;
                r.edi += 0x40;
                r.ecx = 0;
            }
        }
        chat::MOST_PLAYERS => {
            r.eax = a(NUMBER_TEXT);
            part(PARSE_NUMBER, &mut r)?;
            r.eax = chat::most_players(r.eax, gb(NOTICE_KIND));
            p(MOST_PLAYERS, r.eax);
        }
        _ => {}
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, setting_key);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call sub_416E43
    let copy = match crate::net::lobby::copy(
        0x416C76,
        0x6A,
        &[0xE8, 0xC8, 0x01, 0x00, 0x00],
        &[0x0, 0x36],
        &[EDIT_KEY, PARSE_NUMBER],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        dword(CHOICE),
        dword(MOST_PLAYERS),
        region(MACHINE_NAME, 0x40),
        region(EDIT_SAVED, 0x40),
        region(NOTICE_KIND, 1),
    ];
    let saved = Saved::new(&all);
    // (still editing, the item, the name empty, the number, the notice kind)
    let cases: [(bool, u32, bool, u32, u8); 9] = [
        (true, 1, true, 0, 0),
        (false, 1, true, 0, 0),
        (false, 1, false, 0, 0),
        (false, 2, false, 1, 0),
        (false, 2, false, 6, 0),
        (false, 2, false, 9, 0),
        (false, 2, false, 6, 2),
        (false, 2, false, 0xFFFF_FFFF, 2),
        (false, 0, false, 0, 0),
    ];
    for (editing, item, empty, number, kind) in cases {
        run_both(
            &mut t,
            &format!(
                "editing {editing}, item {item}, empty {empty}, number {number:#x}, kind {kind}"
            ),
            copy.addr(),
            Regs {
                eax: 0x1E,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                p(CHOICE, item);
                p(MOST_PLAYERS, 3);
                fill(MACHINE_NAME, 0x40, 6);
                fill(EDIT_SAVED, 0x40, 7);
                if empty {
                    pb(MACHINE_NAME, 0);
                } else {
                    pb(MACHINE_NAME, b'A');
                }
                pb(NOTICE_KIND, kind);
                answer(Box::new(move |place, r| match place {
                    EDIT_KEY => r.set_zero(!editing),
                    _ => r.eax = number,
                }));
            },
            &setting_key,
        );
    }
    drop(saved);
    t.finish()
}
