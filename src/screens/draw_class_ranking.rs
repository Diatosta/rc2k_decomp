//! `screens::draw_class_ranking` (0x47C16A): a page of the times of one class
//! (`dword_7558F4`) from the list at `dword_7556EC` (entries of 0x9C bytes, -1 ends it;
//! +4 the driver's kind or player, +8 the nationality, +0xC the name (a string id when +4
//! is 0, else a text), +0x10 the flag, +0x18 the time, +0x1C the class), ten rows from the
//! place `dword_7556B4` on.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! `g_firstPlayerTime` the class's first time (`menu::find_class_time`, ebx 1). The rows
//! before the place are passed over (counting the class's entries from 1 until the place).
//! Each row (`seconds` its place, and one past the last when the list ends after it, y from 0xF0, 0x19 apart, each draw's y handed on as edx
//! is): the name at x 0xFA (font 0x51E584; a string for kind 0, else the text; the player
//! `dword_7556C0` in 0x51E524 with the fonts below changed to theirs), the time at 0x258
//! (`draw_time_fixed_pitch`, 0x51E564 / 0x51E504), the place at 0xBE (0x51E5A4 / 0x51E544)
//! and below the first the gap (`glyph`): string 0x579 at 0x276 and the time at 0x2E4;
//! then `dword_7559B0` = y + 4, the nationality's sprite (0x1C + +8) at x 0xD2 and the
//! flag's (`+0x10` with 0x18 and 0x19 swapped, + 0x44) at 0x1AE, both from `g_spriteTable`
//! with ebx as the draws leave it as the palette.
//!
//! In the draw list 0x758E46, whose address nothing in RAL.EXE holds (every byte searched,
//! 2026-10-20): unreached, only the self-test runs it. Compared in shadow mode
//! (`dispatch_draw`; the line buffer and the four variables as state). Refuses (before
//! anything) a list without its end in 0x400 entries, a page that starts past the list
//! (the original reads past its end), a row drawn just before the end mark (the original
//! then leaves with a dword still pushed: its `popa` and `retn` take the wrong ones, the
//! original's slip), a name past the line buffer. Self-test: lists of the test's, places
//! 1 and 2, the player in them or not, nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::find_class_time::{ENTRY_LEN, find_class_time};
use crate::screens::LINE_BUFFER_RVA;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_class_ranking",
    rva(0x47C16A),
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

const LIST_RVA: usize = rva(0x7556EC); // dword_7556EC
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const FIRST_PLACE_RVA: usize = rva(0x7556B4); // dword_7556B4
const PLAYER_RVA: usize = rva(0x7556C0); // dword_7556C0
const FIRST_TIME_RVA: usize = rva(0x7559BC); // g_firstPlayerTime
const PLACE_RVA: usize = rva(0x7559AC); // seconds
const GAP_RVA: usize = rva(0x7559C0); // glyph
const SPRITE_Y_RVA: usize = rva(0x7559B0); // dword_7559B0
const LINE_LEN: usize = 0x100;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// Each row's entry and place, and the place left after the last.
type Rows = (Vec<(u32, u32)>, Option<u32>);

/// The rows to draw, worked out before anything is drawn.
fn rows() -> Result<Rows, Refused> {
    // Converted: `rc2k_core::screens::results_pages::class_rows`.
    let list = g(LIST_RVA);
    let at = |n: u32| list.wrapping_add(n.wrapping_mul(ENTRY_LEN));
    let (rows, after) = rc2k_core::screens::results_pages::class_rows(
        |n| get(at(n) + 0x1C),
        |n| get(at(n)) == u32::MAX,
        g(CLASS_RVA),
        g(FIRST_PLACE_RVA),
    )
    .map_err(BadInput::from)?;
    Ok((
        rows.into_iter().map(|(n, place)| (at(n), place)).collect(),
        after,
    ))
}

/// The Rust version.
pub unsafe fn draw_class_ranking(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let (rows, last_place) = rows()?;
    let first_time = find_class_time(g(LIST_RVA), g(CLASS_RVA), true)?;
    // Converted: `rc2k_core::screens::results_pages::class_ranking` (the rows found here).
    use rc2k_core::screens::results_pages::{Name, RankedRow, class_ranking};
    let player = g(PLAYER_RVA);
    let rows: Vec<RankedRow<u32>> = rows
        .iter()
        .map(|&(e, place)| {
            let kind = get(e + 4);
            RankedRow {
                place,
                mine: kind != 0 && kind == player,
                name: if kind == 0 {
                    Name::String(get(e + 0xC))
                } else {
                    Name::Text(get(e + 0xC))
                },
                time: get(e + 0x18),
                nation: get(e + 8),
                car: get(e + 0x10),
            }
        })
        .collect();
    class_ranking(
        &mut crate::screens::game_screen::GameScreen::new(canvas),
        first_time,
        crate::strings::append_time::show_hours(),
        &rows,
        last_place,
    )?;
    Ok(())
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_class_ranking(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
        at(FIRST_TIME_RVA),
        at(PLACE_RVA),
        at(GAP_RVA),
        at(SPRITE_Y_RVA),
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
    let inputs: [Region; 4] = [
        at(LIST_RVA),
        at(CLASS_RVA),
        at(FIRST_PLACE_RVA),
        at(PLAYER_RVA),
    ];
    let saved = (capture(&inputs), state());
    let names: Vec<Vec<u8>> = (0..16)
        .map(|n| format!("Driver {n}\0").into_bytes())
        .collect();
    // Sixteen entries, classes 2, 3, 2, 2, 3, ...; the last two of another class, so that
    // no drawn row is just before the end mark.
    let mut list = vec![0u32; 17 * ENTRY_LEN as usize / 4];
    let base = list.as_mut_ptr() as u32;
    for n in 0..16u32 {
        let e = base + n * ENTRY_LEN;
        let put = |o: u32, v: u32| unsafe { ((e + o) as *mut u32).write_unaligned(v) };
        let kind = [0u32, 1, 5, 0, 2][n as usize % 5];
        put(0, n);
        put(4, kind);
        put(8, n % 7);
        put(
            0xC,
            if kind == 0 {
                0x249 + n
            } else {
                names[n as usize].as_ptr() as u32
            },
        );
        put(0x10, [0x18u32, 0x19, 3, 0x20][n as usize % 4]);
        put(0x18, 6000 + 137 * n);
        put(
            0x1C,
            if n >= 14 {
                9
            } else {
                [2u32, 3, 2][n as usize % 3]
            },
        );
    }
    unsafe { ((base + 16 * ENTRY_LEN) as *mut u32).write_unaligned(u32::MAX) };
    for (class, first, player) in [(2u32, 1u32, 1u32), (3, 1, 5), (2, 2, 2), (2, 3, 7)] {
        check_draw(
            &mut t,
            &format!("class {class}, from {first}, player {player}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(LIST_RVA, base);
                game_write(CLASS_RVA, class);
                game_write(FIRST_PLACE_RVA, first);
                game_write(PLAYER_RVA, player);
            },
            &rust,
            &state,
        );
    }
    restore(&inputs, &saved.0);
    restore(
        &[
            (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
            at(FIRST_TIME_RVA),
            at(PLACE_RVA),
            at(GAP_RVA),
            at(SPRITE_Y_RVA),
        ],
        &saved.1,
    );
    drop(list);
    t.finish()
}
