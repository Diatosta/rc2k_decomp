//! `screens::draw_players_standings` (0x483FDE): the players' standings: the heads (strings
//! 0x218, 0x219, 0x21B at y 0xE7), then four rows (three when `dword_755984` is 3: two
//! players still get four, as the original) in the order `dword_758720` .. (byte offsets into
//! `g_players`): the names at x 0x132, the cars' sprites at 0x114, the flags' at 0x1DA, the
//! places (strings 0x4FE ..) at 0xEA and the points (`off_7589A6[..]`'s first dword, kept in
//! `dword_7559B8`) at 0x217; rows 0x17 apart from y 0xFE; the sprites' palette ebx after the
//! last name.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758ECE). Refuses (before anything) a row's player without
//! its record or name, or without its points. Compared in shadow mode (`dispatch_draw`).
//! Self-test: player counts 2, 3 and 4 and two orders, nothing drawn (skipped without the
//! fonts or four players).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::line_buffer;
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_players_standings",
    rva(0x483FDE),
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

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The row's offset (`dword_758720` + 4 x row) into `g_players` / `off_7589A6`.
fn offset(row: usize) -> u32 {
    k(0x758720 + 4 * row)
}

fn table(base: usize, row: usize) -> u32 {
    unsafe {
        ((game_addr(rva(base)) as u32).wrapping_add(offset(row)) as *const u32).read_unaligned()
    }
}

/// The Rust version.
pub unsafe fn draw_players_standings(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let rows = if k(0x755984) == 3 { 3 } else { 4 };
        for row in 0..rows {
            let p = table(0x75647C, row);
            if p == 0
                || ((p + 0x18) as *const u32).read_unaligned() == 0
                || table(0x7589A6, row) == 0
            {
                return Err(BadInput("a standings row without its player, name or points").into());
            }
        }
        // Converted: `rc2k_core::players_results::standings`.
        let field =
            |row: usize, o: u32| ((table(0x75647C, row) + o) as *const u32).read_unaligned();
        let rows: Vec<rc2k_core::players_results::Standing<u32>> = (0..rows)
            .map(|row| rc2k_core::players_results::Standing {
                name: field(row, 0x18),
                nation: field(row, 4),
                car: field(row, 8),
                points: (table(0x7589A6, row) as *const u32).read_unaligned(),
            })
            .collect();
        rc2k_core::players_results::standings(
            &mut crate::screens::game_screen::GameScreen::new(canvas),
            &rows,
        )?;
        Ok(())
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_players_standings(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (line_buffer() as u32, 0x20),
        at(rva(0x7559B8)),
        (crate::strings::text_from_ids::text() as u32, 0x100),
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
    if (0..4).any(|n| crate::menu::player::player_at(n) == 0) {
        return t.skip("fewer than four player records");
    }
    let regions: Vec<Region> = vec![at(rva(0x755984)), (game_addr(rva(0x758720)) as u32, 0x10)];
    let saved = capture(&regions);
    for (count, order) in [
        (2u32, [0u32, 4, 8, 12]),
        (3, [8, 0, 4, 12]),
        (4, [12, 8, 4, 0]),
    ] {
        unsafe {
            game_write(rva(0x755984), count);
            game_write(rva(0x758720), order);
        }
        if unsafe { draw_players_standings_refuses() } {
            continue;
        }
        check_draw(
            &mut t,
            &format!("count {count} order {order:?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(rva(0x755984), count);
                game_write(rva(0x758720), order);
            },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}

/// Whether the rows' players, names or points are missing (for the test).
unsafe fn draw_players_standings_refuses() -> bool {
    let rows = if k(0x755984) == 3 { 3 } else { 4 };
    (0..rows).any(|row| {
        let p = table(0x75647C, row);
        p == 0
            || unsafe { ((p + 0x18) as *const u32).read_unaligned() } == 0
            || table(0x7589A6, row) == 0
    })
}
