//! `race::loadlevel_init`'s "init ed textures" (`scene::init_texture_levels`, 0x424001,
//! IDA 2026-10-16): which texture slots are wanted at which level (`g_textureSystemLevels`,
//! `g_textureVideoLevels`) and tagged how (`g_textureTags`), 0x400 slots each, for the
//! stage: each car's runs, each kind's slots by the stage prefs, then fixed runs. With
//! `dword_5CDADC` set every level is another (it is set to 0 here first, and nothing sets
//! it otherwise).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::scene::texture_needs::{SYSTEM_LEVELS_RVA, TAGS_RVA, VIDEO_LEVELS_RVA};

pub const OTHER_LEVELS_RVA: usize = rva(0x5CDADC); // dword_5CDADC
pub const KIND_SLOTS_RVA: usize = rva(0x5DC978); // g_kindSlotBytes: + 4 (or 1) a kind's slot
pub const CARS_RVA: usize = rva(0x63DB90); // g_raceCars
const SECOND_CAR_RVA: usize = rva(0x63DB94); // dword_63DB94
const EXTRA_RVA: usize = rva(0x621514); // dword_621514
const WANTED_RVA: usize = rva(0x5D8968); // g_textureWantedPictures
pub const SLOTS: u32 = 0x400;
const FIXED_TAG: u32 = 0x63;

/// A kind's block: its first-texture variable (only its place in `g_textureWantedPictures`
/// is worked out, and not used: section 16), its prefs entries, the first slot, how many,
/// the tag, the level, the other level, and what `dword_5DC978` gets a slot (4, or 1 for
/// the kinds of slots 0x40 .. 0x6F).
type Block = (usize, usize, u32, u32, u32, u32, u32, u32);
const BLOCKS: [Block; 26] = [
    (0x5D0AE8, 0x60A57C, 0x00, 4, 0x0D, 0, 2, 4),
    (0x5D0AF8, 0x60A58C, 0x04, 4, 0x0C, 0, 2, 4),
    (0x5D0B08, 0x60A59C, 0x08, 4, 0x0B, 0, 2, 4),
    (0x5D0B18, 0x60A5AC, 0x0C, 4, 0x0A, 0, 2, 4),
    (0x5D0B28, 0x60A5BC, 0x10, 4, 0x09, 0, 2, 4),
    (0x5D0B38, 0x60A5CC, 0x14, 4, 0x08, 0, 2, 4),
    (0x5D0B48, 0x60A5DC, 0x18, 4, 0x07, 0, 2, 4),
    (0x5D0B58, 0x60A5EC, 0x1C, 4, 0x06, 0, 2, 4),
    (0x5D0B68, 0x60A5FC, 0x20, 4, 0x05, 0, 2, 4),
    (0x5D0B78, 0x60A60C, 0x24, 4, 0x04, 0, 2, 4),
    (0x5D0B88, 0x60A61C, 0x28, 4, 0x03, 0, 2, 4),
    (0x5D0B98, 0x60A62C, 0x2C, 4, 0x02, 0, 2, 4),
    (0x5D0BA8, 0x60A63C, 0x30, 4, 0x01, 0, 2, 4),
    (0x5D0BE8, 0x60A67C, 0x40, 4, 0x20, 1, 2, 1),
    (0x5D0BF8, 0x60A68C, 0x44, 4, 0x1F, 1, 2, 1),
    (0x5D0C08, 0x60A69C, 0x48, 4, 0x1E, 1, 2, 1),
    (0x5D0C18, 0x60A6AC, 0x4C, 4, 0x1D, 1, 2, 1),
    (0x5D0C28, 0x60A6BC, 0x50, 4, 0x1C, 1, 2, 1),
    (0x5D0C38, 0x60A6CC, 0x54, 4, 0x1B, 1, 2, 1),
    (0x5D0C48, 0x60A6DC, 0x58, 4, 0x1A, 1, 2, 1),
    (0x5D0C58, 0x60A6EC, 0x5C, 4, 0x19, 1, 2, 1),
    (0x5D0C68, 0x60A6FC, 0x60, 4, 0x18, 1, 2, 1),
    (0x5D0C78, 0x60A70C, 0x64, 4, 0x17, 1, 2, 1),
    (0x5D0C88, 0x60A71C, 0x68, 4, 0x16, 1, 2, 1),
    (0x5D0C98, 0x60A72C, 0x6C, 4, 0x15, 1, 2, 1),
    (0x5D0CE8, 0x60A77C, 0x80, 7, 0x3D, 0, 1, 4),
];

/// A run of `set_texture_levels`: count, first slot, level, other level, tag.
type Run = (u32, u32, u32, u32, u32);
const AFTER_BLOCKS: [Run; 15] = [
    (1, 0x87, 0, 0, FIXED_TAG),
    (2, 0xC0, 0, 0, FIXED_TAG),
    (1, 0xC2, 0, 1, FIXED_TAG),
    (1, 0x257, 0, 0, FIXED_TAG),
    (1, 0xCA, 0, 0, FIXED_TAG),
    (1, 0xC8, 3, 3, FIXED_TAG),
    (1, 0xD3, 0, 0, FIXED_TAG),
    (1, 0xD4, 0, 0, FIXED_TAG),
    (1, 0xC3, 0, 0, FIXED_TAG),
    (1, 0xD2, 0, 0, FIXED_TAG),
    (1, 0xC6, 1, 1, FIXED_TAG),
    (1, 0xC7, 1, 1, FIXED_TAG),
    (1, 0xC4, 1, 1, FIXED_TAG),
    (1, 0xC5, 1, 1, FIXED_TAG),
    (1, 0xC9, 1, 1, FIXED_TAG),
];
const LAST: Run = (2, 0x21C, 0, 0, FIXED_TAG);
const EXTRA: [Run; 3] = [
    (0x10, 0x200, 2, 2, FIXED_TAG),
    (1, 0x218, 0, 0, FIXED_TAG),
    (8, 0x210, 0, 0, FIXED_TAG),
];

unsafe fn put(table: usize, slot: u32, value: u32) {
    unsafe { game_write(table + 4 * slot as usize, value) };
}

fn other() -> bool {
    unsafe { game_read::<u32>(OTHER_LEVELS_RVA) != 0 }
}

/// `scene::set_texture_levels` (0x422426): ecx slots from ebx get level eax (edx with
/// `dword_5CDADC` set) in both tables, tag ebp. Leaves ebx past them, ecx 0. Refuses no
/// slots (the original counts round) or slots past 0x400.
pub unsafe fn set_texture_levels(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.ecx == 0 || g.ebx >= SLOTS || g.ecx > SLOTS - g.ebx {
        return Err(BadInput("no texture slots, or past 0x400").into());
    }
    let level = if other() { g.edx } else { g.eax };
    unsafe {
        while g.ecx != 0 {
            put(SYSTEM_LEVELS_RVA, g.ebx, level);
            put(VIDEO_LEVELS_RVA, g.ebx, level);
            put(TAGS_RVA, g.ebx, g.ebp);
            g.ebx += 1;
            g.ecx -= 1;
        }
    }
    Ok(g)
}

fn run(g: &mut Regs, (count, first, level, other_level, tag): Run) -> Result<(), Refused> {
    g.ecx = count;
    g.ebx = first;
    g.eax = level;
    g.edx = other_level;
    g.ebp = tag;
    *g = unsafe { set_texture_levels(g) }?;
    Ok(())
}

/// `init_texture_levels`' calls by place: 0 `scene::reset_texture_tables`, 1 and 2 the
/// log's (eax the text; the number with a line's end), 3 `input::update`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

const COUNT_TEXT_RVA: usize = rva(0x4DB7B9); // "NUM DIFFERENT CARS/OBJECTS/TEXTURE SETS "

/// `scene::init_texture_levels` (0x424001), see the header: `g_raceCars` held to 1 .. 8
/// (by the signs of the differences, as the original); the tables reset, the count in the
/// log; `dword_5CDADC` 0 and `input::update`; the first car's two runs (slots 0x100,
/// 0x104; the second's level by `dword_63DB94`), each further car's two (0x108 + 8 a car,
/// tag 0x28 + the car); each kind's slots (its prefs entry not 0 or -1: tag, level, and
/// `dword_5DC978` + 4, or + 1 for slots 0x40 .. 0x6F); the fixed runs, two loops at slot 0xDC (the system level of six,
/// the video level of two), and with `dword_621514` three more. Leaves ebx past the last
/// run, eax, ecx, edx 0, ebp 0x63, esi past the prefs, edi the last kind's wanted picture.
pub unsafe fn init_texture_levels(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let mut cars: u32 = game_read(CARS_RVA);
        if (cars.wrapping_sub(1) as i32) < 0 {
            cars = 1;
        }
        if (cars.wrapping_sub(8) as i32) >= 0 {
            cars = 8;
        }
        game_write(CARS_RVA, cars);
        part(0, &mut g)?;
        let mut r = Regs {
            eax: game_addr(COUNT_TEXT_RVA) as u32,
            ..g
        };
        part(1, &mut r)?;
        let mut r = Regs { eax: cars, ..g };
        part(2, &mut r)?;
        game_write(OTHER_LEVELS_RVA, 0u32);
        part(3, &mut g)?;
        run(&mut g, (4, 0x100, 0, 1, 0x29))?;
        let second = if game_read::<u32>(SECOND_CAR_RVA) == 1 {
            0
        } else {
            1
        };
        run(&mut g, (4, 0x104, second, 1, 0x29))?;
        for car in 2..=8u32 {
            if cars == car - 1 {
                break;
            }
            let first = 0x108 + 8 * (car - 2);
            run(&mut g, (4, first, 0, 2, 0x28 + car))?;
            run(&mut g, (4, first + 4, 1, 2, 0x28 + car))?;
        }
        for (first_var, prefs, slot, count, tag, level, other_level, step) in BLOCKS {
            let kind_first: u32 = game_read(rva(first_var));
            g.edi = (game_addr(WANTED_RVA) as u32).wrapping_add(kind_first.wrapping_mul(4));
            g.esi = game_addr(rva(prefs)) as u32;
            g.edx = slot;
            g.ecx = count;
            while g.ecx != 0 {
                g.eax = (g.esi as *const u32).read_unaligned();
                if g.eax != 0 && g.eax != u32::MAX {
                    game_write(
                        KIND_SLOTS_RVA,
                        game_read::<u32>(KIND_SLOTS_RVA).wrapping_add(step),
                    );
                    put(TAGS_RVA, g.edx, tag);
                    let l = if other() { other_level } else { level };
                    put(SYSTEM_LEVELS_RVA, g.edx, l);
                    put(VIDEO_LEVELS_RVA, g.edx, l);
                }
                g.esi += 4;
                g.edx += 1;
                g.ecx -= 1;
            }
        }
        for r in AFTER_BLOCKS {
            run(&mut g, r)?;
        }
        for (count, table) in [(6u32, SYSTEM_LEVELS_RVA), (2, VIDEO_LEVELS_RVA)] {
            g.ecx = count;
            g.ebx = 0xDC;
            while g.ecx != 0 {
                put(table, g.ebx, 1);
                put(TAGS_RVA, g.ebx, FIXED_TAG);
                g.ebx += 1;
                g.ecx -= 1;
            }
        }
        run(&mut g, LAST)?;
        if game_read::<u32>(EXTRA_RVA) != 0 {
            for r in EXTRA {
                run(&mut g, r)?;
            }
        }
    }
    Ok(g)
}

/// The game's calls: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            0 => crate::scene::reset_texture_tables::reset_texture_tables().apply(r),
            1 => log_write(r.eax as *const u8, false, 1),
            2 => log_int(r.eax as i32, true, 1),
            _ => *r = crate::input::keys::update(r, &mut crate::input::keys::game_screenshot),
        }
    }
    Ok(())
}

/// The three texture tables.
pub fn tables() -> Vec<crate::service::harness::Region> {
    [TAGS_RVA, SYSTEM_LEVELS_RVA, VIDEO_LEVELS_RVA]
        .map(|t| (game_addr(t) as u32, 4 * SLOTS as usize))
        .to_vec()
}
