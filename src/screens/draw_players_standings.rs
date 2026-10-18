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
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::{font_at, line_buffer};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
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
        let font = font_at(0x51E584);
        for (id, x) in [(0x218u32, 0xEA), (0x219, 0x132), (0x21B, 0x217)] {
            draw_string_by_id_at(id, font, x, 0xE7, canvas)?;
        }
        let y = |row: usize| 0xFE + 0x17 * row as i32;
        let mut palette = 0;
        for row in 0..rows {
            let name = ((table(0x75647C, row) + 0x18) as *const u32).read_unaligned();
            palette = draw_text_at(font, name as *const u8, 0x132, y(row), canvas)?.text as u32;
        }
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        let field =
            |row: usize, o: u32| ((table(0x75647C, row) + o) as *const u32).read_unaligned();
        for row in 0..rows {
            canvas.blit_glyph(BlitCall::new(
                0x1C + field(row, 4),
                0x114,
                y(row),
                palette,
                sprites,
            ))?;
        }
        for row in 0..rows {
            let mut flag = field(row, 8);
            if (0x18..=0x19).contains(&flag) {
                flag ^= 1;
            }
            canvas.blit_glyph(BlitCall::new(
                flag.wrapping_add(0x44),
                0x1DA,
                y(row),
                palette,
                sprites,
            ))?;
        }
        for row in 0..rows {
            draw_string_by_id_at(0x4FE + row as u32, font, 0xEA, y(row), canvas)?;
        }
        for row in 0..rows {
            let points = (table(0x7589A6, row) as *const u32).read_unaligned();
            game_write(rva(0x7559B8), points);
            let mut line = crate::strings::append_u32::append_u32(points);
            line.push(0);
            std::ptr::copy_nonoverlapping(line.as_ptr(), line_buffer(), line.len());
            draw_text_at(font, line_buffer(), 0x217, y(row), canvas)?;
        }
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
