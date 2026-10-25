//! Front-end pages' "unread" steps that copy settings into the page's work variables (stream
//! A, part F1, 2026-10-18).

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::menu::state_store::MENU_STATE;
use crate::store::{game_addr, game_read, game_write};

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

/// `screens::pick_player_1` .. `_4` (0x46AEC5, 0x46AF21, 0x46AF58, 0x46AF8F): player `n`
/// the current one (`dword_75598C`), its +0x2C into `dword_7558C4` and its 0x2C bytes at
/// +0x50 into the page's copy (`unk_7555B1`). Every register kept. Refuses (before
/// anything) a player without its record.
pub unsafe fn pick_player(n: u32) -> Result<(), Refused> {
    unsafe {
        let p = crate::menu::player::player_at(n);
        if p == 0 {
            return Err(BadInput("a player without its record").into());
        }
        // Converted: `rc2k_core::screens::page_values` (`Player::picked`).
        let (value, copy) = (p as *const rc2k_core::menu::player::Player)
            .read_unaligned()
            .picked();
        w(0x75598C, n);
        w(0x7558C4, value);
        game_write(rva(0x7555B1), copy);
        Ok(())
    }
}

/// `screens::load_audio_settings` (0x4707E6): the audio options (`g_audioOptions`: quality,
/// channels, Dolby, A3D, EAX, DS3D) into the sound page's work variables (`dword_75566C`,
/// `dword_755680`, `dword_75567C`, `dword_755678`, `dword_755674`, `dword_755670`). Every
/// register kept.
pub unsafe fn load_audio_settings() {
    // Converted: `rc2k_core::screens::page_values::audio_page`.
    let options: rc2k_core::audio::modes::AudioOptions =
        crate::audio::store::AUDIO.audio_options.get();
    unsafe {
        game_write(
            rva(0x75566C),
            rc2k_core::screens::page_values::audio_page(&options),
        )
    };
}

/// `strings::append_string_by_id` into `out`.
unsafe fn push_id(out: &mut Vec<u8>, id: u32) {
    unsafe { crate::strings::append_string_by_id::push_string(out, id) };
}

/// The stage line string `id`, " ", the record's +0x40 as a fraction of 6
/// (`strings::append_fraction`), ": ", the name of its track-pack entry
/// (`records::trakpak_entry`'s first dword, a string id), and its NUL.
pub unsafe fn stage_line_bytes(id: u32, record: u32) -> Result<Vec<u8>, Refused> {
    Ok(unsafe { crate::screens::draw_stage_heading::heading_text(record as *const u8, id) }?)
}

/// The stage line both stage pages build in `byte_A8BD54`.
unsafe fn stage_line(id: u32, record: u32) -> Result<(), Refused> {
    unsafe {
        let line = stage_line_bytes(id, record)?;
        if line.len() > rc2k_core::screens::page_values::STAGE_LINE_ROOM {
            return Err(BadInput("a stage line past its buffer").into());
        }
        std::ptr::copy_nonoverlapping(
            line.as_ptr(),
            game_addr(rva(0xA8BD54)) as *mut u8,
            line.len(),
        );
        Ok(())
    }
}

/// Menu item `n`'s value (+0x14) of the item table that starts 4 bytes into `items`.
fn set_item(items: u32, n: u32, v: u32) {
    unsafe { ((items + 4 + 0x5C * n + 0x14) as *mut u32).write_unaligned(v) };
}

/// The stage record `off_735512[a][b]` (its +0x38 the page's value), and the record + `c` x
/// 0xA4 the line is about.
unsafe fn stage_record(a: u32, b: u32, c: u32) -> (u32, u32) {
    unsafe {
        let table = ((game_addr(rva(0x735512)) as u32 + 4 * a) as *const u32).read_unaligned();
        let base = ((table + 4 * b) as *const u32).read_unaligned();
        (base, base.wrapping_add(c.wrapping_mul(0xA4)))
    }
}

/// What both stage pages read through: refused before anything.
unsafe fn stage_refuses(a: u32, b: u32) -> Result<(), Refused> {
    unsafe {
        let table = ((game_addr(rva(0x735512)) as u32 + 4 * a) as *const u32).read_unaligned();
        if table == 0 || ((table + 4 * b) as *const u32).read_unaligned() == 0 {
            return Err(BadInput("no stage records").into());
        }
        Ok(())
    }
}

/// `screens::stage_options_texts` (0x4692A9): the stage options page's items in the table at
/// `off_74DCE0` (its address: items 1 and 2 string 0x19C / 0x19B by `dword_7556F8`, 0x19E /
/// 0x19D by `dword_7556FC`, item 3 the record's +0x38), the stage line (string 0x481) and the
/// caption string 0x4FC + `dword_75AB30`. Every register kept. Refuses (before anything) no
/// stage records; a record of another track pack is the original's fatal error.
pub unsafe fn stage_options_texts() -> Result<(), Refused> {
    unsafe {
        let (a, b) = (k(0x7556F8), k(0x75880E));
        stage_refuses(a, b)?;
        // Converted: `rc2k_core::screens::page_values` (the items and the caption).
        use rc2k_core::screens::page_values::{stage_options_caption, stage_options_items};
        let items = game_addr(rva(0x74DCE0)) as u32;
        let (first, second) = stage_options_items(a, k(0x7556FC));
        set_item(items, 1, first);
        set_item(items, 2, second);
        let (base, record) = stage_record(a, b, k(0x758812));
        set_item(items, 3, ((base + 0x38) as *const u32).read_unaligned());
        stage_line(0x481, record)?;
        let mut caption = Vec::new();
        push_id(
            &mut caption,
            stage_options_caption(MENU_STATE.menus.caption_choice.get()),
        );
        caption.push(0);
        std::ptr::copy_nonoverlapping(
            caption.as_ptr(),
            game_addr(crate::screens::best_scores::CAPTION_RVA) as *mut u8,
            caption.len(),
        );
        Ok(())
    }
}

/// `screens::stage_choice_texts` (0x469434): the stage choice page's items: item 0 of the
/// table at `off_74DB68` (its address) `dword_758718[dword_7558C8]`, item 1 of the table it
/// points to the record's +0x38 (the original reads the table both ways: `notes/bugs.md`),
/// then the stage line (string 0x482) for record `dword_7558CC` of `off_735512[dword_7558C8]
/// [dword_7558D0]`. Every register kept. Refuses (before anything) no stage records or no
/// item table; another track pack is the original's fatal error.
pub unsafe fn stage_choice_texts() -> Result<(), Refused> {
    unsafe {
        let (a, b) = (k(0x7558C8), k(0x7558D0));
        stage_refuses(a, b)?;
        let pointed = k(0x74DB68);
        if pointed == 0 {
            return Err(BadInput("no item table").into());
        }
        let items = game_addr(rva(0x74DB68)) as u32;
        set_item(
            items,
            0,
            ((game_addr(rva(0x758718)) as u32 + 4 * a) as *const u32).read_unaligned(),
        );
        let (base, record) = stage_record(a, b, k(0x7558CC));
        set_item(pointed, 1, ((base + 0x38) as *const u32).read_unaligned());
        stage_line(0x482, record)
    }
}
