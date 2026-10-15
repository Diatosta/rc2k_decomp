//! The stage choice's own steps (IDA, 2026-10-15): the UK map with the chosen stage
//! (`menu::draw_stage_map`, 14 callers), and the choice's three settings stepped, each then
//! drawing the map again.

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::screens::draw_stage_records::selection;

pub const MODE_RVA: usize = rva(0x755700); // dword_755700: 2 a championship's, 3 a record's
pub const GROUP_RVA: usize = rva(0x7558C8); // dword_7558C8: 0 or 1
pub const SUB_RVA: usize = rva(0x7558D0); // dword_7558D0: 0 .. 5
pub const NUMBER_RVA: usize = rva(0x7558CC); // dword_7558CC: 1 .. 6
const CHAMPIONSHIP_TABLES_RVA: usize = rva(0x73555A); // off_73555A
const CHAMPIONSHIP_TABLE_RVA: usize = rva(0x7587EC); // dword_7587EC
const CHAMPIONSHIP_STAGE_RVA: usize = rva(0x7587F0); // dword_7587F0
const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
const SECTION_RVA: usize = rva(0x75595C); // section
const INDEX_RVA: usize = rva(0x755958); // index
const PALETTES_RVA: usize = rva(0x7598AE); // off_7598AE: the map's overlays by the stage's +0x3C
const MAP_PATH_RVA: usize = rva(0x73218B); // "files\g\graphics\screens\ukmap.png"
const SCREEN_MODE_FLAGS_RVA: usize = rva(0x4F4934); // g_screenModeFlags
const PICTURE_FLAGS_RVA: usize = rva(0x4F49CC); // g_pictureFlags
const SPRITE_TABLE_RVA: usize = rva(0x755581); // g_spriteTable
const RECORD_SIZE: u32 = 0xA4;
const MAP_SPRITE: u32 = 0xA0;
/// The map that cannot be loaded: no message.
const NO_MAP: FatalError = FatalError {
    code: 0x8071_2345,
    message: "",
};

/// A callee by place, with the registers it gets and leaves (and its carry):
/// 0 `records::find_record` (edi the table, ebx the section, eax the index: edi the
/// record), 1 `gfx::load_picture_with_overlay` (eax the path, ebx the overlay), 2
/// `gfx::convert_picture`, 3 `gfx::picture_as_surface`, 4 `gfx::blit_glyph` (eax the
/// sprite, esi the table, ecx and edx 0).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// `menu::draw_stage_map`: the stage by `dword_755700` (2: `off_73555A[dword_7587EC]`'s
/// `dword_7587F0`th; 3: `records::find_record` of `g_recordTable` with `section` and
/// `index`; else the choice's: `off_735512[group][sub]`'s `number`th), its overlay
/// `off_7598AE[+0x3C]`; `ukmap.png` loaded with it (the carry: fatal error 0x80712345),
/// converted, `g_pictureFlags` = `g_screenModeFlags` & 0xFFF, made the surface, sprite 0xA0
/// at (0, 0). Refuses a choice without its tables (the original reads through null).
pub unsafe fn draw_stage_map(entry: &Regs, part: Part) -> Result<(), Refused> {
    let mut g = *entry;
    unsafe {
        let mode: u32 = game_read(MODE_RVA);
        g.edi = match mode {
            2 => {
                g.ebx = game_read(CHAMPIONSHIP_TABLE_RVA);
                g.eax = game_read::<u32>(CHAMPIONSHIP_STAGE_RVA).wrapping_mul(RECORD_SIZE);
                dword(game_addr(CHAMPIONSHIP_TABLES_RVA) as u32 + g.ebx.wrapping_mul(4))
                    .wrapping_add(g.eax)
            }
            3 => {
                g.edi = game_addr(RECORD_TABLE_RVA) as u32;
                g.ebx = game_read(SECTION_RVA);
                g.eax = game_read(INDEX_RVA);
                part(0, &mut g)?;
                g.edi
            }
            _ => {
                let s = selection().ok_or(crate::errors::BadInput(
                    "no stage tables for the choice: the original reads through null",
                ))?;
                g.eax = game_read::<u32>(NUMBER_RVA).wrapping_mul(RECORD_SIZE);
                g.ebx = s.sub;
                s.table.wrapping_add(g.eax)
            }
        };
        g.ebx = dword(game_addr(PALETTES_RVA) as u32 + dword(g.edi + 0x3C).wrapping_mul(4));
        g.eax = game_addr(MAP_PATH_RVA) as u32;
        part(1, &mut g)?;
        if g.carry() {
            return Err(NO_MAP.into());
        }
        part(2, &mut g)?;
        g.eax = game_read::<u32>(SCREEN_MODE_FLAGS_RVA) & 0xFFF;
        game_write(PICTURE_FLAGS_RVA, g.eax);
        part(3, &mut g)?;
        g.eax = MAP_SPRITE;
        g.esi = game_read(SPRITE_TABLE_RVA);
        g.ecx = 0;
        g.edx = 0;
        part(4, &mut g)?;
    }
    Ok(())
}

/// The game's callees: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => r.edi = crate::records::find_record::find_record_in(r.edi, r.eax as i32, r.ebx),
            1 => {
                let carry = crate::gfx::load_picture_with_overlay::load_picture_with_overlay(
                    r.eax as *mut u8,
                    r.ebx as *mut u8,
                )?;
                r.set_carry(carry);
            }
            2 => crate::gfx::convert_picture::convert_picture_now()?,
            3 => crate::gfx::picture_as_surface::picture_as_surface()?,
            _ => {
                crate::gfx::blit_glyph::blit_glyph_on_surface(
                    r.eax,
                    r.esi as *const u8,
                    r.ecx as i32,
                    r.edx as i32,
                    r.ebx as *const u32,
                )?;
            }
        }
    }
    Ok(())
}

pub unsafe fn draw_stage_map_in_game() -> Result<(), Refused> {
    unsafe { draw_stage_map(&Regs::default(), &mut game_part) }
}

/// How a setting draws the map: the Rust version in the game, a counter in the tests.
pub type Draw<'a> = &'a mut dyn FnMut() -> Result<(), Refused>;
/// A setting's Rust version.
pub type SettingFn = unsafe fn(Draw) -> Result<(), Refused>;

/// The choice's settings: the variable stepped, then the map drawn again (`draw`: the Rust
/// version in the game). When the map refuses, the variable is put back first, so that the
/// original, which then runs, steps it once.
unsafe fn step(rva: usize, next: impl Fn(u32) -> u32, draw: Draw) -> Result<(), Refused> {
    unsafe {
        let old: u32 = game_read(rva);
        game_write(rva, next(old));
        draw().inspect_err(|_| game_write(rva, old))
    }
}

/// `menu::switch_stage_group`: the group 0 <-> 1.
pub unsafe fn switch_stage_group(draw: Draw) -> Result<(), Refused> {
    unsafe { step(GROUP_RVA, |g| g ^ 1, draw) }
}

/// `menu::next_stage_sub`: 5 or more (unsigned) to 0, else on.
pub unsafe fn next_stage_sub(draw: Draw) -> Result<(), Refused> {
    unsafe { step(SUB_RVA, |s| if s >= 5 { 0 } else { s + 1 }, draw) }
}

/// `menu::previous_stage_sub`: below 1 (signed) to 5, else back.
pub unsafe fn previous_stage_sub(draw: Draw) -> Result<(), Refused> {
    unsafe { step(SUB_RVA, |s| if (s as i32) < 1 { 5 } else { s - 1 }, draw) }
}

/// `menu::next_stage_number`: 6 or more (unsigned) to 1, else on.
pub unsafe fn next_stage_number(draw: Draw) -> Result<(), Refused> {
    unsafe { step(NUMBER_RVA, |n| if n >= 6 { 1 } else { n + 1 }, draw) }
}

// ---- for the settings' self-tests ----

/// A setting's function: its address and length as IDA shows them, its first bytes, the
/// offset of its call of the map.
pub struct Setting {
    pub address: usize,
    pub len: usize,
    pub entry: &'static [u8],
    pub call: usize,
    pub variable: usize,
}

static MAPS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn count_map(_: u32, _: &mut Regs) {
    MAPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// A copy whose map call goes to a stand-in that counts it, against the Rust version with a
/// `draw` that counts; on the game's variable with each of `values`, put back afterwards.
pub fn check_setting(
    t: &mut crate::framework::SelfTest,
    s: &Setting,
    values: &[u32],
    rust: SettingFn,
) -> Result<(), &'static str> {
    use crate::install::stand_ins::{set_handler, stand_in};
    use crate::service::harness::{at, capture, restore};
    use std::sync::atomic::Ordering;
    let copy = unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(s.address)),
            s.len,
            s.entry,
            &[s.call],
            &|_| Some(stand_in(0)),
        )
    }?;
    set_handler(count_map);
    let var = [at(rva(s.variable))];
    let saved = capture(&var);
    for &value in values {
        let run = |rust_side: bool| {
            unsafe { game_write(rva(s.variable), value) };
            MAPS.store(0, Ordering::SeqCst);
            let before = Regs::SENTINEL;
            let mut ro = before;
            let ok = if rust_side {
                unsafe {
                    rust(&mut || {
                        MAPS.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    })
                }
                .is_ok()
            } else {
                unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
                ro.preserved_except(
                    &before,
                    crate::framework::EAX | crate::framework::ECX | crate::framework::EDX,
                )
            };
            (
                ok,
                unsafe { game_read::<u32>(rva(s.variable)) },
                MAPS.load(Ordering::SeqCst),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("{value:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&var, &saved);
    Ok(())
}
