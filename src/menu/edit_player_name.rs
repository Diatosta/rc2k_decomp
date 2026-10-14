//! `menu::edit_player_name` (0x467C63): one step of typing a player's name on the players'
//! details pages.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The name is the current player's (`Player::name`, `menu/player_name.rs`). One pending
//! key is taken:
//!   none          nothing;
//!   1 (Escape)    the name kept at the start comes back, the entry ends;
//!   0x1C (Enter)  an empty name is replaced by the one kept; the entry ends;
//!   0x6E          the name is cleared (`menu::clear_text`);
//!   0x0E          the last character goes;
//!   any other     `key_to_char`; a character is appended unless the length is already above
//!                 the maximum (`dword_755571`, signed), and taken off again if the name then
//!                 measures 0xEC pixels or more in the font at 0x51E824 (measured from the x
//!                 that is the length before the character, as ecx held it).
//! No cursor is written: the page draws it (`screens::draw_name_cursor`). Called from the
//! four `player*_details_loop`s while `dword_7556A8` is 1.
//! Refuses, before the key is taken: no current player, a player without a name, a length
//! outside 0 .. 0x1F, no keyboard layout.
//! Compared in shadow mode (the key replayed). Self-test: the first player with a name,
//! eight keys on five names.

use crate::errors::{BadInput, Refused};
use crate::framework::shadow;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::input::key_to_char::{
    ALT_GR_RVA, ALT_RVA, DEAD_KEY_RVA, LEFT_SHIFT_RVA, RIGHT_SHIFT_RVA, has_layout, key_to_char,
    regions as key_regions,
};
use crate::input::take_key::{PENDING_KEY_RVA, pending_key, take_key};
use crate::menu::clear_text::clear_text;
use crate::menu::player::{PLAYERS, Player, player_at};
use crate::menu::player_name::{backspace_name, bring_back_name, kept, kept_region, name};
use crate::menu::text_entry::{ENTERING_RVA, LENGTH_RVA, MAX_LENGTH_RVA, TEXT_LEN, set_length};
use crate::screens::harness::Left;
use crate::screens::{font_at, font_is_loaded};
use crate::service::harness::{
    Region, at, capture, check_state_with, dispatch_keyed_state, restore,
};
use crate::service::state::PLAYER_INDEX_RVA;
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
use crate::text::measure_text::measure_text_at;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::edit_player_name", rva(0x467C63), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const KEY_ESCAPE: u8 = 1;
const KEY_ENTER: u8 = 0x1C;
const KEY_CLEAR: u8 = 0x6E;
const KEY_BACKSPACE: u8 = 0x0E;

const MEASURING_FONT: usize = 0x51E824; // dword_51E824
const WIDEST_NAME: i32 = 0xEC;

// ---- Rust version ----

pub unsafe fn edit_player_name(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Everything that could be refused, before the key is taken.
    let name = name()?;
    let len: i32 = unsafe { game_read(LENGTH_RVA) };
    if !(0..TEXT_LEN as i32).contains(&len) {
        return Err(BadInput("an entry length outside 0 .. 0x1F").into());
    }
    if !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }

    unsafe {
        let key = take_key();
        match key {
            0 => {}
            KEY_ESCAPE => {
                bring_back_name(name);
                game_write(ENTERING_RVA, 0u32);
            }
            KEY_ENTER => {
                if len == 0 {
                    bring_back_name(name);
                }
                game_write(ENTERING_RVA, 0u32);
            }
            KEY_CLEAR => clear_text(name),
            KEY_BACKSPACE => backspace_name(name),
            _ => {
                let c = key_to_char(key as u32)? as u8;
                if c != 0 && len <= game_read::<i32>(MAX_LENGTH_RVA) {
                    name.offset(len as isize).write(c);
                    set_length(len.wrapping_add(1));
                    let measured = measure_text_at(font_at(MEASURING_FONT), name, len, canvas)?;
                    if (measured.width as i32) >= WIDEST_NAME {
                        backspace_name(name);
                    }
                }
            }
        }
    }
    Ok(())
}

// ---- hook ----

/// What a step may write, except the pending key.
fn regions() -> Vec<Region> {
    let mut regions = vec![at(LENGTH_RVA), at(ENTERING_RVA)];
    regions.extend(key_regions());
    if let Ok(name) = name() {
        // One past the field: a character at 0x1F goes there before it is measured.
        regions.push((name as u32, TEXT_LEN + 2));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    if shadow() {
        unsafe { edit_player_name(&mut RecordingCanvas::new(false)) }?;
    } else {
        unsafe { edit_player_name(&mut GameCanvas) }?;
    }
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

unsafe extern "C" fn original_without_drawing(r: *mut Regs) {
    record_calls(true, || unsafe { call_original(r) });
}

fn rust_without_drawing(_: &Regs) -> Result<Left, Refused> {
    unsafe { edit_player_name(&mut RecordingCanvas::new(false)) }?;
    Ok(Left::nothing())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off");
    }
    let Some(index) = (0..PLAYERS).find(|&i| {
        let p = player_at(i);
        p != 0 && unsafe { (p as *const Player).read_unaligned() }.name != 0
    }) else {
        return t.skip("no player with a name");
    };
    if !font_is_loaded(MEASURING_FONT) || !has_layout() {
        return t.skip("no measuring font or keyboard layout");
    }
    let font = font_at(MEASURING_FONT);
    let saved_font = unsafe { font.read_unaligned() };
    let inputs = [
        at(PLAYER_INDEX_RVA),
        at(MAX_LENGTH_RVA),
        kept_region(),
        (game_addr(PENDING_KEY_RVA) as u32, 1),
        (game_addr(LEFT_SHIFT_RVA) as u32, 1),
        (game_addr(RIGHT_SHIFT_RVA) as u32, 1),
        (game_addr(ALT_RVA) as u32, 1),
        (game_addr(ALT_GR_RVA) as u32, 1),
    ];
    let saved_inputs = capture(&inputs);
    unsafe { game_write(PLAYER_INDEX_RVA, index) };
    let regions = regions();
    let saved = capture(&regions);
    let field = name().unwrap_or(std::ptr::null_mut());

    let wide = [b'W'; 29];
    let names: [(&[u8], i32); 5] = [
        (b"abc", 3),
        (b"", 0),
        (b"abcdefghijklmnopqrstuvwxyz1234", 30),
        (&wide, 29),
        (b"x", 1),
    ];
    for max in [0x1Eu32, 2] {
        for key in [
            KEY_ESCAPE,
            KEY_ENTER,
            KEY_CLEAR,
            KEY_BACKSPACE,
            0x1E,
            0x39,
            0x9E,
            0,
        ] {
            for (text, len) in names {
                let prepare = || unsafe {
                    font.write_unaligned(saved_font);
                    restore(&regions, &saved);
                    field.write_bytes(0, TEXT_LEN + 1);
                    std::ptr::copy_nonoverlapping(text.as_ptr(), field, text.len());
                    set_length(len);
                    kept().write_bytes(0, TEXT_LEN);
                    std::ptr::copy_nonoverlapping(b"before".as_ptr(), kept(), 6);
                    game_write(MAX_LENGTH_RVA, max);
                    game_write(ENTERING_RVA, 1u32);
                    game_write(DEAD_KEY_RVA, 0u8);
                    for flag in [LEFT_SHIFT_RVA, RIGHT_SHIFT_RVA, ALT_RVA, ALT_GR_RVA] {
                        game_write(flag, 0u8);
                    }
                    pending_key().store(key, Ordering::SeqCst);
                };
                check_state_with(
                    &mut t,
                    &format!(
                        "player {index}, max {max}, key={key:#x} name={:?}",
                        String::from_utf8_lossy(text)
                    ),
                    original_without_drawing,
                    Regs::SENTINEL,
                    &prepare,
                    &regions,
                    &rust_without_drawing,
                    &|| vec![pending_key().load(Ordering::SeqCst)],
                );
            }
        }
    }
    unsafe { font.write_unaligned(saved_font) };
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    t.finish()
}
