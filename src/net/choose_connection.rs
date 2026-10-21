//! `net::choose_connection` (0x4174C5): the list screen of the DirectPlay connections
//! (`net::connections_frame` through `screens::run_frame`): the clocks from `dword_518998`,
//! the top and the highlight 0; each frame a key (`menu::list_take_key`): up (0x66) and down
//! (0x6B) move the highlight (within 0 .. the count - 1, signed) and the top with it
//! (sixteen rows), Enter (0x1C) chooses, Escape (1) or the game closing gives -1; 'R' takes a
//! screenshot (with screenshots on), any other key may compile the lexicon again.
//!
//! In:  nothing
//! Out: eax = the connection chosen (`g_listChoice`), carry set if negative (else the flags
//!      of `test eax, eax`); the rest kept (`pusha` .. `popa`)
//! Callers: `net::start_link`, `net::settings_link`. Its calls through `part` (the frames
//! through the game). The whole screen runs inside it: ungated. Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins (the keys scripted),
//! against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    CHOICE, CHOICE_BEFORE, CLOCK, CONNECTIONS, FLAGS, FRAME_CLOCK, KEY_CLOCK, LIST_TAKE_KEY,
    MAYBE_LEXICON, MOVE_CLOCK, Part, QUITTING, RUN_FRAME, SCREENSHOT, SCREENSHOTS_ON, Saved,
    answer, dispatch_lobby, dword, g, game, p, run_both, test_eax,
};
use rc2k_core::net::screens::{self, ListKey};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::choose_connection", rva(0x4174C5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const FRAME: u32 = 0x4175C7; // net::connections_frame

/// A list's up (0x66) or down (0x6B) on a list of `count` items: the highlight and the top
/// moved, as the original's eax and ebx leave them; false for another key.
pub fn list_move(r: &mut Regs, key: u8, count: u32) -> bool {
    // Converted: `rc2k_core::net::screens` (`ListKey`, `up`, `down`).
    match ListKey::of(key) {
        ListKey::Up => {
            let (choice, top) = screens::up(g(CHOICE), g(CHOICE_BEFORE));
            p(CHOICE, choice);
            p(CHOICE_BEFORE, top);
            r.eax = top;
            true
        }
        ListKey::Down => {
            let (choice, top) = screens::down(g(CHOICE), g(CHOICE_BEFORE), count);
            p(CHOICE, choice);
            p(CHOICE_BEFORE, top);
            r.eax = top;
            r.ebx = choice.wrapping_sub(screens::LIST_ROWS - 1);
            true
        }
        _ => false,
    }
}

/// The end of the list screens: eax the choice, carry if negative.
pub fn chosen(entry: &Regs, r: &Regs) -> Regs {
    let mut left = Regs {
        eflags: r.eflags,
        ..*entry
    };
    left.eax = g(CHOICE);
    test_eax(&mut left);
    if (left.eax as i32) < 0 {
        left.set_carry(true);
    }
    left
}

/// The Rust version: the registers it leaves.
pub fn choose_connection(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(CLOCK);
    p(MOVE_CLOCK, r.eax);
    p(KEY_CLOCK, r.eax);
    p(CHOICE_BEFORE, 0);
    p(CHOICE, 0);
    loop {
        r.eax = game(FRAME);
        part(RUN_FRAME, &mut r)?;
        r.eax = g(CLOCK);
        p(FRAME_CLOCK, r.eax);
        part(LIST_TAKE_KEY, &mut r)?;
        let key = r.eax as u8;
        if list_move(&mut r, key, g(CONNECTIONS)) {
            r.ebx = g(FRAME_CLOCK);
            p(MOVE_CLOCK, r.ebx);
        } else if ListKey::of(key) == ListKey::Enter {
            r.eax = g(CHOICE);
            return Ok(chosen(entry, &r));
        } else if ListKey::of(key) == ListKey::Escape {
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
    dispatch_lobby(&HOOK, r, call_original, choose_connection);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [RUN_FRAME, LIST_TAKE_KEY, SCREENSHOT, MAYBE_LEXICON];
    // pusha; mov eax, dword_518998
    let copy = match crate::net::lobby::copy(
        0x4174C5,
        0x102,
        &[0x60, 0xA1, 0x98, 0x89, 0x51, 0x00],
        &[0x2A, 0x39, 0xC3, 0xC8],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        dword(CONNECTIONS),
        dword(CHOICE),
        dword(CHOICE_BEFORE),
        dword(KEY_CLOCK),
        dword(FRAME_CLOCK),
        dword(MOVE_CLOCK),
        dword(SCREENSHOTS_ON),
    ];
    let saved = Saved::new(&all);
    let down = |n: usize| vec![0x6Bu32; n];
    let scripts: Vec<(&str, u32, Vec<u32>)> = vec![
        ("Escape", 3, vec![]),
        ("down twice, Enter", 3, [down(2), vec![0x1C]].concat()),
        (
            "down past the end, up, Enter",
            3,
            [down(5), vec![0x66, 0x1C]].concat(),
        ),
        (
            "a long list",
            40,
            [down(20), vec![0x66; 3], vec![0x52, 0x31, 0x1C]].concat(),
        ),
        ("none listed, Enter", 0, vec![0x6B, 0x1C]),
        ("up at the top", 5, vec![0x66, 0x66, 0x1C]),
    ];
    for (what, count, script) in scripts {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(CONNECTIONS, count);
                p(SCREENSHOTS_ON, 1);
                answer(crate::net::settings_loop::keys(script.clone()));
            },
            &choose_connection,
        );
    }
    drop(saved);
    t.finish()
}
