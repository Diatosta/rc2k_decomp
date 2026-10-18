//! Front-end pages' "unread" steps that copy settings into the page's work variables (stream
//! A, part F1, 2026-10-18).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};

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
        w(0x75598C, n);
        w(0x7558C4, ((p + 0x2C) as *const u32).read_unaligned());
        std::ptr::copy(
            (p + 0x50) as *const u8,
            game_addr(rva(0x7555B1)) as *mut u8,
            0x2C,
        );
        Ok(())
    }
}

/// `screens::load_audio_settings` (0x4707E6): the audio options (`g_audioOptions`: quality,
/// channels, Dolby, A3D, EAX, DS3D) into the sound page's work variables (`dword_75566C`,
/// `dword_755680`, `dword_75567C`, `dword_755678`, `dword_755674`, `dword_755670`). Every
/// register kept.
pub unsafe fn load_audio_settings() {
    for (from, to) in [
        (0x506CA4u32, 0x75566Cu32),
        (0x506CA8, 0x755680),
        (0x506CB8, 0x75567C),
        (0x506CB4, 0x755678),
        (0x506CB0, 0x755674),
        (0x506CAC, 0x755670),
    ] {
        w(to, k(from));
    }
}

/// `strings::append_string_by_id` into `out`.
unsafe fn push_id(out: &mut Vec<u8>, id: u32) {
    unsafe { crate::strings::append_string_by_id::push_string(out, id) };
}

/// The stage line string `id`, " ", the record's +0x40 as a fraction of 6
/// (`strings::append_fraction`), ": ", the name of its track-pack entry
/// (`records::trakpak_entry`'s first dword, a string id), and its NUL.
pub unsafe fn stage_line_bytes(id: u32, record: u32) -> Result<Vec<u8>, Refused> {
    unsafe {
        let mut line = Vec::new();
        push_id(&mut line, id);
        line.push(b' ');
        line.extend(crate::strings::append_fraction::append_fraction(
            ((record + 0x40) as *const u32).read_unaligned(),
            6,
        ));
        line.extend(b": ");
        let entry = crate::records::trakpak_entry::record_entry(record as *const u8)?;
        push_id(&mut line, (entry as *const u32).read_unaligned());
        line.push(0);
        Ok(line)
    }
}

/// The stage line both stage pages build in `byte_A8BD54`.
unsafe fn stage_line(id: u32, record: u32) -> Result<(), Refused> {
    unsafe {
        let line = stage_line_bytes(id, record)?;
        if line.len() > 0x100 {
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
        let items = game_addr(rva(0x74DCE0)) as u32;
        set_item(items, 1, if a == 1 { 0x19C } else { 0x19B });
        set_item(items, 2, if k(0x7556FC) == 1 { 0x19E } else { 0x19D });
        let (base, record) = stage_record(a, b, k(0x758812));
        set_item(items, 3, ((base + 0x38) as *const u32).read_unaligned());
        stage_line(0x481, record)?;
        let mut caption = Vec::new();
        push_id(&mut caption, 0x4FC + k(0x75AB30));
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

/// Whether `stage_options_texts` would refuse (or meet another track pack): for its test.
pub unsafe fn stage_options_texts_refuses() -> bool {
    unsafe { stage_texts_refuse(k(0x7556F8), k(0x75880E), k(0x758812)) }
}

/// Whether `stage_choice_texts` would refuse (or meet another track pack): for its test.
pub unsafe fn stage_choice_texts_refuses() -> bool {
    unsafe { k(0x74DB68) == 0 || stage_texts_refuse(k(0x7558C8), k(0x7558D0), k(0x7558CC)) }
}

unsafe fn stage_texts_refuse(a: u32, b: u32, c: u32) -> bool {
    unsafe {
        if stage_refuses(a, b).is_err() {
            return true;
        }
        let (_, record) = stage_record(a, b, c);
        crate::records::trakpak_entry::record_entry(record as *const u8).is_err()
    }
}
