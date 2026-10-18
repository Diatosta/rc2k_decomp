//! `screens::draw_control_bindings` (0x46A829): what each of the current player's ten
//! controls is bound to, on the controller pages.
//!
//! In:  nothing
//! Out: every register preserved; carry of the last row's `draw_text` (clear when that row is
//!      the one being learnt)
//!
//! The current player's (`dword_75598C`) bindings at +0x50 .. +0x74 (a dword each: dl a
//! button, dh a direction, 0 nothing), at x 0x1B8, y 0xD5 and on every 0x1B, in the font at
//! 0x51E8C4, built in `byte_A8BC54`: string 0x3BD for nothing; else dh's name (string
//! `word_755200[dh]` for a device kind, +0x34, below 2, else `word_755212[dh]`), for kind 3 on
//! the first two rows the axis (`strings::append_axis_name` of +0x3B, +0x3A), then for a
//! button ('+' after a direction) string 0x3C0, a space and the button's number. The row being
//! learnt (`dword_7551F4` 1, `dword_7551FC` the row from 1) is left out. In the draw lists of
//! the four controller pages (0x758D9A ..). Refuses (before anything) no current player and
//! a direction past the two 9-entry tables (dh above 8: the original reads past them).
//! Compared in shadow mode (`dispatch_draw`). Self-test: the players' own bindings and test
//! ones of each shape and kind, the learnt row off / first / fifth / last, nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::player::{PLAYERS, player_at};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::PLAYER_INDEX_RVA;
use crate::strings::append_axis_name::append_axis_name;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_control_bindings",
    rva(0x46A829),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT_RVA: usize = rva(0xA8BC54); // byte_A8BC54
const TEXT_LEN: usize = 0x100;
const LEARNING_RVA: usize = rva(0x7551F4); // dword_7551F4
const ROW_RVA: usize = rva(0x7551FC); // dword_7551FC
const PAD_NAMES_RVA: usize = rva(0x755200); // word_755200
const STICK_NAMES_RVA: usize = rva(0x755212); // word_755212

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// Row n's text (without its NUL).
pub unsafe fn binding_text(player: u32, n: u32) -> Result<Vec<u8>, Refused> {
    unsafe {
        let binding = get(player + 0x50 + 4 * n);
        let (dl, dh) = (binding as u8, (binding >> 8) as u8);
        let mut out = Vec::new();
        if binding == 0 {
            push_string(&mut out, 0x3BD);
            return Ok(out);
        }
        let kind = get(player + 0x34);
        if dh != 0 {
            if dh > 8 {
                return Err(BadInput("a direction past the name tables").into());
            }
            let table = if kind < 2 {
                PAD_NAMES_RVA
            } else {
                STICK_NAMES_RVA
            };
            let id: u16 = game_read(table + 2 * dh as usize);
            push_string(&mut out, id as u32);
            if kind == 3 && n < 2 {
                out.extend(append_axis_name(*((player + 0x3B - n) as *const u8)));
            }
        }
        if dl != 0 {
            if dh != 0 {
                out.push(b'+');
            }
            push_string(&mut out, 0x3C0);
            out.push(b' ');
            out.extend(crate::strings::append_u32::append_u32(dl as u32));
        }
        Ok(out)
    }
}

/// Returns the carry.
pub unsafe fn draw_control_bindings(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        let player = player_at(game_read(PLAYER_INDEX_RVA));
        if player == 0 {
            return Err(BadInput("no current player").into());
        }
        let learning: u32 = game_read(LEARNING_RVA);
        let row: u32 = game_read(ROW_RVA);
        let mut texts = Vec::new();
        for n in 0..10u32 {
            let mut text = binding_text(player, n)?;
            text.push(0);
            if text.len() > TEXT_LEN {
                return Err(BadInput("a binding's text past its buffer").into());
            }
            texts.push(text);
        }
        let at = game_addr(TEXT_RVA) as *mut u8;
        let mut carry = false;
        for (n, text) in texts.iter().enumerate() {
            if learning == 1 && row == n as u32 + 1 {
                carry = false;
                continue;
            }
            std::ptr::copy_nonoverlapping(text.as_ptr(), at, text.len());
            carry = draw_text_at(font_at(0x51E8C4), at, 0x1B8, 0xD5 + 0x1B * n as i32, canvas)?
                .too_wide;
        }
        Ok(carry)
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_control_bindings(canvas) }?))
}

fn state() -> Vec<u8> {
    capture(&[
        (game_addr(TEXT_RVA) as u32, TEXT_LEN),
        (
            crate::strings::text_from_ids::text() as u32,
            crate::strings::text_from_ids::TEXT_LEN,
        ),
    ])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    let mut regions: Vec<Region> = vec![
        at(PLAYER_INDEX_RVA),
        at(LEARNING_RVA),
        at(ROW_RVA),
        (game_addr(TEXT_RVA) as u32, TEXT_LEN),
    ];
    for index in 0..PLAYERS {
        let p = player_at(index);
        if p != 0 {
            regions.push((p + 0x34, 0x44));
        }
    }
    let saved = capture(&regions);
    // (kind, axis bytes +0x3A / +0x3B, the ten bindings)
    let shapes: [(u32, u16, [u32; 10]); 3] = [
        (
            0,
            0,
            [0, 1, 0x100, 0x203, 0x400, 0x80C, 0x0101, 0x8FF, 0, 0x210],
        ),
        (
            2,
            0x0102,
            [0x100, 0x200, 0x400, 0x800, 5, 0, 0x104, 0x220, 0x101, 0],
        ),
        (
            3,
            0x1314,
            [0x101, 0x202, 0, 0x400, 3, 0x800, 0x100, 0, 0x201, 7],
        ),
    ];
    for index in 0..PLAYERS {
        let p = player_at(index);
        if p == 0 {
            continue;
        }
        for shape in [None, Some(0usize), Some(1), Some(2)] {
            for (learning, row) in [(0u32, 0u32), (1, 1), (1, 5), (1, 10)] {
                let set = || unsafe {
                    game_write(PLAYER_INDEX_RVA, index);
                    game_write(LEARNING_RVA, learning);
                    game_write(ROW_RVA, row);
                    if let Some(s) = shape {
                        let (kind, axes, bindings) = shapes[s];
                        ((p + 0x34) as *mut u32).write_unaligned(kind);
                        ((p + 0x3A) as *mut u16).write_unaligned(axes);
                        for (n, b) in bindings.iter().enumerate() {
                            ((p + 0x50 + 4 * n as u32) as *mut u32).write_unaligned(*b);
                        }
                    }
                };
                set();
                if unsafe { (0..10).any(|n| binding_text(p, n).is_err()) } {
                    restore(&regions, &saved);
                    continue;
                }
                check_draw(
                    &mut t,
                    &format!("player {index}, shape {shape:?}, learning {learning} row {row}"),
                    call_original,
                    Regs::SENTINEL,
                    &set,
                    &rust,
                    &state,
                );
                restore(&regions, &saved);
            }
        }
    }
    restore(&regions, &saved);
    t.finish()
}
