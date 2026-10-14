//! `screens::draw_name_cursor` (0x467D6C): the cursor after a player's name being typed.
//!
//! In:  edi = the player's player (`Player`)
//! Out: every register preserved; carry of the last text
//!
//! The name (`Player::name`) measured with the font at 0x51E824 (its line count and widest
//! line are left in it); not drawn, the carry is that of `cmp dword_7559A8, 1`; with the
//! text cursor's blink (`dword_7559A8`) at 1, string 4
//! ("_") at x = 0x1BB + the name's width, y = 0xD4, in the font at 0x51E8C4. Three
//! callers (the player details screens).
//! Refuses no player or no name.
//! Self-test: the four players' names, both blinks, nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::player::{Player, player_at};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;
use crate::text::measure_text::measure_text_at;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_name_cursor", rva(0x467D6C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const BLINK_RVA: usize = rva(0x7559A8); // dword_7559A8
pub const MEASURING_FONT: usize = 0x51E824;
pub const FONT: usize = 0x51E8C4;
const CURSOR: u32 = 4;

// ---- Rust version ----

/// `x` is ecx at entry (the measuring's x). Returns the carry.
pub unsafe fn draw_name_cursor(
    player: u32,
    x: u32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    if player == 0 {
        return Err(BadInput("no player object").into());
    }
    unsafe {
        let name = (player as *const Player).read_unaligned().name;
        if name == 0 {
            return Err(BadInput("a player without its name").into());
        }
        let measured =
            measure_text_at(font_at(MEASURING_FONT), name as *const u8, x as i32, canvas)?;
        let blink: u32 = game_read(BLINK_RVA);
        if blink == 1 {
            let at = measured.width.wrapping_add(0x1BB) as i32;
            Ok(draw_string_by_id_at(CURSOR, font_at(FONT), at, 0xD4, canvas)?.too_wide)
        } else {
            // Not drawn: the carry of its `cmp dword_7559A8, 1`.
            Ok(blink < 1)
        }
    }
}

// ---- hook ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe {
        draw_name_cursor(r.edi, r.ecx, canvas)
    }?))
}

/// What it leaves behind: the text buffer the cursor goes through. The measuring font's
/// leftovers are not compared: the shadow run's canvas does not write font objects back
/// (as for the other hooks that measure).
pub fn state() -> Vec<u8> {
    capture(&[(text() as u32, TEXT_LEN)])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    // The blink, the measuring font (written) and the text buffer.
    let regions: [Region; 3] = [
        at(BLINK_RVA),
        (
            crate::framework::game_addr(rva(MEASURING_FONT)) as u32,
            0x20,
        ),
        (text() as u32, TEXT_LEN),
    ];
    let saved = capture(&regions);
    for index in 0..4u32 {
        let player = player_at(index);
        if player == 0 || unsafe { (player as *const Player).read_unaligned() }.name == 0 {
            continue;
        }
        for blink in [0u32, 1] {
            check_draw(
                &mut t,
                &format!("player {index}, blink {blink}"),
                call_original,
                Regs {
                    edi: player,
                    ecx: 0x1BB,
                    ..Regs::SENTINEL
                },
                &|| unsafe { game_write(BLINK_RVA, blink) },
                &rust,
                &state,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
