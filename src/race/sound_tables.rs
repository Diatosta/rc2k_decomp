//! The race's sound set-up (`sub_415C93`, IDA 2026-10-17): the sound tables made ready
//! once (`race::prepare_sound_tables`: whole numbers made floats, spans, reciprocals, a
//! sine table) and the race's sounds restarted or silenced. Here the steps.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::math::{x87, x87_load};
use crate::service::harness::Region;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

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

/// The span's subtraction: the x87's (exact) or the core's (plain).
type Sub = fn(u32, u32) -> u32;

fn span_plain(second: u32, first: u32) -> u32 {
    rc2k_core::race::sound_tables::span(f32::from_bits(first), f32::from_bits(second)).to_bits()
}

unsafe fn span_with(entry: &Regs, sub: Sub) -> Regs {
    let mut g = *entry;
    unsafe {
        whole_to_float(g.ebx);
        g.ebx += 4;
        g.eax = whole_to_float(g.ebx);
        put(g.ebx + 4, sub(get(g.ebx), get(g.ebx - 4)));
    }
    g
}

/// `race::sound_span` (0x40D71B): two dwords from ebx made floats, the third their
/// difference (`fld` / `fsub` / `fstp`). Leaves eax as the second `whole_to_float`, ebx
/// at the second. The exact x87 version (`prepare_sound_tables`' for
/// `race::set_up_race_sound_objects`, not yet converted).
pub unsafe fn sound_span(entry: &Regs) -> Regs {
    unsafe { span_with(entry, x87::sub) }
}

/// `sound_span` converted: the hook's.
// Converted: `rc2k_core::race::sound_tables::span`.
pub unsafe fn sound_span_plain(entry: &Regs) -> Regs {
    unsafe { span_with(entry, span_plain) }
}

/// `race::sound_row` (0x40D731): a row at edx: +4, +8, +0xC made floats, spans at +0x10
/// and +0x1C. Leaves ebx at edx + 0x20, eax as the last.
pub unsafe fn sound_row(entry: &Regs) -> Regs {
    unsafe { row_with(entry, x87::sub) }
}

/// `sound_row` converted: the hook's.
// Converted: `rc2k_core::race::sound_tables::{whole_to_float, span}`.
pub unsafe fn sound_row_plain(entry: &Regs) -> Regs {
    unsafe { row_with(entry, span_plain) }
}

unsafe fn row_with(entry: &Regs, sub: Sub) -> Regs {
    let mut g = *entry;
    unsafe {
        for off in [4, 8, 0xC] {
            g.ebx = g.edx + off;
            g.eax = whole_to_float(g.ebx);
        }
        g.ebx = g.edx + 0x10;
        g = span_with(&g, sub);
        g.ebx = g.edx + 0x1C;
        g = span_with(&g, sub);
    }
    g
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
            done: game_addr(rva(0x516358)) as u32,
            records: game_addr(rva(0x514F90)) as u32,
            sines: game_addr(rva(0x516258)) as u32,
            scratch: game_addr(rva(0x5183CC)) as u32,
        }
    }
}

pub const RECORDS: u32 = 0x22;
pub const RECORD_LEN: usize = 0x88;
pub const SINES: u32 = 0x40;

/// `race::prepare_sound_tables` (0x40D636): once (`dword_516358`): each of the 0x22
/// records (`off_514F90`) its span at +4 (`sound_span`), +0xC made its reciprocal
/// (`fld1` / `fdiv`), its three rows (`sound_row`, 0x28 apart from +0x10); then the 0x40
/// floats at 0x516258 1 + 0.05 sin(x) for x from 0 by pi / 32 (each step rounded to a
/// float through `dword_5183CC`, left the last). Every register kept.
pub unsafe fn prepare_sound_tables(v: &TableVars) {
    unsafe {
        if get(v.done) == 1 {
            return;
        }
        put(v.done, 1);
        for n in 0..RECORDS {
            let record = get(v.records + 4 * n);
            sound_span(&Regs {
                ebx: record + 4,
                ..Regs::default()
            });
            put(record + 0xC, x87::reciprocal(get(record + 0xC)));
            for k in 0..3 {
                sound_row(&Regs {
                    edx: record + 0x10 + 0x28 * k,
                    ..Regs::default()
                });
            }
        }
        put(v.scratch, 0x20);
        let step = x87::pi_over(0x20);
        put(v.scratch, step);
        let mut x = 0u32;
        for k in 0..SINES {
            put(
                v.sines + 4 * k,
                x87_load::sine_times_plus(x, 0x3D4C_CCCD, 0x3F80_0000),
            );
            x = x87::add(x, step);
            put(v.scratch, x);
        }
    }
}

/// `prepare_sound_tables` converted: the hook's.
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
            sound_span_plain(&Regs {
                ebx: record + 4,
                ..Regs::default()
            });
            put(
                record + 0xC,
                reciprocal(f32::from_bits(get(record + 0xC))).to_bits(),
            );
            for k in 0..3 {
                sound_row_plain(&Regs {
                    edx: record + 0x10 + 0x28 * k,
                    ..Regs::default()
                });
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

/// What `prepare_sound_tables` writes.
/// The float places `prepare_sound_tables` writes: each record's span and reciprocal
/// (+0xC) and its rows' spans; the sines and the scratch.
pub fn table_floats(v: &TableVars) -> Vec<(Region, crate::service::floats::Tolerance)> {
    use crate::service::floats::FLOAT;
    let mut f = Vec::new();
    for n in 0..RECORDS {
        let record = unsafe { get(v.records + 4 * n) };
        f.push(((record + 0xC, 4), FLOAT));
        for k in 0..3 {
            for o in [0x28, 0x34] {
                f.push(((record + o + 0x28 * k, 4), FLOAT));
            }
        }
    }
    f.extend([
        ((v.sines, 4 * SINES as usize), FLOAT),
        ((v.scratch, 4), FLOAT),
    ]);
    f
}

pub fn table_regions(v: &TableVars) -> Vec<Region> {
    let mut r: Vec<Region> = (0..RECORDS)
        .map(|n| (unsafe { get(v.records + 4 * n) }, RECORD_LEN))
        .collect();
    r.extend([(v.done, 4), (v.sines, 4 * SINES as usize), (v.scratch, 4)]);
    r
}

/// The sound parts of `restart_race_sounds` and `silence_race_sounds`, by place.
pub const WAIT: u32 = 1;
pub const RESET: u32 = 2;
pub const LET_PLAY: u32 = 3;
pub const COMMAND_4: u32 = 4;

/// `race::restart_race_sounds` (0x40D34A): the commands waited for, the race's sounds
/// reset (`race::reset_race_sounds`), let play; `dword_5124E4` 1. Every register kept.
pub unsafe fn restart_race_sounds(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let g = *entry;
    let mut r = g;
    part(WAIT, &mut r)?;
    part(RESET, &mut r)?;
    part(LET_PLAY, &mut r)?;
    unsafe { crate::framework::game_write(rva(0x5124E4), 1u32) };
    Ok(g)
}

/// `race::silence_race_sounds` (0x40D366): the race's sounds reset, command 4, the
/// commands waited for. Every register kept.
pub unsafe fn silence_race_sounds(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let g = *entry;
    let mut r = g;
    part(RESET, &mut r)?;
    part(COMMAND_4, &mut r)?;
    part(WAIT, &mut r)?;
    Ok(g)
}

/// Their parts in the game: the Rust versions.
pub fn sound_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, reset_race_sounds};
    use crate::sound::queue::{Queue, send_command, sent};
    use crate::sound::wait_for_commands::{Game, wait_for_commands};
    unsafe {
        match place {
            WAIT => wait_for_commands(&mut Game(sent(r))),
            RESET => {
                *r = reset_race_sounds(
                    r,
                    &SoundVars::game(),
                    &mut crate::race::race_end::sound_part,
                )?
            }
            LET_PLAY => crate::sound::let_sounds_play::let_sounds_play(),
            _ => send_command(&Queue::game(), 4, sent(r)),
        }
    }
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

/// The objects' addresses as IDA shows them.
pub const OBJECTS_IDA: [usize; 8] = [
    0x512A30, 0x512ED8, 0x513380, 0x513828, 0x513CD0, 0x514178, 0x514620, 0x514AC8,
];
pub const OBJECT_LEN: usize = 0x4A8;

impl ObjectVars {
    pub fn game() -> Self {
        Self {
            table: game_addr(rva(0x512A10)) as u32,
            objects: OBJECTS_IDA.map(|a| game_addr(rva(a)) as u32),
        }
    }
}

/// Places of `set_up_race_sound_objects`' and `clear_race_sound_objects`' calls.
pub const PREPARE: u32 = 5;
pub const RESTART: u32 = 6;
pub const SILENCE: u32 = 7;

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

fn rd(a: usize) -> u32 {
    unsafe { crate::framework::game_read(rva(a)) }
}

fn wr(a: usize, v: u32) {
    unsafe { crate::framework::game_write(rva(a), v) };
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
pub unsafe fn set_up_race_sound_objects(
    entry: &Regs,
    v: &ObjectVars,
    part: Part,
) -> Result<Regs, Refused> {
    let g = *entry;
    unsafe {
        let one = rd(0x52A910) == 1;
        if one && rd(0x52A920) > 7 {
            return Err(crate::errors::BadInput("a chosen car past the eight").into());
        }
        let objects: Vec<u32> = (0..8).map(|n| get(v.table + 4 * n)).collect();
        if objects.contains(&0) {
            return Err(crate::errors::BadInput("a race sound object missing").into());
        }
        let mut r = g;
        part(PREPARE, &mut r)?;
        let players = if rd(0x75ACE0) == 1 { 1 } else { 2 };
        wr(0x514F78, players);
        let mut count = players;
        if rd(0x75ACD8) == 2 {
            wr(0x514F8C, 1);
            count = 4;
        }
        wr(0x514F7C, count);
        wr(0x514F70, 0);
        wr(0x514F74, 1);
        if one {
            wr(0x514F70, rd(0x52A920));
            wr(0x514F7C, rd(0x52AD98));
        }
        for (n, &o) in v.objects.iter().enumerate() {
            put(o, (-(n as i32) - 1) as u32);
        }
        let first = objects[rd(0x514F70) as usize];
        put(first, 0);
        put(first + 4, rd(0x75ACE8));
        put(first + 0x298, rd(0x75AE20));
        if rd(0x514F78) != 1 {
            let second = objects[rd(0x514F74) as usize];
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
        // The registers at the call: the last object's loop's (eax 0 and ecx 0 after
        // `rep stosb`, ebx the object, edx 8, edi past its 0x68 bytes).
        let last = objects[7];
        let mut r = Regs {
            eax: 0,
            ebx: last,
            ecx: 0,
            edx: 8,
            edi: last + 0x2C + 0x68,
            ..g
        };
        part(RESTART, &mut r)?;
    }
    Ok(g)
}

/// `race::clear_race_sound_objects` (0x415C6D): `silence_race_sounds`; `dword_514F78`,
/// `dword_514F7C`, `dword_514F8C` 0. Every register kept.
pub unsafe fn clear_race_sound_objects(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let g = *entry;
    let mut r = g;
    part(SILENCE, &mut r)?;
    for a in [0x514F78, 0x514F7C, 0x514F8C] {
        wr(a, 0);
    }
    Ok(g)
}

/// Their parts in the game (with `sound_part`'s).
pub fn object_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            PREPARE => prepare_sound_tables(&TableVars::game()),
            RESTART => *r = restart_race_sounds(r, &mut sound_part)?,
            SILENCE => *r = silence_race_sounds(r, &mut sound_part)?,
            _ => sound_part(place, r)?,
        }
    }
    Ok(())
}

/// The plain variables the two write.
pub fn object_regions() -> Vec<Region> {
    [
        0x514F78usize,
        0x514F7C,
        0x514F8C,
        0x514F70,
        0x514F74,
        0x5BEBE8,
        0x52A910,
        0x52A920,
        0x52AD98,
        0x75ACE0,
        0x75ACD8,
    ]
    .iter()
    .map(|&a| (game_addr(rva(a)) as u32, 4))
    .collect()
}
