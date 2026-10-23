//! `install::run_page` (0x4144CF): one of the installer's pages (a list menu) run until it
//! is left: the answer 1, not leaving, the clocks, the highlight from the list's +4; the
//! marks cleared (`menu::list_clear_marks`) and the list's action (+8) told the start (ebx
//! 0). A list with 0x40 or 0x80 is only that action. Else the items marked
//! (`install::mark_page`), the highlight moved on past items that cannot be chosen
//! (`menu::list_item_unusable`), the backdrop shown (`install::show_backdrop`); then each
//! frame: the pieces laid out, measured, placed, a frame's messages (`screens::run_frame`,
//! eax 0), the page drawn (lock, backdrop, `install::draw_pieces`, unlock, show), the
//! clocks, a key (`menu::list_take_key`). The game quitting ends it. A key (al 1 .. 0x7F)
//! goes to the item (`install::page_key`); if it did nothing, to the list's key action
//! (+0x14); if that did nothing either, it moves the highlight (`menu::list_step`, the
//! neighbours at +0x10); what took the key restarts the move clock. Then the marks again,
//! the action told (ebx 1), the items marked, and a highlight that cannot be chosen moved
//! on (an arrow on to the same item as before: the key again on the new one; else back for
//! 0x66 and 0x68, on for the rest, round the list). Leaving (`g_listLeave`): the backdrop,
//! the action told the end (ebx -1). At the end the marks cleared and the highlight kept.
//!
//! In:  edi the list
//! Out: eax the answer (`byte_51D415`); esi, edi, ebp kept; ebx, ecx, edx as left
//! A page whose every item cannot be chosen would go round for ever: refused. Caller
//! `install::run_pages`. Its calls through `part` (the list's actions by their address).
//! Ungated. Shadow mode runs the original alone. Self-test: a copy whose sixteen calls and
//! the list's actions go to stand-ins (keys, moves and answers scripted), against the Rust
//! version. The quitting way is not tested (it would set the game's `g_quitting`).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CHOICE, CHOICE_BEFORE, CLOCK, COPY_PICTURE, FRAME_CLOCK, KEY_CLOCK, LIST_ANSWER, LIST_LEAVE,
    LOCK, MOVE_CLOCK, Part, QUITTING, RUN_FRAME, SHOW, UNLOCK, ZF, a, answer, at, both, copy, g,
    gb, p, pb, quiet, rb, rd, region, stand_in, wd,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts, regs_entry};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::run_page", rva(0x4144CF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CLEAR_MARKS: usize = 0x414AC9; // menu::list_clear_marks
const MARK_PAGE: usize = 0x414B53; // install::mark_page
const UNUSABLE: usize = 0x414C1B; // menu::list_item_unusable (carry)
const SHOW_BACKDROP: usize = 0x41525D; // install::show_backdrop
const LAY_OUT: usize = 0x414C7E; // install::lay_out_pieces
const MEASURE: usize = 0x414DF5; // install::measure_pieces
const PLACE: usize = 0x414ED1; // install::place_pieces
const DRAW: usize = 0x415058; // install::draw_pieces
const TAKE_KEY: usize = 0x41469A; // menu::list_take_key
const PAGE_KEY: usize = 0x4146E4; // install::page_key (zero)
const STEP: usize = 0x414A55; // menu::list_step (zero)

fn zero(r: &Regs) -> bool {
    r.eflags & ZF != 0
}

/// A list's action (+8) told where the page is (ebx), eax kept.
fn tell(r: &mut Regs, list: u32, ebx: u32, part: Part) -> Result<(), Refused> {
    let action = rd(list + 8);
    if action != 0 {
        r.ebx = ebx;
        let eax = r.eax;
        part(action, r)?;
        r.eax = eax;
    }
    Ok(())
}

pub fn run_page(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let list = entry.edi;
    let call = |r: &mut Regs, x: usize, part: Part| part(at(x), r);
    pb(LIST_ANSWER, 1);
    pb(LIST_LEAVE, 0);
    r.eax = g(CLOCK);
    p(MOVE_CLOCK, r.eax);
    p(KEY_CLOCK, r.eax);
    r.eax = rd(list + 4);
    p(CHOICE, r.eax);
    let count = rb(list + 1) as u32;
    let round = 2 * count as usize + 2;
    if count != 0 {
        call(&mut r, CLEAR_MARKS, part)?;
    }
    tell(&mut r, list, 0, part)?;
    'page: {
        if rb(list) & 0xC0 != 0 {
            break 'page;
        }
        call(&mut r, MARK_PAGE, part)?;
        r.eax = g(CHOICE);
        r.ecx = count;
        let mut steps = 0;
        loop {
            call(&mut r, UNUSABLE, part)?;
            if !r.carry() {
                break;
            }
            steps += 1;
            if steps > round {
                return Err(BadInput("a page whose every item cannot be chosen").into());
            }
            p(CHOICE, g(CHOICE).wrapping_add(1));
            if r.ecx == g(CHOICE) {
                p(CHOICE, 0);
            }
        }
        r.set_carry(true);
        call(&mut r, SHOW_BACKDROP, part)?;
        loop {
            for x in [LAY_OUT, MEASURE, PLACE] {
                call(&mut r, x, part)?;
            }
            let mut f = Regs { eax: 0, ..r };
            call(&mut f, RUN_FRAME, part)?;
            call(&mut r, LOCK, part)?;
            if !r.carry() {
                for x in [COPY_PICTURE, DRAW, UNLOCK, SHOW] {
                    call(&mut r, x, part)?;
                }
            }
            r.eax = g(CLOCK);
            p(FRAME_CLOCK, r.eax);
            r.eax = g(CHOICE);
            p(CHOICE_BEFORE, r.eax);
            call(&mut r, TAKE_KEY, part)?;
            if g(QUITTING) == 1 {
                break 'page;
            }
            let key = r.eax as u8;
            if key != 0 && key & 0x80 == 0 {
                let mut again = 0;
                loop {
                    r.esi = list + 0x18 + 0x10 * g(CHOICE);
                    call(&mut r, PAGE_KEY, part)?;
                    let mut took = zero(&r);
                    if !took {
                        let keys = rd(list + 0x14);
                        if keys != 0 {
                            part(keys, &mut r)?;
                            took = zero(&r);
                        }
                    }
                    let mut moved = true;
                    if !took {
                        r.ebx = rd(list + 0x10);
                        call(&mut r, STEP, part)?;
                        moved = zero(&r);
                    }
                    if moved {
                        r.ebx = g(FRAME_CLOCK);
                        p(MOVE_CLOCK, r.ebx);
                    }
                    call(&mut r, CLEAR_MARKS, part)?;
                    tell(&mut r, list, 1, part)?;
                    call(&mut r, MARK_PAGE, part)?;
                    call(&mut r, UNUSABLE, part)?;
                    if !r.carry() {
                        break;
                    }
                    r.ebx = g(CHOICE);
                    let arrow = matches!(r.eax as u8, 0x66 | 0x6B | 0x68 | 0x69);
                    if arrow && r.ebx != g(CHOICE_BEFORE) {
                        again += 1;
                        if again > round {
                            return Err(BadInput("a page whose every item cannot be chosen").into());
                        }
                        continue;
                    }
                    r.ecx = count;
                    let mut steps = 0;
                    loop {
                        let k = r.eax as u8;
                        if k == 0x66 || k == 0x68 {
                            let c = g(CHOICE).wrapping_sub(1);
                            p(CHOICE, c);
                            if c as i32 >= 0 {
                            } else {
                                p(CHOICE, r.ecx.wrapping_sub(1));
                            }
                        } else {
                            let c = g(CHOICE).wrapping_add(1);
                            p(CHOICE, c);
                            if r.ecx == c {
                                p(CHOICE, 0);
                            }
                        }
                        call(&mut r, UNUSABLE, part)?;
                        if !r.carry() {
                            break;
                        }
                        steps += 1;
                        if steps > round {
                            return Err(BadInput("a page whose every item cannot be chosen").into());
                        }
                    }
                    break;
                }
            } else {
                call(&mut r, CLEAR_MARKS, part)?;
                tell(&mut r, list, 1, part)?;
                call(&mut r, MARK_PAGE, part)?;
                call(&mut r, UNUSABLE, part)?;
                if r.carry() {
                    r.ebx = g(CHOICE);
                    let arrow = matches!(r.eax as u8, 0x66 | 0x6B | 0x68 | 0x69);
                    if arrow && r.ebx != g(CHOICE_BEFORE) {
                        return Err(BadInput("an arrow that is no key").into());
                    }
                    r.ecx = count;
                    let mut steps = 0;
                    loop {
                        let c = g(CHOICE).wrapping_add(1);
                        p(CHOICE, c);
                        if r.ecx == c {
                            p(CHOICE, 0);
                        }
                        call(&mut r, UNUSABLE, part)?;
                        if !r.carry() {
                            break;
                        }
                        steps += 1;
                        if steps > round {
                            return Err(BadInput("a page whose every item cannot be chosen").into());
                        }
                    }
                }
            }
            if gb(LIST_LEAVE) == 1 {
                break;
            }
        }
        r.set_carry(false);
        call(&mut r, SHOW_BACKDROP, part)?;
        tell(&mut r, list, u32::MAX, part)?;
    }
    if count != 0 {
        call(&mut r, CLEAR_MARKS, part)?;
    }
    wd(list + 4, g(CHOICE));
    r.eax = gb(LIST_ANSWER) as u32;
    Ok(Regs {
        esi: entry.esi,
        edi: entry.edi,
        ebp: entry.ebp,
        ..r
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        run_page(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        run_page(r, &mut crate::install::installer::game_part)
    })
}

// ---- self-test ----

const CALLEES: [u32; 16] = [
    0x414AC9, 0x414B53, 0x414C1B, 0x41525D, 0x414C7E, 0x414DF5, 0x414ED1, 0x405A71, 0x4058AD,
    0x4060E3, 0x415058, 0x405949, 0x4057D1, 0x41469A, 0x4146E4, 0x414A55,
];
const CALLS: [(usize, u32); 22] = [
    (0x2E, 0x414AC9),
    (0x49, 0x414B53),
    (0x57, 0x414C1B),
    (0x79, 0x41525D),
    (0x7E, 0x414C7E),
    (0x83, 0x414DF5),
    (0x88, 0x414ED1),
    (0x90, 0x405A71),
    (0x96, 0x4058AD),
    (0x9D, 0x4060E3),
    (0xA2, 0x415058),
    (0xA7, 0x405949),
    (0xAC, 0x4057D1),
    (0xC5, 0x41469A),
    (0xE6, 0x4146E4),
    (0xFB, 0x414A55),
    (0x10E, 0x414AC9),
    (0x121, 0x414B53),
    (0x126, 0x414C1B),
    (0x185, 0x414C1B),
    (0x19A, 0x41525D),
    (0x1B3, 0x414AC9),
];

/// A case's script: the keys, which items cannot be chosen, what the item takes, what the
/// list's key action takes, what the step moves, whether the lock fails.
#[derive(Clone)]
struct Script {
    keys: Vec<u8>,
    unusable: u32,
    item_takes: bool,
    list_takes: bool,
    step_moves: bool,
    lock_fails: bool,
}

static STATE: Mutex<(usize, u32)> = Mutex::new((0, 0));

fn scripted(s: Script) -> crate::install::installer::Answer {
    *STATE.lock().unwrap() = (0, 0);
    Box::new(move |place, r| {
        let mut state = STATE.lock().unwrap();
        let callee = |x: u32| place == crate::install::installer::a(x as usize);
        if callee(0x41469A) {
            let n = state.0;
            state.0 += 1;
            let key = s.keys.get(n).copied().unwrap_or(1);
            r.eax = r.eax & !0xFF | key as u32;
            if n + 1 >= s.keys.len() {
                pb(LIST_LEAVE, 1);
            }
        } else if callee(0x414C1B) {
            r.set_carry(s.unusable >> (g(CHOICE) & 31) & 1 != 0);
        } else if callee(0x4146E4) {
            r.set_zero(s.item_takes);
        } else if callee(0x414A55) {
            r.set_zero(s.step_moves);
            if s.step_moves {
                p(CHOICE, (g(CHOICE) + 1) % 6);
            }
        } else if callee(0x4058AD) {
            r.set_carry(s.lock_fails);
        } else if place == 0xC0DE_0014 {
            r.set_zero(s.list_takes);
        } else if place == 0xC0DE_0008 {
            state.1 += 1;
            r.eax = 0x5555;
        }
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // push esi; push edi; push ebp; mov byte_51D415, 1
    let copy = match copy(
        0x4144CF,
        0x1CB,
        &[0x56, 0x57, 0x55, 0xC6, 0x05, 0x15, 0xD4, 0x51, 0x00, 0x01],
        &CALLS,
        &CALLEES,
        &[],
        &[],
        &[0xC0DE_0008, 0xC0DE_0014],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let list = TestList::new();
    let regions = [list.region(), region(CHOICE, 0x20), region(CLOCK, 4)];
    let saved = Saved::new(&regions);
    let base = Script {
        keys: vec![0x66, 0, 0x1C],
        unusable: 0,
        item_takes: true,
        list_takes: false,
        step_moves: false,
        lock_fails: false,
    };
    let mut scripts = vec![base.clone()];
    for (keys, unusable) in [
        (vec![0x66u8, 0x6B, 0x68, 0x69, 0x30, 0x90], 0b000100u32),
        (vec![0x66, 0x66, 0x6B], 0b101001),
        (vec![0x68, 0x69, 0x1C], 0b010110),
        (vec![0x6B], 0b011111),
    ] {
        for (item_takes, list_takes, step_moves) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (false, false, false),
        ] {
            scripts.push(Script {
                keys: keys.clone(),
                unusable,
                item_takes,
                list_takes,
                step_moves,
                lock_fails: unusable & 1 != 0,
            });
        }
    }
    for flags in [0u8, 0x40, 0x80] {
        for actions in [false, true] {
            for s in &scripts {
                if flags != 0 && s.keys != base.keys {
                    continue;
                }
                let prepare = || {
                    list.set(flags, &[1, 2, 3, 4, 5, 6], 2, 0, 0, 0, 100, b"");
                    if actions {
                        wd(list.base() + 8, stand_in(16));
                        wd(list.base() + 0x14, stand_in(17));
                    }
                    wd(list.base() + 4, 2);
                    p(CLOCK, 0x4321);
                    answer(scripted(s.clone()));
                };
                let before = Regs {
                    edi: list.base(),
                    ..Regs::SENTINEL
                };
                both(
                    &mut t,
                    &format!(
                        "flags {flags:#x}, actions {actions}, keys {:x?}, unusable {:#b}, item {} \
                         list {} step {} lock fails {}",
                        s.keys, s.unusable, s.item_takes, s.list_takes, s.step_moves, s.lock_fails
                    ),
                    &copy,
                    before,
                    &regions,
                    0,
                    &[],
                    &prepare,
                    &|r, part, _| run_page(r, part),
                );
            }
        }
    }
    quiet();
    let _ = (a, QUITTING);
    drop(saved);
    t.finish()
}
