//! `screens::draw_redefined_keys` (0x47CFA7): the keys on a redefine keys page.
//!
//! In:  nothing
//! Out: every register preserved; carry of the last row
//!
//! The current player's (`dword_75598C`) keys (`Player::keys`, +0x1C), places 0 .. 7 and
//! 9 .. 11 (place 8 is not drawn), at x = 0x1CC, y = 0xAF and on every 0x1D, in the font at
//! 0x51E8C4: each key's name (`input::append_key_name` and a NUL into `byte_A8BC54`, then
//! `draw_text`), but at the place being redefined (`g_redefinedKeyPlace`) the cursor,
//! string 4, while the text cursor's blink (`dword_7559A8`) is 1, else nothing (the carry
//! then that of `cmp dword_7559A8, 1`). The player's keys, not the page's work copy: a key
//! shows when the redefining ends. In the draw lists of the four pages (0x758DE6 ..).
//! Refuses no current player and what the key's name refuses.
//! Compared in shadow mode (`dispatch_draw`). Self-test: the four players, four places
//! and both blinks, nothing drawn.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::input::append_key_name::append_key_name;
use crate::menu::player::{PLAYERS, player_at};
use crate::menu::player_settings::current;
use crate::menu::redefine_keys::{NO_PLACE, PLACE_RVA};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::draw_name_cursor::BLINK_RVA;
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::PLAYER_INDEX_RVA;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_redefined_keys",
    rva(0x47CFA7),
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

// ---- game data ----

const NAME_RVA: usize = rva(0xA8BC54); // byte_A8BC54
const NAME_LEN: usize = 0x100; // up to byte_A8BD54
const FONT: usize = 0x51E8C4;
const CURSOR: u32 = 4;
const X: i32 = 0x1CC;
/// The places drawn, in order; row n is at y = 0xAF + n * 0x1D.
const PLACES: [usize; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11];

// ---- Rust version ----

/// Returns the carry.
pub unsafe fn draw_redefined_keys(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let player = current()?;
    unsafe {
        let keys = player.read_unaligned().keys;
        let redefined: u32 = game_read(PLACE_RVA);
        let font = font_at(FONT);
        let mut carry = false;
        for (row, &place) in PLACES.iter().enumerate() {
            let y = 0xAF + row as i32 * 0x1D;
            if redefined == place as u32 {
                let blink: u32 = game_read(BLINK_RVA);
                carry = if blink == 1 {
                    draw_string_by_id_at(CURSOR, font, X, y, canvas)?.too_wide
                } else {
                    blink < 1
                };
            } else {
                let (mut name, _) = append_key_name(keys[place] as u32)?;
                name.push(0);
                if name.len() > NAME_LEN {
                    return Err(crate::errors::BadInput("a key's name past its buffer").into());
                }
                let at = game_addr(NAME_RVA) as *mut u8;
                std::ptr::copy_nonoverlapping(name.as_ptr(), at, name.len());
                carry = draw_text_at(font, at, X, y, canvas)?.too_wide;
            }
        }
        Ok(carry)
    }
}

// ---- hook ----

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_redefined_keys(canvas) }?))
}

/// What it leaves behind: the key's name buffer and the text buffer the cursor goes through.
fn state() -> Vec<u8> {
    capture(&[
        (game_addr(NAME_RVA) as u32, NAME_LEN),
        (text() as u32, TEXT_LEN),
    ])
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
    if !crate::input::key_to_char::has_layout() {
        return t.skip("no keyboard layout");
    }
    let regions: [Region; 5] = [
        at(PLAYER_INDEX_RVA),
        at(PLACE_RVA),
        at(BLINK_RVA),
        (game_addr(NAME_RVA) as u32, NAME_LEN),
        (text() as u32, TEXT_LEN),
    ];
    let saved = capture(&regions);
    for index in 0..PLAYERS {
        if player_at(index) == 0 {
            continue;
        }
        for place in [0u32, 8, 11, NO_PLACE] {
            for blink in [0u32, 1] {
                check_draw(
                    &mut t,
                    &format!("player {index}, place {place}, blink {blink}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        game_write(PLAYER_INDEX_RVA, index);
                        game_write(PLACE_RVA, place);
                        game_write(BLINK_RVA, blink);
                    },
                    &rust,
                    &state,
                );
            }
        }
    }
    restore(&regions, &saved);
    t.finish()
}
