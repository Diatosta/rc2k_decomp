//! `menu::edit_savegame_name` (0x46D685): one step of typing the name of a saved game; Enter
//! saves the game.
//!
//! In:  nothing
//! Out: every register preserved (carry is set on Escape; the caller does not read it)
//! The name is the highlighted entry of the saved game list. One pending key is taken:
//!   none          the cursor is written again;
//!   1 (Escape)    the name from before comes back, the entry ends;
//!   0x1C (Enter)  an empty name is replaced by the one from before; the entry ends, the
//!                 name is cut at its length and the game is saved in that slot
//!                 (`saves::write_savegame_slot`);
//!   0x6E          the name is cleared;
//!   0x0E          the last character goes (`backspace_savegame_name`);
//!   any other     `key_to_char`; a character is appended unless the length is already above
//!                 the maximum, and taken off again if the name then measures 0xEC pixels
//!                 or more.
//! After anything but Escape and Enter the cursor (string 4) is written after the name if
//! `dword_7559A8` is 1, then a NUL.
//! Called from the screen loop `sub_47F034` while `dword_7556A8` is 1.
//! Refuses, before the key is taken: a name past the list, a length outside 0 .. 0x1F, no
//! keyboard layout. Manual patch with a private copy (0x20A bytes, 14 calls relocated).
//! In shadow mode the game is saved once, by the original; the replayed Rust version leaves
//! the save out. The self-test runs a copy of the original whose call to the save function
//! goes to a stand-in that notes the slot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::input::key_to_char::{
    ALT_GR_RVA, ALT_RVA, DEAD_KEY_RVA, LEFT_SHIFT_RVA, RIGHT_SHIFT_RVA, has_layout, key_to_char,
    regions as key_regions,
};
use crate::input::take_key::{PENDING_KEY_RVA, pending_key, take_key};
use crate::menu::add_text_length::add_text_length;
use crate::menu::backspace_savegame_name::backspace_savegame_name;
use crate::menu::clear_text::clear_text;
use crate::menu::name_lists::{NAME_LEN, SAVEGAME_PAGE_RVA, savegame_name};
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{
    ENTERING_RVA, LENGTH_RVA, MAX_LENGTH_RVA, TEXT_LEN, backup, backup_region, length, set_length,
};
use crate::redirect::RedirectedCopy;
use crate::saves::write_savegame_slot::write_savegame_slot;
use crate::screens::harness::Left;
use crate::screens::{font_at, font_is_loaded};
use crate::service::harness::{
    Region, at, capture, check_state_with, dispatch_keyed_state, restore,
};
use crate::service::state::{MENU_ITEM_RVA, menu_item};
use crate::strings::append_string_by_id::push_string;
use crate::strings::string_table::{blob, is_table_id};
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
use crate::text::measure_text::measure_text_at;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::edit_savegame_name", rva(0x46D685), thunk, &ORIGINAL)
        .private_copy(0x20A)
        .relocate_calls(&[
            0x01, 0x30, 0x76, 0x7B, 0xB9, 0xCE, 0xFC, 0x10A, 0x114, 0x16D, 0x17A, 0x1B0, 0x1FE,
            0x203,
        ])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SAVE_GAME_RVA: usize = rva(0x4AB558); // saves::write_savegame_slot: ebx = slot
const CURSOR_ON_RVA: usize = rva(0x7559A8); // dword_7559A8: 1 in the visible half of the cursor's blink

const KEY_ESCAPE: u8 = 1;
const KEY_ENTER: u8 = 0x1C;
const KEY_CLEAR: u8 = 0x6E;
const KEY_BACKSPACE: u8 = 0x0E;
const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: usize = 0x51E824; // dword_51E824

// ---- Rust version ----

unsafe fn end_name(name: *mut u8) {
    unsafe { name.offset(length() as isize).write(0) };
}

/// The slot of the highlighted saved game: `page * 10 + item - 1`.
fn highlighted_slot() -> u32 {
    unsafe { game_read::<u32>(SAVEGAME_PAGE_RVA) }
        .wrapping_mul(10)
        .wrapping_add(menu_item())
        .wrapping_sub(1)
}

/// Saves the game in `slot`. Whether it worked is not looked at, as in the original. A slot
/// or a state the Rust writer refuses goes to the original's.
pub fn save_game(slot: u32) {
    if unsafe { write_savegame_slot(slot) }.is_err() {
        let mut regs = Regs {
            ebx: slot,
            ..Regs::default()
        };
        unsafe { call_game(&mut regs, game_addr(SAVE_GAME_RVA)) };
    }
}

/// `save` is called with the slot when Enter ends the entry.
pub unsafe fn edit_savegame_name(
    canvas: &mut dyn Canvas,
    save: &mut dyn FnMut(u32),
) -> Result<(), Refused> {
    // Everything that could be refused, before the key is taken.
    let name = savegame_name()?;
    if !(0..NAME_LEN as i32).contains(&length()) {
        return Err(BadInput("text length outside the name").into());
    }
    if !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }

    unsafe {
        let key = take_key();
        // Converted: `rc2k_core::menu::text_entry` (`key`, `appends`, `too_wide`).
        use rc2k_core::menu::text_entry::{self as entry, Key};
        match entry::key(key) {
            Key::None => {}
            Key::Escape => {
                restore_text(name);
                game_write(ENTERING_RVA, 0u32);
                return Ok(());
            }
            Key::Enter => {
                if length() == 0 {
                    restore_text(name);
                    add_text_length(name)?;
                }
                game_write(ENTERING_RVA, 0u32);
                end_name(name);
                save(highlighted_slot());
                end_name(name);
                game_write(ENTERING_RVA, 0u32);
                return Ok(());
            }
            Key::Clear => clear_text(name),
            Key::Backspace => backspace_savegame_name()?,
            Key::Other(key) => {
                let c = key_to_char(key as u32)? as u8;
                let len = length();
                if entry::appends(c, len, game_read(MAX_LENGTH_RVA)) {
                    name.offset(len as isize).write(c);
                    set_length(len.wrapping_add(1));
                    let measured = measure_text_at(font_at(MEASURING_FONT), name, len, canvas)?;
                    if entry::too_wide(measured.width as i32) {
                        backspace_savegame_name()?;
                    }
                }
            }
        }
        let mut tail = Vec::new();
        if game_read::<u32>(CURSOR_ON_RVA) == 1 {
            push_string(&mut tail, CURSOR_STRING);
        }
        tail.push(0);
        std::ptr::copy_nonoverlapping(tail.as_ptr(), name.offset(length() as isize), tail.len());
    }
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![at(LENGTH_RVA), at(ENTERING_RVA)];
    regions.extend(key_regions());
    if let Ok(name) = savegame_name() {
        regions.push((name as u32, NAME_LEN + 8));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    if shadow() {
        // The original has saved the game already.
        unsafe { edit_savegame_name(&mut RecordingCanvas::new(false), &mut |_| {}) }?;
    } else {
        unsafe { edit_savegame_name(&mut GameCanvas, &mut save_game) }?;
    }
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: the last name of the list, with the save replaced by a stand-in ----

const FUNCTION_LEN: usize = 0x20A;
const ENTRY_BYTES: [u8; 6] = [0x60, 0xE8, 0x0A, 0xA9, 0xF9, 0xFF]; // pusha; call take_key
const CALL_OFFSETS: [usize; 14] = [
    0x01, 0x30, 0x76, 0x7B, 0xB9, 0xCE, 0xFC, 0x10A, 0x114, 0x16D, 0x17A, 0x1B0, 0x1FE, 0x203,
];

static SAVES: AtomicU32 = AtomicU32::new(0);
static SAVED_SLOT: AtomicU32 = AtomicU32::new(0);

/// Stands in for the save function: notes the slot in ebx and counts the call.
#[unsafe(naked)]
unsafe extern "C" fn fake_save_game() {
    core::arch::naked_asm!(
        "mov [{slot}], ebx",
        "inc dword ptr [{saves}]",
        "ret",
        slot = sym SAVED_SLOT,
        saves = sym SAVES,
    )
}

fn rust_for_test(_: &Regs) -> Result<Left, Refused> {
    unsafe {
        edit_savegame_name(&mut RecordingCanvas::new(false), &mut |slot| {
            SAVED_SLOT.store(slot, Ordering::SeqCst);
            SAVES.fetch_add(1, Ordering::SeqCst);
        })
    }?;
    Ok(Left::nothing())
}

static COPY: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn copy_without_drawing(r: *mut Regs) {
    record_calls(true, || unsafe {
        call_game(r, COPY.load(Ordering::SeqCst))
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off");
    }
    if blob().is_null()
        || !is_table_id(CURSOR_STRING)
        || !font_is_loaded(MEASURING_FONT)
        || !has_layout()
    {
        return t.skip("no strings, measuring font or keyboard layout");
    }
    let redirect =
        |target: usize| (target == SAVE_GAME_RVA).then_some(fake_save_game as *const () as usize);
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x46D685)),
            FUNCTION_LEN,
            &ENTRY_BYTES,
            &CALL_OFFSETS,
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    COPY.store(copy.addr(), Ordering::SeqCst);

    let font = font_at(MEASURING_FONT);
    let saved_font = unsafe { font.read_unaligned() };
    let inputs = [
        at(SAVEGAME_PAGE_RVA),
        at(MENU_ITEM_RVA),
        at(CURSOR_ON_RVA),
        backup_region(),
        (game_addr(PENDING_KEY_RVA) as u32, 1),
        (game_addr(LEFT_SHIFT_RVA) as u32, 1),
        (game_addr(RIGHT_SHIFT_RVA) as u32, 1),
        (game_addr(ALT_RVA) as u32, 1),
        (game_addr(ALT_GR_RVA) as u32, 1),
    ];
    let saved_inputs = capture(&inputs);
    // Item 10 of page 9 is entry 99.
    unsafe {
        game_write(SAVEGAME_PAGE_RVA, 9u32);
        game_write(MENU_ITEM_RVA, 10u32);
    }
    let Ok(name) = savegame_name() else {
        restore(&inputs, &saved_inputs);
        return t.skip("the test entry is not in the list");
    };
    let regions = regions();
    let saved = capture(&regions);

    let wide = [b'W'; 29];
    let names: [(&[u8], i32); 5] = [
        (b"abc", 3),
        (b"", 0),
        (b"abcdefghijklmnopqrstuvwxyz1234", 30),
        (&wide, 29),
        (b"x", 1),
    ];
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
            for cursor_on in [0u32, 1] {
                let prepare = || unsafe {
                    font.write_unaligned(saved_font);
                    name.write_bytes(0xAA, NAME_LEN + 8);
                    std::ptr::copy_nonoverlapping(text.as_ptr(), name, text.len());
                    name.add(text.len()).write(0);
                    set_length(len);
                    backup().write_bytes(0, TEXT_LEN);
                    std::ptr::copy_nonoverlapping(b"before".as_ptr(), backup(), 6);
                    game_write(ENTERING_RVA, 1u32);
                    game_write(CURSOR_ON_RVA, cursor_on);
                    game_write(DEAD_KEY_RVA, 0u8);
                    for flag in [LEFT_SHIFT_RVA, RIGHT_SHIFT_RVA, ALT_RVA, ALT_GR_RVA] {
                        game_write(flag, 0u8);
                    }
                    pending_key().store(key, Ordering::SeqCst);
                    SAVES.store(0, Ordering::SeqCst);
                    SAVED_SLOT.store(0x5EED, Ordering::SeqCst);
                };
                check_state_with(
                    &mut t,
                    &format!(
                        "key={key:#x} name={:?} cursor_on={cursor_on}",
                        String::from_utf8_lossy(text)
                    ),
                    copy_without_drawing,
                    Regs::SENTINEL,
                    &prepare,
                    &regions,
                    &rust_for_test,
                    &|| {
                        vec![
                            pending_key().load(Ordering::SeqCst),
                            SAVES.load(Ordering::SeqCst) as u8,
                            SAVED_SLOT.load(Ordering::SeqCst) as u8,
                        ]
                    },
                );
            }
        }
    }

    unsafe { font.write_unaligned(saved_font) };
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    t.finish()
}
