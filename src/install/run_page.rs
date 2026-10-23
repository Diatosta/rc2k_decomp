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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::install::installer::{
    CHOICE, CHOICE_BEFORE, CLOCK, COPY_PICTURE, FRAME_CLOCK, KEY_CLOCK, LIST_ANSWER, LIST_LEAVE,
    LOCK, MOVE_CLOCK, Part, QUITTING, RUN_FRAME, SHOW, UNLOCK, ZF, at, g, gb, p, pb, rb, rd, wd,
};
use crate::race::viewer::regs_entry;

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        run_page(r, &mut crate::install::installer::game_part)
    })
}
