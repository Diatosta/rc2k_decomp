//! `race::loadlevel_init`'s "init ed textures" (`scene::init_texture_levels`, 0x424001,
//! IDA 2026-10-16): which texture slots are wanted at which level (`g_textureSystemLevels`,
//! `g_textureVideoLevels`) and tagged how (`g_textureTags`), 0x400 slots each, for the
//! stage: each car's runs, each kind's slots by the stage prefs, then fixed runs. With
//! `dword_5CDADC` set every level is another (it is set to 0 here first, and nothing sets
//! it otherwise).

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::view_store::RACE_VIEW;
use crate::scene::texture_needs::{SYSTEM_LEVELS_RVA, TAGS_RVA, VIDEO_LEVELS_RVA};

pub const OTHER_LEVELS_RVA: usize = rva(0x5CDADC); // dword_5CDADC
pub const KIND_SLOTS_RVA: usize = rva(0x5DC978); // g_kindSlotBytes: + 4 (or 1) a kind's slot
const EXTRA_RVA: usize = rva(0x621514); // dword_621514
pub const SLOTS: u32 = 0x400;

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

fn other() -> bool {
    unsafe { game_read::<u32>(OTHER_LEVELS_RVA) != 0 }
}

const COUNT_TEXT_RVA: &std::ffi::CStr = c"NUM DIFFERENT CARS/OBJECTS/TEXTURE SETS "; // "NUM DIFFERENT CARS/OBJECTS/TEXTURE SETS "

/// `scene::init_texture_levels` (0x424001), see the header: `g_raceCars` held to 1 .. 8
/// (by the signs of the differences, as the original); the tables reset, the count in the
/// log; `dword_5CDADC` 0 and `input::update`; the first car's two runs (slots 0x100,
/// 0x104; the second's level by `dword_63DB94`), each further car's two (0x108 + 8 a car,
/// tag 0x28 + the car); each kind's slots (its prefs entry not 0 or -1: tag, level, and
/// `dword_5DC978` + 4, or + 1 for slots 0x40 .. 0x6F); the fixed runs, two loops at slot 0xDC (the system level of six,
/// the video level of two), and with `dword_621514` three more. What the original
/// leaves in the registers its one caller reads none of (register liveness, 2026-10-23:
/// the site in `race::loadlevel_init`, every register followed, no read).
pub unsafe fn init_texture_levels() -> Result<(), Refused> {
    // Converted: `rc2k_core::scene::texture_levels::{held_cars, set_levels}`.
    use crate::logfile::{log_int::log_int, log_write::log_write};
    use crate::scene::textures::slice;
    use rc2k_core::scene::texture_levels::{KindSlots, held_cars, set_levels};
    unsafe {
        let cars = held_cars(RACE_VIEW.race_cars.bits());
        RACE_VIEW.race_cars.set_bits(cars);
        crate::scene::reset_texture_tables::reset_texture_tables();
        log_write(COUNT_TEXT_RVA.as_ptr() as usize as *const u8, false, 1);
        log_int(cars as i32, true, 1);
        game_write(OTHER_LEVELS_RVA, 0u32);
        crate::input::keys::update_input(&mut crate::input::keys::game_screenshot);
        let kinds: Vec<(KindSlots, Vec<u32>)> = BLOCKS
            .iter()
            .map(|&(_, prefs, first, count, tag, level, other_level, step)| {
                let k = KindSlots {
                    first,
                    tag,
                    level,
                    other_level,
                    step,
                };
                let at = game_addr(rva(prefs)) as *const u32;
                (
                    k,
                    (0..count as usize)
                        .map(|n| at.add(n).read_unaligned())
                        .collect(),
                )
            })
            .collect();
        let set = set_levels(
            &mut rc2k_core::scene::textures::LevelTables {
                tags: slice(TAGS_RVA, SLOTS),
                system: slice(SYSTEM_LEVELS_RVA, SLOTS),
                video: slice(VIDEO_LEVELS_RVA, SLOTS),
            },
            cars,
            RACE_VIEW.players.bits() == 1,
            &kinds,
            game_read::<u32>(EXTRA_RVA) != 0,
            other(),
        )?;
        game_write(
            KIND_SLOTS_RVA,
            game_read::<u32>(KIND_SLOTS_RVA).wrapping_add(set.kind_slots_added),
        );
    }
    Ok(())
}
