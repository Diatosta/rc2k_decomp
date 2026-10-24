//! The race's sound set-up (`sub_415C93`, IDA 2026-10-17): the sound tables made ready
//! once (`race::prepare_sound_tables`: whole numbers made floats, spans, reciprocals, a
//! sine table) and the race's sounds restarted or silenced. Here the steps.

use crate::errors::Refused;
use crate::framework::rva;
use crate::sound::store::ClockSlot;
use crate::store::game_addr;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) };
}

/// `race::whole_to_float` (0x40D706): the dword at ebx made a float (`fild` / `fstp`)
/// when it looks like a whole number (its top byte 0 or 0xFF). Leaves eax its top byte's
/// mask (`and 0xFF000000`).
pub unsafe fn whole_to_float(at: u32) -> u32 {
    // Converted: `rc2k_core::race::sound_tables::whole_to_float` (one rounding: the same
    // at every precision, so no exact twin).
    unsafe {
        let (top, float) = rc2k_core::race::sound_tables::whole_to_float(get(at));
        if let Some(f) = float {
            put(at, f);
        }
        top
    }
}

fn span_plain(second: u32, first: u32) -> u32 {
    rc2k_core::race::sound_tables::span(f32::from_bits(first), f32::from_bits(second)).to_bits()
}

/// `race::sound_span` (0x40D71B), converted: the two dwords at `at` made floats, the third
/// (at `at` + 8) their difference (`fld` / `fsub` / `fstp`). Its leftovers (eax, ebx) are
/// read by no caller (`prepare_sound_tables` and `sound_row` only).
// Converted: `rc2k_core::race::sound_tables::span`.
pub unsafe fn sound_span(at: u32) {
    unsafe {
        whole_to_float(at);
        whole_to_float(at + 4);
        put(at + 8, span_plain(get(at + 4), get(at)));
    }
}

/// `race::sound_row` (0x40D731), converted: a row at `row`: +4, +8, +0xC made floats, spans
/// at +0x10 and +0x1C. Its leftovers are read by no caller.
// Converted: `rc2k_core::race::sound_tables::{whole_to_float, span}`.
pub unsafe fn sound_row(row: u32) {
    unsafe {
        for off in [4, 8, 0xC] {
            whole_to_float(row + off);
        }
        sound_span(row + 0x10);
        sound_span(row + 0x1C);
    }
}

/// Where `prepare_sound_tables` finds its variables (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct TableVars {
    /// dword_516358: 1 once done.
    pub done: u32,
    /// off_514F90: 0x22 record pointers.
    pub records: u32,
    /// unk_516258: 0x40 floats.
    pub sines: u32,
    /// dword_5183CC: the scratch.
    pub scratch: u32,
}

impl TableVars {
    pub fn game() -> Self {
        Self {
            done: crate::sound::store::SurfaceSlot::at(0x516358).address() as u32,
            records: game_addr(rva(0x514F90)) as u32,
            sines: game_addr(rva(0x516258)) as u32,
            scratch: crate::session::MACHINE.scratch.addr() as u32,
        }
    }
}

pub const RECORDS: u32 = 0x22;
/// `race::prepare_sound_tables` (0x40D636), converted (`set_up_race_sound_objects`'s too): once (`dword_516358`): each of the 0x22 records
/// (`off_514F90`) its span at +4 (`sound_span`), +0xC made its reciprocal (`fld1` /
/// `fdiv`), its three rows (`sound_row`, 0x28 apart from +0x10); then the 0x40 floats at
/// 0x516258 1 + 0.05 sin(x) for x from 0 by pi / 32 (the scratch `dword_5183CC` left the
/// last x). Every register kept.
// Converted: `rc2k_core::race::sound_tables::{reciprocal, sines}`.
pub unsafe fn prepare_sound_tables_plain(v: &TableVars) {
    use rc2k_core::race::sound_tables::{reciprocal, sines};
    unsafe {
        if get(v.done) == 1 {
            return;
        }
        put(v.done, 1);
        for n in 0..RECORDS {
            let record = get(v.records + 4 * n);
            sound_span(record + 4);
            put(
                record + 0xC,
                reciprocal(f32::from_bits(get(record + 0xC))).to_bits(),
            );
            for k in 0..3 {
                sound_row(record + 0x10 + 0x28 * k);
            }
        }
        let (table, last) = sines();
        for (k, f) in table.iter().enumerate() {
            put(v.sines + 4 * k as u32, f.to_bits());
        }
        // The scratch: 0x20, the step, then each x: the last left.
        put(v.scratch, last.to_bits());
    }
}

/// `race::restart_race_sounds`: the commands waited for, the race's sounds reset, let
/// play, `dword_5124E4` 1. Every register kept.
pub fn restart_race_sounds() -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, reset_race_sounds, wait_for_commands};
    wait_for_commands();
    reset_race_sounds(&SoundVars::game())?;
    crate::sound::let_sounds_play::let_sounds_play();
    unsafe { crate::store::game_write(ClockSlot::at(0x5124E4), 1u32) };
    Ok(())
}

/// `race::silence_race_sounds` (0x40D366): the race's sounds reset, command 4, the
/// commands waited for. Every register kept.
pub fn silence_race_sounds() -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, command, reset_race_sounds, wait_for_commands};
    reset_race_sounds(&SoundVars::game())?;
    command(4, 0);
    wait_for_commands();
    Ok(())
}

// ---- the race's sound objects ----

/// Where `set_up_race_sound_objects` finds the objects: the pointer table
/// (`g_raceSoundObjects`, 0x512A10) and the eight objects it names, also written by their
/// own addresses (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct ObjectVars {
    pub table: u32,
    pub objects: [u32; 8],
}

impl ObjectVars {
    pub fn game() -> Self {
        Self {
            table: crate::sound::store::OBJECT_TABLE.address() as u32,
            objects: std::array::from_fn(|k| {
                (crate::sound::store::FIRST_OBJECT + crate::sound::store::OBJECT_LEN * k).address()
                    as u32
            }),
        }
    }
}

/// The fields each object's set-up clears (0) or sets (1): (offset, value).
const OBJECT_FIELDS: [(u32, u32); 24] = [
    (0x10, 0),
    (0x8, 0),
    (0xC, 1),
    (0x14, 0),
    (0x18, 0),
    (0x1C, 0),
    (0x20, 0),
    (0x94, 0),
    (0x98, 0),
    (0x9C, 0),
    (0xA0, 0),
    (0xA4, 0),
    (0xA8, 0),
    (0x24, 1),
    (0xF0, 0),
    (0xF4, 0),
    (0x84, 0),
    (0xC0, 0),
    (0xDC, 0),
    (0xAC, 0),
    (0x278, 0),
    (0x27C, 0),
    (0x280, 0),
    (0x10C, 0),
];

fn rd(a: impl crate::store::At) -> u32 {
    unsafe { crate::store::at_read(a) }
}

fn wr(a: impl crate::store::At, v: u32) {
    unsafe { crate::store::at_write(a, v) };
}

/// `race::set_up_race_sound_objects` (0x415C93): `prepare_sound_tables`; the players'
/// count (`dword_514F78`: 1, 2 with `dword_75ACE0` not 1), the objects' count
/// (`dword_514F7C`: that, 4 in mode 2 with `dword_514F8C` 1), the players' objects
/// (`dword_514F70` 0 or the chosen car `dword_52A920` with `dword_52A910` 1, which also
/// gives the count `dword_52AD98`; `dword_514F74` 1); each object's first dword -1 .. -8;
/// the players' objects their number, kind and setting; each object's 0x68 bytes from
/// +0x2C and 24 fields reset; `dword_5BEBE8` 0; `restart_race_sounds`. Every register
/// kept. Refuses, before anything, a chosen car past 7 (the original reads past the
/// table) or an object missing.
pub fn set_up_race_sound_objects(v: &ObjectVars) -> Result<(), Refused> {
    unsafe {
        let one = rd(0x52A910) == 1;
        if one && rd(0x52A920) > 7 {
            return Err(crate::errors::BadInput("a chosen car past the eight").into());
        }
        let objects: Vec<u32> = (0..8).map(|n| get(v.table + 4 * n)).collect();
        if objects.contains(&0) {
            return Err(crate::errors::BadInput("a race sound object missing").into());
        }
        // Converted: `rc2k_core::race::sound_objects::{sound_players, sound_objects,
        // object_mark}`; the tables through `prepare_sound_tables_plain`.
        use rc2k_core::race::sound_objects::{object_mark, sound_objects, sound_players};
        prepare_sound_tables_plain(&TableVars::game());
        let players = sound_players(rd(0x75ACE0));
        wr(crate::sound::store::ObjectsSlot::at(0x514F78), players);
        let (count, arcade) = sound_objects(players, rd(0x75ACD8));
        if arcade {
            wr(crate::sound::store::ObjectsSlot::at(0x514F8C), 1);
        }
        wr(crate::sound::store::ObjectsSlot::at(0x514F7C), count);
        wr(crate::sound::store::ObjectsSlot::at(0x514F70), 0);
        wr(crate::sound::store::ObjectsSlot::at(0x514F74), 1);
        if one {
            wr(crate::sound::store::ObjectsSlot::at(0x514F70), rd(0x52A920));
            wr(crate::sound::store::ObjectsSlot::at(0x514F7C), rd(0x52AD98));
        }
        for (n, &o) in v.objects.iter().enumerate() {
            put(o, object_mark(n as u32));
        }
        let first = objects[rd(crate::sound::store::ObjectsSlot::at(0x514F70)) as usize];
        put(first, 0);
        put(first + 4, rd(0x75ACE8));
        put(first + 0x298, rd(0x75AE20));
        if rd(crate::sound::store::ObjectsSlot::at(0x514F78)) != 1 {
            let second = objects[rd(crate::sound::store::ObjectsSlot::at(0x514F74)) as usize];
            put(second, 1);
            put(second + 4, rd(0x75ACEC));
            put(second + 0x298, rd(0x75AF8C));
        }
        for &o in &objects {
            std::ptr::write_bytes((o + 0x2C) as *mut u8, 0, 0x68);
            for &(off, val) in &OBJECT_FIELDS {
                put(o + off, val);
            }
        }
        wr(0x5BEBE8, 0);
        restart_race_sounds()?;
    }
    Ok(())
}

/// `race::clear_race_sound_objects` (0x415C6D): `silence_race_sounds`; `dword_514F78`,
/// `dword_514F7C`, `dword_514F8C` 0. Every register kept.
pub fn clear_race_sound_objects() -> Result<(), Refused> {
    silence_race_sounds()?;
    for a in [
        crate::sound::store::ObjectsSlot::at(0x514F78),
        crate::sound::store::ObjectsSlot::at(0x514F7C),
        crate::sound::store::ObjectsSlot::at(0x514F8C),
    ] {
        wr(a, 0);
    }
    Ok(())
}
