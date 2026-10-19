//! `net::choose_session` (0x417726): the list screen of the sessions (`net::sessions_frame`
//! through `screens::run_frame`), as `net::choose_connection`'s, and once a second
//! (`system::tick_count` against `dword_52A4C4`) the sessions listed again
//! (`net::find_sessions_regs`, with `g_netPosting` 0 for it): the highlighted session's GUID
//! kept first (`unk_52AD00`) and found again after (`system::same_guid`), the highlight on
//! it (the top moved by as much, from 0 when sixteen or fewer, not below 0), or at the top
//! when it is gone; a failed listing gives -2. With no session listed only Escape, 'R' and
//! the lexicon's key work.
//!
//! In:  nothing
//! Out: eax = the session chosen (`g_listChoice`: -1 Escape, -2 the listing failed), carry
//!      set if negative (else the flags of `test eax, eax`); the rest kept
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game). The
//! whole screen runs inside it: ungated. Shadow mode runs the original alone. Self-test: a
//! copy whose calls go to stand-ins (the keys and the clock scripted, the listings giving
//! records of a test table), against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva};
use crate::net::choose_connection::{chosen, list_move};
use crate::net::lobby::{
    CHOICE, CHOICE_BEFORE, CLOCK, CLOCK_A, FLAGS, FRAME_CLOCK, JOINED, KEY_CLOCK, LIST_TAKE_KEY,
    MAYBE_LEXICON, MOVE_CLOCK, POSTING, Part, QUITTING, RUN_FRAME, SAME_GUID, SCREENSHOT,
    SCREENSHOTS_ON, SESSIONS, Saved, TICK, a, answer, dispatch_lobby, dword, g, game, movsb, p,
    region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::choose_session", rva(0x417726), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const FRAME: u32 = 0x41790E; // net::sessions_frame
const SESSION_ENTRY: u32 = 0x418553; // net::session_entry
const FIND_SESSIONS: u32 = 0x41771E; // net::find_sessions_regs

/// The sessions listed again, the highlight kept on the same one. Err(()) for a failed
/// listing.
fn refresh(r: &mut Regs, part: Part) -> Result<bool, Refused> {
    r.ebx = g(SESSIONS);
    if r.ebx != 0 {
        r.eax = g(CHOICE);
        part(SESSION_ENTRY, r)?;
        r.esi = r.eax + 0x40;
        r.edi = a(JOINED);
        movsb(r.edi, r.esi, 0x10);
        r.esi += 0x10;
        r.edi += 0x10;
        r.ecx = 0;
    }
    let posting = g(POSTING);
    p(POSTING, 0);
    part(FIND_SESSIONS, r)?;
    p(POSTING, posting);
    if r.carry() {
        return Ok(false);
    }
    if r.ebx == 0 || g(SESSIONS) == 0 {
        // loc_4177F1: xor ecx, ecx; xor edx, edx
        r.ecx = 0;
        r.edx = 0;
    } else {
        r.ebx = a(JOINED);
        r.ecx = 0;
        loop {
            r.eax = r.ecx;
            part(SESSION_ENTRY, r)?;
            r.eax = r.eax.wrapping_add(0x40);
            part(SAME_GUID, r)?;
            if r.eflags & ZERO_FLAG != 0 {
                r.edx = 0;
                if g(SESSIONS) > 0x10 {
                    r.edx = g(CHOICE_BEFORE).wrapping_sub(g(CHOICE)).wrapping_add(r.ecx);
                    if (r.edx as i32) < 0 {
                        r.edx = 0;
                    }
                }
                break;
            }
            r.ecx = r.ecx.wrapping_add(1);
            if r.ecx == g(SESSIONS) {
                r.ecx = 0;
                r.edx = 0;
                break;
            }
        }
    }
    p(CHOICE, r.ecx);
    p(CHOICE_BEFORE, r.edx);
    Ok(true)
}

/// The Rust version: the registers it leaves.
pub fn choose_session(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(CLOCK);
    p(MOVE_CLOCK, r.eax);
    p(KEY_CLOCK, r.eax);
    p(CHOICE, 0);
    p(CHOICE_BEFORE, 0);
    part(TICK, &mut r)?;
    p(CLOCK_A, r.eax);
    loop {
        r.eax = game(FRAME);
        part(RUN_FRAME, &mut r)?;
        part(TICK, &mut r)?;
        r.ebx = r.eax;
        r.eax = r.eax.wrapping_sub(g(CLOCK_A));
        if (r.eax.wrapping_sub(1000) as i32) >= 0 {
            p(CLOCK_A, r.ebx);
            if !refresh(&mut r, part)? {
                p(CHOICE, 0xFFFF_FFFE);
                return Ok(chosen(entry, &r));
            }
        }
        r.eax = g(CLOCK);
        p(FRAME_CLOCK, r.eax);
        part(LIST_TAKE_KEY, &mut r)?;
        let key = r.eax as u8;
        let any = g(SESSIONS) != 0;
        if any && list_move(&mut r, key, g(SESSIONS)) {
            r.ebx = g(FRAME_CLOCK);
            p(MOVE_CLOCK, r.ebx);
        } else if any && key == 0x1C {
            r.eax = g(CHOICE);
            return Ok(chosen(entry, &r));
        } else if key == 1 {
            break;
        } else {
            if g(SCREENSHOTS_ON) != 0 && key == 0x52 {
                part(SCREENSHOT, &mut r)?;
            }
            part(MAYBE_LEXICON, &mut r)?;
        }
        if g(QUITTING) == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(chosen(entry, &r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, choose_session);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        TICK,
        RUN_FRAME,
        SESSION_ENTRY,
        FIND_SESSIONS,
        SAME_GUID,
        LIST_TAKE_KEY,
        SCREENSHOT,
        MAYBE_LEXICON,
    ];
    // pusha; mov eax, dword_518998
    let copy = match crate::net::lobby::copy(
        0x417726,
        0x1E8,
        &[0x60, 0xA1, 0x98, 0x89, 0x51, 0x00],
        &[
            0x24, 0x34, 0x39, 0x66, 0x8B, 0xB3, 0xBB, 0x10A, 0x19D, 0x1A2,
        ],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut table = vec![0u8; 0x68 * 40];
    let tab = table.as_mut_ptr() as u32;
    let all = [
        dword(SESSIONS),
        dword(CHOICE),
        dword(CHOICE_BEFORE),
        dword(KEY_CLOCK),
        dword(FRAME_CLOCK),
        dword(MOVE_CLOCK),
        dword(SCREENSHOTS_ON),
        dword(CLOCK_A),
        dword(POSTING),
        region(JOINED, 0x10),
        (tab, 0x68 * 40),
    ];
    let saved = Saved::new(&all);
    // (what, sessions at first, after each listing, the listing fails, keys, the clock's
    // step a frame, which session's GUID moves where (the old index's GUID given to index))
    type Case = (&'static str, u32, u32, bool, Vec<u32>, u32, u32);
    let cases: Vec<Case> = vec![
        ("Escape", 2, 2, false, vec![], 10, 0),
        ("Enter at once", 2, 2, false, vec![0x1C], 10, 0),
        ("none listed", 0, 0, false, vec![0x6B, 0x1C, 0x52], 10, 0),
        (
            "refreshed, kept",
            3,
            3,
            false,
            vec![0x6B, 0x6B, 0x10, 0x10, 0x1C],
            600,
            0,
        ),
        (
            "refreshed, moved",
            20,
            30,
            false,
            [vec![0x6B; 18], vec![0x10, 0x1C]].concat(),
            700,
            5,
        ),
        (
            "refreshed, gone",
            4,
            2,
            false,
            vec![0x6B, 0x6B, 0x6B, 0x10, 0x10, 0x1C],
            900,
            9,
        ),
        (
            "the listing fails",
            3,
            3,
            true,
            vec![0x10, 0x10, 0x10],
            1000,
            0,
        ),
    ];
    for (what, first, after, fails, keys, step, shift) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(SESSIONS, first);
                p(SCREENSHOTS_ON, 0);
                p(POSTING, 1);
                // Record n's GUID is n, or n - shift after a listing.
                let set = move |listed: bool| unsafe {
                    for n in 0..40u32 {
                        let id = if listed { n.wrapping_sub(shift) } else { n };
                        ((tab + 0x68 * n + 0x40) as *mut [u32; 4]).write_unaligned([id, 0, 0, 0]);
                    }
                };
                set(false);
                let mut take = crate::net::settings_loop::keys(keys.clone());
                let mut clock = 0x10000u32;
                answer(Box::new(move |place, r| {
                    take(place, r);
                    match place {
                        TICK => {
                            clock += step;
                            r.eax = clock;
                        }
                        SESSION_ENTRY => r.eax = tab + 0x68 * (r.eax % 40),
                        SAME_GUID => r.set_zero(crate::system::same_guid::same_guid(r.eax, r.ebx)),
                        FIND_SESSIONS => {
                            r.set_carry(fails);
                            p(SESSIONS, after);
                            set(true);
                        }
                        _ => {}
                    }
                }));
            },
            &choose_session,
        );
    }
    drop(saved);
    let _ = &mut table;
    t.finish()
}
