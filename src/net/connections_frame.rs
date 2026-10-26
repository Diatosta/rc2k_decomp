//! `net::connections_frame` (0x4175C7): the frame of `net::choose_connection`: on the locked
//! surface the background, the title (string 0x14E, `stru_51E9A4` at 0x316, 0x6F), and from
//! y 0x9B sixteen of the connections' names from `g_listChoiceBefore` (the list's top) in
//! `font`, the highlighted one in `unk_51E644` in the blink's lit half
//! (`net::connection_entry`, `text::draw_text` at 0x190, each the font's line height lower);
//! unlocked and shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Called by `screens::run_frame` for `net::choose_connection`. Its calls through `part`.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins, against the Rust version: the lock failing, lists of 0, 3 and 20 from tops 0
//! and 4, highlighted and blinking.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest};
use crate::net::connecting::STATUS_FONT;
use crate::net::lobby::{
    CHOICE, CHOICE_BEFORE, CONNECTIONS, COPY_PICTURE, DRAW_STRING, DRAW_TEXT, FONT, FRAME_CLOCK,
    LOCK, MOVE_CLOCK, Part, SHOW_FRAME, Saved, UNLOCK, a, answer, dword, g, lobby_entry, p,
    run_both,
};
use rc2k_core::net::screens;

const CONNECTION_ENTRY: u32 = 0x418530; // net::connection_entry
const LIT: usize = 0x51E644; // unk_51E644

/// A list screen's frame: the title, then up to sixteen names from the list's top, each the
/// record (`entry`) of an item below the count. `lit_ebx` is what ebx gets for the
/// highlighted item. Shared with `net::sessions_frame`.
pub fn list_frame(
    entry: &Regs,
    title: u32,
    count: usize,
    record: u32,
    lit_ebx: usize,
    part: Part,
) -> Result<Regs, Refused> {
    let mut r = *entry;
    let done = |r: &Regs| {
        Ok(Regs {
            eflags: r.eflags,
            ..*entry
        })
    };
    part(LOCK, &mut r)?;
    if r.carry() {
        return done(&r);
    }
    part(COPY_PICTURE, &mut r)?;
    r.ebp = 0x6F;
    r.eax = title;
    r.ebx = a(STATUS_FONT);
    r.ecx = 0x316;
    r.edx = r.ebp;
    r.ebp += 0x2C;
    part(DRAW_STRING, &mut r)?;
    // Converted: `rc2k_core::net::screens` (`LIST_ROWS`, `listed`, `blink_lit`).
    for row in 0..screens::LIST_ROWS {
        let Some(item) = screens::listed(row, g(CHOICE_BEFORE), g(count)) else {
            continue;
        };
        r.esi = item;
        r.eax = a(FONT);
        if item == g(CHOICE) {
            r.ebx = a(lit_ebx);
            let (frame, moved) = (g(FRAME_CLOCK), g(MOVE_CLOCK));
            r.ecx = frame.wrapping_sub(moved);
            if screens::blink_lit(frame, moved) {
                r.eax = a(LIT);
            }
        }
        std::mem::swap(&mut r.eax, &mut r.esi);
        part(record, &mut r)?;
        std::mem::swap(&mut r.eax, &mut r.esi);
        r.ebx = r.esi;
        r.ecx = 0x190;
        r.edx = r.ebp;
        r.ebp = r
            .ebp
            .wrapping_add(unsafe { ((r.eax + 8) as *const u32).read_unaligned() });
        part(DRAW_TEXT, &mut r)?;
    }
    r.esi = screens::LIST_ROWS;
    part(UNLOCK, &mut r)?;
    part(SHOW_FRAME, &mut r)?;
    done(&r)
}

/// The Rust version: the registers it leaves (the entry's).
pub fn connections_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    list_frame(
        entry,
        screens::CONNECTIONS_TITLE,
        CONNECTIONS,
        CONNECTION_ENTRY,
        0x51E704,
        part,
    )
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, connections_frame)
}

/// The test of the two list frames.
#[allow(clippy::too_many_arguments)] // one test's whole description
pub fn list_frame_test(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    count: usize,
    record: u32,
    rust: fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let places = [
        LOCK,
        COPY_PICTURE,
        DRAW_STRING,
        record,
        DRAW_TEXT,
        UNLOCK,
        SHOW_FRAME,
    ];
    let copy = match crate::net::lobby::copy(address, len, entry, calls, &places) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        dword(count),
        dword(CHOICE),
        dword(CHOICE_BEFORE),
        dword(FRAME_CLOCK),
        dword(MOVE_CLOCK),
    ];
    let saved = Saved::new(&all);
    // (the lock failing, the count, the top, the highlighted, the blink)
    let cases: [(bool, u32, u32, u32, u32); 6] = [
        (true, 3, 0, 0, 0),
        (false, 0, 0, 0, 0),
        (false, 3, 0, 1, 0),
        (false, 3, 0, 1, 8),
        (false, 20, 4, 9, 0),
        (false, 20, 0, 0x30, 0),
    ];
    for (fails, n, top, choice, clock) in cases {
        run_both(
            &mut t,
            &format!(
                "lock failing {fails}, {n} listed, top {top}, highlighted {choice}, clock {clock}"
            ),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                p(count, n);
                p(CHOICE_BEFORE, top);
                p(CHOICE, choice);
                p(FRAME_CLOCK, 0x300 + clock);
                p(MOVE_CLOCK, 0x300);
                answer(Box::new(move |place, r| {
                    if place == LOCK {
                        r.set_carry(fails);
                    }
                    if place == record {
                        r.eax = 0x0070_0000 + 0x100 * r.eax;
                    }
                    if place == DRAW_TEXT {
                        r.eax = 0;
                        r.ebx += 5;
                    }
                }));
            },
            &rust,
        );
    }
    drop(saved);
    t.finish()
}
