//! `service::edit_setup_name` (0x46F1A9): one step of typing the name of a setup slot on the
//! "save setup" screen; Enter saves the setup.
//!
//! In:  nothing
//! Out: every register preserved (carry is set on Escape; the caller does not read it)
//! The name is the highlighted slot's entry in the slot list, with the text entry's cursor
//! (string 4) appended while typing. One pending key is taken:
//!   none          the cursor is written again;
//!   1 (Escape)    the name from before comes back, the entry ends;
//!   0x1C (Enter)  an empty name is replaced by the one from before; the entry ends, the
//!                 name is cut at its length and `write_setup_slot` saves the slot (whether
//!                 that worked is not looked at);
//!   0x6E          the name is cleared;
//!   0x0E          the last character goes (`backspace_setup_name`);
//!   any other     `key_to_char`; a character is appended unless the length is already above
//!                 the maximum (`dword_755571`), and taken off again if the name then
//!                 measures 0xEC pixels or more in the font at 0x51E824.
//! Called from the screen loop `sub_46EB7A` while `dword_7556A8` is 1.
//! Refuses, before the key is taken: a slot past 99, a length outside 0 .. 0x1F, no car
//! object, no keyboard layout. Manual patch with a private copy (0x251 bytes, 14 calls
//! relocated): a call sits in the first bytes.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::buffer as path_buffer;
use crate::files::read_file::LAST_FILE_PATH_RVA;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::input::key_to_char::{
    ALT_GR_RVA, ALT_RVA, DEAD_KEY_RVA, LEFT_SHIFT_RVA, RIGHT_SHIFT_RVA, has_layout, key_to_char,
    regions as key_regions,
};
use crate::input::take_key::{pending_key, take_key};
use crate::menu::add_text_length::add_text_length;
use crate::menu::clear_text::clear_text;
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{
    ENTERING_RVA, LENGTH_RVA, MAX_LENGTH_RVA, TEXT_LEN, backup, backup_region, set_length,
};
use crate::screens::harness::Left;
use crate::screens::{font_at, font_is_loaded};
use crate::service::backspace_setup_name::{TEST_ITEM, TEST_PAGE, backspace_setup_name, entry};
use crate::service::harness::{
    Region, at, capture, check_state_with, dispatch_keyed_state, restore,
};
use crate::service::load_setup_slot::{SLOT_PAGE_RVA, highlighted_slot};
use crate::service::read_setup_slot::{
    NAME_LEN, SETUP_FILE_LEN, SETUP_FILE_RVA, SLOT_COUNT, slot_name,
};
use crate::service::state::{MENU_ITEM_RVA, current_player};
use crate::service::write_setup_slot::write_setup_slot;
use crate::strings::append_string_by_id::push_string;
use crate::strings::string_table::{blob, is_table_id};
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
use crate::text::measure_text::measure_text_at;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::edit_setup_name", rva(0x46F1A9), thunk, &ORIGINAL)
        .private_copy(0x251)
        .relocate_calls(&[
            0x01, 0x3A, 0x8A, 0x8F, 0xD7, 0xF6, 0x12E, 0x13C, 0x146, 0x1A9, 0x1B6, 0x1F6, 0x245,
            0x24A,
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

const KEY_ESCAPE: u8 = 1;
const KEY_ENTER: u8 = 0x1C;
const KEY_CLEAR: u8 = 0x6E;
const KEY_BACKSPACE: u8 = 0x0E;

const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: usize = 0x51E824; // dword_51E824

// ---- Rust version ----

/// Writes the cursor and a NUL after the name's `len` characters.
unsafe fn put_cursor(name: *mut u8) {
    unsafe {
        let mut tail = Vec::new();
        push_string(&mut tail, CURSOR_STRING);
        tail.push(0);
        let len: i32 = game_read(LENGTH_RVA);
        std::ptr::copy_nonoverlapping(tail.as_ptr(), name.offset(len as isize), tail.len());
    }
}

unsafe fn end_name(name: *mut u8) {
    unsafe {
        let len: i32 = game_read(LENGTH_RVA);
        name.offset(len as isize).write(0);
    }
}

pub unsafe fn edit_setup_name(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Everything that could be refused, before the key is taken.
    let (name, _) = entry()?;
    let slot = highlighted_slot();
    if current_player().is_none() {
        return Err(BadInput("no current player object").into());
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
                if game_read::<i32>(LENGTH_RVA) == 0 {
                    restore_text(name);
                    add_text_length(name)?;
                }
                game_write(ENTERING_RVA, 0u32);
                end_name(name);
                write_setup_slot(slot)?;
                end_name(name);
                game_write(ENTERING_RVA, 0u32);
                return Ok(());
            }
            Key::Clear => clear_text(name),
            Key::Backspace => backspace_setup_name()?,
            Key::Other(key) => {
                let c = key_to_char(key as u32)? as u8;
                let len: i32 = game_read(LENGTH_RVA);
                if entry::appends(c, len, game_read(MAX_LENGTH_RVA)) {
                    name.offset(len as isize).write(c);
                    set_length(len.wrapping_add(1));
                    // x is whatever ecx held: the length before this character.
                    let measured = measure_text_at(font_at(MEASURING_FONT), name, len, canvas)?;
                    if entry::too_wide(measured.width as i32) {
                        backspace_setup_name()?;
                    }
                }
            }
        }
        put_cursor(name);
    }
    Ok(())
}

// ---- hook ----

/// What a step may write, except the pending key: the name (and a little past it, where the
/// cursor of a full name lands), the entry's state, what `key_to_char` and `write_setup_slot`
/// write.
fn regions() -> Vec<Region> {
    let mut regions = vec![at(LENGTH_RVA), at(ENTERING_RVA)];
    regions.extend(key_regions());
    let slot = highlighted_slot();
    if slot < SLOT_COUNT {
        regions.push((slot_name(slot) as u32, NAME_LEN + 8));
    }
    let file = game_addr(SETUP_FILE_RVA) as u32;
    regions.push((file, 0xC));
    regions.push((file + 0x10, SETUP_FILE_LEN - 0x10));
    regions.push((path_buffer() as u32, 0x100));
    regions.push(at(LAST_FILE_PATH_RVA));
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    // In shadow mode the original has drawn already (the measuring font draws nothing).
    if shadow() {
        unsafe { edit_setup_name(&mut RecordingCanvas::new(false)) }?;
    } else {
        unsafe { edit_setup_name(&mut GameCanvas) }?;
    }
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: slot 99 (page 3, item 10); its name and file are put back ----

const TEST_FILE: &str = "savedata\\carsetup\\carset99";

/// The test slot's file without its time stamp, or one byte if there is none; the file is
/// removed, so that the next run has to write it again.
fn written_file() -> Vec<u8> {
    let Ok(mut bytes) = std::fs::read(TEST_FILE) else {
        return vec![0xFF];
    };
    let _ = std::fs::remove_file(TEST_FILE);
    if bytes.len() >= 0x10 {
        bytes[0xC..0x10].fill(0);
    }
    bytes
}

unsafe extern "C" fn original_without_drawing(r: *mut Regs) {
    record_calls(true, || unsafe { call_original(r) });
}

fn rust_without_drawing(_: &Regs) -> Result<Left, Refused> {
    unsafe { edit_setup_name(&mut RecordingCanvas::new(false)) }?;
    Ok(Left::nothing())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off");
    }
    if current_player().is_none()
        || blob().is_null()
        || !is_table_id(CURSOR_STRING)
        || !font_is_loaded(MEASURING_FONT)
        || !has_layout()
    {
        return t.skip("no player object, strings, measuring font or keyboard layout");
    }
    let old_file = std::fs::read(TEST_FILE).ok();
    let font = font_at(MEASURING_FONT);
    let saved_font = unsafe { font.read_unaligned() };
    let inputs = [
        at(SLOT_PAGE_RVA),
        at(MENU_ITEM_RVA),
        backup_region(),
        (game_addr(crate::input::take_key::PENDING_KEY_RVA) as u32, 1),
        (game_addr(LEFT_SHIFT_RVA) as u32, 1),
        (game_addr(RIGHT_SHIFT_RVA) as u32, 1),
        (game_addr(ALT_RVA) as u32, 1),
        (game_addr(ALT_GR_RVA) as u32, 1),
    ];
    let saved_inputs = capture(&inputs);
    unsafe {
        game_write(SLOT_PAGE_RVA, TEST_PAGE);
        game_write(MENU_ITEM_RVA, TEST_ITEM);
    }
    let regions = regions();
    let saved = capture(&regions);
    let name = slot_name(99);

    // (what the name holds, the length typed so far)
    let wide = [b'W'; 29];
    let names: [(&[u8], i32); 5] = [
        (b"abc", 3),
        (b"", 0),
        (b"abcdefghijklmnopqrstuvwxyz1234", 30),
        (&wide, 29),
        (b"x", 1),
    ];
    // Escape, Enter, clear, backspace, 'a', space, a key release, no key.
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
                name.write_bytes(0xAA, NAME_LEN + 8);
                std::ptr::copy_nonoverlapping(text.as_ptr(), name, text.len());
                set_length(len);
                put_cursor(name);
                backup().write_bytes(0, TEXT_LEN);
                std::ptr::copy_nonoverlapping(b"before".as_ptr(), backup(), 6);
                game_write(ENTERING_RVA, 1u32);
                game_write(DEAD_KEY_RVA, 0u8);
                for flag in [LEFT_SHIFT_RVA, RIGHT_SHIFT_RVA, ALT_RVA, ALT_GR_RVA] {
                    game_write(flag, 0u8);
                }
                pending_key().store(key, Ordering::SeqCst);
            };
            check_state_with(
                &mut t,
                &format!("key={key:#x} name={:?}", String::from_utf8_lossy(text)),
                original_without_drawing,
                Regs::SENTINEL,
                &prepare,
                &regions,
                &rust_without_drawing,
                &|| {
                    let mut seen = written_file();
                    seen.push(pending_key().load(Ordering::SeqCst));
                    seen
                },
            );
        }
    }

    unsafe { font.write_unaligned(saved_font) };
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    match old_file {
        Some(bytes) => {
            let _ = std::fs::write(TEST_FILE, bytes);
        }
        None => {
            let _ = std::fs::remove_file(TEST_FILE);
        }
    }
    t.finish()
}
